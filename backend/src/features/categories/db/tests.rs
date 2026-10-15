use sqlx::Row;

use crate::core::{init_db, is_foreign_key_violation, is_unique_violation, AppConfig};

use super::*;

async fn setup_test_db() -> crate::core::DbPool {
    let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
        .await
        .expect("init test sqlite in-memory db");
    let mut tx = pool.begin().await.expect("begin tx");
    init_category_schema(&mut tx)
        .await
        .expect("init category schema");
    tx.commit().await.expect("commit tx");
    seed_default_categories(&pool).await.expect("seed defaults");
    pool
}

#[tokio::test]
async fn test_cascade_delete_type_removes_categories_and_subcategories() {
    let pool = setup_test_db().await;

    // 1. Insert transaction type
    let typ_res = sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('Test Type', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let typ_id = typ_res.last_insert_rowid();

    // 2. Insert category under type
    let cat_res = sqlx::query(
        "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (?, 'Test Cat', 1, 0, 0)",
    )
    .bind(typ_id)
    .execute(&pool)
    .await
    .unwrap();
    let cat_id = cat_res.last_insert_rowid();

    // 3. Insert subcategory under category
    let sub_res = sqlx::query(
        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (?, 'Test Sub', 1, 0, 0)",
    )
    .bind(cat_id)
    .execute(&pool)
    .await
    .unwrap();
    let sub_id = sub_res.last_insert_rowid();

    // 4. Verify all 3 exist
    let (typ_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM transaction_types WHERE id = ?")
            .bind(typ_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let (cat_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories WHERE id = ?")
        .bind(cat_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (sub_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = ?")
        .bind(sub_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(typ_count, 1);
    assert_eq!(cat_count, 1);
    assert_eq!(sub_count, 1);

    // 5. Delete transaction type
    sqlx::query("DELETE FROM transaction_types WHERE id = ?")
        .bind(typ_id)
        .execute(&pool)
        .await
        .unwrap();

    // 6. Verify cascade: categories and subcategories are both deleted
    let (cat_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories WHERE id = ?")
        .bind(cat_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (sub_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = ?")
        .bind(sub_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        cat_after, 0,
        "categories must be cascade deleted when type is deleted"
    );
    assert_eq!(
        sub_after, 0,
        "subcategories must be cascade deleted when parent type is deleted"
    );
}

#[tokio::test]
async fn test_cascade_delete_category_removes_subcategories() {
    let pool = setup_test_db().await;

    let typ_res = sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('Test Type', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let typ_id = typ_res.last_insert_rowid();

    let cat_res = sqlx::query(
        "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (?, 'Test Cat', 1, 0, 0)",
    )
    .bind(typ_id)
    .execute(&pool)
    .await
    .unwrap();
    let cat_id = cat_res.last_insert_rowid();

    let sub_res = sqlx::query(
        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (?, 'Test Sub', 1, 0, 0)",
    )
    .bind(cat_id)
    .execute(&pool)
    .await
    .unwrap();
    let sub_id = sub_res.last_insert_rowid();

    // Delete category only
    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(cat_id)
        .execute(&pool)
        .await
        .unwrap();

    let (sub_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = ?")
        .bind(sub_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        sub_after, 0,
        "subcategory must be cascade deleted when category is deleted"
    );

    // Type still remains
    let (typ_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM transaction_types WHERE id = ?")
            .bind(typ_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(typ_after, 1, "parent type must remain untouched");
}

#[tokio::test]
async fn test_view_v_category_hierarchy_aggregates_properly() {
    let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
        .await
        .expect("init test sqlite in-memory db");
    let mut tx = pool.begin().await.expect("begin tx");
    init_category_schema(&mut tx)
        .await
        .expect("init category schema");
    tx.commit().await.expect("commit tx");

    sqlx::query(
        "INSERT INTO colors (id, name, hex, sort_order, created_at, updated_at) VALUES (1, 'Green', '#00ff00', 1, 0, 0), (2, 'Red', '#ff0000', 2, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Type 1 with Category and Subcategory
    let typ1_res = sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('Type 1', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let typ1_id = typ1_res.last_insert_rowid();

    let cat1_res = sqlx::query(
        "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (?, 'Cat 1', 1, 0, 0)",
    )
    .bind(typ1_id)
    .execute(&pool)
    .await
    .unwrap();
    let cat1_id = cat1_res.last_insert_rowid();

    sqlx::query(
        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (?, 'Sub 1', 1, 0, 0)",
    )
    .bind(cat1_id)
    .execute(&pool)
    .await
    .unwrap();

    // Type 2 with no categories (empty type)
    sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('Type 2 Empty', 2, 2, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let hierarchy = fetch_hierarchy(&pool).await.expect("fetch hierarchy");
    assert_eq!(hierarchy.types().len(), 2);
    assert_eq!(hierarchy.types()[0].categories().len(), 1);
    assert_eq!(
        hierarchy.types()[0].categories()[0].subcategories().len(),
        1
    );
    assert_eq!(hierarchy.colors().len(), 2);

    // Verify exact column order in v_category_hierarchy view
    let view_columns: Vec<String> = sqlx::query("PRAGMA table_info(v_category_hierarchy)")
        .fetch_all(&pool)
        .await
        .expect("query pragma table_info for view")
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect();

    assert_eq!(
        view_columns,
        vec![
            "type_color_id",
            "type_color",
            "type_id",
            "type_name",
            "type_sort_order",
            "category_id",
            "category_name",
            "category_sort_order",
            "subcategory_id",
            "subcategory_name",
            "subcategory_sort_order",
        ],
        "v_category_hierarchy column order must match canonical sequence"
    );
}

#[tokio::test]
async fn test_reset_categories_to_defaults_transaction() {
    let pool = setup_test_db().await;

    // Hierarchy starts with 4 types, 8 categories, 12 colors
    let before = fetch_hierarchy(&pool).await.unwrap();
    assert_eq!(before.types().len(), 4);
    let total_cats: usize = before.types().iter().map(|t| t.categories().len()).sum();
    assert_eq!(total_cats, 8);
    assert_eq!(before.colors().len(), 12);

    // Delete a type
    delete_type(&pool, before.types()[0].id()).await.unwrap();
    let after_delete = fetch_hierarchy(&pool).await.unwrap();
    assert_eq!(after_delete.types().len(), 3);

    // Reset to defaults
    reset_defaults(&pool)
        .await
        .expect("reset defaults transaction");
    let after_reset = fetch_hierarchy(&pool).await.unwrap();
    assert_eq!(after_reset.types().len(), 4);
    let reset_cats: usize = after_reset
        .types()
        .iter()
        .map(|t| t.categories().len())
        .sum();
    assert_eq!(reset_cats, 8);
    assert_eq!(after_reset.colors().len(), 12);
}

#[tokio::test]
async fn test_foreign_key_restrict_deleting_color_in_use_fails() {
    let pool = setup_test_db().await;

    // Default seeded hierarchy has transaction types referencing color_id = 1.
    // Attempting to delete color 1 must fail due to foreign key ON DELETE RESTRICT.
    let delete_result = sqlx::query("DELETE FROM colors WHERE id = 1")
        .execute(&pool)
        .await;

    assert!(
        delete_result.is_err(),
        "deleting a color actively referenced by a transaction type must violate ON DELETE RESTRICT"
    );
    let err = delete_result.unwrap_err();
    assert!(
        is_foreign_key_violation(&err),
        "expected foreign key constraint violation, got: {err}"
    );
    assert!(
        !is_unique_violation(&err),
        "foreign key error is not a unique violation"
    );

    // Verify an unused color can be deleted cleanly without violating foreign key constraints
    sqlx::query(
        "INSERT INTO colors (id, name, hex, sort_order, created_at, updated_at) VALUES (999, 'Unused Color', '#123456', 999, 0, 0)",
    )
    .execute(&pool)
    .await
    .expect("insert unused color");

    let delete_unused = sqlx::query("DELETE FROM colors WHERE id = 999")
        .execute(&pool)
        .await;
    assert!(
        delete_unused.is_ok(),
        "deleting an unreferenced color must succeed"
    );
}

#[tokio::test]
async fn test_category_schema_constraint_violation_detection() {
    let pool = setup_test_db().await;

    // 1. Type name unique violation
    let dup_type_err = sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('Income', 1, 99, 0, 0)",
    )
    .execute(&pool)
    .await
    .expect_err("insert duplicate type name must fail");
    assert!(
        is_unique_violation(&dup_type_err),
        "duplicate type name must be a unique violation: {dup_type_err}"
    );
    assert!(!is_foreign_key_violation(&dup_type_err));

    // 2. Type with non-existent color_id foreign key violation
    let fk_type_err = sqlx::query(
        "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES ('New Type', 99999, 99, 0, 0)",
    )
    .execute(&pool)
    .await
    .expect_err("insert type with invalid color_id must fail");
    assert!(
        is_foreign_key_violation(&fk_type_err),
        "invalid color_id must be a foreign key violation: {fk_type_err}"
    );
    assert!(!is_unique_violation(&fk_type_err));

    // 3. Category under non-existent type_id foreign key violation
    let fk_cat_err = sqlx::query(
        "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (99999, 'Orphan Cat', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .expect_err("insert category with invalid type_id must fail");
    assert!(
        is_foreign_key_violation(&fk_cat_err),
        "invalid type_id must be a foreign key violation: {fk_cat_err}"
    );
    assert!(!is_unique_violation(&fk_cat_err));

    // 4. Duplicate category name under same type_id unique violation
    let (expense_id,): (i64,) =
        sqlx::query_as("SELECT id FROM transaction_types WHERE name = 'Expense'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let dup_cat_err = sqlx::query(
        "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (?, 'Housing', 99, 0, 0)",
    )
    .bind(expense_id)
    .execute(&pool)
    .await
    .expect_err("insert duplicate category name under same type must fail");
    assert!(
        is_unique_violation(&dup_cat_err),
        "duplicate category name under type must be unique violation: {dup_cat_err}"
    );
    assert!(!is_foreign_key_violation(&dup_cat_err));

    // 5. Subcategory under non-existent category_id foreign key violation
    let fk_sub_err = sqlx::query(
        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (99999, 'Orphan Sub', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .expect_err("insert subcategory with invalid category_id must fail");
    assert!(
        is_foreign_key_violation(&fk_sub_err),
        "invalid category_id must be a foreign key violation: {fk_sub_err}"
    );
    assert!(!is_unique_violation(&fk_sub_err));

    // 6. Duplicate subcategory name under same category_id unique violation
    let (housing_id,): (i64,) = sqlx::query_as("SELECT id FROM categories WHERE name = 'Housing'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let dup_sub_err = sqlx::query(
        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (?, 'Rent & Mortgage', 99, 0, 0)",
    )
    .bind(housing_id)
    .execute(&pool)
    .await
    .expect_err("insert duplicate subcategory name under same category must fail");
    assert!(
        is_unique_violation(&dup_sub_err),
        "duplicate subcategory name under category must be unique violation: {dup_sub_err}"
    );
    assert!(!is_foreign_key_violation(&dup_sub_err));
}
