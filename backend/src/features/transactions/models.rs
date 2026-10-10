//! Transactions domain models, Value Objects, and wire Data Transfer Objects.
//!
//! Enforces parse-don't-validate invariants at the domain boundary:
//! - All struct fields are private and accessed via reference getters.
//! - Monetary values are stored and computed strictly as integer minor units ([`TransactionAmount`]).
//! - Dates are validated against the ISO calendar and represented consistently as UTC epoch seconds ([`TransactionDate`]).
//! - Inbound request DTOs reject unknown fields via `#[serde(deny_unknown_fields)]`.

use serde::{Deserialize, Serialize};

use super::error::TransactionError;

// ============================================================================
// Value Objects & Pure Calendar Math
// ============================================================================

/// Validated non-empty transaction description Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransactionDescription(String);

impl TransactionDescription {
    /// Validates and trims a raw description string.
    ///
    /// # Errors
    /// Returns [`TransactionError::EmptyDescription`] if the trimmed description is empty.
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, TransactionError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(TransactionError::EmptyDescription { action });
        }
        Ok(Self(trimmed))
    }

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }

    /// Returns a string slice reference to the validated description.
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated transaction counterparty/payee Value Object.
///
/// Trims surrounding whitespace and converts empty strings to `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransactionPayee(Option<String>);

impl TransactionPayee {
    /// Trims the input and normalizes empty or whitespace strings to `None`.
    pub(crate) fn new(raw: Option<impl Into<String>>) -> Self {
        let cleaned = raw.and_then(|r| {
            let trimmed = r.into().trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        Self(cleaned)
    }

    /// Consumes the wrapper, returning the inner `Option<String>`.
    pub(crate) fn into_inner(self) -> Option<String> {
        self.0
    }
}

/// Howard Hinnant's civil day algorithm: converts civil calendar (year, month, day) to days since UTC epoch (1970-01-01).
fn civil_to_epoch_days(y: i32, m: u32, d: u32) -> i64 {
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era as i64) * 146097 + (doe as i64) - 719468
}

/// Howard Hinnant's civil day algorithm: converts days since UTC epoch (1970-01-01) to civil calendar (year, month, day).
fn epoch_days_to_civil(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

/// Validated transaction date Value Object representing UTC epoch seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TransactionDate(i64);

impl TransactionDate {
    /// Creates a [`TransactionDate`] from raw UTC epoch seconds.
    pub(crate) fn from_epoch_secs(secs: i64) -> Self {
        Self(secs)
    }

    /// Parses an ISO date string (`YYYY-MM-DD`) into UTC epoch seconds (midnight UTC).
    ///
    /// # Errors
    /// Returns [`TransactionError::InvalidDate`] if the date cannot be parsed or values are out of range.
    pub(crate) fn try_from_iso(raw: &str, action: &'static str) -> Result<Self, TransactionError> {
        let trimmed = raw.trim();
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() != 3 {
            return Err(TransactionError::InvalidDate {
                action,
                raw: raw.to_string(),
            });
        }

        let year: i32 = parts[0]
            .parse()
            .map_err(|_| TransactionError::InvalidDate {
                action,
                raw: raw.to_string(),
            })?;
        let month: u32 = parts[1]
            .parse()
            .map_err(|_| TransactionError::InvalidDate {
                action,
                raw: raw.to_string(),
            })?;
        let day: u32 = parts[2]
            .parse()
            .map_err(|_| TransactionError::InvalidDate {
                action,
                raw: raw.to_string(),
            })?;

        if !(1..=12).contains(&month) || !(1..=31).contains(&day) || !(1900..=2200).contains(&year)
        {
            return Err(TransactionError::InvalidDate {
                action,
                raw: raw.to_string(),
            });
        }

        let days = civil_to_epoch_days(year, month, day);
        let secs = days * 86_400;
        Ok(Self(secs))
    }

    /// Returns the raw integer value in UTC epoch seconds.
    pub(crate) fn epoch_secs(&self) -> i64 {
        self.0
    }

    /// Formats the epoch seconds into an ISO date string (`YYYY-MM-DD`).
    pub(crate) fn to_iso_date(self) -> String {
        let days = self.0.div_euclid(86_400);
        let (y, m, d) = epoch_days_to_civil(days);
        format!("{y:04}-{m:02}-{d:02}")
    }
}

/// Validated non-zero monetary amount in integer minor units Value Object.
///
/// Transactions represent real fund movements; zero amounts are forbidden.
/// Positive quantities indicate income/inflows; negative quantities indicate expenses/outflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TransactionAmount(i64);

impl TransactionAmount {
    /// Validates that the monetary amount in minor units is non-zero.
    ///
    /// # Errors
    /// Returns [`TransactionError::ZeroAmount`] if `minor_units == 0`.
    pub(crate) fn try_new(
        minor_units: i64,
        action: &'static str,
    ) -> Result<Self, TransactionError> {
        if minor_units == 0 {
            return Err(TransactionError::ZeroAmount { action });
        }
        Ok(Self(minor_units))
    }

    /// Returns the raw integer value in minor units.
    pub(crate) fn get(&self) -> i64 {
        self.0
    }
}

/// Validated settlement status of a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransactionStatus {
    /// Fully settled and finalized.
    Cleared,
    /// Pending authorization or settlement.
    Pending,
}

impl TransactionStatus {
    /// Parses a raw status string into a [`TransactionStatus`].
    pub(crate) fn try_new(raw: &str, action: &'static str) -> Result<Self, TransactionError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cleared" => Ok(Self::Cleared),
            "pending" => Ok(Self::Pending),
            _ => Err(TransactionError::InvalidStatus {
                action,
                raw: raw.to_string(),
            }),
        }
    }

