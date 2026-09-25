use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;

pub(crate) use db::{init_schema, seed_default_categories};
pub use error::CategoryError;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
