use std::time::{SystemTime, UNIX_EPOCH};

use super::models::{
    CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, CreateCategoryRequest,
    CreateSubcategoryRequest, CreateTypeRequest, SubcategoryItem, TransactionTypeItem,
    UpdateNameRequest, UpdateTypeColorRequest,
};
use crate::{
    core::{db::DbPool, error::AppError},
    features::auth::generate_token,
};

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
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

/// Creates category hierarchy domain tables, indices, and views.
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transaction_types (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            color TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY NOT NULL,
            type_id TEXT NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, name)
        );

        CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);

        CREATE TABLE IF NOT EXISTS subcategories (
            id TEXT PRIMARY KEY NOT NULL,
            category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(category_id, name)
        );

        CREATE INDEX IF NOT EXISTS idx_subcategories_category_id ON subcategories(category_id);

        CREATE VIEW IF NOT EXISTS v_category_hierarchy AS
        SELECT
            t.id AS type_id,
            t.name AS type_name,
            t.color AS type_color,
            t.sort_order AS type_sort_order,
            c.id AS category_id,
            c.name AS category_name,
            c.sort_order AS category_sort_order,
            s.id AS subcategory_id,
            s.name AS subcategory_name,
            s.sort_order AS subcategory_sort_order
        FROM transaction_types t
        LEFT JOIN categories c ON c.type_id = t.id
        LEFT JOIN subcategories s ON s.category_id = c.id
        ORDER BY t.sort_order, t.name, c.sort_order, c.name, s.sort_order, s.name;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves the complete category hierarchy from the database view.
pub(crate) async fn fetch_hierarchy(pool: &DbPool) -> Result<CategoryHierarchyResponse, AppError> {
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

/// Atomically creates a new transaction type.
pub(crate) async fn create_type(
    pool: &DbPool,
    payload: CreateTypeRequest,
) -> Result<TransactionTypeItem, AppError> {
    let name = payload.name.trim().to_string();
    let color = payload.color.trim().to_string();

    if name.is_empty() || color.is_empty() {
        return Err(AppError::BadRequest(
            "Name and color cannot be empty".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
        .fetch_one(&mut *tx)
        .await?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("type-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&color)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit().await?;
            Ok(TransactionTypeItem { id, name, color })
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::Conflict(format!(
                    "Transaction type '{name}' already exists"
                )))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

/// Updates display color of a transaction type.
pub(crate) async fn update_type_color(
    pool: &DbPool,
    id: &str,
    payload: UpdateTypeColorRequest,
) -> Result<(), AppError> {
    let color = payload.color.trim().to_string();
    if color.is_empty() {
        return Err(AppError::BadRequest("Color cannot be empty".to_string()));
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE transaction_types SET color = ?, updated_at = ? WHERE id = ?")
        .bind(&color)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        Err(AppError::NotFound(format!(
            "Transaction type '{id}' not found"
        )))
    } else {
        Ok(())
    }
}

/// Deletes a transaction type and cascades to associated categories.
pub(crate) async fn delete_type(pool: &DbPool, id: &str) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM transaction_types WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        Err(AppError::NotFound(format!(
            "Transaction type '{id}' not found"
        )))
    } else {
        Ok(())
    }
}

/// Atomically creates a new category under a transaction type.
pub(crate) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> {
    let type_name_or_id = payload.type_name.trim();
    let name = payload.name.trim().to_string();

    if type_name_or_id.is_empty() || name.is_empty() {
        return Err(AppError::BadRequest(
            "Type and category name cannot be empty".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    let parent_type: Option<(String, String)> = sqlx::query_as(
        "SELECT id, name FROM transaction_types WHERE id = ? OR name = ? COLLATE NOCASE LIMIT 1",
    )
    .bind(type_name_or_id)
    .bind(type_name_or_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (type_id, canonical_type_name) = match parent_type {
        Some((tid, tname)) => (tid, tname),
        None => {
            return Err(AppError::NotFound(format!(
                "Transaction type '{type_name_or_id}' not found"
            )))
        }
    };

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM categories WHERE type_id = ?")
            .bind(&type_id)
            .fetch_one(&mut *tx)
            .await?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("cat-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&type_id)
    .bind(&name)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit().await?;
            Ok(CategoryItem {
                id,
                name,
                type_name: canonical_type_name,
                subcategories: Vec::new(),
            })
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::Conflict(format!(
                    "Category '{name}' already exists under type"
                )))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

/// Updates the name of an existing category.
pub(crate) async fn update_category_name(
    pool: &DbPool,
    id: &str,
    payload: UpdateNameRequest,
) -> Result<(), AppError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Category name cannot be empty".to_string(),
        ));
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE categories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(AppError::NotFound(format!("Category '{id}' not found")))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::Conflict(format!(
                    "Category name '{name}' already exists under this type"
                )))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

/// Deletes a category and cascades to its subcategories.
pub(crate) async fn delete_category(pool: &DbPool, id: &str) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        Err(AppError::NotFound(format!("Category '{id}' not found")))
    } else {
        Ok(())
    }
}

/// Atomically creates a new subcategory under an existing category.
pub(crate) async fn create_subcategory(
    pool: &DbPool,
    payload: CreateSubcategoryRequest,
) -> Result<SubcategoryItem, AppError> {
    let category_id = payload.category_id.trim();
    let name = payload.name.trim().to_string();

    if category_id.is_empty() || name.is_empty() {
        return Err(AppError::BadRequest(
            "Category ID and subcategory name cannot be empty".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    let cat_exists: Option<(String,)> = sqlx::query_as("SELECT id FROM categories WHERE id = ?")
        .bind(category_id)
        .fetch_optional(&mut *tx)
        .await?;

    if cat_exists.is_none() {
        return Err(AppError::NotFound(format!(
            "Category '{category_id}' not found"
        )));
    }

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM subcategories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(&mut *tx)
            .await?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("sub-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(category_id)
    .bind(&name)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit().await?;
            Ok(SubcategoryItem { id, name })
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::Conflict(format!(
                    "Subcategory '{name}' already exists under category"
                )))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

/// Updates the name of an existing subcategory.
pub(crate) async fn update_subcategory_name(
    pool: &DbPool,
    id: &str,
    payload: UpdateNameRequest,
) -> Result<(), AppError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Subcategory name cannot be empty".to_string(),
        ));
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE subcategories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(AppError::NotFound(format!("Subcategory '{id}' not found")))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(AppError::Conflict(format!(
                    "Subcategory '{name}' already exists under this category"
                )))
            } else {
                Err(AppError::from(err))
            }
        }
    }
}

/// Deletes a subcategory.
pub(crate) async fn delete_subcategory(pool: &DbPool, id: &str) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM subcategories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        Err(AppError::NotFound(format!("Subcategory '{id}' not found")))
    } else {
        Ok(())
    }
}