    /// Returns the lowercase string representation of the status.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Pending => "pending",
        }
    }
}

/// Validated source origin discriminator of a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransactionSourceType {
    /// Ingested from an imported statement file.
    Import,
    /// Recorded manually by a user.
    Manual,
}

impl TransactionSourceType {
    /// Parses a raw source type string into a [`TransactionSourceType`].
    pub(crate) fn try_new(raw: &str, action: &'static str) -> Result<Self, TransactionError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "import" => Ok(Self::Import),
            "manual" => Ok(Self::Manual),
            _ => Err(TransactionError::InvalidSourceType {
                action,
                raw: raw.to_string(),
            }),
        }
    }

    /// Returns the lowercase string representation of the source type.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Manual => "manual",
        }
    }
}

// ============================================================================
// Wire Request DTOs
// ============================================================================

/// Wire Request DTO for creating a manual transaction.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTransactionRequest {
    date: String,
    description: Option<String>,
    payee: Option<String>,
    amount: i64,
    #[serde(alias = "type_id")]
    type_id: i64,
    #[serde(default)]
    _type: Option<String>,
    #[serde(default, alias = "type_color")]
    _type_color: Option<String>,
    #[serde(default, alias = "member_id")]
    member_id: Option<i64>,
    #[serde(alias = "account_id")]
    account_id: i64,
    #[serde(default, alias = "category_id")]
    category_id: Option<i64>,
    #[serde(default, alias = "subcategory_id")]
    subcategory_id: Option<i64>,
    notes: Option<String>,
    status: Option<String>,
}

impl CreateTransactionRequest {
    /// Returns the inbound date string (`YYYY-MM-DD`).
    pub(crate) fn date(&self) -> &str {
        &self.date
    }

    /// Returns the optional transaction description.
    pub(crate) fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional counterparty/payee.
    pub(crate) fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    /// Returns the monetary amount in integer minor units.
    pub(crate) fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the transaction type ID.
    pub(crate) fn type_id(&self) -> i64 {
        self.type_id
    }

    /// Returns the optional attributing member ID.
    pub(crate) fn member_id(&self) -> Option<i64> {
        self.member_id
    }

    /// Returns the financial account ID.
    pub(crate) fn account_id(&self) -> i64 {
        self.account_id
    }

    /// Returns the optional category ID.
    pub(crate) fn category_id(&self) -> Option<i64> {
        self.category_id
    }

    /// Returns the optional subcategory ID.
    pub(crate) fn subcategory_id(&self) -> Option<i64> {
        self.subcategory_id
    }

    /// Returns the optional user notes.
    pub(crate) fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    /// Returns the optional status string.
    pub(crate) fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

/// Wire Request DTO for modifying an existing transaction.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateTransactionRequest {
    date: Option<String>,
    description: Option<String>,
    payee: Option<String>,
    amount: Option<i64>,
    #[serde(default, alias = "type_id")]
    type_id: Option<i64>,
    #[serde(default)]
    _type: Option<String>,
    #[serde(default, alias = "type_color")]
    _type_color: Option<String>,
    #[serde(default, alias = "member_id")]
    member_id: Option<i64>,
    #[serde(default, alias = "account_id")]
    account_id: Option<i64>,
    #[serde(default, alias = "category_id")]
    category_id: Option<i64>,
    #[serde(default, alias = "subcategory_id")]
    subcategory_id: Option<i64>,
    notes: Option<String>,
    status: Option<String>,
}

