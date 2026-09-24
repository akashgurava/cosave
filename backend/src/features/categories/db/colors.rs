use crate::core::{AppError, DbPool, DbResultExt};

use super::super::error::CategoryError;
use super::super::models::ColorItem;
use super::util::now_epoch_secs;

/// Retrieves all available palette colors from the database.
pub(crate) async fn fetch_colors(pool: &DbPool) -> Result<Vec<ColorItem>, AppError> {
    let colors =
        sqlx::query_as::<_, ColorItem>("SELECT id, name, hex FROM colors ORDER BY sort_order, id")
            .fetch_all(pool)
            .await
            .db_context("CONFIG.CATEGORIES.FETCH_COLORS.QUERY")?;
    Ok(colors)
}

/// Helper function to resolve color ID and hex from either integer ID or color string.
pub(crate) async fn resolve_color_id(
    pool: &DbPool,
    color_id: Option<i64>,
    color: Option<&str>,
) -> Result<(i64, String), AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.RESOLVE_COLOR";
    if let Some(cid) = color_id {
        let row: Option<(i64, String)> = sqlx::query_as("SELECT id, hex FROM colors WHERE id = ?")
            .bind(cid)
            .fetch_optional(pool)
            .await
            .db_context("CONFIG.CATEGORIES.RESOLVE_COLOR.QUERY_ID")?;
        if let Some((id, hex)) = row {
            return Ok((id, hex));
        }
        return Err(CategoryError::ColorNotFound {
            action: ACTION,
            id: cid,
        }
        .into());
    }

    if let Some(c) = color {
        let trimmed = c.trim();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyColor { action: ACTION }.into());
        }

        // Try parsing string as integer ID
        if let Ok(parsed_id) = trimmed.parse::<i64>() {
            let row: Option<(i64, String)> =
                sqlx::query_as("SELECT id, hex FROM colors WHERE id = ?")
                    .bind(parsed_id)
                    .fetch_optional(pool)
                    .await
                    .db_context("CONFIG.CATEGORIES.RESOLVE_COLOR.QUERY_PARSED_ID")?;
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
        .await
        .db_context("CONFIG.CATEGORIES.RESOLVE_COLOR.QUERY_NAME_OR_HEX")?;

        if let Some((id, hex)) = row {
            return Ok((id, hex));
        }

        return Err(CategoryError::UnrecognizedColor {
            action: ACTION,
            color: trimmed.to_string(),
        }
        .into());
    }

    Err(CategoryError::MissingColor { action: ACTION }.into())
}

/// Seeds the default 12 palette colors if table is empty.
pub(crate) async fn seed_default_colors(pool: &DbPool) -> Result<(), AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM colors")
        .fetch_one(pool)
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_COLORS.COUNT")?;
    if count.0 > 0 {
        return Ok(());
    }

    let now = now_epoch_secs();
    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_COLORS.BEGIN_TRANSACTION")?;

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
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_COLORS.INSERT")?;
    }

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.SEED_DEFAULT_COLORS.COMMIT_TRANSACTION")?;
    Ok(())
}
