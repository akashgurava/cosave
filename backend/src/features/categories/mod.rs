pub(crate) mod db;
pub(crate) mod models;
pub(crate) mod routes;

use crate::core::state::AppState;
use axum::Router;

pub(crate) const TASK_NAME: &str = "CONFIG.CATEGORIES";

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
