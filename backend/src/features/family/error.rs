//! Family domain error types, screaming status codes, and HTTP response mapping.
//!
//! Provides the strongly typed [`FamilyError`] enum representing domain validation,
//! currency constraints, and relational lookup failures across families, members,
//! and accounts. Every variant carries a globally unique compile-time action token
//! identifying the exact failure site. When transformed into an HTTP response, these
//! errors produce structured client envelopes without leaking database internals.

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

/// Domain and validation errors arising within family and account workflows.
#[derive(Debug)]
pub enum FamilyError {
    /// Family display name is empty or whitespace-only.
    EmptyFamilyName { action: &'static str },
    /// Family member display name is empty or whitespace-only.
    EmptyMemberName { action: &'static str },
    /// Currency code is not a valid 3-letter ISO-4217 code.
    InvalidCurrency {
        action: &'static str,
        currency: String,
    },
    /// Bank institution name is empty or whitespace-only.
    EmptyBankName { action: &'static str },
    /// Account display name is empty or whitespace-only.
    EmptyAccountName { action: &'static str },
    /// Credit card display name is empty or whitespace-only.
    EmptyCardName { action: &'static str },
    /// Last 4 digits identifier is not exactly 4 digits.
    InvalidLast4 { action: &'static str },
    /// Account type discriminator is unrecognized.
    InvalidAccountType { action: &'static str, raw: String },
    /// Monetary quantity is negative where non-negative is required.
    NegativeAmount { action: &'static str },
    /// Target family was not found.
    FamilyNotFound { action: &'static str },
    /// Target currency ID was not found.
    CurrencyNotFound { action: &'static str, id: i64 },
    /// Account currency does not match household family base currency.
    CurrencyMismatch {
        action: &'static str,
        family_currency_id: i64,
        account_currency_id: i64,
    },
    /// Target member ID was not found.
    MemberNotFound { action: &'static str, id: i64 },
    /// Target account ID was not found.
    AccountNotFound { action: &'static str, id: i64 },
    /// A family with the given name already exists in the system.
    FamilyAlreadyExists {
        action: &'static str,
        family_name: String,
    },
    /// A member with the given name already exists in this family.
    MemberAlreadyExists {
        action: &'static str,
        member_name: String,
    },
    /// An account with the given name already exists for this member.
    AccountAlreadyExists {
        action: &'static str,
        account_name: String,
    },
}

impl FamilyError {
    /// Returns the globally unique compile-time action token pinpointing the failure site.
    pub fn action(&self) -> &'static str {
        match self {
            Self::EmptyFamilyName { action } => action,
            Self::EmptyMemberName { action } => action,
            Self::InvalidCurrency { action, .. } => action,
            Self::EmptyBankName { action } => action,
            Self::EmptyAccountName { action } => action,
            Self::EmptyCardName { action } => action,
            Self::InvalidLast4 { action } => action,
            Self::InvalidAccountType { action, .. } => action,
            Self::NegativeAmount { action } => action,
            Self::FamilyNotFound { action } => action,
            Self::CurrencyNotFound { action, .. } => action,
            Self::CurrencyMismatch { action, .. } => action,
            Self::MemberNotFound { action, .. } => action,
            Self::AccountNotFound { action, .. } => action,
            Self::FamilyAlreadyExists { action, .. } => action,
            Self::MemberAlreadyExists { action, .. } => action,
            Self::AccountAlreadyExists { action, .. } => action,
        }
    }

    /// Returns the screaming machine-readable status code for the error variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyFamilyName { .. } => "EMPTY_FAMILY_NAME",
            Self::EmptyMemberName { .. } => "EMPTY_MEMBER_NAME",
            Self::InvalidCurrency { .. } => "INVALID_CURRENCY",
            Self::EmptyBankName { .. } => "EMPTY_BANK_NAME",
            Self::EmptyAccountName { .. } => "EMPTY_ACCOUNT_NAME",
            Self::EmptyCardName { .. } => "EMPTY_CARD_NAME",
            Self::InvalidLast4 { .. } => "INVALID_LAST4",
            Self::InvalidAccountType { .. } => "INVALID_ACCOUNT_TYPE",
            Self::NegativeAmount { .. } => "NEGATIVE_AMOUNT",
            Self::FamilyNotFound { .. } => "FAMILY_NOT_FOUND",
            Self::CurrencyNotFound { .. } => "CURRENCY_NOT_FOUND",
            Self::CurrencyMismatch { .. } => "FAMILY_CURRENCY_MISMATCH",
            Self::MemberNotFound { .. } => "MEMBER_NOT_FOUND",
            Self::AccountNotFound { .. } => "ACCOUNT_NOT_FOUND",
            Self::FamilyAlreadyExists { .. } => "FAMILY_ALREADY_EXISTS",
            Self::MemberAlreadyExists { .. } => "MEMBER_ALREADY_EXISTS",
            Self::AccountAlreadyExists { .. } => "ACCOUNT_ALREADY_EXISTS",
        }
    }
}

impl fmt::Display for FamilyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::EmptyFamilyName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptyMemberName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::InvalidCurrency { action, currency } => {
                write!(f, "{code}. ACTION: {action}. Currency: '{currency}'")
            }
            Self::EmptyBankName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptyAccountName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptyCardName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::InvalidLast4 { action } => write!(f, "{code}. ACTION: {action}"),
            Self::InvalidAccountType { action, raw } => {
                write!(f, "{code}. ACTION: {action}. Type: '{raw}'")
            }
            Self::NegativeAmount { action } => write!(f, "{code}. ACTION: {action}"),
            Self::FamilyNotFound { action } => write!(f, "{code}. ACTION: {action}"),
            Self::CurrencyNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Currency ID: {id}")
            }
            Self::CurrencyMismatch {
                action,
                family_currency_id,
                account_currency_id,
            } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. Account currency ID {account_currency_id} does not match household base currency ID {family_currency_id}"
                )
            }
            Self::MemberNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Member ID: {id}")
            }
            Self::AccountNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Account ID: {id}")
            }
            Self::FamilyAlreadyExists {
                action,
                family_name,
            } => {
                write!(f, "{code}. ACTION: {action}. Family: '{family_name}'")
            }
            Self::MemberAlreadyExists {
                action,
                member_name,
            } => {
                write!(f, "{code}. ACTION: {action}. Member: '{member_name}'")
            }
            Self::AccountAlreadyExists {
                action,
                account_name,
            } => {
                write!(f, "{code}. ACTION: {action}. Account: '{account_name}'")
            }
        }
    }
}

impl Error for FamilyError {}

impl IntoResponse for FamilyError {
    fn into_response(self) -> Response {
        let action = self.action();
        let code_str = self.code();

        let (status_code, code, message) = match &self {
            Self::EmptyFamilyName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Family name cannot be empty.".to_string(),
            ),
            Self::EmptyMemberName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Member name cannot be empty.".to_string(),
            ),
            Self::InvalidCurrency { currency, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Currency '{currency}' is not a valid 3-letter ISO-4217 code."),
            ),
            Self::EmptyBankName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Bank institution name cannot be empty.".to_string(),
            ),
            Self::EmptyAccountName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Account name cannot be empty.".to_string(),
            ),
            Self::EmptyCardName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Card name cannot be empty.".to_string(),
            ),
            Self::InvalidLast4 { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Last 4 digits must contain exactly 4 numeric characters.".to_string(),
            ),
            Self::InvalidAccountType { raw, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!(
                    "Account type '{raw}' is invalid. Expected 'bank_account' or 'credit_card'."
                ),
            ),
            Self::NegativeAmount { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Amount cannot be negative.".to_string(),
            ),
            Self::FamilyNotFound { .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                "Family was not found.".to_string(),
            ),
            Self::CurrencyNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Currency with ID {id} was not found."),
            ),
            Self::CurrencyMismatch {
                family_currency_id,
                account_currency_id,
                ..
            } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!(
                    "Account currency ID {account_currency_id} does not match household base currency ID {family_currency_id}."
                ),
            ),
            Self::MemberNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Family member with ID {id} was not found."),
            ),
            Self::AccountNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Account with ID {id} was not found."),
            ),
            Self::FamilyAlreadyExists { family_name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Family '{family_name}' already exists."),
            ),
            Self::MemberAlreadyExists { member_name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Member '{member_name}' already exists in this family."),
            ),
            Self::AccountAlreadyExists { account_name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Account '{account_name}' already exists for this member at this bank."),
            ),
        };

        if status_code.is_server_error() {
            tracing::error!(action = action, code = code_str, error = %self, "request failed");
        } else {
            tracing::warn!(action = action, code = code_str, error = %self, "family request rejected");
        }

        let body = Json(ApiResponse::err(
            code,
            Status::custom(code_str),
            ErrorPayload::new(action, message),
        ));
        (status_code, body).into_response()
    }
}
