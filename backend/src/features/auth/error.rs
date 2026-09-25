use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

#[derive(Debug)]
pub enum AuthError {
    InvalidUsername {
        action: &'static str,
        username: String,
        min_len: usize,
    },
    InvalidPassword {
        action: &'static str,
        min_len: usize,
    },
    UserExists {
        action: &'static str,
        username: String,
    },
    InvalidCredentials {
        action: &'static str,
    },
    Unauthenticated {
        action: &'static str,
    },
    InsertNewSessionError {
        action: &'static str,
        user_id: String,
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
            Self::InsertNewSessionError { .. } => "INSERT_NEW_SESSION_ERROR",
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
            Self::UserExists { action, username } => {
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
            Self::InsertNewSessionError {
                action,
                user_id,
                source,
            } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. UserID: '{user_id}'. ERROR: {source}"
                )
            }
        }
    }
}

impl Error for AuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InsertNewSessionError { source, .. } => Some(source),
            _ => None,
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
