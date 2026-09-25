#![deny(dead_code)]

mod core;
mod features;

#[cfg(feature = "cli")]
pub use core::Cli;
pub use core::{init_db, AppError, AppState, DbPool};
pub use features::{init_features, router, AuthError, CategoryError};
