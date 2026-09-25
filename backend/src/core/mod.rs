#[cfg(feature = "cli")]
mod cli;
mod db;
mod error;
mod response;
mod state;

#[cfg(feature = "cli")]
pub use cli::Cli;
pub(crate) use db::{create_db_object, db_err, DbResultExt};
pub use db::{init_db, DbPool};
pub use error::AppError;
pub(crate) use response::{ApiResponse, Code, ErrorPayload, Status};
pub use state::AppState;

#[cfg(test)]
pub(crate) mod test_utils;
#[cfg(test)]
pub(crate) use test_utils::TestApp;
