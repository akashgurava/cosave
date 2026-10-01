use crate::core::{init_db, AppConfig};

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
    assert_eq!(hierarchy.categories().len(), 1);
    assert_eq!(hierarchy.categories()[0].subcategories().len(), 1);
    assert_eq!(hierarchy.colors().len(), 2);
}

#[tokio::test]
async fn test_reset_categories_to_defaults_transaction() {
    let pool = setup_test_db().await;

    // Hierarchy starts with 4 types, 8 categories, 12 colors
    let before = fetch_hierarchy(&pool).await.unwrap();
    assert_eq!(before.types().len(), 4);
    assert_eq!(before.categories().len(), 8);
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
    assert_eq!(after_reset.categories().len(), 8);
    assert_eq!(after_reset.colors().len(), 12);
}
