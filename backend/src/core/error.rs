use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use super::response::{ApiResponse, Code, ErrorPayload, Status};
use crate::features::{auth::AuthError, categories::CategoryError};

/// Central application error type.
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Category(#[from] CategoryError),

    #[error("INIT_SCHEMA_ERROR. ACTION: {action}. TABLE: {table}. ERROR: {source}")]
    InitSchema {
        action: &'static str,
        table: &'static str,
        #[source]
        source: sqlx::Error,
    },

    #[error("SHOULD_NOT_BE_HAPPENING. ACTION: {action}. REASON: {reason}")]
    ShouldNotBeHappening {
        action: &'static str,
        reason: String,
    },
}

impl AppError {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.action(),
            Self::Category(err) => err.action(),
            Self::InitSchema { action, .. } => action,
            Self::ShouldNotBeHappening { action, .. } => action,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.code(),
            Self::Category(err) => err.code(),
            Self::InitSchema { .. } => "INIT_SCHEMA_ERROR",
            Self::ShouldNotBeHappening { .. } => "SHOULD_NOT_BE_HAPPENING",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Auth(err) => err.into_response(),
            Self::Category(err) => err.into_response(),
            _ => {
                let action = self.action();
                let code_str = self.code();

                let (status_code, code, message) = match &self {
                    Self::Auth(_) | Self::Category(_) => unreachable!(),
                    Self::InitSchema { table, source, .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Code::internal_error(),
                        format!("Failed initializing table '{table}': {source}"),
                    ),
                    Self::ShouldNotBeHappening { reason, .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Code::internal_error(),
                        format!("Unexpected invariant violation in {action}: {reason}"),
                    ),
                };

                tracing::error!(action = action, code = code_str, error = %self, "request failed");

                let body = Json(ApiResponse::err(
                    code,
                    Status::custom(code_str),
                    ErrorPayload::new(action, message),
                ));
                (status_code, body).into_response()
            }
        }
    }
}
