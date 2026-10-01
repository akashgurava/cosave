//! Aggregated domain features, API routing, and database lifecycle for the application.
//!
//! This module coordinates application feature slices at the system boundary:
//!
//! - **Unified API Routing**: The [`router`] function mounts all feature endpoints (`/auth`,
//!   `/config`), merges the health check, and attaches a standard 404 fallback.
//! - **Atomic Schema Migrations**: The [`init_schemas`] function executes all feature DDL migrations
//!   inside a single database transaction on startup so the app is always up to date.
//! - **Default Data Seeding**: The [`init_features`] function populates new instances with sensible
//!   defaults (such as standard budgeting categories and palette colors) on first launch.
//!
//! # Implemented Features
//!
//! The application organizes its business capabilities into modular, self-contained feature slices:
//! - **Auth**: Handles user onboarding, Argon2id credential verification, and dual-channel session authentication, exposing [`AuthError`].
//! - **Categories**: Organizes cashflows into a 3-tier hierarchy (types, categories, and subcategories) with customizable palette colors for budgeting and visualization, exposing [`CategoryError`].

use axum::Router;

use crate::core::{api_not_found, health_router, AppError, AppState, DbPool, DbResultExt};

pub(crate) mod auth;
pub(crate) mod categories;

pub use auth::AuthError;
pub use categories::CategoryError;

/// Assembles the unified REST API router with standard 404 envelope fallback.
///
/// Merges the core health router, mounts `/auth` and `/config` feature routers,
/// and attaches the fallback handler for unmatched API routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health_router())
        .nest("/auth", auth::router())
        .nest("/config", categories::router())
        .fallback(api_not_found)
}

/// Runs idempotent table, index, and view creation migrations across all domain features inside an atomic transaction.
///
/// Invoked once during application startup immediately after connection pool initialization.
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL migration fails, or
/// [`AppError::ShouldNotBeHappening`] if the transaction cannot be opened or committed.
pub async fn init_schemas(pool: &DbPool) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .db_context("FEATURES.INIT_SCHEMAS.TX_BEGIN")?;
    auth::init_auth_schema(&mut tx).await?;
    categories::init_category_schema(&mut tx).await?;
    tx.commit()
        .await
        .db_context("FEATURES.INIT_SCHEMAS.TX_COMMIT")?;
    Ok(())
}

/// Initializes feature domain modules and seeds initial defaults if database tables are empty.
///
/// Invoked once during application startup after all schemas have been initialized.
///
/// # Errors
/// Returns [`AppError`] if initial seeding fails.
pub async fn init_features(pool: &DbPool) -> Result<(), AppError> {
    categories::seed_default_categories(pool).await?;
    Ok(())
}
