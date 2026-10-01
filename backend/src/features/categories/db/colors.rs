use crate::core::{AppError, DbPool, DbResultExt};

use super::super::error::CategoryError;
use super::super::models::ColorItem;

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
            action: "CONFIG.CATEGORIES.RESOLVE_COLOR.COLOR_ID_NOT_FOUND",
            id: cid,
        }
        .into());
    }

    if let Some(c) = color {
        let trimmed = c.trim();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyColor {
                action: "CONFIG.CATEGORIES.RESOLVE_COLOR.EMPTY_COLOR",
            }
            .into());
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
            action: "CONFIG.CATEGORIES.RESOLVE_COLOR.UNRECOGNIZED_COLOR",
            color: trimmed.to_string(),
        }
        .into());
    }

    Err(CategoryError::MissingColor {
        action: "CONFIG.CATEGORIES.RESOLVE_COLOR.MISSING_COLOR",
    }
    .into())
}
