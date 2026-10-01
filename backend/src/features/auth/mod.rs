//! Authentication and session management feature slice.
//!
//! # Responsibilities
//! - **Identity & Registration**: Transactional user provisioning with Argon2id password hashing
//!   and initial role assignment (first registered user is granted Admin, subsequent users Member).
//! - **Session Management**: Opaque cryptographic session token issuance, validation, and revocation.
//! - **Access Control**: Request authentication extractor (`AuthUser`) for guarding protected routes.
//!
//! # Submodules
//! - `db`: SQLite persistence for `users` and `sessions` tables and indexes.
//! - `error`: Strongly typed domain errors ([`AuthError`]) and HTTP failure responses.
//! - `models`: Inbound request DTOs, outbound payloads, and validated domain types.
//! - `routes`: HTTP endpoints mounted under `/auth` (`/register`, `/login`, `/logout`, `/me`).
//! - `security`: Password hashing, cryptographic token generation, and the `AuthUser` extractor.

use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;
mod security;

pub(crate) use db::init_auth_schema;
pub(crate) use security::AuthUser;

pub use error::AuthError;

/// Returns the authentication feature router mounted under `/auth`.
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
