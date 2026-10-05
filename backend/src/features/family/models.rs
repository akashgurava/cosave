//! Family and accounts domain models, Value Objects, and wire DTOs.
//!
//! Enforces parse-don't-validate invariants at the domain boundary:
//! - All struct fields are private and accessed via getters or moved via `into_inner()`.
//! - Money is strictly represented as integer cents ([`AmountCents`]).
//! - Last 4 digits are strictly 4 numeric characters ([`Last4`]).
//!
//! # Architecture & Three-Tier Separation
//! - **Value Objects**: Encapsulate domain validation rules and string trimming on construction.
//!   Invalid domain state is unrepresentable.
//! - **Wire Request DTOs**: Strict deserialization structures enforcing `#[serde(deny_unknown_fields)]`
//!   and handling wire aliases (`name` -> `family_name`/`member_name`).
//! - **Wire Response DTOs**: Serialized API payload structures matching the frontend contract.

use serde::{Deserialize, Serialize};

use super::error::FamilyError;

// ============================================================================
// Value Objects
// ============================================================================

/// Validated household family name Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FamilyName(String);

impl FamilyName {
    /// Validates and trims a raw family name string.
    ///
    /// # Errors
    /// Returns [`FamilyError::EmptyFamilyName`] if the trimmed name is empty.
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

    /// Returns a string slice reference to the validated family name.
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated individual family member display name Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberName(String);

impl MemberName {
    /// Validates and trims a raw member display name.
    ///
    /// # Errors
    /// Returns [`FamilyError::EmptyMemberName`] if the trimmed name is empty.
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

    /// Returns a string slice reference to the validated member name.
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated banking institution name Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BankName(String);

impl BankName {
    /// Validates and trims a raw bank institution name.
    ///
    /// # Errors
    /// Returns [`FamilyError::EmptyBankName`] if the trimmed name is empty.
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

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated 4-digit financial account identification string Value Object.
///
/// Trims whitespace and guarantees exactly 4 ASCII numeric digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Last4(String);

impl Last4 {
    /// Validates and trims a 4-digit account identifier.
    ///
    /// # Errors
    /// Returns [`FamilyError::InvalidLast4`] if the string is not exactly 4 ASCII numeric digits.
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

    /// Returns a string slice reference to the validated 4 digits.
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated depository bank account display name Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccountName(String);

impl AccountName {
    /// Validates and trims a raw bank account name.
    ///
    /// # Errors
    /// Returns [`FamilyError::EmptyAccountName`] if the trimmed name is empty.
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

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated revolving credit card display name Value Object.
///
/// Trims surrounding whitespace on creation and guarantees non-empty content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CardName(String);

impl CardName {
    /// Validates and trims a raw credit card name.
    ///
    /// # Errors
    /// Returns [`FamilyError::EmptyCardName`] if the trimmed name is empty.
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

    /// Consumes the wrapper, returning the inner [`String`].
    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated non-negative financial amount in integer minor units Value Object.
///
/// Prevents floating point inaccuracy by storing money strictly as 64-bit integer minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AmountMinorUnits(i64);

pub(crate) type AmountCents = AmountMinorUnits;

impl AmountMinorUnits {
    /// Validates that the monetary amount in minor units is non-negative.
    ///
    /// # Errors
    /// Returns [`FamilyError::NegativeAmount`] if `minor_units < 0`.
    pub(crate) fn try_new(minor_units: i64, action: &'static str) -> Result<Self, FamilyError> {
        if minor_units < 0 {
            return Err(FamilyError::NegativeAmount { action });
        }
        Ok(Self(minor_units))
    }

    /// Returns the raw integer value in minor units.
    pub(crate) fn get(&self) -> i64 {
        self.0
    }
}

// ============================================================================
// Wire Request DTOs
// ============================================================================

/// Wire Request DTO for updating household family metadata and default currency.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateFamilyRequest {
    family_name: Option<String>,
    currency_id: i64,
}

impl UpdateFamilyRequest {
    /// Returns the optional updated family name.
    pub(crate) fn family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    /// Returns the mandatory updated currency ID.
    pub(crate) fn currency_id(&self) -> i64 {
        self.currency_id
    }
}

/// Wire Request DTO for creating a new family member within a household.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateMemberRequest {
    family_id: i64,
    member_name: String,
}

impl CreateMemberRequest {
    /// Returns the target family ID.
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    /// Returns the requested member display name.
    pub(crate) fn member_name(&self) -> &str {
        &self.member_name
    }
}

/// Wire Request DTO for updating an existing member's display name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateMemberRequest {
    member_name: String,
}

impl UpdateMemberRequest {
    /// Returns the updated member display name.
    pub(crate) fn member_name(&self) -> &str {
        &self.member_name
    }
}

/// Wire Request DTO for creating a new manually tracked depository checking or savings account.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateBankAccountRequest {
    family_id: i64,
    owner_member_id: i64,
    currency_id: Option<i64>,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
}

impl CreateBankAccountRequest {
    /// Returns the parent family ID.
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    /// Returns the owning member ID.
    pub(crate) fn owner_member_id(&self) -> i64 {
        self.owner_member_id
    }

