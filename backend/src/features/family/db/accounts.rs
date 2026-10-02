//! Bank account and credit card persistence queries and lifecycle workflows.

use sqlx::Row;

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};

use super::super::error::FamilyError;
use super::super::models::{
    AccountName, AmountCents, BankAccountDto, BankName, CardName, CreateBankAccountRequest,
    CreateCreditCardRequest, CreditCardDto, CurrencyCode, Last4, UpdateBankAccountRequest,
    UpdateCreditCardRequest,
};

/// Creates a new depository bank account as a single atomic INSERT operation.
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
    let currency =
        CurrencyCode::try_new(payload.currency(), "FAMILY.CREATE_BANK.INVALID_CURRENCY")?;
    let available_balance_cents = AmountCents::try_new(
        payload.available_balance_cents(),
        "FAMILY.CREATE_BANK.NEGATIVE_BALANCE",
    )?;

    let family_id = payload.family_id();
    let owner_member_id = payload.owner_member_id();
    let raw_currency = currency.into_inner();
    let raw_bank_name = bank_name.into_inner();
    let raw_account_name = account_name.into_inner();
    let raw_last4 = last4.into_inner();
    let now = now_epoch_secs();

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(owner_member_id)
    .bind(&raw_currency)
    .bind(&raw_bank_name)
    .bind(&raw_last4)
    .bind(&raw_account_name)
    .bind(available_balance_cents.get())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match res {
        Ok(id) => Ok(BankAccountDto::new(
            id,
            family_id,
            owner_member_id,
            raw_currency,
            raw_bank_name,
            raw_account_name,
            raw_last4,
            available_balance_cents.get(),
            now,
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.CREATE_BANK.ALREADY_EXISTS",
                    name: raw_account_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(FamilyError::MemberNotFound {
                    action: "FAMILY.CREATE_BANK.MEMBER_NOT_FOUND",
                    id: owner_member_id,
                }
                .into())
            } else {
                Err(db_err("FAMILY.CREATE_BANK.INSERT", err))
            }
        }
    }
}

/// Updates an existing bank account as a single atomic UPDATE operation.
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
    let available_balance_cents = AmountCents::try_new(
        payload.available_balance_cents(),
        "FAMILY.UPDATE_BANK.NEGATIVE_BALANCE",
    )?;

    let validated_currency = match payload.currency() {
        Some(c) => {
            Some(CurrencyCode::try_new(c, "FAMILY.UPDATE_BANK.INVALID_CURRENCY")?.into_inner())
        }
        None => None,
    };

    let now = now_epoch_secs();
    let raw_bank_name = bank_name.into_inner();
    let raw_account_name = account_name.into_inner();
    let raw_last4 = last4.into_inner();

    let res = sqlx::query(
        r#"
        UPDATE accounts
        SET currency = COALESCE(?, currency),
            bank_name = ?,
            name = ?,
            last4 = ?,
            available_balance_cents = ?,
            updated_at = ?
        WHERE id = ? AND type = 'bank_account'
        RETURNING id, family_id, owner_member_id, currency, created_at;
        "#,
    )
    .bind(validated_currency.as_deref())
    .bind(&raw_bank_name)
    .bind(&raw_account_name)
    .bind(&raw_last4)
    .bind(available_balance_cents.get())
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(r)) => Ok(BankAccountDto::new(
            r.get("id"),
            r.get("family_id"),
            r.get("owner_member_id"),
            r.get::<String, _>("currency"),
            raw_bank_name,
            raw_account_name,
            raw_last4,
            available_balance_cents.get(),
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
                    name: raw_account_name,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_BANK.EXECUTE", err))
            }
        }
    }
}

