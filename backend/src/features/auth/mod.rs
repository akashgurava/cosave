pub(crate) mod db;
pub(crate) mod models;
pub(crate) mod routes;
pub(crate) mod security;

#[allow(unused_imports)]
pub(crate) use models::{Role, User, UserDto};
#[allow(unused_imports)]
pub(crate) use security::{generate_token, AuthUser, OptionalAuthUser};

use crate::core::state::AppState;
use axum::Router;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
