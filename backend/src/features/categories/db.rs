use crate::{
    core::{create_db_object, db::DbPool, error::AppError, NewAppError},
    features::categories::{
        models::{
            CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, ColorItem,
            CreateCategoryRequest, CreateSubcategoryRequest, CreateTypeRequest, SubcategoryItem,
            TransactionTypeItem, UpdateNameRequest, UpdateTypeColorRequest,
        },
        TASK_NAME,
    },
};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(32);
    for b in bytes {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.is_unique_violation() {
            return true;
        }
        let msg = db_err.message().to_lowercase();
        if msg.contains("unique") || msg.contains("primary key") {
            return true;
        }
    }
    false
}

/// Creates category hierarchy domain tables, indices, and views.
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), NewAppError> {
    create_db_object(
        TASK_NAME.to_string(),
        "colors".to_string(),
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS colors (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT UNIQUE NOT NULL,
            hex TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        TASK_NAME.to_string(),
        "transaction_types".to_string(),
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS transaction_types (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            color_id INTEGER NOT NULL REFERENCES colors(id),
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        TASK_NAME.to_string(),
        "categories".to_string(),
        pool,
        r#"
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
        "#,
    )
    .await?;

    create_db_object(
        TASK_NAME.to_string(),
        "subcategories".to_string(),
        pool,
        r#"
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
        "#,
    )
    .await?;

    create_db_object(
        TASK_NAME.to_string(),
        "v_category_hierarchy".to_string(),
        pool,
        r#"
        CREATE VIEW IF NOT EXISTS v_category_hierarchy AS
        SELECT
            t.id AS type_id,
            t.name AS type_name,
            t.color_id AS type_color_id,
            col.hex AS type_color,
            t.sort_order AS type_sort_order,
            c.id AS category_id,
            c.name AS category_name,
            c.sort_order AS category_sort_order,
            s.id AS subcategory_id,
            s.name AS subcategory_name,
            s.sort_order AS subcategory_sort_order
        FROM transaction_types t
        JOIN colors col ON col.id = t.color_id
        LEFT JOIN categories c ON c.type_id = t.id
        LEFT JOIN subcategories s ON s.category_id = c.id
        ORDER BY t.sort_order, t.name, c.sort_order, c.name, s.sort_order, s.name;
        "#,
    )
    .await?;

    Ok(())
}

/// Retrieves all available palette colors from the database.
pub(crate) async fn fetch_colors(pool: &DbPool) -> Result<Vec<ColorItem>, AppError> {
    let colors =
        sqlx::query_as::<_, ColorItem>("SELECT id, name, hex FROM colors ORDER BY sort_order, id")
            .fetch_all(pool)
            .await?;
    Ok(colors)
}

/// Helper function to resolve color ID and hex from either integer ID or color string.
pub(crate) async fn resolve_color_id(
    pool: &DbPool,
    color_id: Option<i64>,
    color: Option<&str>,
) -> Result<(i64, String), AppError> {
    if let Some(cid) = color_id {
        let row: Option<(i64, String)> = sqlx::query_as("SELECT id, hex FROM colors WHERE id = ?")
            .bind(cid)
            .fetch_optional(pool)
            .await?;
        if let Some((id, hex)) = row {
            return Ok((id, hex));
        }
        return Err(AppError::BadRequest(format!(
            "Color with id {cid} not found"
        )));
    }

    if let Some(c) = color {
        let trimmed = c.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("Color cannot be empty".to_string()));
        }

        // Try parsing string as integer ID
        if let Ok(parsed_id) = trimmed.parse::<i64>() {
            let row: Option<(i64, String)> =
                sqlx::query_as("SELECT id, hex FROM colors WHERE id = ?")
                    .bind(parsed_id)
                    .fetch_optional(pool)
                    .await?;
            if let Some((id, hex)) = row {
                return Ok((id, hex));
            }
        }

        // Try matching by hex or by name (case-insensitive)
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, hex FROM colors WHERE LOWER(hex) = LOWER(?) OR LOWER(name) = LOWER(?) LIMIT 1",
        )
        .bind(trimmed)
        .bind(trimmed)
        .fetch_optional(pool)
        .await?;

        if let Some((id, hex)) = row {
            return Ok((id, hex));
        }

        return Err(AppError::BadRequest(format!(
            "Color '{trimmed}' is not recognized in the colors table"
        )));
    }

    Err(AppError::BadRequest("Color must be specified".to_string()))
}

/// Retrieves the complete category hierarchy from the database view.
pub(crate) async fn fetch_hierarchy(pool: &DbPool) -> Result<CategoryHierarchyResponse, AppError> {
    let rows: Vec<CategoryHierarchyRow> = sqlx::query_as(
        r#"
        SELECT
            type_id,
            type_name,
            type_color,
            type_color_id,
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
                color_id: row.type_color_id,
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

    let colors = fetch_colors(pool).await?;

    Ok(CategoryHierarchyResponse {
        types,
        categories,
        colors,
    })
}

/// Atomically creates a new transaction type.
pub(crate) async fn create_type(
    pool: &DbPool,
    payload: CreateTypeRequest,
) -> Result<TransactionTypeItem, AppError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    let (color_id, color_hex) =
        resolve_color_id(pool, payload.color_id, payload.color.as_deref()).await?;

    let mut tx = pool.begin().await?;

    let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
        .fetch_one(&mut *tx)
        .await?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("type-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(color_id)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit().await?;
            Ok(TransactionTypeItem {
                id,
                name,
                color: color_hex,
                color_id,
            })
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
    let (color_id, _) = resolve_color_id(pool, payload.color_id, payload.color.as_deref()).await?;

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE transaction_types SET color_id = ?, updated_at = ? WHERE id = ?")
        .bind(color_id)
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
                    "Category '{name}' already exists under type '{canonical_type_name}'"
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

/// Seeds the default 12 palette colors if table is empty.
pub(crate) async fn seed_default_colors(pool: &DbPool) -> Result<(), AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM colors")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }

    let now = now_epoch_secs();
    let mut tx = pool.begin().await?;

    let default_colors = [
        (1, "Emerald", "#10b981", 1),
        (2, "Rose", "#f43f5e", 2),
        (3, "Grey", "#71717a", 3),
        (4, "Blue", "#3b82f6", 4),
        (5, "Amber", "#f59e0b", 5),
        (6, "Violet", "#8b5cf6", 6),
        (7, "Cyan", "#06b6d4", 7),
        (8, "Orange", "#f97316", 8),
        (9, "Pink", "#ec4899", 9),
        (10, "Teal", "#14b8a6", 10),
        (11, "Indigo", "#6366f1", 11),
        (12, "Lime", "#84cc16", 12),
    ];

    for (id, name, hex, sort_order) in default_colors {
        sqlx::query(
            "INSERT INTO colors (id, name, hex, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(hex)
        .bind(sort_order)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Seeds the default 4 types, 8 categories, and 14 subcategories if empty.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), AppError> {
    seed_default_colors(pool).await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_types")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }

    let now = now_epoch_secs();
    tracing::info!("seeding default transaction types, categories, and subcategories");

    let mut tx = pool.begin().await?;

    // 4 Types with color_id
    // 1: Emerald (#10b981), 2: Rose (#f43f5e), 4: Blue (#3b82f6), 6: Violet (#8b5cf6)
    let types = [
        ("type-income", "Income", 1, 1),
        ("type-expense", "Expense", 2, 2),
        ("type-transfer", "Transfer", 4, 3),
        ("type-invest", "Invest", 6, 4),
    ];

    for (id, name, color_id, sort_order) in types {
        sqlx::query(
            "INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(color_id)
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
