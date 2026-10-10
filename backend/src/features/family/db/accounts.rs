//! Database persistence and single-shot atomic queries for financial accounts.
//!
//! Manages depository bank accounts and revolving credit cards:
//! - **Value Object Validation**: Validates bank names, account names, 4-digit last4 codes,
//!   currencies, and non-negative integer minor unit amounts at the database ingress boundary.
//! - **Single-Shot Insert Operations**: Executes atomic `INSERT ... RETURNING id` queries,
//!   populating accounts with typed discriminators (`bank_account` or `credit_card`).
//! - **Constraint Classification**: Translates SQLite constraint violations
//!   (`UNIQUE(owner_member_id, type, bank_name, account_name)`, foreign key to owning member) to domain errors.
//! - **Single-Shot Atomic Updates**: Uses `UPDATE ... WHERE type = ? RETURNING` to modify
//!   account balances and metadata while safeguarding account type integrity.

use sqlx::Row;

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};

use super::super::error::FamilyError;
use super::super::models::{
    AccountName, AmountMinorUnits, BankAccountDto, BankName, CardName, CreateBankAccountRequest,
    CreateCreditCardRequest, CreditCardDto, Last4, UpdateBankAccountRequest,
    UpdateCreditCardRequest,
};

/// Creates a new depository bank account as a single atomic INSERT operation.
///
/// Validates incoming Value Objects, inserts into `accounts` with account type `"bank_account"`,
/// and returns the newly created bank account entity.
///
/// # Execution Model
/// Executes a single atomic `INSERT ... RETURNING id` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`]
/// (`UNIQUE(owner_member_id, type, bank_name, account_name)`) and [`is_foreign_key_violation`] (`FOREIGN KEY(owner_member_id)`).
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateBankAccountRequest`] containing family ID, owner member ID, bank name,
///   account name, last4, optional currency ID, and available balance in minor units.
///
/// # Returns
/// - `Ok(BankAccountDto)` representing the newly created depository bank account with generated ID.
///
/// # Errors
/// - Returns [`FamilyError::EmptyBankName`] if bank name fails Value Object validation.
/// - Returns [`FamilyError::EmptyAccountName`] if account name fails Value Object validation.
/// - Returns [`FamilyError::InvalidLast4`] if last4 is not exactly 4 ASCII digits.
/// - Returns [`FamilyError::CurrencyNotFound`] if currency ID does not exist in currencies.
/// - Returns [`FamilyError::CurrencyMismatch`] if requested currency does not match family base currency.
/// - Returns [`FamilyError::NegativeAmount`] if available balance in minor units is negative.
/// - Returns [`FamilyError::MemberNotFound`] if the owner member ID does not exist.
/// - Returns [`FamilyError::AccountAlreadyExists`] if this owner already has an account with this name at this bank.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn create_bank_account(
    pool: &DbPool,
    payload: CreateBankAccountRequest,
) -> Result<BankAccountDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.CREATE_BANK.EMPTY_BANK_NAME")?;
    let account_name = AccountName::try_new(
        payload.account_name(),
        "FAMILY.CREATE_BANK.EMPTY_ACCOUNT_NAME",
    )?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.CREATE_BANK.INVALID_LAST4")?;
    let available_balance = AmountMinorUnits::try_new(
        payload.available_balance(),
        "FAMILY.CREATE_BANK.NEGATIVE_BALANCE",
    )?;

    let family_id = payload.family_id();
    let owner_member_id = payload.owner_member_id();
    let raw_bank_name = bank_name.into_inner();
    let raw_account_name = account_name.into_inner();
    let raw_last4 = last4.into_inner();
    let now = now_epoch_secs();

    let family_currency_id: i64 =
        sqlx::query_scalar("SELECT currency_id FROM families WHERE id = ?")
            .bind(family_id)
            .fetch_optional(pool)
            .await
            .db_context("FAMILY.CREATE_BANK.GET_FAMILY_CURRENCY")?
            .ok_or(FamilyError::FamilyNotFound {
                action: "FAMILY.CREATE_BANK.FAMILY_NOT_FOUND",
            })?;

    let currency_id = match payload.currency_id() {
        Some(req_curr_id) => {
            if req_curr_id != family_currency_id {
                return Err(FamilyError::CurrencyMismatch {
                    action: "FAMILY.CREATE_BANK.CURRENCY_MISMATCH",
                    family_currency_id,
                    account_currency_id: req_curr_id,
                }
                .into());
            }
            req_curr_id
        }
        None => family_currency_id,
    };

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, available_balance, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(owner_member_id)
    .bind(currency_id)
    .bind(&raw_bank_name)
    .bind(&raw_last4)
    .bind(&raw_account_name)
    .bind(available_balance.get())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match res {
        Ok(id) => Ok(BankAccountDto::new(
            id,
            family_id,
            owner_member_id,
            currency_id,
            raw_bank_name,
            raw_account_name,
            raw_last4,
            available_balance.get(),
            now,
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.CREATE_BANK.ALREADY_EXISTS",
                    account_name: raw_account_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                let currency_exists: Option<i64> =
                    sqlx::query_scalar("SELECT 1 FROM currencies WHERE id = ?")
                        .bind(currency_id)
                        .fetch_optional(pool)
                        .await
                        .unwrap_or(None);
                if currency_exists.is_none() {
                    Err(FamilyError::CurrencyNotFound {
                        action: "FAMILY.CREATE_BANK.CURRENCY_NOT_FOUND",
                        id: currency_id,
                    }
                    .into())
                } else {
                    Err(FamilyError::MemberNotFound {
                        action: "FAMILY.CREATE_BANK.MEMBER_NOT_FOUND",
                        id: owner_member_id,
                    }
                    .into())
                }
            } else {
                Err(db_err("FAMILY.CREATE_BANK.INSERT", err))
            }
        }
    }
}

/// Updates an existing bank account as a single atomic UPDATE operation.
///
/// Validates incoming Value Objects and applies updates guarded by `type = 'bank_account'`
/// to prevent cross-type mutation. Fetches the updated record via `RETURNING`.
///
/// # Execution Model
/// Executes a single atomic `UPDATE ... RETURNING` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`].
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target bank account.
/// - `payload`: Inbound [`UpdateBankAccountRequest`] containing updated bank name, account name,
///   last4, optional currency, and available balance in minor units.
///
/// # Returns
/// - `Ok(BankAccountDto)` representing the updated bank account entity.
///
/// # Errors
/// - Returns [`FamilyError::EmptyBankName`] if bank name fails Value Object validation.
/// - Returns [`FamilyError::EmptyAccountName`] if account name fails Value Object validation.
/// - Returns [`FamilyError::InvalidLast4`] if last4 is not exactly 4 digits.
/// - Returns [`FamilyError::CurrencyNotFound`] if currency ID does not exist in currencies.
/// - Returns [`FamilyError::CurrencyMismatch`] if requested currency does not match family base currency.
/// - Returns [`FamilyError::NegativeAmount`] if available balance in minor units is negative.
/// - Returns [`FamilyError::AccountNotFound`] if the target account ID does not exist or is not a bank account.
/// - Returns [`FamilyError::AccountAlreadyExists`] if renaming conflicts with an existing account for this member at this bank.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn update_bank_account(
    pool: &DbPool,
    id: i64,
    payload: UpdateBankAccountRequest,
) -> Result<BankAccountDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.UPDATE_BANK.EMPTY_BANK_NAME")?;
    let account_name = AccountName::try_new(
        payload.account_name(),
        "FAMILY.UPDATE_BANK.EMPTY_ACCOUNT_NAME",
    )?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.UPDATE_BANK.INVALID_LAST4")?;
    let available_balance = AmountMinorUnits::try_new(
        payload.available_balance(),
        "FAMILY.UPDATE_BANK.NEGATIVE_BALANCE",
    )?;

    let now = now_epoch_secs();
    let raw_bank_name = bank_name.into_inner();
    let raw_account_name = account_name.into_inner();
    let raw_last4 = last4.into_inner();

    let account_row = sqlx::query(
        r#"
        SELECT f.currency_id as family_currency_id, a.currency_id as account_currency_id
        FROM accounts a
        JOIN families f ON a.family_id = f.id
        WHERE a.id = ? AND a.type = 'bank_account';
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .db_context("FAMILY.UPDATE_BANK.GET_ACCOUNT_CURRENCY")?
    .ok_or(FamilyError::AccountNotFound {
        action: "FAMILY.UPDATE_BANK.NOT_FOUND",
        id,
    })?;

    let family_currency_id: i64 = account_row.get("family_currency_id");
    let existing_currency_id: i64 = account_row.get("account_currency_id");

    let currency_id = match payload.currency_id() {
        Some(req_curr_id) => {
            if req_curr_id != family_currency_id {
                return Err(FamilyError::CurrencyMismatch {
                    action: "FAMILY.UPDATE_BANK.CURRENCY_MISMATCH",
                    family_currency_id,
                    account_currency_id: req_curr_id,
                }
                .into());
            }
            req_curr_id
        }
        None => existing_currency_id,
    };

    let res = sqlx::query(
        r#"
        UPDATE accounts
        SET currency_id = ?,
            bank_name = ?,
            account_name = ?,
            last4 = ?,
            available_balance = ?,
            updated_at = ?
        WHERE id = ? AND type = 'bank_account'
        RETURNING id, family_id, owner_member_id, created_at;
        "#,
    )
    .bind(currency_id)
    .bind(&raw_bank_name)
    .bind(&raw_account_name)
    .bind(&raw_last4)
    .bind(available_balance.get())
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(r)) => Ok(BankAccountDto::new(
            r.get("id"),
            r.get("family_id"),
            r.get("owner_member_id"),
            currency_id,
            raw_bank_name,
            raw_account_name,
            raw_last4,
            available_balance.get(),
            r.get("created_at"),
        )),
        Ok(None) => Err(FamilyError::AccountNotFound {
            action: "FAMILY.UPDATE_BANK.NOT_FOUND",
            id,
        }
        .into()),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.UPDATE_BANK.ALREADY_EXISTS",
                    account_name: raw_account_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(FamilyError::CurrencyNotFound {
                    action: "FAMILY.UPDATE_BANK.CURRENCY_NOT_FOUND",
                    id: currency_id,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_BANK.EXECUTE", err))
            }
        }
    }
}