/// Seeds the default 4 types, 8 categories, and 14 subcategories if empty.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_types")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }

    let now = now_epoch_secs();
    tracing::info!("seeding default transaction types, categories, and subcategories");

    let mut tx = pool.begin().await?;

    // 4 Types
    let types = [
        ("type-income", "Income", "#10b981", 1),
        ("type-expense", "Expense", "#f43f5e", 2),
        ("type-transfer", "Transfer", "#3b82f6", 3),
        ("type-invest", "Invest", "#8b5cf6", 4),
    ];

    for (id, name, color, sort_order) in types {
        sqlx::query(
            "INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(color)
        .bind(sort_order)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 8 Categories
    let categories = [
        ("cat-inc-salary", "type-income", "Salary & Wages", 1),
        ("cat-inc-invest", "type-income", "Investment Income", 2),
        ("cat-exp-housing", "type-expense", "Housing", 1),
        ("cat-exp-food", "type-expense", "Food & Dining", 2),
        ("cat-exp-transport", "type-expense", "Transportation", 3),
        ("cat-trf-internal", "type-transfer", "Account Transfer", 1),
        ("cat-inv-retirement", "type-invest", "Retirement", 1),
        ("cat-inv-stocks", "type-invest", "Brokerage", 2),
    ];

    for (id, type_id, name, sort_order) in categories {
        sqlx::query(
            "INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(type_id)
        .bind(name)
        .bind(sort_order)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 14 Subcategories
    let subcategories = [
        ("sub-sal-primary", "cat-inc-salary", "Primary Employer", 1),
        ("sub-sal-bonus", "cat-inc-salary", "Bonus & Commission", 2),
        ("sub-inv-div", "cat-inc-invest", "Dividends", 1),
        ("sub-house-rent", "cat-exp-housing", "Rent & Mortgage", 1),
        ("sub-house-util", "cat-exp-housing", "Utilities", 2),
        ("sub-food-groc", "cat-exp-food", "Groceries", 1),
        ("sub-food-rest", "cat-exp-food", "Restaurants", 2),
        ("sub-food-cafe", "cat-exp-food", "Coffee & Cafes", 3),
        ("sub-tran-fuel", "cat-exp-transport", "Fuel & Gas", 1),
        ("sub-tran-pub", "cat-exp-transport", "Public Transit", 2),
        ("sub-trf-save", "cat-trf-internal", "Savings Transfer", 1),
        (
            "sub-inv-401k",
            "cat-inv-retirement",
            "401(k) Contribution",
            1,
        ),
        ("sub-inv-ira", "cat-inv-retirement", "Roth IRA", 2),
        ("sub-inv-etf", "cat-inv-stocks", "Index Funds & ETFs", 1),
    ];

    for (id, category_id, name, sort_order) in subcategories {
        sqlx::query(
            "INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(category_id)
        .bind(name)
        .bind(sort_order)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Atomically clears and resets all categories and types to standard defaults.
pub(crate) async fn reset_defaults(pool: &DbPool) -> Result<CategoryHierarchyResponse, AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM subcategories")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM categories")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM transaction_types")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    seed_default_categories(pool).await?;
    fetch_hierarchy(pool).await
}
