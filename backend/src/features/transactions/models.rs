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
    type_id: i64,
    account_id: i64,
    category_id: i64,
    #[serde(default)]
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

    /// Returns the financial account ID.
    pub(crate) fn account_id(&self) -> i64 {
        self.account_id
    }

    /// Returns the mandatory category ID.
    pub(crate) fn category_id(&self) -> i64 {
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
    source: String,
    date: String,
    description: Option<String>,
    payee: Option<String>,
    amount: i64,
    type_id: i64,
    account_id: i64,
    category_id: i64,
    #[serde(default)]
    subcategory_id: Option<i64>,
    notes: Option<String>,
    status: Option<String>,
}

impl UpdateTransactionRequest {
    /// Returns the transaction source ("manual" | "import").
    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    /// Returns the updated date string.
    pub(crate) fn date(&self) -> &str {
        &self.date
    }

    /// Returns the optional updated description.
    pub(crate) fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional updated counterparty/payee.
    pub(crate) fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    /// Returns the updated amount in minor units.
    pub(crate) fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the updated transaction type ID.
    pub(crate) fn type_id(&self) -> i64 {
        self.type_id
    }

    /// Returns the updated account ID.
    pub(crate) fn account_id(&self) -> i64 {
        self.account_id
    }

    /// Returns the updated category ID.
    pub(crate) fn category_id(&self) -> i64 {
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

/// Wire Request DTO for deleting an existing transaction.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteTransactionRequest {
    source: String,
}

impl DeleteTransactionRequest {
    /// Returns the transaction source ("manual" | "import").
    pub(crate) fn source(&self) -> &str {
        &self.source
    }
}

/// Deserializes an optional list of `i64` from either repeated keys (`?account_ids=1&account_ids=2`),
/// a single scalar (`?account_ids=1`), or a comma-separated string (`?account_ids=1,2`).
fn deserialize_optional_vec_i64<'de, D>(deserializer: D) -> Result<Option<Vec<i64>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawList {
        List(Vec<i64>),
        Single(i64),
        Comma(String),
    }

    match Option::<RawList>::deserialize(deserializer)? {
        None => Ok(None),
        Some(RawList::List(items)) => {
            if items.is_empty() {
                Ok(None)
            } else {
                Ok(Some(items))
            }
        }
        Some(RawList::Single(val)) => Ok(Some(vec![val])),
        Some(RawList::Comma(s)) => {
            let parsed: Vec<i64> = s
                .split(',')
                .map(|token| token.trim())
                .filter(|token| !token.is_empty())
                .filter_map(|token| token.parse::<i64>().ok())
                .collect();
            if parsed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(parsed))
            }
        }
    }
}

/// Deserializes an optional list of `String` from repeated keys, single scalar, or comma-separated string.
fn deserialize_optional_vec_string<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawList {
        List(Vec<String>),
        Single(String),
    }

    match Option::<RawList>::deserialize(deserializer)? {
        None => Ok(None),
        Some(RawList::List(items)) => {
            let cleaned: Vec<String> = items
                .into_iter()
                .flat_map(|s| {
                    s.split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect::<Vec<String>>()
                })
                .collect();
            if cleaned.is_empty() {
                Ok(None)
            } else {
                Ok(Some(cleaned))
            }
        }
        Some(RawList::Single(s)) => {
            let cleaned: Vec<String> = s
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            if cleaned.is_empty() {
                Ok(None)
            } else {
                Ok(Some(cleaned))
            }
        }
    }
}

