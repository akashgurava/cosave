//! Core foundational infrastructure, runtime state, and framework contracts.
//!
//! # Architecture & Subsystem Facade
//! The `core` module encapsulates domain-agnostic infrastructure, networking, database lifecycle,
//! application state, and cross-cutting HTTP contracts for CoSave.
//!
//! All internal submodules are strictly private to this folder. External callers across feature boundaries
//! and crate entry points import exclusively through this root `core` facade:
//! - Feature modules import via `crate::core::{...}`.
//! - External crate consumers import via `cosave::{...}` (re-exported by `lib.rs`).
//!
//! # Core Subsystems & Responsibilities
//! - **CLI (`cli`)**: Command-line interface definitions and parsing (compiled when the `cli` feature is enabled).
//! - **Configuration (`config`)**: Immutable startup configuration Single Source of Truth ([`AppConfig`], [`AppEnv`]),
//!   unifying CLI flags and authoritative environment variables.
//! - **Database (`db`)**: SQLite connection pool management ([`DbPool`]), WAL pragma configuration,
//!   path directory safety, isolated DDL execution, and database error mapping.
//! - **Errors (`error`)**: Root application error taxonomy ([`AppError`]), single source of truth for screaming
//!   error codes, granular action tokens, and authoritative server-side logging sink.
//! - **Health (`health`)**: Infallible `/health` liveness probe and lightweight tracing router.
//! - **Metadata (`meta`)**: Core system metadata persistence (`app_meta` table), schema flags, and seed state.
//! - **Response Envelopes (`response`)**: Authoritative REST wire envelope ([`ApiResponse`]), standard numeric status codes
//!   ([`Code`]), alphanumeric status tokens ([`Status`]), typed [`ErrorPayload`], and 404 fallbacks.
//! - **State (`state`)**: Thread-safe shared application runtime state ([`AppState`]) encapsulated across Axum routes.
//! - **Time (`time`)**: Authoritative timestamp generator ensuring consistent UTC epoch seconds.

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
