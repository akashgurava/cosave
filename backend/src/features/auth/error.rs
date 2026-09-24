use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use thiserror::Error;

use crate::core::response::{ApiResponse, Code, Status};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("INVALID_USERNAME. Need atleast 2 characters")]
    InvalidUsername,
    #[error("INVALID_PASSWORD. Need atleast 6 characters")]
    InvalidPassword,
    #[error("USER_EXISTS")]
    UserExists,
    #[error("INSERT_NEW_USER_ERROR. Name: {name}. Error: {error}")]
    InsertNewUserError { name: String, error: sqlx::Error },

    #[error("INSERT_NEW_SESSION_ERROR. Name: {name}. Error: {error}")]
    InsertNewSessionError { name: String, error: sqlx::Error },
}

impl AuthError {
    pub(crate) fn insert_new_user_error(name: String, error: sqlx::Error) -> Self {
        Self::InsertNewUserError { name, error }
    }

    pub(crate) fn insert_new_session_error(name: String, error: sqlx::Error) -> Self {
        Self::InsertNewSessionError { name, error }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status_code, code, status, message) = match self {
            Self::InvalidUsername => {
                tracing::warn!("invalid username");
                (
                    StatusCode::BAD_REQUEST,
                    Code::bad_request(),
                    Status::bad_request(),
                    Some(self.to_string()),
                )
            }
            Self::InvalidPassword => {
                tracing::warn!("invalid password");
                (
                    StatusCode::BAD_REQUEST,
                    Code::bad_request(),
                    Status::bad_request(),
                    Some(self.to_string()),
                )
            }
            Self::UserExists => {
                tracing::warn!("user already exists");
                (
                    StatusCode::CONFLICT,
                    Code::conflict(),
                    Status::conflict(),
                    Some(self.to_string()),
                )
            }
            Self::InsertNewUserError { name, error } => {
                tracing::error!("failed to insert new user {name}: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                    Some(self.to_string()),
                )
            }
            Self::InsertNewSessionError { name, error } => {
                tracing::error!("failed to insert new session for {name}: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                    Some(self.to_string()),
                )
            }
        };

        let body = Json(ApiResponse::err(code, status, message));
        (status_code, body).into_response()
    }
}