/// Query parameters for filtering, searching, and paginating transactions.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionFilterQuery {
    #[serde(default, alias = "q")]
    query: Option<String>,

    // Pagination
    #[serde(default)]
    page: Option<u32>,
    #[serde(default, alias = "pageSize", alias = "limit")]
    page_size: Option<u32>,

    // Date bounds
    #[serde(default, alias = "from_date", alias = "startDate")]
    from_date: Option<String>,
    #[serde(default, alias = "to_date", alias = "endDate")]
    to_date: Option<String>,

    // Amount bounds (minor units)
    #[serde(default, alias = "min_amount", alias = "minAmount")]
    amount_min: Option<i64>,
    #[serde(default, alias = "max_amount", alias = "maxAmount")]
    amount_max: Option<i64>,

    // Hierarchy of Multi-Criteria Filter Lists
    #[serde(
        default,
        alias = "account_ids",
        alias = "accountIds",
        alias = "account_id",
        alias = "accountId",
        deserialize_with = "deserialize_optional_vec_i64"
    )]
    account_ids: Option<Vec<i64>>,

    #[serde(
        default,
        alias = "type_ids",
        alias = "typeIds",
        alias = "type_id",
        alias = "typeId",
        deserialize_with = "deserialize_optional_vec_i64"
    )]
    type_ids: Option<Vec<i64>>,

    #[serde(
        default,
        alias = "category_ids",
        alias = "categoryIds",
        alias = "category_id",
        alias = "categoryId",
        deserialize_with = "deserialize_optional_vec_i64"
    )]
    category_ids: Option<Vec<i64>>,

    #[serde(
        default,
        alias = "subcategory_ids",
        alias = "subcategoryIds",
        alias = "subcategory_id",
        alias = "subcategoryId",
        deserialize_with = "deserialize_optional_vec_i64"
    )]
    subcategory_ids: Option<Vec<i64>>,

    #[serde(
        default,
        alias = "statuses",
        alias = "status",
        deserialize_with = "deserialize_optional_vec_string"
    )]
    statuses: Option<Vec<String>>,
}