    /// Returns the optional currency ID foreign key (defaults to family base currency).
    pub(crate) fn currency_id(&self) -> Option<i64> {
        self.currency_id
    }

    /// Returns the banking institution name.
    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    /// Returns the account display name.
    pub(crate) fn account_name(&self) -> &str {
        &self.account_name
    }

    /// Returns the 4-digit account identification string.
    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    /// Returns the initial available balance in integer cents.
    pub(crate) fn available_balance_cents(&self) -> i64 {
        self.available_balance_cents
    }
}

/// Wire Request DTO for updating an existing depository bank account's details.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateBankAccountRequest {
    currency_id: Option<i64>,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
}

impl UpdateBankAccountRequest {
    /// Returns the optional currency ID foreign key.
    pub(crate) fn currency_id(&self) -> Option<i64> {
        self.currency_id
    }

    /// Returns the updated banking institution name.
    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    /// Returns the updated account display name.
    pub(crate) fn account_name(&self) -> &str {
        &self.account_name
    }

    /// Returns the updated 4-digit identification string.
    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    /// Returns the updated available balance in integer cents.
    pub(crate) fn available_balance_cents(&self) -> i64 {
        self.available_balance_cents
    }
}

/// Wire Request DTO for creating a new manually tracked revolving credit card account.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateCreditCardRequest {
    family_id: i64,
    owner_member_id: i64,
    currency_id: Option<i64>,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
}

impl CreateCreditCardRequest {
    /// Returns the parent family ID.
    pub(crate) fn family_id(&self) -> i64 {
        self.family_id
    }

    /// Returns the owning member ID.
    pub(crate) fn owner_member_id(&self) -> i64 {
        self.owner_member_id
    }

    /// Returns the optional currency ID foreign key (defaults to family base currency).
    pub(crate) fn currency_id(&self) -> Option<i64> {
        self.currency_id
    }

    /// Returns the issuing bank or institution name.
    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    /// Returns the credit card display name.
    pub(crate) fn card_name(&self) -> &str {
        &self.card_name
    }

    /// Returns the 4-digit card identification string.
    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    /// Returns the credit limit in integer cents.
    pub(crate) fn credit_limit_cents(&self) -> i64 {
        self.credit_limit_cents
    }

    /// Returns the available credit amount in integer cents.
    pub(crate) fn available_cents(&self) -> i64 {
        self.available_cents
    }
}

/// Wire Request DTO for updating an existing credit card account's details and limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateCreditCardRequest {
    currency_id: Option<i64>,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
}

impl UpdateCreditCardRequest {
    /// Returns the optional currency ID foreign key.
    pub(crate) fn currency_id(&self) -> Option<i64> {
        self.currency_id
    }

    /// Returns the updated issuing institution name.
    pub(crate) fn bank_name(&self) -> &str {
        &self.bank_name
    }

    /// Returns the updated credit card display name.
    pub(crate) fn card_name(&self) -> &str {
        &self.card_name
    }

    /// Returns the updated 4-digit card identifier.
    pub(crate) fn last4(&self) -> &str {
        &self.last4
    }

    /// Returns the updated credit limit in integer cents.
    pub(crate) fn credit_limit_cents(&self) -> i64 {
        self.credit_limit_cents
    }

    /// Returns the updated available credit in integer cents.
    pub(crate) fn available_cents(&self) -> i64 {
        self.available_cents
    }
}

/// Wire Request DTO for regional default currency resolution query parameters.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefaultCurrencyQuery {
    region: Option<String>,
}

impl DefaultCurrencyQuery {
    /// Returns the optional ISO country/region code provided by the client.
    pub(crate) fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}

// ============================================================================
// Wire Response DTOs
// ============================================================================

/// Wire Response DTO representing household metadata and preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FamilyDto {
    id: i64,
    family_name: String,
    currency_id: i64,
    created_at: i64,
}

impl FamilyDto {
    /// Constructs a new [`FamilyDto`].
    pub(crate) fn new(
        id: i64,
        family_name: impl Into<String>,
        currency_id: i64,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            family_name: family_name.into(),
            currency_id,
            created_at,
        }
    }

    /// Returns the family primary key identifier.
    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the household family display name.
    #[cfg(test)]
    pub(crate) fn family_name(&self) -> &str {
        &self.family_name
    }
}

/// Wire Response DTO representing an individual household member.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MemberDto {
    id: i64,
    family_id: i64,
    member_name: String,
    created_at: i64,
}

impl MemberDto {
    /// Constructs a new [`MemberDto`].
    pub(crate) fn new(
        id: i64,
        family_id: i64,
        member_name: impl Into<String>,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            family_id,
            member_name: member_name.into(),
            created_at,
        }
    }

    /// Returns the member primary key identifier.
    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

