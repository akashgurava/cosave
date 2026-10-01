//! Authentication, user onboarding, and session security for the application.
//!
//! This module provides identity management, credential verification, and route protection:
//!
//! - **First-User Bootstrap**: Simplifies initial setup by automatically designating the very first
//!   registered user as an administrator (`Admin`), while subsequent signups join as standard members (`Member`).
//! - **Dual-Channel Session Handling**: Supports session verification through HTTP-only cookies
//!   for web browser sessions alongside `Authorization: Bearer <token>` headers for scripts and API clients.
//! - **Credential Security**: Protects stored credentials using Argon2id password hashing with unique random salts.
//! - **Session Lifecycle**: Issues cryptographically random session tokens with a 30-day lifetime, supporting
//!   seamless login retention and immediate revocation on logout.
//! - **Route Protection & Error Handling**: Provides the `AuthUser` request extractor for securing feature
//!   endpoints, and exposes [`AuthError`] for domain error classification and targeted client messaging.

use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;
mod security;

pub(crate) use db::init_auth_schema;
pub use error::AuthError;
pub(crate) use security::AuthUser;

/// Returns the authentication feature router mounted under `/auth`.
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
