use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt;

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
        let (status_code, code, status) = match &self {
            Self::BadRequest(msg) => {
                tracing::warn!(error = %msg, "client bad request");
                (
                    StatusCode::BAD_REQUEST,
                    Code::bad_request(),
                    Status::bad_request(),
                )
            }
            Self::Unauthorized(msg) => {
                tracing::warn!(error = %msg, "client unauthorized");
                (
                    StatusCode::UNAUTHORIZED,
                    Code::unauthorized(),
                    Status::unauthenticated(),
                )
            }
            Self::InvalidCredentials => {
                tracing::warn!("client invalid credentials");
                (
                    StatusCode::UNAUTHORIZED,
                    Code::unauthorized(),
                    Status::invalid_credentials(),
                )
            }
            Self::NotFound(msg) => {
                tracing::warn!(error = %msg, "resource not found");
                (
                    StatusCode::NOT_FOUND,
                    Code::not_found(),
                    Status::not_found(),
                )
            }
            Self::Conflict(msg) => {
                tracing::warn!(error = %msg, "resource conflict");
                (StatusCode::CONFLICT, Code::conflict(), Status::conflict())
            }
            Self::UserExists => {
                tracing::warn!("user already exists");
                (
                    StatusCode::CONFLICT,
                    Code::conflict(),
                    Status::user_exists(),
                )
            }
            Self::Database(err) => {
                tracing::error!(error = %err, "unhandled database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                )
            }
            Self::Internal(msg) => {
                tracing::error!(error = %msg, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Code::internal_error(),
                    Status::internal_error(),
                )
            }
        };

        let body = Json(ApiResponse::err(code, status, None::<()>));
        (status_code, body).into_response()
    }
}
