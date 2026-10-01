//! Core library for the application backend.
//!
//! Provides the foundational runtime infrastructure, database pooling, and domain features
//! powering the application. The crate coordinates application configuration via [`AppConfig`],
//! initializes SQLite connections through [`init_db`], manages shared state with [`AppState`],
//! and exports the unified HTTP [`router`].

#![deny(dead_code)]

mod core;
mod features;

#[cfg(feature = "cli")]
pub use core::Cli;
pub use core::{
    api_only_root_fallback, init_db, ApiResponse, AppConfig, AppEnv, AppError, AppState, Code,
    DbPool, ErrorPayload, Status,
};
pub use features::{init_features, init_schemas, router, AuthError, CategoryError};
