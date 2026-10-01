//! Domain feature modules, unified routing, and schema orchestration.
//!
//! # Architecture & Vertical Feature Slices
//! The `features` module organizes business logic into cohesive, feature-first domain slices.
//! Each feature encapsulates its domain models, persistence queries, error taxonomy, and HTTP
//! route handlers behind an internal module facade.
//!
//! # Domain Features
//! - **Authentication (`auth`)**: User registration, Argon2id password hashing, session management,
//!   role-based access control (Admin vs Member), and authentication request extractors.
//! - **Categories (`categories`)**: Transaction classification hierarchy, encompassing transaction
//!   types, categories, subcategories, color palettes, and hierarchical rollup views.
//!
//! # Central Orchestration
//! This root `features` module acts as the unified orchestrator:
//! - [`router`]: Assembles the unified REST API router, mounting feature routes alongside core health
//!   probes and standardized 404 fallbacks.
//! - [`init_schemas`]: Bootstraps SQLite tables, indexes, and views across all domain features.
//! - [`init_features`]: Seeds initial declarative defaults (such as default category hierarchies)
//!   on first run.

use axum::Router;

use crate::core::{api_not_found, health_router, AppError, AppState, DbPool, DbResultExt};

pub(crate) mod auth;
pub(crate) mod categories;

pub use auth::AuthError;
pub use categories::CategoryError;

/// Assembles the unified REST API router with standard 404 envelope fallback.
///
/// Merges the core health router, mounts `/auth` and `/categories` feature routers,
/// and attaches the fallback handler for unmatched API routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health_router())
        .nest("/auth", auth::router())
        .nest("/categories", categories::router())
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
