use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt;

use crate::features::auth::AuthError;

use super::response::{ApiResponse, Code, Status};

/// Central application error type.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum AppError {
    BadRequest(String),
    Unauthorized(String),
    InvalidCredentials,
    NotFound(String),
    Conflict(String),
    UserExists,
    Database(sqlx::Error),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Conflict(msg) => write!(f, "Conflict: {msg}"),
            Self::UserExists => write!(f, "User already exists"),
            Self::Database(err) => write!(f, "Database error: {err}"),
            Self::Internal(msg) => write!(f, "Internal server error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(err) => Some(err),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, code, status, message) = match &self {
            Self::BadRequest(msg) => {
                tracing::warn!(error = %msg, "client bad request");
                (
                    StatusCode::BAD_REQUEST,
                    Code::bad_request(),
                    Status::bad_request(),
                    Some(msg.clone()),
                )
            }
            Self::Unauthorized(msg) => {
                tracing::warn!(error = %msg, "client unauthorized");
                (
                    StatusCode::UNAUTHORIZED,
                    Code::unauthorized(),
                    Status::unauthenticated(),
                    Some(msg.clone()),
                )
            }
            Self::InvalidCredentials => {
                tracing::warn!("client invalid credentials");
                (
                    StatusCode::UNAUTHORIZED,
                    Code::unauthorized(),
                    Status::invalid_credentials(),
                    Some("Invalid credentials".to_string()),
                )
            }
            Self::NotFound(msg) => {
                tracing::warn!(error = %msg, "resource not found");
                (
                    StatusCode::NOT_FOUND,
                    Code::not_found(),
                    Status::not_found(),
                    Some(msg.clone()),
                )
            }
            Self::Conflict(msg) => {
                tracing::warn!(error = %msg, "resource conflict");
                (
                    StatusCode::CONFLICT,
                    Code::conflict(),
                    Status::conflict(),
                    Some(msg.clone()),
                )
            }
            Self::UserExists => {
                tracing::warn!("user already exists");
                (
                    StatusCode::CONFLICT,
                    Code::conflict(),
                    Status::user_exists(),
                    Some("User already exists".to_string()),
                )
            }
            Self::Database(err) => {
                tracing::error!(error = %err, "unhandled database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                    Some("Internal database error".to_string()),
                )
            }
            Self::Internal(msg) => {
                tracing::error!(error = %msg, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                    Some(msg.clone()),
                )
            }
        };

        let body = Json(ApiResponse::err(code, status, message));
        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NewAppError {
    #[error("SHOULD_NOT_BE_HAPPENING. TASK: {0}. ACTION: {1}. ERROR: {1}")]
    ShouldNotBeHappening(String, String, String),

    #[error("INIT_SCHEMA_ERROR. TASK: {0}. TABLE: {1}. ERROR: {2}")]
    InitSchemaError(String, String, sqlx::Error),

    // DB Errors
    #[error("DB_ERROR. TASK: {0}. ACTION: {1}. ERROR: {2}")]
    TransactionError(String, String, sqlx::Error),

    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl NewAppError {
    pub(crate) fn should_not_be_happening(task: String, action: String, err: String) -> Self {
        Self::ShouldNotBeHappening(task, action, err)
    }

    pub(crate) fn init_schema(task: String, table: String, err: sqlx::Error) -> Self {
        Self::InitSchemaError(task, table, err)
    }

    pub(crate) fn transaction(task: String, action: String, err: sqlx::Error) -> Self {
        Self::TransactionError(task, action, err)
    }
}

impl IntoResponse for NewAppError {
    fn into_response(self) -> Response {
        tracing::error!("{}", &self);

        let (status_code, code, status, message) = match self {
            Self::ShouldNotBeHappening(_, _, _) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Code::internal_error(),
                Status::internal_error(),
                Some(self.to_string()),
            ),

            Self::InitSchemaError(_, _, _) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Code::internal_error(),
                Status::internal_error(),
                Some(self.to_string()),
            ),

            Self::TransactionError(_, _, _) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Code::internal_error(),
                Status::internal_error(),
                Some(self.to_string()),
            ),

            Self::Auth(err) => return err.into_response(),
        };

        let body = Json(ApiResponse::err(code, status, message));
        (status_code, body).into_response()
    }
}
