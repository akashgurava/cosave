//! Palette color persistence.
//!
//! Provides database routines for retrieving curated theme colors from the predefined palette.

use crate::core::{AppError, DbPool, DbResultExt};
use crate::features::categories::models::ColorItem;

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
