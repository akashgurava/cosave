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
