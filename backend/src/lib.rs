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
