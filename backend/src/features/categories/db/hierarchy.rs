use crate::core::{AppError, DbPool, DbResultExt};

use super::super::models::{
    CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, SubcategoryItem,
    TransactionTypeItem,
};
use super::colors::{fetch_colors, seed_default_colors};
use super::util::now_epoch_secs;

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
    .await
    .db_context("CONFIG.CATEGORIES.FETCH_HIERARCHY.QUERY")?;

    let mut types: Vec<TransactionTypeItem> = Vec::new();
    let mut categories: Vec<CategoryItem> = Vec::new();

    for row in rows {
        if !types.iter().any(|t| t.id() == row.type_id()) {
            types.push(TransactionTypeItem::new(
                row.type_id(),
                row.type_name(),
                row.type_color(),
                row.type_color_id(),
            ));
        }

        if let (Some(cat_id), Some(cat_name)) = (row.category_id(), row.category_name()) {
            if let Some(cat) = categories.iter_mut().find(|c| c.id() == cat_id) {
                if let (Some(sub_id), Some(sub_name)) =
                    (row.subcategory_id(), row.subcategory_name())
                {
                    if !cat.subcategories().iter().any(|s| s.id() == sub_id) {
                        cat.subcategories_mut()
                            .push(SubcategoryItem::new(sub_id, sub_name));
                    }
                }
            } else {
                let mut cat = CategoryItem::new(cat_id, cat_name, row.type_name());
                if let (Some(sub_id), Some(sub_name)) =
                    (row.subcategory_id(), row.subcategory_name())
                {
                    cat.subcategories_mut()
                        .push(SubcategoryItem::new(sub_id, sub_name));
                }
                categories.push(cat);
            }
        }
    }

    let colors = fetch_colors(pool).await?;

    Ok(CategoryHierarchyResponse::new(types, categories, colors))
}

/// Seeds the default 4 types, 8 categories, and 14 subcategories if empty.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), AppError> {
    seed_default_colors(pool).await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_types")
        .fetch_one(pool)
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.COUNT")?;
    if count.0 > 0 {
        return Ok(());
    }

    let now = now_epoch_secs();
    tracing::info!("seeding default transaction types, categories, and subcategories");

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.BEGIN_TRANSACTION")?;

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
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.INSERT_TYPES")?;
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
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.INSERT_CATEGORIES")?;
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
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.INSERT_SUBCATEGORIES")?;
    }

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_CATEGORIES.COMMIT_TRANSACTION")?;
    Ok(())
}

/// Atomically clears and resets all categories and types to standard defaults.
pub(crate) async fn reset_defaults(pool: &DbPool) -> Result<CategoryHierarchyResponse, AppError> {
    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.BEGIN_TRANSACTION")?;
    sqlx::query("DELETE FROM subcategories")
        .execute(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_SUBCATEGORIES")?;
    sqlx::query("DELETE FROM categories")
        .execute(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_CATEGORIES")?;
    sqlx::query("DELETE FROM transaction_types")
        .execute(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_TRANSACTION_TYPES")?;
    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.COMMIT_TRANSACTION")?;

    seed_default_categories(pool).await?;
    fetch_hierarchy(pool).await
}