/// Creates a new credit card account as a single atomic INSERT operation.
///
/// Validates incoming Value Objects, inserts into `accounts` with account type `"credit_card"`,
/// and returns the newly created credit card entity with automatically derived outstanding balance in minor units.
///
/// # Execution Model
/// Executes a single atomic `INSERT ... RETURNING id` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`]
/// (`UNIQUE(owner_member_id, type, bank_name, account_name)`) and [`is_foreign_key_violation`] (`FOREIGN KEY(owner_member_id)`).
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateCreditCardRequest`] containing family ID, owner member ID, bank name,
///   card name, last4, optional currency, credit limit in minor units, and available credit in minor units.
///
/// # Returns
/// - `Ok(CreditCardDto)` representing the newly created credit card account with generated ID.
///
/// # Errors
/// - Returns [`FamilyError::EmptyBankName`] if bank name fails Value Object validation.
/// - Returns [`FamilyError::EmptyCardName`] if card name fails Value Object validation.
/// - Returns [`FamilyError::InvalidLast4`] if last4 is not exactly 4 digits.
/// - Returns [`FamilyError::CurrencyNotFound`] if currency ID does not exist in currencies.
/// - Returns [`FamilyError::CurrencyMismatch`] if requested currency does not match family base currency.
/// - Returns [`FamilyError::NegativeAmount`] if credit limit or available credit in minor units is negative.
/// - Returns [`FamilyError::MemberNotFound`] if the owner member ID does not exist.
/// - Returns [`FamilyError::AccountAlreadyExists`] if this owner already has a card with this name at this bank.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn create_credit_card(
    pool: &DbPool,
    payload: CreateCreditCardRequest,
) -> Result<CreditCardDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.CREATE_CREDIT.EMPTY_BANK_NAME")?;
    let card_name = CardName::try_new(payload.card_name(), "FAMILY.CREATE_CREDIT.EMPTY_CARD_NAME")?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.CREATE_CREDIT.INVALID_LAST4")?;
    let credit_limit = AmountMinorUnits::try_new(
        payload.credit_limit(),
        "FAMILY.CREATE_CREDIT.NEGATIVE_LIMIT",
    )?;
    let available_credit = AmountMinorUnits::try_new(
        payload.available_credit(),
        "FAMILY.CREATE_CREDIT.NEGATIVE_AVAILABLE",
    )?;

    let family_id = payload.family_id();
    let owner_member_id = payload.owner_member_id();
    let raw_bank_name = bank_name.into_inner();
    let raw_card_name = card_name.into_inner();
    let raw_last4 = last4.into_inner();
    let now = now_epoch_secs();

    let family_currency_id: i64 =
        sqlx::query_scalar("SELECT currency_id FROM families WHERE id = ?")
            .bind(family_id)
            .fetch_optional(pool)
            .await
            .db_context("FAMILY.CREATE_CREDIT.GET_FAMILY_CURRENCY")?
            .ok_or(FamilyError::FamilyNotFound {
                action: "FAMILY.CREATE_CREDIT.FAMILY_NOT_FOUND",
            })?;

    let currency_id = match payload.currency_id() {
        Some(req_curr_id) => {
            if req_curr_id != family_currency_id {
                return Err(FamilyError::CurrencyMismatch {
                    action: "FAMILY.CREATE_CREDIT.CURRENCY_MISMATCH",
                    family_currency_id,
                    account_currency_id: req_curr_id,
                }
                .into());
            }
            req_curr_id
        }
        None => family_currency_id,
    };

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, credit_limit, available_credit, created_at, updated_at
        ) VALUES (?, ?, 'credit_card', ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(owner_member_id)
    .bind(currency_id)
    .bind(&raw_bank_name)
    .bind(&raw_last4)
    .bind(&raw_card_name)
    .bind(credit_limit.get())
    .bind(available_credit.get())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match res {
        Ok(id) => Ok(CreditCardDto::new(
            id,
            family_id,
            owner_member_id,
            currency_id,
            raw_bank_name,
            raw_card_name,
            raw_last4,
            credit_limit.get(),
            available_credit.get(),
            now,
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.CREATE_CREDIT.ALREADY_EXISTS",
                    account_name: raw_card_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                let currency_exists: Option<i64> =
                    sqlx::query_scalar("SELECT 1 FROM currencies WHERE id = ?")
                        .bind(currency_id)
                        .fetch_optional(pool)
                        .await
                        .unwrap_or(None);
                if currency_exists.is_none() {
                    Err(FamilyError::CurrencyNotFound {
                        action: "FAMILY.CREATE_CREDIT.CURRENCY_NOT_FOUND",
                        id: currency_id,
                    }
                    .into())
                } else {
                    Err(FamilyError::MemberNotFound {
                        action: "FAMILY.CREATE_CREDIT.MEMBER_NOT_FOUND",
                        id: owner_member_id,
                    }
                    .into())
                }
            } else {
                Err(db_err("FAMILY.CREATE_CREDIT.INSERT", err))
            }
        }
    }
}

/// Updates an existing credit card account as a single atomic UPDATE operation.
///
/// Validates incoming Value Objects and applies updates guarded by `type = 'credit_card'`
/// to prevent cross-type mutation. Fetches the updated record via `RETURNING`.
///
/// # Execution Model
/// Executes a single atomic `UPDATE ... RETURNING` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`].
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target credit card account.
/// - `payload`: Inbound [`UpdateCreditCardRequest`] containing updated card name, bank name,
///   last4, optional currency, credit limit in minor units, and available credit in minor units.
///
/// # Returns
/// - `Ok(CreditCardDto)` representing the updated credit card entity.
///
/// # Errors
/// - Returns [`FamilyError::EmptyBankName`] if bank name fails Value Object validation.
/// - Returns [`FamilyError::EmptyCardName`] if card name fails Value Object validation.
/// - Returns [`FamilyError::InvalidLast4`] if last4 is not exactly 4 digits.
/// - Returns [`FamilyError::CurrencyNotFound`] if currency ID does not exist in currencies.
/// - Returns [`FamilyError::CurrencyMismatch`] if requested currency does not match family base currency.
/// - Returns [`FamilyError::NegativeAmount`] if credit limit or available credit in minor units is negative.
/// - Returns [`FamilyError::AccountNotFound`] if the target account ID does not exist or is not a credit card.
/// - Returns [`FamilyError::AccountAlreadyExists`] if renaming conflicts with an existing account for this member at this bank.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn update_credit_card(
    pool: &DbPool,
    id: i64,
    payload: UpdateCreditCardRequest,
) -> Result<CreditCardDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.UPDATE_CREDIT.EMPTY_BANK_NAME")?;
    let card_name = CardName::try_new(payload.card_name(), "FAMILY.UPDATE_CREDIT.EMPTY_CARD_NAME")?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.UPDATE_CREDIT.INVALID_LAST4")?;
    let credit_limit = AmountMinorUnits::try_new(
        payload.credit_limit(),
        "FAMILY.UPDATE_CREDIT.NEGATIVE_LIMIT",
    )?;
    let available_credit = AmountMinorUnits::try_new(
        payload.available_credit(),
        "FAMILY.UPDATE_CREDIT.NEGATIVE_AVAILABLE",
    )?;

    let now = now_epoch_secs();
    let raw_bank_name = bank_name.into_inner();
    let raw_card_name = card_name.into_inner();
    let raw_last4 = last4.into_inner();

    let account_row = sqlx::query(
        r#"
        SELECT f.currency_id as family_currency_id, a.currency_id as account_currency_id
        FROM accounts a
        JOIN families f ON a.family_id = f.id
        WHERE a.id = ? AND a.type = 'credit_card';
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .db_context("FAMILY.UPDATE_CREDIT.GET_ACCOUNT_CURRENCY")?
    .ok_or(FamilyError::AccountNotFound {
        action: "FAMILY.UPDATE_CREDIT.NOT_FOUND",
        id,
    })?;

    let family_currency_id: i64 = account_row.get("family_currency_id");
    let existing_currency_id: i64 = account_row.get("account_currency_id");

    let currency_id = match payload.currency_id() {
        Some(req_curr_id) => {
            if req_curr_id != family_currency_id {
                return Err(FamilyError::CurrencyMismatch {
                    action: "FAMILY.UPDATE_CREDIT.CURRENCY_MISMATCH",
                    family_currency_id,
                    account_currency_id: req_curr_id,
                }
                .into());
            }
            req_curr_id
        }
        None => existing_currency_id,
    };

    let res = sqlx::query(
        r#"
        UPDATE accounts
        SET currency_id = ?,
            bank_name = ?,
            account_name = ?,
            last4 = ?,
            credit_limit = ?,
            available_credit = ?,
            updated_at = ?
        WHERE id = ? AND type = 'credit_card'
        RETURNING id, family_id, owner_member_id, created_at;
        "#,
    )
    .bind(currency_id)
    .bind(&raw_bank_name)
    .bind(&raw_card_name)
    .bind(&raw_last4)
    .bind(credit_limit.get())
    .bind(available_credit.get())
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(r)) => Ok(CreditCardDto::new(
            r.get("id"),
            r.get("family_id"),
            r.get("owner_member_id"),
            currency_id,
            raw_bank_name,
            raw_card_name,
            raw_last4,
            credit_limit.get(),
            available_credit.get(),
            r.get("created_at"),
        )),
        Ok(None) => Err(FamilyError::AccountNotFound {
            action: "FAMILY.UPDATE_CREDIT.NOT_FOUND",
            id,
        }
        .into()),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.UPDATE_CREDIT.ALREADY_EXISTS",
                    account_name: raw_card_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(FamilyError::CurrencyNotFound {
                    action: "FAMILY.UPDATE_CREDIT.CURRENCY_NOT_FOUND",
                    id: currency_id,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_CREDIT.EXECUTE", err))
            }
        }
    }
}

/// Deletes a financial account by ID.
///
/// Removes the account entity from `accounts` regardless of whether it is a bank account
/// or credit card.
///
/// # Execution Model
/// Executes a single atomic `DELETE FROM accounts WHERE id = ?` directly against [`DbPool`],
/// validating that at least one row was affected.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target account to delete.
///
/// # Returns
/// - `Ok(())` on successful deletion.
///
/// # Errors
/// - Returns [`FamilyError::AccountNotFound`] if no account matching `id` was found.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn delete_account(pool: &DbPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = ?;")
        .bind(id)
        .execute(pool)
        .await
        .db_context("FAMILY.DELETE_ACCOUNT.EXECUTE")?;

    if result.rows_affected() == 0 {
        return Err(FamilyError::AccountNotFound {
            action: "FAMILY.DELETE_ACCOUNT.NOT_FOUND",
            id,
        }
        .into());
    }

    Ok(())
}
