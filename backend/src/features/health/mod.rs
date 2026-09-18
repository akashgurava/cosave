pub(crate) mod routes;

use crate::core::state::AppState;
use axum::Router;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
