use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::features::{auth::AuthError, categories::CategoryError};

use super::response::{ApiResponse, Code, ErrorPayload, Status};

/// Central application error type.
#[derive(Debug)]
pub enum AppError {
    Auth(AuthError),
    Category(CategoryError),
    InitSchema {
        action: &'static str,
        table: &'static str,
        source: sqlx::Error,
    },
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

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Auth(err) => write!(f, "{err}"),
            Self::Category(err) => write!(f, "{err}"),
            Self::InitSchema {
                action,
                table,
                source,
            } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. TABLE: {table}. ERROR: {source}"
                )
            }
            Self::ShouldNotBeHappening { action, reason } => {
                write!(f, "{code}. ACTION: {action}. REASON: {reason}")
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Auth(err) => Some(err),
            Self::Category(err) => Some(err),
            Self::InitSchema { source, .. } => Some(source),
            Self::ShouldNotBeHappening { .. } => None,
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}

impl From<CategoryError> for AppError {
    fn from(err: CategoryError) -> Self {
        Self::Category(err)
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
