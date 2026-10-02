//! Family and accounts domain models, Value Objects, and wire DTOs.
//!
//! Enforces parse-don't-validate invariants at the domain boundary:
//! - All struct fields are private and accessed via getters or moved via `into_inner()`.
//! - Money is strictly represented as integer cents ([`AmountCents`]).
//! - Currencies are strictly uppercase 3-letter ISO-4217 codes ([`CurrencyCode`]).
//! - Last 4 digits are strictly 4 numeric characters ([`Last4`]).

use serde::{Deserialize, Serialize};

use super::error::FamilyError;

// ============================================================================
// Value Objects
// ============================================================================

/// Validated family name value object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FamilyName(String);

impl FamilyName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(FamilyError::EmptyFamilyName { action });
        }
        Ok(Self(trimmed))
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated member display name value object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberName(String);

impl MemberName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(FamilyError::EmptyMemberName { action });
        }
        Ok(Self(trimmed))
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated ISO-4217 3-letter uppercase currency code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CurrencyCode(String);

impl CurrencyCode {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_uppercase();
        if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(FamilyError::InvalidCurrency {
                action,
                currency: trimmed,
            });
        }
        Ok(Self(trimmed))
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated bank institution name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BankName(String);

impl BankName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(FamilyError::EmptyBankName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated 4-digit account identification string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Last4(String);

impl Last4 {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.len() != 4 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err(FamilyError::InvalidLast4 { action });
        }
        Ok(Self(trimmed))
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated depository bank account name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccountName(String);

impl AccountName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(FamilyError::EmptyAccountName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated credit card name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CardName(String);

impl CardName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, FamilyError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(FamilyError::EmptyCardName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated non-negative financial amount in integer minor units (cents).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AmountCents(i64);

impl AmountCents {
    pub(crate) fn try_new(cents: i64, action: &'static str) -> Result<Self, FamilyError> {
        if cents < 0 {
            return Err(FamilyError::NegativeAmount { action });
        }
        Ok(Self(cents))
    }

    pub(crate) fn get(&self) -> i64 {
        self.0
    }
}

// ============================================================================
// Wire Request DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateFamilyRequest {
    name: Option<String>,
    currency: Option<String>,
}

impl UpdateFamilyRequest {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateMemberRequest {
    family_id: i64,
    name: String,
}

impl CreateMemberRequest {
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateMemberRequest {
    name: String,
}

impl UpdateMemberRequest {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateBankAccountRequest {
    family_id: i64,
    owner_member_id: i64,
    currency: String,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
}

impl CreateBankAccountRequest {
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    pub(crate) fn owner_member_id(&self) -> i64 {
        self.owner_member_id
    }

    pub(crate) fn currency(&self) -> &str {
        &self.currency
    }

    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    pub(crate) fn account_name(&self) -> &str {
        &self.account_name
    }

    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    pub(crate) fn available_balance_cents(&self) -> i64 {
        self.available_balance_cents
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateBankAccountRequest {
    currency: Option<String>,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
}

impl UpdateBankAccountRequest {
    pub(crate) fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }

    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    pub(crate) fn account_name(&self) -> &str {
        &self.account_name
    }

    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    pub(crate) fn available_balance_cents(&self) -> i64 {
        self.available_balance_cents
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateCreditCardRequest {
    family_id: i64,
    owner_member_id: i64,
    currency: String,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
}

impl CreateCreditCardRequest {
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    pub(crate) fn owner_member_id(&self) -> i64 {
        self.owner_member_id
    }

    pub(crate) fn currency(&self) -> &str {
        &self.currency
    }

    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    pub(crate) fn card_name(&self) -> &str {
        &self.card_name
    }

    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    pub(crate) fn credit_limit_cents(&self) -> i64 {
        self.credit_limit_cents
    }

    pub(crate) fn available_cents(&self) -> i64 {
        self.available_cents
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateCreditCardRequest {
    currency: Option<String>,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
}

impl UpdateCreditCardRequest {
    pub(crate) fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }

    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    pub(crate) fn card_name(&self) -> &str {
        &self.card_name
    }

    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    pub(crate) fn credit_limit_cents(&self) -> i64 {
        self.credit_limit_cents
    }

    pub(crate) fn available_cents(&self) -> i64 {
        self.available_cents
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DefaultCurrencyQuery {
    region: Option<String>,
}

impl DefaultCurrencyQuery {
    pub(crate) fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}

// ============================================================================
// Wire Response DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FamilyDto {
    id: i64,
    name: String,
    currency: String,
    created_at: i64,
}

impl FamilyDto {
    pub(crate) fn new(
        id: i64,
        name: impl Into<String>,
        currency: impl Into<String>,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            currency: currency.into(),
            created_at,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn currency(&self) -> &str {
        &self.currency
    }

    pub(crate) fn created_at(&self) -> i64 {
        self.created_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MemberDto {
    id: i64,
    family_id: i64,
    name: String,
    created_at: i64,
}

impl MemberDto {
    pub(crate) fn new(id: i64, family_id: i64, name: impl Into<String>, created_at: i64) -> Self {
        Self {
            id,
            family_id,
            name: name.into(),
            created_at,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BankAccountDto {
    id: i64,
    family_id: i64,
    owner_member_id: i64,
    #[serde(rename = "type")]
    account_type: String,
    currency: String,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
    created_at: i64,
}

impl BankAccountDto {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: i64,
        family_id: i64,
        owner_member_id: i64,
        currency: impl Into<String>,
        bank_name: impl Into<String>,
        account_name: impl Into<String>,
        last4: impl Into<String>,
        available_balance_cents: i64,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            family_id,
            owner_member_id,
            account_type: "bank_account".to_string(),
            currency: currency.into(),
            bank_name: bank_name.into(),
            account_name: account_name.into(),
            last4: last4.into(),
            available_balance_cents,
            created_at,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CreditCardDto {
    id: i64,
    family_id: i64,
    owner_member_id: i64,
    #[serde(rename = "type")]
    account_type: String,
    currency: String,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
    outstanding_cents: i64,
    created_at: i64,
}

impl CreditCardDto {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: i64,
        family_id: i64,
        owner_member_id: i64,
        currency: impl Into<String>,
        bank_name: impl Into<String>,
        card_name: impl Into<String>,
        last4: impl Into<String>,
        credit_limit_cents: i64,
        available_cents: i64,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            family_id,
            owner_member_id,
            account_type: "credit_card".to_string(),
            currency: currency.into(),
            bank_name: bank_name.into(),
            card_name: card_name.into(),
            last4: last4.into(),
            credit_limit_cents,
            available_cents,
            outstanding_cents: credit_limit_cents.saturating_sub(available_cents),
            created_at,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum AccountDto {
    Bank(BankAccountDto),
    Credit(CreditCardDto),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FamilyDetailsDto {
    family: FamilyDto,
    members: Vec<MemberDto>,
    accounts: Vec<AccountDto>,
}

impl FamilyDetailsDto {
    pub(crate) fn new(
        family: FamilyDto,
        members: Vec<MemberDto>,
        accounts: Vec<AccountDto>,
    ) -> Self {
        Self {
            family,
            members,
            accounts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DefaultCurrencyDto {
    currency: String,
}

impl DefaultCurrencyDto {
    pub(crate) fn new(currency: impl Into<String>) -> Self {
        Self {
            currency: currency.into(),
        }
    }
}

// ============================================================================
// Tier 1 Pure Domain Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_name_validation() {
        assert!(FamilyName::try_new("  ", "TEST").is_err());
        assert!(FamilyName::try_new("", "TEST").is_err());
        let valid = FamilyName::try_new("  The Miller Family  ", "TEST").unwrap();
        assert_eq!(valid.as_str(), "The Miller Family");
        assert_eq!(valid.into_inner(), "The Miller Family");
    }

    #[test]
    fn test_member_name_validation() {
        assert!(MemberName::try_new("  ", "TEST").is_err());
        assert!(MemberName::try_new("", "TEST").is_err());
        let valid = MemberName::try_new("  Sarah Miller ", "TEST").unwrap();
        assert_eq!(valid.as_str(), "Sarah Miller");
    }

    #[test]
    fn test_currency_code_validation() {
        assert!(CurrencyCode::try_new("", "TEST").is_err());
        assert!(CurrencyCode::try_new("US", "TEST").is_err());
        assert!(CurrencyCode::try_new("USDD", "TEST").is_err());
        assert!(CurrencyCode::try_new("123", "TEST").is_err());
        let inr = CurrencyCode::try_new("inr", "TEST").unwrap();
        assert_eq!(inr.as_str(), "INR");
        let usd = CurrencyCode::try_new("USD", "TEST").unwrap();
        assert_eq!(usd.as_str(), "USD");
    }

    #[test]
    fn test_last4_validation() {
        assert!(Last4::try_new("", "TEST").is_err());
        assert!(Last4::try_new("123", "TEST").is_err());
        assert!(Last4::try_new("12345", "TEST").is_err());
        assert!(Last4::try_new("abcd", "TEST").is_err());
        let valid = Last4::try_new("  4821  ", "TEST").unwrap();
        assert_eq!(valid.as_str(), "4821");
    }

    #[test]
    fn test_amount_cents_validation() {
        assert!(AmountCents::try_new(-1, "TEST").is_err());
        assert!(AmountCents::try_new(-100, "TEST").is_err());
        let zero = AmountCents::try_new(0, "TEST").unwrap();
        assert_eq!(zero.get(), 0);
        let pos = AmountCents::try_new(250000, "TEST").unwrap();
        assert_eq!(pos.get(), 250000);
    }
}
