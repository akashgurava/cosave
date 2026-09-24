use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::core::response::{ApiResponse, Code, ErrorPayload, Status};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("INVALID_USERNAME. ACTION: {action}. Username: '{username}'. Need at least {min_len} characters")]
    InvalidUsername {
        action: &'static str,
        username: String,
        min_len: usize,
    },

    #[error("INVALID_PASSWORD. ACTION: {action}. Need at least {min_len} characters")]
    InvalidPassword {
        action: &'static str,
        min_len: usize,
    },

    #[error("USER_EXISTS. ACTION: {action}. User '{username}' already exists")]
    UserExists {
        action: &'static str,
        username: String,
    },

    #[error("INVALID_CREDENTIALS. ACTION: {action}")]
    InvalidCredentials { action: &'static str },

    #[error("UNAUTHENTICATED. ACTION: {action}")]
    Unauthenticated { action: &'static str },

    #[error("INSERT_NEW_USER_ERROR. ACTION: {action}. Username: '{username}'. ERROR: {source}")]
    InsertNewUserError {
        action: &'static str,
        username: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("INSERT_NEW_SESSION_ERROR. ACTION: {action}. UserID: '{user_id}'. ERROR: {source}")]
    InsertNewSessionError {
        action: &'static str,
        user_id: String,
        #[source]
        source: sqlx::Error,
    },
}

impl AuthError {
    pub fn action(&self) -> &'static str {
        match self {
            Self::InvalidUsername { action, .. } => action,
            Self::InvalidPassword { action, .. } => action,
            Self::UserExists { action, .. } => action,
            Self::InvalidCredentials { action, .. } => action,
            Self::Unauthenticated { action, .. } => action,
            Self::InsertNewUserError { action, .. } => action,
            Self::InsertNewSessionError { action, .. } => action,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidUsername { .. } => "INVALID_USERNAME",
            Self::InvalidPassword { .. } => "INVALID_PASSWORD",
            Self::UserExists { .. } => "USER_EXISTS",
            Self::InvalidCredentials { .. } => "INVALID_CREDENTIALS",
            Self::Unauthenticated { .. } => "UNAUTHENTICATED",
            Self::InsertNewUserError { .. } => "INSERT_NEW_USER_ERROR",
            Self::InsertNewSessionError { .. } => "INSERT_NEW_SESSION_ERROR",
        }
    }
}

impl IntoResponse for AuthError {
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
            Self::UserExists { username, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!(
                    "Username '{username}' already exists. Please sign in or choose another name."
                ),
            ),
            Self::InsertNewUserError {
                username, source, ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Code::internal_error(),
                format!("Failed to register '{username}'. Database constraint violation: {source}"),
            ),
            Self::InsertNewSessionError {
                user_id, source, ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Code::internal_error(),
                format!("Failed to create session for user '{user_id}'. Database error: {source}"),
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
