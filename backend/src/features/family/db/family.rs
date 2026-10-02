//! Database queries, updates, and initial seeding for the primary household family.
//!
//! Manages the top-level family entity lifecycle:
//! - **Default Currency Inference**: Resolves localized currency from client region or family record.
//! - **Family Lifecycle**: Fetches, creates, and modifies household name and primary base currency.
//! - **Household Overview Assembly**: Concurrently collects family metadata, member roster,
//!   and polymorphic financial accounts into a unified [`FamilyDetailsDto`].
//! - **Initial Seeding**: Idempotently copies currencies from JSON template via `app_meta` tracking.

use sqlx::Row;

use crate::core::{
    db_err, get_meta, is_unique_violation, now_epoch_secs, set_meta_tx, AppError, DbPool,
    DbResultExt,
};

use super::super::error::FamilyError;
use super::super::models::{
    AccountDto, BankAccountDto, CreditCardDto, CurrencyDto, FamilyDetailsDto, FamilyDto,
    FamilyName, MemberDto,
};

const DEFAULT_CURRENCY_JSON: &str = include_str!("../../../../resources/default_currency.json");
const META_KEY_SEED_CURRENCIES: &str = "seed.currencies.v1";

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultCurrencyConfig {
    currencies: Vec<DefaultCurrencyItem>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultCurrencyItem {
    code: String,
    name: String,
    symbol: String,
    scale: i64,
    sort_order: i64,
    region: String,
}

/// Resolves the default currency for a client:
/// - If a household family exists, returns the family's base currency code.
/// - Otherwise, infers the currency from the client region code via the `currencies` table.
/// - If region is unknown or missing, resolves to `"USD"`.
pub(crate) async fn get_default_currency(
    pool: &DbPool,
    region: Option<&str>,
) -> Result<String, AppError> {
    let family_currency: Option<String> = sqlx::query_scalar(
        r#"
        SELECT c.code
        FROM families f
        JOIN currencies c ON f.currency_id = c.id
        ORDER BY f.id ASC
        LIMIT 1;
        "#,
    )
    .fetch_optional(pool)
    .await
    .db_context("FAMILY.DEFAULT_CURRENCY.FAMILY_QUERY")?;

    if let Some(curr) = family_currency {
        if !curr.is_empty() {
            return Ok(curr);
        }
    }

    if let Some(r) = region {
        let trimmed = r.trim().to_ascii_uppercase();
        if !trimmed.is_empty() {
            let matched_currency: Option<String> =
                sqlx::query_scalar("SELECT code FROM currencies WHERE region = ? LIMIT 1;")
                    .bind(&trimmed)
                    .fetch_optional(pool)
                    .await
                    .db_context("FAMILY.DEFAULT_CURRENCY.REGION_QUERY")?;

            if let Some(c) = matched_currency {
                return Ok(c);
            }
        }
    }

    Ok("USD".to_string())
}

/// Fetches the optional primary household family record without erroring on empty state.
pub(crate) async fn get_optional_family(pool: &DbPool) -> Result<Option<FamilyDto>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT f.id, f.family_name, f.currency_id, f.created_at
        FROM families f
        ORDER BY f.id ASC
        LIMIT 1;
        "#,
    )
    .fetch_optional(pool)
    .await
    .db_context("FAMILY.GET_OPTIONAL_FAMILY.QUERY")?;

    Ok(row.map(|r| {
        FamilyDto::new(
            r.get("id"),
            r.get::<String, _>("family_name"),
            r.get::<i64, _>("currency_id"),
            r.get("created_at"),
        )
    }))
}