/// Creates a new credit card account as a single atomic INSERT operation.
pub(crate) async fn create_credit_card(
    pool: &DbPool,
    payload: CreateCreditCardRequest,
) -> Result<CreditCardDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.CREATE_CREDIT.EMPTY_BANK_NAME")?;
    let card_name = CardName::try_new(payload.card_name(), "FAMILY.CREATE_CREDIT.EMPTY_CARD_NAME")?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.CREATE_CREDIT.INVALID_LAST4")?;
    let currency =
        CurrencyCode::try_new(payload.currency(), "FAMILY.CREATE_CREDIT.INVALID_CURRENCY")?;
    let credit_limit_cents = AmountCents::try_new(
        payload.credit_limit_cents(),
        "FAMILY.CREATE_CREDIT.NEGATIVE_LIMIT",
    )?;
    let available_cents = AmountCents::try_new(
        payload.available_cents(),
        "FAMILY.CREATE_CREDIT.NEGATIVE_AVAILABLE",
    )?;

    let family_id = payload.family_id();
    let owner_member_id = payload.owner_member_id();
    let raw_currency = currency.into_inner();
    let raw_bank_name = bank_name.into_inner();
    let raw_card_name = card_name.into_inner();
    let raw_last4 = last4.into_inner();
    let now = now_epoch_secs();

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            name, credit_limit_cents, available_cents, created_at, updated_at
        ) VALUES (?, ?, 'credit_card', ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(owner_member_id)
    .bind(&raw_currency)
    .bind(&raw_bank_name)
    .bind(&raw_last4)
    .bind(&raw_card_name)
    .bind(credit_limit_cents.get())
    .bind(available_cents.get())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match res {
        Ok(id) => Ok(CreditCardDto::new(
            id,
            family_id,
            owner_member_id,
            raw_currency,
            raw_bank_name,
            raw_card_name,
            raw_last4,
            credit_limit_cents.get(),
            available_cents.get(),
            now,
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::AccountAlreadyExists {
                    action: "FAMILY.CREATE_CREDIT.ALREADY_EXISTS",
                    name: raw_card_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(FamilyError::MemberNotFound {
                    action: "FAMILY.CREATE_CREDIT.MEMBER_NOT_FOUND",
                    id: owner_member_id,
                }
                .into())
            } else {
                Err(db_err("FAMILY.CREATE_CREDIT.INSERT", err))
            }
        }
    }
}

/// Updates an existing credit card account as a single atomic UPDATE operation.
pub(crate) async fn update_credit_card(
    pool: &DbPool,
    id: i64,
    payload: UpdateCreditCardRequest,
) -> Result<CreditCardDto, AppError> {
    let bank_name = BankName::try_new(payload.bank_name(), "FAMILY.UPDATE_CREDIT.EMPTY_BANK_NAME")?;
    let card_name = CardName::try_new(payload.card_name(), "FAMILY.UPDATE_CREDIT.EMPTY_CARD_NAME")?;
    let last4 = Last4::try_new(payload.last4(), "FAMILY.UPDATE_CREDIT.INVALID_LAST4")?;
    let credit_limit_cents = AmountCents::try_new(
        payload.credit_limit_cents(),
        "FAMILY.UPDATE_CREDIT.NEGATIVE_LIMIT",
    )?;
    let available_cents = AmountCents::try_new(
        payload.available_cents(),
        "FAMILY.UPDATE_CREDIT.NEGATIVE_AVAILABLE",
    )?;

    let validated_currency = match payload.currency() {
        Some(c) => {
            Some(CurrencyCode::try_new(c, "FAMILY.UPDATE_CREDIT.INVALID_CURRENCY")?.into_inner())
        }
        None => None,
    };

    let now = now_epoch_secs();
    let raw_bank_name = bank_name.into_inner();
    let raw_card_name = card_name.into_inner();
    let raw_last4 = last4.into_inner();

    let res = sqlx::query(
        r#"
        UPDATE accounts
        SET currency = COALESCE(?, currency),
            bank_name = ?,
            name = ?,
            last4 = ?,
            credit_limit_cents = ?,
            available_cents = ?,
            updated_at = ?
        WHERE id = ? AND type = 'credit_card'
        RETURNING id, family_id, owner_member_id, currency, created_at;
        "#,
    )
    .bind(validated_currency.as_deref())
    .bind(&raw_bank_name)
    .bind(&raw_card_name)
    .bind(&raw_last4)
    .bind(credit_limit_cents.get())
    .bind(available_cents.get())
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(r)) => Ok(CreditCardDto::new(
            r.get("id"),
            r.get("family_id"),
            r.get("owner_member_id"),
            r.get::<String, _>("currency"),
            raw_bank_name,
            raw_card_name,
            raw_last4,
            credit_limit_cents.get(),
            available_cents.get(),
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
                    name: raw_card_name,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_CREDIT.EXECUTE", err))
            }
        }
    }
}

/// Deletes an account by ID.
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
