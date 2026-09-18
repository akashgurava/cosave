pub(crate) mod auth;
pub(crate) mod categories;
pub(crate) mod health;

use crate::core::state::AppState;
use axum::Router;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::router())
        .nest("/categories", categories::router())
}
