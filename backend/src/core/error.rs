//! Central error classification, propagation rollups, and HTTP response mapping.
//!
//! All domain and infrastructure failures roll up into the top-level [`AppError`] enum,
//! which acts as the unified error boundary for the application backend. Each error carries
//! a globally unique compile-time action identifier pinpointing the exact failure site.
//! When converted into an HTTP response, internal server errors are logged authoritatively
//! while clients receive sanitized, human-actionable error envelopes without leaking raw database details.

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::features::{AuthError, CategoryError, FamilyError, TransactionError};

use super::response::{ApiResponse, Code, ErrorPayload, Status};

/// Central application error type unifying feature domain errors and infrastructure failures.
#[derive(Debug)]
pub enum AppError {
    /// Authentication or authorization failure.
    Auth(AuthError),
    /// Category or hierarchy domain error.
    Category(CategoryError),
    /// Family and accounts domain error.
    Family(FamilyError),
    /// Transaction and ledger domain error.
    Transaction(TransactionError),

    /// Database table or index DDL initialization failure.
    InitSchema {
        /// Dedicated compile-time action identifier.
        action: &'static str,
        /// Table name associated with the schema migration.
        table: &'static str,
        /// Underlying SQLite / SQLx engine error.
        source: sqlx::Error,
    },
    /// Unexpected runtime invariant violation or unhandled database failure.
    ShouldNotBeHappening {
        /// Dedicated compile-time action identifier.
        action: &'static str,
        /// Contextual description of the invariant violation.
        reason: String,
    },
}

impl AppError {
    /// Returns the globally unique compile-time action string pinpointing the failure site.
    pub fn action(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.action(),
            Self::Category(err) => err.action(),
            Self::Family(err) => err.action(),
            Self::Transaction(err) => err.action(),
            Self::InitSchema { action, .. } => action,
            Self::ShouldNotBeHappening { action, .. } => action,
        }
    }

    /// Single source of truth for the screaming snake_case error code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.code(),
            Self::Category(err) => err.code(),
            Self::Family(err) => err.code(),
            Self::Transaction(err) => err.code(),
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
            Self::Family(err) => write!(f, "{err}"),
            Self::Transaction(err) => write!(f, "{err}"),
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
            Self::Family(err) => Some(err),
            Self::Transaction(err) => Some(err),
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

impl From<FamilyError> for AppError {
    fn from(err: FamilyError) -> Self {
        Self::Family(err)
    }
}

impl From<TransactionError> for AppError {
    fn from(err: TransactionError) -> Self {
        Self::Transaction(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Auth(err) => err.into_response(),
            Self::Category(err) => err.into_response(),
            Self::Family(err) => err.into_response(),
            Self::Transaction(err) => err.into_response(),
            _ => {
                let action = self.action();
                let code_str = self.code();

                let (status_code, code, message) = match &self {
                    Self::Auth(_) | Self::Category(_) | Self::Family(_) | Self::Transaction(_) => {
                        unreachable!()
                    }
                    Self::InitSchema { table, .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Code::internal_error(),
                        format!("Failed initializing database table '{table}'."),
                    ),
                    Self::ShouldNotBeHappening { .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Code::internal_error(),
                        "An unexpected internal error occurred.".to_string(),
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
