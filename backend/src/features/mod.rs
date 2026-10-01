use axum::Router;

use crate::core::{health_router, AppError, AppState, DbPool};

pub(crate) mod auth;
pub(crate) mod categories;

pub use auth::AuthError;
pub use categories::CategoryError;

/// Assembles the unified REST API router.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health_router())
        .nest("/auth", auth::router())
        .nest("/categories", categories::router())
}

/// Runs table and view creation migrations across all domain features.
pub async fn init_schemas(pool: &DbPool) -> Result<(), AppError> {
    auth::init_auth_schema(pool).await?;
    categories::init_category_schema(pool).await?;
    Ok(())
}

/// Initializes feature domain modules and seeds initial defaults if empty.
pub async fn init_features(pool: &DbPool) -> Result<(), AppError> {
    categories::seed_default_categories(pool).await?;
    Ok(())
}
