//! Palette color persistence and dynamic identifier resolution.
//!
//! Provides database routines for retrieving curated theme colors and resolving color inputs
//! from numeric identifiers, hexadecimal strings, or color names. Flexible lookup ensures
//! that user-selected colors can be assigned to transaction types accurately while maintaining
//! referential integrity against the predefined palette.

use crate::core::{AppError, DbPool, DbResultExt};
use crate::features::categories::models::ColorItem;
use crate::features::categories::CategoryError;

/// Retrieves all available palette colors from the database ordered by sort order and ID.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
///
/// # Returns
/// - `Ok(Vec<ColorItem>)` containing all active color options.
///
/// # Errors
/// Returns [`AppError::ShouldNotBeHappening`] with action `CONFIG.CATEGORIES.FETCH_COLORS.QUERY` if the query fails.
pub(in crate::features::categories) async fn fetch_colors(
    pool: &DbPool,
) -> Result<Vec<ColorItem>, AppError> {
    let colors =
        sqlx::query_as::<_, ColorItem>("SELECT id, name, hex FROM colors ORDER BY sort_order, id")
            .fetch_all(pool)
            .await
            .db_context("CONFIG.CATEGORIES.FETCH_COLORS.QUERY")?;
    Ok(colors)
}

/// Resolves a color database ID and hex string from an explicit numeric ID or fallback color string.
///
/// Resolution operates in precedence order:
/// 1. If `color_id` is provided, looks up `colors` by primary key `id`.
/// 2. If `color` string is provided:
///    a. Checks if non-empty; rejects whitespace-only strings.
///    b. Attempts parsing as an integer primary key.
///    c. Performs case-insensitive matching on `hex` or `name` (`LOWER(hex) = LOWER(?) OR LOWER(name) = LOWER(?)`).
/// 3. If neither is provided, rejects as missing color.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `color_id`: Optional numeric primary key ID.
/// - `color`: Optional candidate color string (numeric ID string, hex code, or color name).
///
/// # Returns
/// - `Ok((i64, String))` containing the verified `(color_id, hex_code)`.
///
/// # Errors
/// - Returns [`CategoryError::ColorNotFound`] if `color_id` does not exist in `colors`.
/// - Returns [`CategoryError::EmptyColor`] if `color` is empty or whitespace-only.
/// - Returns [`CategoryError::UnrecognizedColor`] if `color` cannot be resolved to any palette entry.
/// - Returns [`CategoryError::MissingColor`] if both arguments are [`None`].
pub(super) async fn resolve_color_id(
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
