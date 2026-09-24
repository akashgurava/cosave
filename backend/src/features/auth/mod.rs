mod db;
mod error;
mod models;
mod routes;
mod security;

use axum::Router;

pub use error::AuthError;

pub(crate) use db::init_schema;
pub(crate) use security::AuthUser;

#[allow(dead_code)]
pub(crate) const FEATURE: &str = "AUTH";

#[cfg(test)]
pub(crate) use models::User;

use crate::core::AppState;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
