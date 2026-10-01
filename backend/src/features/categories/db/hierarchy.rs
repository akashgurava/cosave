//! Category hierarchy aggregation, default seeding, and taxonomy reset workflows.
//!
//! Assembles the complete 3-tier category tree by querying the denormalized `v_category_hierarchy`
//! database view and structuring rows into nested objects for the API. This module also manages
//! initial idempotent seeding from default taxonomy definitions and provides an atomic reset
//! routine to restore default categories if needed.

use sqlx::Executor;

use crate::core::{get_meta, now_epoch_secs, set_meta_tx, AppError, DbPool, DbResultExt};
use crate::features::categories::models::{
    CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, SubcategoryItem,
    TransactionTypeItem,
};

use super::colors::fetch_colors;

const META_KEY_SEED_HIERARCHY: &str = "seed.hierarchy.v1";

/// Retrieves the complete category hierarchy from the database view.
///
/// Queries `v_category_hierarchy` ordered by type, category, and subcategory sort orders,
/// assembling rows into a structured hierarchical response containing all transaction types,
/// categories with nested subcategories, and the full palette of selectable colors.
///
/// # Ingress
/// - `pool`: Reference to the active [`DbPool`].
///
/// # Returns
/// - `Ok(CategoryHierarchyResponse)`: Complete nested taxonomy and palette colors.
/// - `Err(AppError)`: Database error if querying the view or colors fails.
pub(in crate::features::categories) async fn fetch_hierarchy(
    pool: &DbPool,
) -> Result<CategoryHierarchyResponse, AppError> {
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

const DEFAULT_HIERARCHY_JSON: &str = include_str!("../../../../resources/default_hierarchy.json");

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultHierarchy {
    colors: Vec<DefaultColor>,
    types: Vec<DefaultType>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultColor {
    id: i64,
    name: String,
    hex: String,
    sort_order: i64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultType {
    name: String,
    color_id: i64,
    sort_order: i64,
    categories: Vec<DefaultCategory>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultCategory {
    name: String,
    sort_order: i64,
    subcategories: Vec<String>,
}

async fn seed_hierarchy_from_json(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<(), AppError> {
    let hierarchy: DefaultHierarchy =
        serde_json::from_str(DEFAULT_HIERARCHY_JSON).map_err(|e| {
            AppError::ShouldNotBeHappening {
                action: "CONFIG.CATEGORIES.SEED_DEFAULTS.PARSE_JSON",
                reason: format!("Failed to parse default hierarchy JSON: {e}"),
            }
        })?;

    for color in hierarchy.colors {
        tx.execute(
            sqlx::query(
                "INSERT INTO colors (id, name, hex, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(color.id)
            .bind(&color.name)
            .bind(&color.hex)
            .bind(color.sort_order)
            .bind(now)
            .bind(now),
        )
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.INSERT_COLORS")?;
    }

    for t in hierarchy.types {
        let type_res = tx.execute(
            sqlx::query(
                "INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&t.name)
            .bind(t.color_id)
            .bind(t.sort_order)
            .bind(now)
            .bind(now),
        )
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.INSERT_TYPES")?;

        let type_id = type_res.last_insert_rowid();

        for c in t.categories {
            let cat_res = tx.execute(
                sqlx::query(
                    "INSERT INTO categories (type_id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(type_id)
                .bind(&c.name)
                .bind(c.sort_order)
                .bind(now)
                .bind(now),
            )
            .await
            .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.INSERT_CATEGORIES")?;

            let cat_id = cat_res.last_insert_rowid();

            for (idx, sub_name) in c.subcategories.into_iter().enumerate() {
                let sub_sort = (idx as i64) + 1;
                tx.execute(
                    sqlx::query(
                        "INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(cat_id)
                    .bind(&sub_name)
                    .bind(sub_sort)
                    .bind(now)
                    .bind(now),
                )
                .await
                .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.INSERT_SUBCATEGORIES")?;
            }
        }
    }

    set_meta_tx(
        "CONFIG.CATEGORIES.SEED_DEFAULTS.SET_META",
        tx,
        META_KEY_SEED_HIERARCHY,
        "1",
    )
    .await?;

    Ok(())
}

/// Seeds the default colors, types, categories, and subcategories from JSON template if not already seeded.
///
/// Checks application metadata for the `seed.hierarchy.v1` key to ensure idempotency. If unseeded,
/// initiates an explicit transaction, inserts all default colors, types, categories, and subcategories,
/// records the migration in `app_meta`, and commits atomically.
///
/// # Ingress
/// - `pool`: Reference to the active [`DbPool`].
///
/// # Returns
/// - `Ok(())`: Defaults already exist or were successfully inserted.
/// - `Err(AppError)`: Database error if transaction execution fails.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), AppError> {
    if get_meta(
        "CONFIG.CATEGORIES.SEED_DEFAULTS.CHECK_META",
        pool,
        META_KEY_SEED_HIERARCHY,
    )
    .await?
    .is_some()
    {
        return Ok(());
    }

    let now = now_epoch_secs();
    tracing::info!(
        "CONFIG.CATEGORIES.SEED_DEFAULTS.START. Seeding default colors, transaction types, categories, and subcategories from template"
    );

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.BEGIN_TRANSACTION")?;

    seed_hierarchy_from_json(&mut tx, now).await?;

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULTS.COMMIT_TRANSACTION")?;
    Ok(())
}

/// Atomically clears and resets all categories, types, and colors to standard defaults.
///
/// Executes an atomic transaction that removes all existing subcategories, categories,
/// transaction types, and colors, then re-seeds the default taxonomy from the embedded JSON template.
/// Following commit, retrieves and returns the newly refreshed hierarchy.
///
/// # Ingress
/// - `pool`: Reference to the active [`DbPool`].
///
/// # Returns
/// - `Ok(CategoryHierarchyResponse)`: Complete fresh hierarchy populated with default items.
/// - `Err(AppError)`: Database error if deletion, re-seeding, or hierarchy retrieval fails.
pub(in crate::features::categories) async fn reset_defaults(
    pool: &DbPool,
) -> Result<CategoryHierarchyResponse, AppError> {
    let now = now_epoch_secs();
    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.BEGIN_TRANSACTION")?;

    tx.execute("DELETE FROM subcategories")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_SUBCATEGORIES")?;
    tx.execute("DELETE FROM categories")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_CATEGORIES")?;
    tx.execute("DELETE FROM transaction_types")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_TRANSACTION_TYPES")?;
    tx.execute("DELETE FROM colors")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_COLORS")?;

    seed_hierarchy_from_json(&mut tx, now).await?;

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.COMMIT_TRANSACTION")?;

    fetch_hierarchy(pool).await
}
