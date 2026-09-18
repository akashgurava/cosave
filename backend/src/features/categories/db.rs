use super::models::{
    CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, SubcategoryItem,
    TransactionTypeItem,
};
use crate::core::db::DbPool;

pub(crate) fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            if code == "2067" || code == "1555" {
                return true;
            }
        }
        let msg = db_err.message();
        if msg.contains("UNIQUE constraint failed") {
            return true;
        }
    }
    false
}

pub(crate) async fn fetch_hierarchy(
    pool: &DbPool,
) -> Result<CategoryHierarchyResponse, sqlx::Error> {
    let rows: Vec<CategoryHierarchyRow> = sqlx::query_as(
        r#"
        SELECT
            type_id,
            type_name,
            type_color,
            type_sort_order,
            category_id,
            category_name,
            category_sort_order,
            subcategory_id,
            subcategory_name,
            subcategory_sort_order
        FROM v_category_hierarchy
        ORDER BY type_sort_order, type_name, category_sort_order, category_name, subcategory_sort_order, subcategory_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut types: Vec<TransactionTypeItem> = Vec::new();
    let mut categories: Vec<CategoryItem> = Vec::new();

    for row in rows {
        if !types.iter().any(|t| t.id == row.type_id) {
            types.push(TransactionTypeItem {
                id: row.type_id.clone(),
                name: row.type_name.clone(),
                color: row.type_color.clone(),
            });
        }

        if let (Some(cat_id), Some(cat_name)) = (row.category_id, row.category_name) {
            if let Some(cat) = categories.iter_mut().find(|c| c.id == cat_id) {
                if let (Some(sub_id), Some(sub_name)) = (row.subcategory_id, row.subcategory_name) {
                    if !cat.subcategories.iter().any(|s| s.id == sub_id) {
                        cat.subcategories.push(SubcategoryItem {
                            id: sub_id,
                            name: sub_name,
                        });
                    }
                }
            } else {
                let mut subcategories = Vec::new();
                if let (Some(sub_id), Some(sub_name)) = (row.subcategory_id, row.subcategory_name) {
                    subcategories.push(SubcategoryItem {
                        id: sub_id,
                        name: sub_name,
                    });
                }
                categories.push(CategoryItem {
                    id: cat_id,
                    name: cat_name,
                    type_name: row.type_name.clone(),
                    subcategories,
                });
            }
        }
    }

    Ok(CategoryHierarchyResponse { types, categories })
}

pub(crate) async fn get_next_type_sort_order(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
        .fetch_one(pool)
        .await?;
    Ok(max_sort.0.unwrap_or(0) + 1)
}

pub(crate) async fn insert_type(
    pool: &DbPool,
    id: &str,
    name: &str,
    color: &str,
    sort_order: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(color)
    .bind(sort_order)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn update_type_color(
    pool: &DbPool,
    id: &str,
    color: &str,
    now: i64,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("UPDATE transaction_types SET color = ?, updated_at = ? WHERE id = ?")
        .bind(color)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub(crate) async fn delete_type(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM transaction_types WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub(crate) async fn find_type_by_id_or_name(
    pool: &DbPool,
    query: &str,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name FROM transaction_types WHERE id = ? OR name = ? COLLATE NOCASE LIMIT 1",
    )
    .bind(query)
    .bind(query)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn get_next_category_sort_order(
    pool: &DbPool,
    type_id: &str,
) -> Result<i64, sqlx::Error> {
    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM categories WHERE type_id = ?")
            .bind(type_id)
            .fetch_one(pool)
            .await?;
    Ok(max_sort.0.unwrap_or(0) + 1)
}

pub(crate) async fn insert_category(
    pool: &DbPool,
    id: &str,
    type_id: &str,
    name: &str,
    sort_order: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(type_id)
    .bind(name)
    .bind(sort_order)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn update_category_name(
    pool: &DbPool,
    id: &str,
    name: &str,
    now: i64,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("UPDATE categories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub(crate) async fn delete_category(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub(crate) async fn check_category_exists(
    pool: &DbPool,
    category_id: &str,
) -> Result<bool, sqlx::Error> {
    let cat_exists: Option<(String,)> = sqlx::query_as("SELECT id FROM categories WHERE id = ?")
        .bind(category_id)
        .fetch_optional(pool)
        .await?;
    Ok(cat_exists.is_some())
}

pub(crate) async fn get_next_subcategory_sort_order(
    pool: &DbPool,
    category_id: &str,
) -> Result<i64, sqlx::Error> {
    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM subcategories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(pool)
            .await?;
    Ok(max_sort.0.unwrap_or(0) + 1)
}

pub(crate) async fn insert_subcategory(
    pool: &DbPool,
    id: &str,
    category_id: &str,
    name: &str,
    sort_order: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(category_id)
    .bind(name)
    .bind(sort_order)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn update_subcategory_name(
    pool: &DbPool,
    id: &str,
    name: &str,
    now: i64,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("UPDATE subcategories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub(crate) async fn delete_subcategory(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM subcategories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Seeds the default 4 types, 8 categories, and 14 subcategories.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), sqlx::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    tracing::info!("seeding default transaction types, categories, and subcategories");

    let mut tx = pool.begin().await?;

    // 4 Types
    let types = [
        ("type-income", "Income", "#10b981", 1),
        ("type-expense", "Expense", "#f43f5e", 2),
        ("type-transfer", "Transfer", "#71717a", 3),
        ("type-invest", "Invest", "#3b82f6", 4),
    ];

    for (id, name, color, sort) in types {
        sqlx::query(
            r#"
            INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(color)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 8 Categories: (id, type_id, name, sort)
    let categories = [
        ("cat-inc-salary", "type-income", "Salary", 1),
        ("cat-inc-freelance", "type-income", "Freelance", 2),
        ("cat-exp-housing", "type-expense", "Housing", 1),
        ("cat-exp-food", "type-expense", "Food", 2),
        ("cat-exp-transport", "type-expense", "Transport", 3),
        ("cat-exp-personal", "type-expense", "Personal", 4),
        ("cat-trf-internal", "type-transfer", "Internal", 1),
        ("cat-inv-equities", "type-invest", "Equities", 1),
    ];

    for (id, type_id, name, sort) in categories {
        sqlx::query(
            r#"
            INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(type_id)
        .bind(name)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 14 Subcategories: (id, category_id, name, sort)
    let subcategories = [
        // Salary
        ("sub-inc-primary", "cat-inc-salary", "Primary Employer", 1),
        ("sub-inc-bonus", "cat-inc-salary", "Bonus", 2),
        // Freelance
        ("sub-inc-consulting", "cat-inc-freelance", "Consulting", 1),
        ("sub-inc-retainers", "cat-inc-freelance", "Retainers", 2),
        // Housing
        ("sub-exp-rent", "cat-exp-housing", "Rent", 1),
        ("sub-exp-utilities", "cat-exp-housing", "Utilities", 2),
        // Food
        ("sub-exp-groceries", "cat-exp-food", "Groceries", 1),
        ("sub-exp-dining", "cat-exp-food", "Dining Out", 2),
        // Transport
        ("sub-exp-fuel", "cat-exp-transport", "Fuel", 1),
        ("sub-exp-transit", "cat-exp-transport", "Public Transit", 2),
        // Personal
        ("sub-exp-fitness", "cat-exp-personal", "Gym & Fitness", 1),
        // Internal
        (
            "sub-trf-checking",
            "cat-trf-internal",
            "Checking to Savings",
            1,
        ),
        ("sub-trf-emergency", "cat-trf-internal", "Emergency Fund", 2),
        // Equities
        ("sub-inv-etf", "cat-inv-equities", "Index ETFs", 1),
    ];

    for (id, cat_id, name, sort) in subcategories {
        sqlx::query(
            r#"
            INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(cat_id)
        .bind(name)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Atomically resets categories to default 4 types, 8 categories, and 14 subcategories.
pub(crate) async fn reset_default_categories(pool: &DbPool) -> Result<(), sqlx::Error> {
    tracing::info!("resetting categories to default configuration");
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM transaction_types")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    seed_default_categories(pool).await
}