impl TransactionFilterQuery {
    /// Parses an HTTP query string (e.g. `q=foo&account_ids=1&account_ids=2` or `page=2&pageSize=20`)
    /// gracefully handling repeated parameters, alias mappings, and comma-separated values.
    pub(crate) fn from_query_str(query_str: &str) -> Self {
        let mut query = None;
        let mut page = None;
        let mut page_size = None;
        let mut from_date = None;
        let mut to_date = None;
        let mut amount_min = None;
        let mut amount_max = None;
        let mut account_ids: Vec<i64> = Vec::new();
        let mut type_ids: Vec<i64> = Vec::new();
        let mut category_ids: Vec<i64> = Vec::new();
        let mut subcategory_ids: Vec<i64> = Vec::new();
        let mut statuses: Vec<String> = Vec::new();

        for pair in query_str.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim();
            if key.is_empty() {
                continue;
            }

            match key {
                "q" | "query" => query = Some(val.to_string()),
                "page" => page = val.parse::<u32>().ok(),
                "pageSize" | "page_size" | "limit" => page_size = val.parse::<u32>().ok(),
                "from_date" | "fromDate" | "startDate" => from_date = Some(val.to_string()),
                "to_date" | "toDate" | "endDate" => to_date = Some(val.to_string()),
                "min_amount" | "minAmount" | "amount_min" => {
                    amount_min = val.parse::<i64>().ok();
                }
                "max_amount" | "maxAmount" | "amount_max" => {
                    amount_max = val.parse::<i64>().ok();
                }
                "account_ids" | "accountIds" | "account_id" | "accountId" => {
                    for token in val.split(',') {
                        if let Ok(id) = token.trim().parse::<i64>() {
                            account_ids.push(id);
                        }
                    }
                }
                "type_ids" | "typeIds" | "type_id" | "typeId" => {
                    for token in val.split(',') {
                        if let Ok(id) = token.trim().parse::<i64>() {
                            type_ids.push(id);
                        }
                    }
                }
                "category_ids" | "categoryIds" | "category_id" | "categoryId" => {
                    for token in val.split(',') {
                        if let Ok(id) = token.trim().parse::<i64>() {
                            category_ids.push(id);
                        }
                    }
                }
                "subcategory_ids" | "subcategoryIds" | "subcategory_id" | "subcategoryId" => {
                    for token in val.split(',') {
                        if let Ok(id) = token.trim().parse::<i64>() {
                            subcategory_ids.push(id);
                        }
                    }
                }
                "statuses" | "status" => {
                    for token in val.split(',') {
                        let t = token.trim();
                        if !t.is_empty() {
                            statuses.push(t.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        Self {
            query,
            page,
            page_size,
            from_date,
            to_date,
            amount_min,
            amount_max,
            account_ids: if account_ids.is_empty() {
                None
            } else {
                Some(account_ids)
            },
            type_ids: if type_ids.is_empty() {
                None
            } else {
                Some(type_ids)
            },
            category_ids: if category_ids.is_empty() {
                None
            } else {
                Some(category_ids)
            },
            subcategory_ids: if subcategory_ids.is_empty() {
                None
            } else {
                Some(subcategory_ids)
            },
            statuses: if statuses.is_empty() {
                None
            } else {
                Some(statuses)
            },
        }
    }

    /// Returns the optional free-text search query.
    pub(crate) fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Returns the requested page number, defaulting to 1 (1-indexed).
    pub(crate) fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    /// Returns the requested page size, defaulting to 20, capped between 1 and 100.
    pub(crate) fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }

    /// Returns the SQL `OFFSET` calculated as `(page - 1) * page_size`.
    pub(crate) fn offset(&self) -> u32 {
        (self.page() - 1) * self.page_size()
    }

    /// Returns the optional start date filter string.
    pub(crate) fn start_date(&self) -> Option<&str> {
        self.from_date.as_deref()
    }

    /// Returns the optional end date filter string.
    pub(crate) fn to_date(&self) -> Option<&str> {
        self.to_date.as_deref()
    }

    /// Returns the optional minimum transaction amount bound.
    pub(crate) fn amount_min(&self) -> Option<i64> {
        self.amount_min
    }

    /// Returns the optional maximum transaction amount bound.
    pub(crate) fn amount_max(&self) -> Option<i64> {
        self.amount_max
    }

    /// Returns the optional list of filtered account IDs.
    pub(crate) fn account_ids(&self) -> Option<&[i64]> {
        self.account_ids.as_deref()
    }

    /// Returns the optional list of filtered transaction type IDs.
    pub(crate) fn type_ids(&self) -> Option<&[i64]> {
        self.type_ids.as_deref()
    }

    /// Returns the optional list of filtered category IDs.
    pub(crate) fn category_ids(&self) -> Option<&[i64]> {
        self.category_ids.as_deref()
    }

    /// Returns the optional list of filtered subcategory IDs.
    pub(crate) fn subcategory_ids(&self) -> Option<&[i64]> {
        self.subcategory_ids.as_deref()
    }

    /// Returns the optional list of filtered settlement statuses.
    pub(crate) fn statuses(&self) -> Option<&[String]> {
        self.statuses.as_deref()
    }
}

// ============================================================================
// Wire Response DTOs
// ============================================================================

/// Wire Response DTO representing a single transaction in the master ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionDto {
    id: String,
    source: String,
    date: String,
    description: String,
    payee: Option<String>,
    amount: i64,
    type_id: i64,
    account_id: i64,
    category_id: i64,
    subcategory_id: Option<i64>,
    notes: Option<String>,
    status: String,
}

impl TransactionDto {
    /// Constructs a new [`TransactionDto`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: String,
        source: String,
        date: String,
        description: String,
        payee: Option<String>,
        amount: i64,
        type_id: i64,
        account_id: i64,
        category_id: i64,
        subcategory_id: Option<i64>,
        notes: Option<String>,
        status: String,
    ) -> Self {
        Self {
            id,
            source,
            date,
            description,
            payee,
            amount,
            type_id,
            account_id,
            category_id,
            subcategory_id,
            notes,
            status,
        }
    }

    /// Returns the transaction ID.
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    /// Returns the transaction source ("manual" | "import").
    #[cfg(test)]
    pub(crate) fn source(&self) -> &str {
        &self.source
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
    pub(crate) fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    /// Returns the amount in minor units.
    pub(crate) fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the settlement status.
    #[cfg(test)]
    pub(crate) fn status(&self) -> &str {
        &self.status
    }
}

/// Paginated list response DTO containing sliced items and pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaginatedTransactionsDto {
    items: Vec<TransactionDto>,
    total_count: i64,
    page: u32,
    page_size: u32,
    total_pages: u32,
}

impl PaginatedTransactionsDto {
    /// Constructs a new [`PaginatedTransactionsDto`].
    pub(crate) fn new(
        items: Vec<TransactionDto>,
        total_count: i64,
        page: u32,
        page_size: u32,
        total_pages: u32,
    ) -> Self {
        Self {
            items,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }

    /// Returns a slice of transactions for the current page.
    pub(crate) fn items(&self) -> &[TransactionDto] {
        &self.items
    }

    /// Returns the total matching record count across all pages.
    pub(crate) fn total_count(&self) -> i64 {
        self.total_count
    }

    /// Returns the current page number (1-indexed).
    pub(crate) fn page(&self) -> u32 {
        self.page
    }

    /// Returns the maximum items per page.
    pub(crate) fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Returns the total number of pages.
    pub(crate) fn total_pages(&self) -> u32 {
        self.total_pages
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

    #[test]
    fn test_transaction_filter_query_pagination_defaults() {
        let q: TransactionFilterQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.page(), 1);
        assert_eq!(q.page_size(), 20);
        assert_eq!(q.offset(), 0);
        assert!(q.account_ids().is_none());
        assert!(q.type_ids().is_none());
        assert!(q.category_ids().is_none());
        assert!(q.subcategory_ids().is_none());
        assert!(q.statuses().is_none());

        let q2: TransactionFilterQuery =
            serde_json::from_str(r#"{"page": 3, "pageSize": 50}"#).unwrap();
        assert_eq!(q2.page(), 3);
        assert_eq!(q2.page_size(), 50);
        assert_eq!(q2.offset(), 100);

        // Clamp upper bound
        let q_clamp: TransactionFilterQuery =
            serde_json::from_str(r#"{"page": 0, "pageSize": 500}"#).unwrap();
        assert_eq!(q_clamp.page(), 1);
        assert_eq!(q_clamp.page_size(), 100);
    }

    #[test]
    fn test_transaction_filter_query_list_deserialization() {
        // Comma-separated strings
        let q1: TransactionFilterQuery = serde_json::from_str(
            r#"{
                "accountIds": "1, 2, 3",
                "typeIds": "10,20",
                "categoryIds": "100",
                "statuses": "cleared, pending"
            }"#,
        )
        .unwrap();
        assert_eq!(q1.account_ids(), Some(&[1, 2, 3][..]));
        assert_eq!(q1.type_ids(), Some(&[10, 20][..]));
        assert_eq!(q1.category_ids(), Some(&[100][..]));
        assert_eq!(
            q1.statuses(),
            Some(&["cleared".to_string(), "pending".to_string()][..])
        );

        // JSON array format
        let q2: TransactionFilterQuery = serde_json::from_str(
            r#"{
                "account_ids": [4, 5],
                "statuses": ["cleared"]
            }"#,
        )
        .unwrap();
        assert_eq!(q2.account_ids(), Some(&[4, 5][..]));
        assert_eq!(q2.statuses(), Some(&["cleared".to_string()][..]));

        // Single integer scalar
        let q3: TransactionFilterQuery =
            serde_json::from_str(r#"{"account_id": 9, "status": "pending"}"#).unwrap();
        assert_eq!(q3.account_ids(), Some(&[9][..]));
        assert_eq!(q3.statuses(), Some(&["pending".to_string()][..]));
    }

    #[test]
    fn test_paginated_transactions_dto_math() {
        let dto = PaginatedTransactionsDto::new(vec![], 55, 2, 20, 3);
        assert_eq!(dto.total_count(), 55);
        assert_eq!(dto.page(), 2);
        assert_eq!(dto.page_size(), 20);
        assert_eq!(dto.total_pages(), 3);
        assert!(dto.items().is_empty());
    }
}
