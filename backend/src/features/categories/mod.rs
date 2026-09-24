pub(crate) mod db;
pub(crate) mod error;
pub(crate) mod models;
pub(crate) mod routes;

pub use error::CategoryError;

use crate::core::state::AppState;
use axum::Router;

#[allow(dead_code)]
pub(crate) const FEATURE: &str = "CONFIG.CATEGORIES";

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
