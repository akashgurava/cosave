use super::*;
use crate::core::init_db;

async fn setup_test_db() -> crate::core::DbPool {
    let pool = init_db("sqlite::memory:")
        .await
        .expect("init test sqlite in-memory db");
    init_schema(&pool).await.expect("init schema");
    seed_default_colors(&pool).await.expect("seed colors");
    pool
}

#[tokio::test]
async fn test_cascade_delete_type_removes_categories_and_subcategories() {
    let pool = setup_test_db().await;

    // 1. Insert transaction type
    sqlx::query(
        "INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at) VALUES ('typ_test', 'Test Type', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2. Insert category under type
    sqlx::query(
        "INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at) VALUES ('cat_test', 'typ_test', 'Test Cat', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Insert subcategory under category
    sqlx::query(
        "INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at) VALUES ('sub_test', 'cat_test', 'Test Sub', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 4. Verify all 3 exist
    let (typ_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM transaction_types WHERE id = 'typ_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (cat_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM categories WHERE id = 'cat_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (sub_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = 'sub_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(typ_count, 1);
    assert_eq!(cat_count, 1);
    assert_eq!(sub_count, 1);

    // 5. Delete transaction type
    sqlx::query("DELETE FROM transaction_types WHERE id = 'typ_test'")
        .execute(&pool)
        .await
        .unwrap();

    // 6. Verify cascade: categories and subcategories are both deleted
    let (cat_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM categories WHERE id = 'cat_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (sub_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = 'sub_test'")
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

    sqlx::query(
        "INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at) VALUES ('typ_test', 'Test Type', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at) VALUES ('cat_test', 'typ_test', 'Test Cat', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at) VALUES ('sub_test', 'cat_test', 'Test Sub', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Delete category only
    sqlx::query("DELETE FROM categories WHERE id = 'cat_test'")
        .execute(&pool)
        .await
        .unwrap();

    let (sub_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM subcategories WHERE id = 'sub_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        sub_after, 0,
        "subcategory must be cascade deleted when category is deleted"
    );

    // Type still remains
    let (typ_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM transaction_types WHERE id = 'typ_test'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(typ_after, 1, "parent type must remain untouched");
}

#[tokio::test]
async fn test_view_v_category_hierarchy_aggregates_properly() {
    let pool = setup_test_db().await;

    // Type 1 with Category and Subcategory
    sqlx::query(
        "INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at) VALUES ('typ_1', 'Type 1', 1, 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at) VALUES ('cat_1', 'typ_1', 'Cat 1', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at) VALUES ('sub_1', 'cat_1', 'Sub 1', 1, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Type 2 with no categories (empty type)
    sqlx::query(
        "INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at) VALUES ('typ_2', 'Type 2 Empty', 2, 2, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let hierarchy = fetch_hierarchy(&pool).await.expect("fetch hierarchy");
    assert_eq!(hierarchy.types().len(), 2);
    assert_eq!(hierarchy.categories().len(), 1);
    assert_eq!(hierarchy.categories()[0].subcategories().len(), 1);
    assert!(!hierarchy.colors().is_empty());
}

#[tokio::test]
async fn test_reset_categories_to_defaults_transaction() {
    let pool = setup_test_db().await;
    seed_default_categories(&pool).await.expect("seed defaults");

    // Hierarchy starts with 4 types, 8 categories
    let before = fetch_hierarchy(&pool).await.unwrap();
    assert_eq!(before.types().len(), 4);
    assert_eq!(before.categories().len(), 8);

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
}
