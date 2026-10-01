//! Authentication domain error taxonomy and HTTP response transformation.
//!
//! Provides the rigidly typed [`AuthError`] enumeration representing anticipated
//! domain failures encountered during user registration, credential authentication,
//! session validation, and authorization boundaries.
//!
//! # HTTP Status Code Mapping
//! | Variant | HTTP Status | Wire Code | Description |
//! | :--- | :--- | :--- | :--- |
//! | [`AuthError::InvalidUsername`] | `400 Bad Request` | `INVALID_USERNAME` | Username fails length or format constraints. |
//! | [`AuthError::InvalidPassword`] | `400 Bad Request` | `INVALID_PASSWORD` | Password fails minimum length constraint. |
//! | [`AuthError::InvalidCredentials`] | `401 Unauthorized` | `INVALID_CREDENTIALS` | Unmatched username or incorrect password. |
//! | [`AuthError::Unauthenticated`] | `401 Unauthorized` | `UNAUTHENTICATED` | Missing, invalid, or expired session token. |
//! | [`AuthError::UserAlreadyExists`] | `409 Conflict` | `USER_ALREADY_EXISTS` | Target username is already registered. |
//!
//! # Invariants
//! - Every variant carries a globally unique compile-time `action` token pinpointing the exact failure site.
//! - Error codes are screaming snake_case string literals defined strictly once in [`AuthError::code`].
//! - Server-side logging is handled exclusively inside [`IntoResponse::into_response`].
//! - Client responses wrap failures in standardized [`ApiResponse`] envelopes with [`ErrorPayload`].

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

/// Domain errors specific to authentication, credential verification, and session authorization.
#[derive(Debug)]
pub enum AuthError {
    /// Inbound username failed length or formatting constraints during value object validation.
    InvalidUsername {
        /// Unique compile-time action token indicating the exact validation failure point.
        action: &'static str,
        /// Inbound raw username that failed validation.
        username: String,
        /// Minimum required character length for usernames.
        min_len: usize,
    },
    /// Inbound password failed minimum length requirements during value object validation.
    InvalidPassword {
        /// Unique compile-time action token indicating the exact validation failure point.
        action: &'static str,
        /// Minimum required character length for passwords.
        min_len: usize,
    },
    /// Registration rejected because a user with the requested username already exists.
    UserAlreadyExists {
        /// Unique compile-time action token identifying the registration conflict check.
        action: &'static str,
        /// Conflicting username string.
        username: String,
    },
    /// Authentication rejected due to unknown username or mismatched password hash.
    InvalidCredentials {
        /// Unique compile-time action token identifying the authentication checkpoint.
        action: &'static str,
    },
    /// Request rejected because an unauthenticated caller attempted to access a protected resource.
    Unauthenticated {
        /// Unique compile-time action token identifying the authorization boundary checkpoint.
        action: &'static str,
    },
}

impl AuthError {
    /// Returns the globally unique compile-time action token pinpointing the exact failure site.
    pub fn action(&self) -> &'static str {
        match self {
            Self::InvalidUsername { action, .. } => action,
            Self::InvalidPassword { action, .. } => action,
            Self::UserAlreadyExists { action, .. } => action,
            Self::InvalidCredentials { action, .. } => action,
            Self::Unauthenticated { action, .. } => action,
        }
    }

    /// Returns the uppercase screaming snake_case error code string for client classification.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidUsername { .. } => "INVALID_USERNAME",
            Self::InvalidPassword { .. } => "INVALID_PASSWORD",
            Self::UserAlreadyExists { .. } => "USER_ALREADY_EXISTS",
            Self::InvalidCredentials { .. } => "INVALID_CREDENTIALS",
            Self::Unauthenticated { .. } => "UNAUTHENTICATED",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::InvalidUsername {
                action,
                username,
                min_len,
            } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. Username: '{username}'. Need at least {min_len} characters"
                )
            }
            Self::InvalidPassword { action, min_len } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. Need at least {min_len} characters"
                )
            }
            Self::UserAlreadyExists { action, username } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. User '{username}' already exists"
                )
            }
            Self::InvalidCredentials { action } => {
                write!(f, "{code}. ACTION: {action}")
            }
            Self::Unauthenticated { action } => {
                write!(f, "{code}. ACTION: {action}")
            }
        }
    }
}

impl Error for AuthError {}

impl IntoResponse for AuthError {
    /// Transforms [`AuthError`] into an Axum HTTP [`Response`].
    ///
    /// Logs structured warnings (for 4xx client errors) or errors (for 5xx server errors) via [`tracing`],
    /// and formats the body into an [`ApiResponse`] JSON failure envelope.
    fn into_response(self) -> Response {
        let action = self.action();
        let code_str = self.code();

        let (status_code, code, message) = match &self {
            Self::InvalidUsername {
                username, min_len, ..
            } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Username '{username}' must be at least {min_len} characters."),
            ),
            Self::InvalidPassword { min_len, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Password must be at least {min_len} characters."),
            ),
            Self::InvalidCredentials { .. } => (
                StatusCode::UNAUTHORIZED,
                Code::unauthorized(),
                "Invalid username or password.".to_string(),
            ),
            Self::Unauthenticated { .. } => (
                StatusCode::UNAUTHORIZED,
                Code::unauthorized(),
                "Authentication required. Please sign in to access this resource.".to_string(),
            ),
            Self::UserAlreadyExists { username, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!(
                    "Username '{username}' already exists. Please sign in or choose another name."
                ),
            ),
        };

        if status_code.is_server_error() {
            tracing::error!(action = action, code = code_str, error = %self, "request failed");
        } else {
            tracing::warn!(action = action, code = code_str, error = %self, "client error");
        }

        let body = Json(ApiResponse::err(
            code,
            Status::custom(code_str),
            ErrorPayload::new(action, message),
        ));
        (status_code, body).into_response()
    }
}