impl UpdateTransactionRequest {
    /// Returns the optional updated date string.
    pub(crate) fn date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    /// Returns the optional updated description.
    pub(crate) fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional updated counterparty/payee.
    pub(crate) fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    /// Returns the optional updated amount in minor units.
    pub(crate) fn amount(&self) -> Option<i64> {
        self.amount
    }

    /// Returns the optional updated transaction type ID.
    pub(crate) fn type_id(&self) -> Option<i64> {
        self.type_id
    }

    /// Returns the optional updated attributing member ID.
    pub(crate) fn member_id(&self) -> Option<i64> {
        self.member_id
    }

    /// Returns the optional updated account ID.
    pub(crate) fn account_id(&self) -> Option<i64> {
        self.account_id
    }

    /// Returns the optional updated category ID.
    pub(crate) fn category_id(&self) -> Option<i64> {
        self.category_id
    }

    /// Returns the optional updated subcategory ID.
    pub(crate) fn subcategory_id(&self) -> Option<i64> {
        self.subcategory_id
    }

    /// Returns the optional updated user notes.
    pub(crate) fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    /// Returns the optional updated settlement status.
    pub(crate) fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

/// Query parameters for filtering and searching transactions.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionFilterQuery {
    #[serde(default, alias = "q")]
    query: Option<String>,
    #[serde(default, alias = "from_date", alias = "startDate")]
    from_date: Option<String>,
    #[serde(default, alias = "to_date", alias = "endDate")]
    to_date: Option<String>,
    #[serde(default, alias = "type_id")]
    type_id: Option<i64>,
    #[serde(default, alias = "type")]
    type_name: Option<String>,
    #[serde(default, alias = "member_id")]
    member_id: Option<i64>,
    #[serde(default, alias = "account_id")]
    account_id: Option<i64>,
    #[serde(default, alias = "category_id")]
    category_id: Option<i64>,
    #[serde(default, alias = "subcategory_id")]
    subcategory_id: Option<i64>,
    #[serde(default)]
    status: Option<String>,
}

impl TransactionFilterQuery {
    /// Returns the optional free-text search query.
    pub(crate) fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Returns the optional start date filter string.
    pub(crate) fn start_date(&self) -> Option<&str> {
        self.from_date.as_deref()
    }

    /// Returns the optional end date filter string.
    pub(crate) fn to_date(&self) -> Option<&str> {
        self.to_date.as_deref()
    }

    /// Returns the optional type ID filter.
    pub(crate) fn type_id(&self) -> Option<i64> {
        self.type_id
    }

    /// Returns the optional type name filter.
    pub(crate) fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// Returns the optional member ID filter.
    pub(crate) fn member_id(&self) -> Option<i64> {
        self.member_id
    }

    /// Returns the optional account ID filter.
    pub(crate) fn account_id(&self) -> Option<i64> {
        self.account_id
    }

    /// Returns the optional category ID filter.
    pub(crate) fn category_id(&self) -> Option<i64> {
        self.category_id
    }

    /// Returns the optional subcategory ID filter.
    pub(crate) fn subcategory_id(&self) -> Option<i64> {
        self.subcategory_id
    }

    /// Returns the optional settlement status filter.
    pub(crate) fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

// ============================================================================
// Wire Response DTOs
// ============================================================================

/// Wire Response DTO representing a single transaction in the master ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionDto {
    id: i64,
    date: String,
    description: String,
    payee: String,
    amount: i64,
    type_id: Option<i64>,
    #[serde(rename = "type")]
    type_name: Option<String>,
    type_color: Option<String>,
    member_id: Option<i64>,
    account_id: Option<i64>,
    category_id: Option<i64>,
    subcategory_id: Option<i64>,
    notes: Option<String>,
    status: String,
}

