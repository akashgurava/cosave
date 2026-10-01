//! Foundational runtime infrastructure and HTTP contracts for the application backend.
//!
//! This module provides the core building blocks required to configure, boot, and run
//! the server:
//!
//! - **Configuration and Bootstrapping**: Loads and validates environment variables and optional
//!   command-line flags into an immutable [`AppConfig`], resolving data directories, bind addresses,
//!   and initializing the SQLite connection pool with WAL mode enabled.
//! - **Shared Server State**: Wraps database connections and system resources in an [`AppState`]
//!   container that is shared across Axum routes and background tasks.
//! - **Consistent API Contracts**: Defines the unified [`ApiResponse`] envelope and [`AppError`]
//!   hierarchy used by all feature modules to ensure predictable JSON responses and human-actionable
//!   error messages for frontend clients.
//! - **System Health Check**: Houses the `/api/v1/health` route used by Docker healthchecks,
//!   local reverse proxies, and startup scripts to verify the backend is up and responsive.
//! - **Application Metadata Storage**: Manages the internal `app_meta` key-value table, providing
//!   an idempotent mechanism for tracking schema versions, recording one-time seed migrations,
//!   and persisting global runtime metadata.

#[cfg(feature = "cli")]
mod cli;
mod config;
mod db;
mod error;
mod health;
mod meta;
mod response;
mod state;
mod time;

pub(crate) use db::{create_db_object, db_err, DbResultExt};
pub(crate) use health::router as health_router;
pub(crate) use meta::{get_meta, init_core_schema, set_meta_tx};
pub(crate) use response::api_not_found;
pub(crate) use time::now_epoch_secs;

#[cfg(feature = "cli")]
pub use cli::Cli;
pub use config::{AppConfig, AppEnv};
pub use db::{init_db, DbPool};
pub use error::AppError;
pub use response::{api_only_root_fallback, ApiResponse, Code, ErrorPayload, Status};
pub use state::AppState;
