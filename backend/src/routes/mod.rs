pub(crate) mod auth;
pub(crate) mod categories;
pub(crate) mod health;

use axum::Router;

use crate::state::AppState;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::router())
        .nest("/categories", categories::router())
}