/// Updates or creates the family record display name and/or base currency.
pub(crate) async fn update_family(
    pool: &DbPool,
    name: Option<FamilyName>,
    currency_id: i64,
) -> Result<FamilyDto, AppError> {
    let now = now_epoch_secs();
    let raw_name = name.map(|n| n.into_inner());

    let row = sqlx::query(
        r#"
        INSERT INTO families (id, family_name, currency_id, created_at, updated_at)
        VALUES (1, COALESCE(?, 'My Family'), ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            family_name = COALESCE(excluded.family_name, families.family_name),
            currency_id = excluded.currency_id,
            updated_at = excluded.updated_at
        RETURNING id, family_name, currency_id, created_at;
        "#,
    )
    .bind(&raw_name)
    .bind(currency_id)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match row {
        Ok(r) => Ok(FamilyDto::new(
            r.get("id"),
            r.get::<String, _>("family_name"),
            r.get::<i64, _>("currency_id"),
            r.get("created_at"),
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::FamilyAlreadyExists {
                    action: "FAMILY.UPDATE_FAMILY.ALREADY_EXISTS",
                    family_name: raw_name.unwrap_or_default(),
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_FAMILY.EXECUTE", err))
            }
        }
    }
}

/// Assembles the complete family details (family, members, accounts, currencies).
/// If no family has been created yet, returns `family: None` with empty members and accounts.
pub(crate) async fn get_family_details(pool: &DbPool) -> Result<FamilyDetailsDto, AppError> {
    let family = get_optional_family(pool).await?;

    let (members, accounts) = match &family {
        Some(f) => {
            let member_rows = sqlx::query(
                r#"
                SELECT id, family_id, member_name, created_at
                FROM members
                WHERE family_id = ?
                ORDER BY id ASC;
                "#,
            )
            .bind(f.id())
            .fetch_all(pool)
            .await
            .db_context("FAMILY.GET_DETAILS.MEMBERS_QUERY")?;

            let members: Vec<MemberDto> = member_rows
                .into_iter()
                .map(|r| {
                    MemberDto::new(
                        r.get("id"),
                        r.get("family_id"),
                        r.get::<String, _>("member_name"),
                        r.get("created_at"),
                    )
                })
                .collect();

            let account_rows = sqlx::query(
                r#"
                SELECT a.id, a.family_id, a.owner_member_id, a.type, a.currency_id, a.bank_name, a.last4,
                       a.account_name, a.available_balance_cents, a.credit_limit_cents,
                       a.available_cents, a.created_at
                FROM accounts a
                WHERE a.family_id = ?
                ORDER BY a.id ASC;
                "#,
            )
            .bind(f.id())
            .fetch_all(pool)
            .await
            .db_context("FAMILY.GET_DETAILS.ACCOUNTS_QUERY")?;

            let mut accounts = Vec::new();
            for r in account_rows {
                let acc_type: String = r.get("type");
                if acc_type == "bank_account" {
                    accounts.push(AccountDto::Bank(BankAccountDto::new(
                        r.get("id"),
                        r.get("family_id"),
                        r.get("owner_member_id"),
                        r.get::<i64, _>("currency_id"),
                        r.get::<String, _>("bank_name"),
                        r.get::<String, _>("account_name"),
                        r.get::<String, _>("last4"),
                        r.get::<Option<i64>, _>("available_balance_cents")
                            .unwrap_or(0),
                        r.get("created_at"),
                    )));
                } else {
                    let limit: i64 = r.get::<Option<i64>, _>("credit_limit_cents").unwrap_or(0);
                    let available: i64 = r.get::<Option<i64>, _>("available_cents").unwrap_or(0);
                    accounts.push(AccountDto::Credit(CreditCardDto::new(
                        r.get("id"),
                        r.get("family_id"),
                        r.get("owner_member_id"),
                        r.get::<i64, _>("currency_id"),
                        r.get::<String, _>("bank_name"),
                        r.get::<String, _>("account_name"),
                        r.get::<String, _>("last4"),
                        limit,
                        available,
                        r.get("created_at"),
                    )));
                }
            }
            (members, accounts)
        }
        None => (Vec::new(), Vec::new()),
    };

    let currencies = get_supported_currencies(pool).await?;

    Ok(FamilyDetailsDto::new(family, members, accounts, currencies))
}

/// Retrieves all supported currencies ordered by configured sort order.
pub(crate) async fn get_supported_currencies(pool: &DbPool) -> Result<Vec<CurrencyDto>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, code, name, symbol, scale
        FROM currencies
        ORDER BY sort_order ASC, code ASC;
        "#,
    )
    .fetch_all(pool)
    .await
    .db_context("FAMILY.DB.GET_SUPPORTED_CURRENCIES")?;

    let currencies = rows
        .into_iter()
        .map(|r| {
            CurrencyDto::new(
                r.get::<i64, _>("id"),
                r.get::<String, _>("code"),
                r.get::<String, _>("name"),
                r.get::<String, _>("symbol"),
                r.get::<i64, _>("scale"),
            )
        })
        .collect();

    Ok(currencies)
}

/// Seeds initial currency options from `default_currency.json` tracked via `app_meta`.
/// Everything else (families, members, accounts) starts completely blank.
pub(crate) async fn seed_currencies(pool: &DbPool) -> Result<(), AppError> {
    if get_meta(
        "FAMILY.SEED_CURRENCIES.CHECK_META",
        pool,
        META_KEY_SEED_CURRENCIES,
    )
    .await?
    .is_some()
    {
        return Ok(());
    }

    let mut tx = pool
        .begin()
        .await
        .db_context("FAMILY.SEED_CURRENCIES.TX_BEGIN")?;
    let now = now_epoch_secs();
    let config: DefaultCurrencyConfig =
        serde_json::from_str(DEFAULT_CURRENCY_JSON).map_err(|e| {
            AppError::ShouldNotBeHappening {
                action: "FAMILY.SEED_CURRENCIES.PARSE_JSON",
                reason: format!("Failed to parse default currency JSON: {e}"),
            }
        })?;

    for c in config.currencies {
        sqlx::query(
            r#"
            INSERT INTO currencies (code, name, symbol, scale, sort_order, region, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(c.code)
        .bind(c.name)
        .bind(c.symbol)
        .bind(c.scale)
        .bind(c.sort_order)
        .bind(c.region)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .db_context("FAMILY.SEED_CURRENCIES.INSERT")?;
    }

    set_meta_tx(
        "FAMILY.SEED_CURRENCIES.SET_META",
        &mut tx,
        META_KEY_SEED_CURRENCIES,
        "1",
    )
    .await?;

    tx.commit()
        .await
        .db_context("FAMILY.SEED_CURRENCIES.TX_COMMIT")?;

    Ok(())
}

/// Backwards-compatible alias for `seed_currencies`.
pub(crate) async fn seed_default_family(pool: &DbPool) -> Result<(), AppError> {
    seed_currencies(pool).await
}