impl TransactionDto {
    /// Constructs a new [`TransactionDto`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: i64,
        date: String,
        description: String,
        payee: String,
        amount: i64,
        type_id: Option<i64>,
        type_name: Option<String>,
        type_color: Option<String>,
        member_id: Option<i64>,
        account_id: Option<i64>,
        category_id: Option<i64>,
        subcategory_id: Option<i64>,
        notes: Option<String>,
        status: String,
    ) -> Self {
        Self {
            id,
            date,
            description,
            payee,
            amount,
            type_id,
            type_name,
            type_color,
            member_id,
            account_id,
            category_id,
            subcategory_id,
            notes,
            status,
        }
    }

    /// Returns the transaction ID.
    #[cfg(test)]
    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the ISO date string.
    #[cfg(test)]
    pub(crate) fn date(&self) -> &str {
        &self.date
    }

    /// Returns the transaction description.
    #[cfg(test)]
    pub(crate) fn description(&self) -> &str {
        &self.description
    }

    /// Returns the counterparty/payee.
    #[cfg(test)]
    pub(crate) fn payee(&self) -> &str {
        &self.payee
    }

    /// Returns the amount in minor units.
    #[cfg(test)]
    pub(crate) fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the settlement status.
    #[cfg(test)]
    pub(crate) fn status(&self) -> &str {
        &self.status
    }
}

// ============================================================================
// Tier 1 Pure Domain Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_trimming_and_validation() {
        let desc = TransactionDescription::try_new("  Whole Foods  ", "TEST.ACTION")
            .expect("Valid description");
        assert_eq!(desc.as_str(), "Whole Foods");

        let err = TransactionDescription::try_new("   ", "TEST.ACTION")
            .expect_err("Empty description must fail");
        assert_eq!(err.code(), "EMPTY_TRANSACTION_DESCRIPTION");
        assert_eq!(err.action(), "TEST.ACTION");
    }

    #[test]
    fn test_payee_normalization() {
        let p1 = TransactionPayee::new(Some(" Blue Bottle ")).into_inner();
        assert_eq!(p1.as_deref(), Some("Blue Bottle"));

        let p2 = TransactionPayee::new(Some("   ")).into_inner();
        assert_eq!(p2, None);

        let p3 = TransactionPayee::new(None::<String>).into_inner();
        assert_eq!(p3, None);
    }

    #[test]
    fn test_date_iso_roundtrip_and_validation() {
        // Epoch anchor: 1970-01-01
        let anchor = TransactionDate::try_from_iso("1970-01-01", "TEST.DATE").unwrap();
        assert_eq!(anchor.epoch_secs(), 0);
        assert_eq!(anchor.to_iso_date(), "1970-01-01");

        // Target test date: 2026-10-05
        let d = TransactionDate::try_from_iso("2026-10-05", "TEST.DATE").unwrap();
        assert_eq!(d.to_iso_date(), "2026-10-05");

        // Invalid format tests
        assert!(TransactionDate::try_from_iso("invalid", "TEST.DATE").is_err());
        assert!(TransactionDate::try_from_iso("2026-13-01", "TEST.DATE").is_err());
        assert!(TransactionDate::try_from_iso("2026-00-01", "TEST.DATE").is_err());
        assert!(TransactionDate::try_from_iso("2026-05-32", "TEST.DATE").is_err());
    }

    #[test]
    fn test_amount_validation() {
        let income = TransactionAmount::try_new(500000, "TEST.AMOUNT").expect("Positive amount");
        assert_eq!(income.get(), 500000);

        let expense = TransactionAmount::try_new(-8420, "TEST.AMOUNT").expect("Negative amount");
        assert_eq!(expense.get(), -8420);

        let err = TransactionAmount::try_new(0, "TEST.AMOUNT").expect_err("Zero amount must fail");
        assert_eq!(err.code(), "ZERO_TRANSACTION_AMOUNT");
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(
            TransactionStatus::try_new("cleared", "TEST.STATUS").unwrap(),
            TransactionStatus::Cleared
        );
        assert_eq!(
            TransactionStatus::try_new("PENDING", "TEST.STATUS").unwrap(),
            TransactionStatus::Pending
        );
        assert!(TransactionStatus::try_new("unknown", "TEST.STATUS").is_err());
    }

    #[test]
    fn test_source_type_parsing() {
        assert_eq!(
            TransactionSourceType::try_new("import", "TEST.SOURCE").unwrap(),
            TransactionSourceType::Import
        );
        assert_eq!(
            TransactionSourceType::try_new("MANUAL", "TEST.SOURCE").unwrap(),
            TransactionSourceType::Manual
        );
        assert!(TransactionSourceType::try_new("invalid", "TEST.SOURCE").is_err());
    }
}
