//! Central application error types, propagation rollups, and HTTP response mapping.
//!
//! # Architecture
//! Backend errors in CoSave operate on a two-tier hierarchy:
//! 1. **Domain Feature Errors**: Encapsulated errors specific to domain subsystems
//!    (such as [`AuthError`] or [`CategoryError`]), representing anticipated business rule
//!    or validation failures.
//! 2. **Application Error ([`AppError`])**: Root error enum that wraps all feature errors via
//!    explicit `From` implementations and owns infrastructure failures ([`AppError::InitSchema`]
//!    and [`AppError::ShouldNotBeHappening`]).
//!
//! # Standards & Invariants
//! - **Single Source of Truth (SSOT)**: Screaming snake_case error code strings are defined exclusively
//!   in [`AppError::code`]. No macro string derivations or duplicate string literals.
//! - **Action Taxonomy**: Every error variant carries a globally unique compile-time `action` token
//!   pinpointing the exact failure site (`FEATURE.WORKFLOW.STEP[.BRANCH]`).
//! - **Authoritative Logging Sink**: [`IntoResponse`] serves as the authoritative server-side logging
//!   sink (`tracing::error!` for 5xx). Zero call-site error logging is permitted.
//! - **No Leaked SQL or Internals**: Server logs record the full internal diagnostic details via `%self`,
//!   while client error envelopes receive sanitized, actionable messages without raw SQL or engine traces.

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::features::{AuthError, CategoryError};

use super::response::{ApiResponse, Code, ErrorPayload, Status};

/// Central application error type unifying feature domain errors and infrastructure failures.
#[derive(Debug)]
pub enum AppError {
    /// Authentication or authorization failure.
    Auth(AuthError),
    /// Category or hierarchy domain error.
    Category(CategoryError),
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
            Self::InitSchema { action, .. } => action,
            Self::ShouldNotBeHappening { action, .. } => action,
        }
    }

    /// Single source of truth for the screaming snake_case error code string.
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
