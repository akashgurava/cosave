use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;
mod security;

pub(crate) use db::init_schema;
pub use error::AuthError;
pub(crate) use security::AuthUser;

#[cfg(test)]
pub(crate) use models::User;

pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
