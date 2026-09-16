pub(crate) mod health;

use axum::Router;

pub(crate) fn router() -> Router {
    Router::new().merge(health::router())
}
