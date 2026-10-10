//! Transaction domain error types, screaming status codes, and HTTP response mapping.
//!
//! Provides the strongly typed [`TransactionError`] enum representing domain validation,
//! relational lookup failures, and state violations across transactions, manual records,
//! and staging entries. Every variant carries a globally unique compile-time action token
//! identifying the exact failure site. When transformed into an HTTP response, these
//! errors produce structured client envelopes without leaking database internals.

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

/// Domain and validation errors arising within transaction workflows.
#[derive(Debug)]
pub enum TransactionError {
    /// Transaction description is empty or whitespace-only.
    EmptyDescription { action: &'static str },
    /// Transaction date format is invalid or cannot be parsed.
    InvalidDate { action: &'static str, raw: String },
    /// Transaction amount is zero (transactions must represent non-zero fund movements).
    ZeroAmount { action: &'static str },
    /// Target transaction was not found.
    TransactionNotFound { action: &'static str, id: String },
    /// Target account was not found.
    AccountNotFound { action: &'static str, id: i64 },
    /// Target family member was not found.
    MemberNotFound { action: &'static str, id: i64 },
    /// Target category was not found.
    CategoryNotFound { action: &'static str, id: i64 },
    /// Target subcategory was not found.
    SubcategoryNotFound { action: &'static str, id: i64 },
    /// Target transaction type was not found.
    TypeNotFound { action: &'static str, id: i64 },
    /// Invalid transaction status string.
    InvalidStatus { action: &'static str, raw: String },
    /// Invalid transaction source type discriminator.
    InvalidSourceType { action: &'static str, raw: String },
    /// Operation is not supported for the given transaction or source.
    UnsupportedOperation {
        action: &'static str,
        reason: String,
    },
}

impl TransactionError {
    /// Returns the globally unique compile-time action token pinpointing the failure site.
    pub fn action(&self) -> &'static str {
        match self {
            Self::EmptyDescription { action } => action,
            Self::InvalidDate { action, .. } => action,
            Self::ZeroAmount { action } => action,
            Self::TransactionNotFound { action, .. } => action,
            Self::AccountNotFound { action, .. } => action,
            Self::MemberNotFound { action, .. } => action,
            Self::CategoryNotFound { action, .. } => action,
            Self::SubcategoryNotFound { action, .. } => action,
            Self::TypeNotFound { action, .. } => action,
            Self::InvalidStatus { action, .. } => action,
            Self::InvalidSourceType { action, .. } => action,
            Self::UnsupportedOperation { action, .. } => action,
        }
    }

    /// Returns the screaming machine-readable status code for the error variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyDescription { .. } => "EMPTY_TRANSACTION_DESCRIPTION",
            Self::InvalidDate { .. } => "INVALID_TRANSACTION_DATE",
            Self::ZeroAmount { .. } => "ZERO_TRANSACTION_AMOUNT",
            Self::TransactionNotFound { .. } => "TRANSACTION_NOT_FOUND",
            Self::AccountNotFound { .. } => "ACCOUNT_NOT_FOUND",
            Self::MemberNotFound { .. } => "MEMBER_NOT_FOUND",
            Self::CategoryNotFound { .. } => "CATEGORY_NOT_FOUND",
            Self::SubcategoryNotFound { .. } => "SUBCATEGORY_NOT_FOUND",
            Self::TypeNotFound { .. } => "TYPE_NOT_FOUND",
            Self::InvalidStatus { .. } => "INVALID_TRANSACTION_STATUS",
            Self::InvalidSourceType { .. } => "INVALID_SOURCE_TYPE",
            Self::UnsupportedOperation { .. } => "UNSUPPORTED_OPERATION",
        }
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::EmptyDescription { action } => write!(f, "{code}. ACTION: {action}"),
            Self::InvalidDate { action, raw } => {
                write!(f, "{code}. ACTION: {action}. Date: '{raw}'")
            }
            Self::ZeroAmount { action } => write!(f, "{code}. ACTION: {action}"),
            Self::TransactionNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Transaction ID: {id}")
            }
            Self::AccountNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Account ID: {id}")
            }
            Self::MemberNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Member ID: {id}")
            }
            Self::CategoryNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Category ID: {id}")
            }
            Self::SubcategoryNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Subcategory ID: {id}")
            }
            Self::TypeNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Type ID: {id}")
            }
            Self::InvalidStatus { action, raw } => {
                write!(f, "{code}. ACTION: {action}. Status: '{raw}'")
            }
            Self::InvalidSourceType { action, raw } => {
                write!(f, "{code}. ACTION: {action}. Source Type: '{raw}'")
            }
            Self::UnsupportedOperation { action, reason } => {
                write!(f, "{code}. ACTION: {action}. Reason: {reason}")
            }
        }
    }
}

impl Error for TransactionError {}

impl IntoResponse for TransactionError {
    fn into_response(self) -> Response {
        let action = self.action();
        let code_str = self.code();

        let (status_code, code, message) = match &self {
            Self::EmptyDescription { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Transaction description cannot be empty.".to_string(),
            ),
            Self::InvalidDate { raw, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Transaction date '{raw}' is invalid. Expected 'YYYY-MM-DD'."),
            ),
            Self::ZeroAmount { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Transaction amount cannot be zero.".to_string(),
            ),
            Self::TransactionNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Transaction with ID {id} was not found."),
            ),
            Self::AccountNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Account with ID {id} was not found."),
            ),
            Self::MemberNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Member with ID {id} was not found."),
            ),
            Self::CategoryNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Category with ID {id} was not found."),
            ),
            Self::SubcategoryNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Subcategory with ID {id} was not found."),
            ),
            Self::TypeNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Transaction type with ID {id} was not found."),
            ),
            Self::InvalidStatus { raw, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Transaction status '{raw}' is invalid. Expected 'cleared' or 'pending'."),
            ),
            Self::InvalidSourceType { raw, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Source type '{raw}' is invalid. Expected 'import' or 'manual'."),
            ),
            Self::UnsupportedOperation { reason, .. } => {
                (StatusCode::BAD_REQUEST, Code::bad_request(), reason.clone())
            }
        };

        if status_code.is_server_error() {
            tracing::error!(action = action, code = code_str, error = %self, "request failed");
        } else {
            tracing::warn!(action = action, code = code_str, error = %self, "transaction request rejected");
        }

        let body = Json(ApiResponse::err(
            code,
            Status::custom(code_str),
            ErrorPayload::new(action, message),
        ));
        (status_code, body).into_response()
    }
}
