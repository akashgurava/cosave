use axum::Router;

use crate::core::{AppError, AppState, DbPool};

pub(crate) mod auth;
pub(crate) mod categories;
pub(crate) mod health;

/// Assembles the unified REST API router.
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::router())
        .nest("/categories", categories::router())
}

/// Runs table and view creation migrations across all domain features.
pub(crate) async fn init_schemas(pool: &DbPool) -> Result<(), AppError> {
    auth::init_schema(pool).await?;
    categories::init_schema(pool).await?;
    Ok(())
}

/// Initializes feature domain modules and seeds initial defaults if empty.
pub(crate) async fn init_features(pool: &DbPool) -> Result<(), AppError> {
    categories::seed_default_categories(pool).await?;
    Ok(())
}