/// Wire Response DTO representing a depository checking or savings bank account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BankAccountDto {
    id: i64,
    family_id: i64,
    owner_member_id: i64,
    #[serde(rename = "type")]
    account_type: String,
    currency_id: i64,
    bank_name: String,
    account_name: String,
    last4: String,
    available_balance_cents: i64,
    created_at: i64,
}

impl BankAccountDto {
    /// Constructs a new [`BankAccountDto`] with account type discriminator set to `"bank_account"`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: i64,
        family_id: i64,
        owner_member_id: i64,
        currency_id: i64,
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
            currency_id,
            bank_name: bank_name.into(),
            account_name: account_name.into(),
            last4: last4.into(),
            available_balance_cents,
            created_at,
        }
    }

    /// Returns the account primary key identifier.
    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

/// Wire Response DTO representing a revolving credit card account with balances and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreditCardDto {
    id: i64,
    family_id: i64,
    owner_member_id: i64,
    #[serde(rename = "type")]
    account_type: String,
    currency_id: i64,
    bank_name: String,
    card_name: String,
    last4: String,
    credit_limit_cents: i64,
    available_cents: i64,
    outstanding_cents: i64,
    created_at: i64,
}

impl CreditCardDto {
    /// Constructs a new [`CreditCardDto`] with account type discriminator `"credit_card"`
    /// and automatically calculates `outstanding_cents` as `credit_limit_cents.saturating_sub(available_cents)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: i64,
        family_id: i64,
        owner_member_id: i64,
        currency_id: i64,
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
            currency_id,
            bank_name: bank_name.into(),
            card_name: card_name.into(),
            last4: last4.into(),
            credit_limit_cents,
            available_cents,
            outstanding_cents: credit_limit_cents.saturating_sub(available_cents),
            created_at,
        }
    }

    /// Returns the account primary key identifier.
    pub(crate) fn id(&self) -> i64 {
        self.id
    }
}

/// Polymorphic Wire Response DTO representing either a bank account or a credit card.
///
/// Serialized untagged to match the frontend TypeScript union representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum AccountDto {
    /// Depository checking or savings account.
    Bank(BankAccountDto),
    /// Revolving credit card account.
    Credit(CreditCardDto),
}

/// Wire Response DTO representing an authoritative supported currency option.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrencyDto {
    id: i64,
    code: String,
    name: String,
    symbol: String,
    scale: i64,
}

impl CurrencyDto {
    /// Constructs a new [`CurrencyDto`].
    pub(crate) fn new(
        id: i64,
        code: impl Into<String>,
        name: impl Into<String>,
        symbol: impl Into<String>,
        scale: i64,
    ) -> Self {
        Self {
            id,
            code: code.into(),
            name: name.into(),
            symbol: symbol.into(),
            scale,
        }
    }

    /// Returns the currency primary key identifier.
    #[cfg(test)]
    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the 3-letter currency code.
    #[cfg(test)]
    pub(crate) fn code(&self) -> &str {
        &self.code
    }

    /// Returns the full currency name.
    #[cfg(test)]
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Returns the native currency symbol.
    #[cfg(test)]
    pub(crate) fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the minor unit scale (number of decimals).
    #[cfg(test)]
    pub(crate) fn scale(&self) -> i64 {
        self.scale
    }
}

/// Composite Wire Response DTO aggregating the household family, all roster members, all accounts, and supported currencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FamilyDetailsDto {
    family: Option<FamilyDto>,
    members: Vec<MemberDto>,
    accounts: Vec<AccountDto>,
    currencies: Vec<CurrencyDto>,
}

impl FamilyDetailsDto {
    /// Constructs a new composite [`FamilyDetailsDto`].
    pub(crate) fn new(
        family: Option<FamilyDto>,
        members: Vec<MemberDto>,
        accounts: Vec<AccountDto>,
        currencies: Vec<CurrencyDto>,
    ) -> Self {
        Self {
            family,
            members,
            accounts,
            currencies,
        }
    }

    #[cfg(test)]
    pub(crate) fn family(&self) -> Option<&FamilyDto> {
        self.family.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn currencies(&self) -> &[CurrencyDto] {
        &self.currencies
    }
}

/// Wire Response DTO returning the resolved default ISO-4217 currency code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultCurrencyDto {
    currency: String,
}

impl DefaultCurrencyDto {
    /// Constructs a new [`DefaultCurrencyDto`].
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

    #[test]
    fn test_currency_dto() {
        let dto = CurrencyDto::new(1, "USD", "US Dollar", "$", 2);
        assert_eq!(dto.id(), 1);
        assert_eq!(dto.code(), "USD");
        assert_eq!(dto.name(), "US Dollar");
        assert_eq!(dto.symbol(), "$");
        assert_eq!(dto.scale(), 2);

        let details = FamilyDetailsDto::new(
            Some(FamilyDto::new(1, "Family", 1, 0)),
            vec![],
            vec![],
            vec![dto],
        );
        assert_eq!(details.currencies().len(), 1);
        assert_eq!(details.family().unwrap().family_name(), "Family");
    }
}
