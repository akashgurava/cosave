use axum::Router;

use crate::core::AppState;

mod routes;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
