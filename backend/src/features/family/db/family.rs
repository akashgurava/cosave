//! Database queries, updates, and initial seeding for the primary household family.
//!
//! Manages the top-level family entity lifecycle:
//! - **Default Currency Inference**: Resolves localized currency from client region or family record.
//! - **Family Lifecycle**: Fetches and modifies household name and primary base currency.
//! - **Household Overview Assembly**: Concurrently collects family metadata, member roster,
//!   and polymorphic financial accounts into a unified [`FamilyDetailsDto`].
//! - **Initial Seeding**: Idempotently provisions canonical household data on initial startup.

use sqlx::{Row, SqlitePool};

use crate::core::{db_err, is_unique_violation, now_epoch_secs, AppError, DbPool, DbResultExt};

use super::super::error::FamilyError;
use super::super::models::{
    AccountDto, BankAccountDto, CreditCardDto, CurrencyCode, FamilyDetailsDto, FamilyDto,
    FamilyName, MemberDto,
};

/// Resolves a localized default currency from an optional ISO country code.
///
/// Avoids arbitrary USD defaulting by inspecting the ISO region string. Defaults to
/// `"INR"` if region is absent or unrecognized.
///
/// # Ingress
/// - `region`: Optional 2-letter ISO country/region code (e.g. `"US"`, `"GB"`, `"DE"`, `"IN"`).
///
/// # Returns
/// - 3-letter uppercase ISO-4217 currency code [`String`].
pub(crate) fn resolve_default_currency(region: Option<&str>) -> String {
    if let Some(r) = region {
        let r_upper = r.trim().to_uppercase();
        let currency = match r_upper.as_str() {
            "IN" => "INR",
            "GB" | "UK" => "GBP",
            "DE" | "FR" | "IT" | "ES" | "NL" | "BE" | "IE" | "PT" | "AT" | "FI" | "GR" => "EUR",
            "CA" => "CAD",
            "AU" => "AUD",
            "JP" => "JPY",
            "CH" => "CHF",
            "SG" => "SGD",
            "NZ" => "NZD",
            "AE" => "AED",
            "CN" => "CNY",
            "BR" => "BRL",
            "MX" => "MXN",
            "KR" => "KRW",
            "SE" => "SEK",
            "NO" => "NOK",
            "DK" => "DKK",
            "ZA" => "ZAR",
            "HK" => "HKD",
            "US" => "USD",
            _ => "INR",
        };
        return currency.to_string();
    }
    "INR".to_string()
}

/// Retrieves the default currency for the household.
///
/// Consults the existing family record in the database first. If found with a configured
/// currency, returns it; otherwise infers the currency from the client's region.
///
/// # Execution Model
/// Single-shot read query attempting [`get_family`], with pure regional inference fallback.
///
/// # Ingress
/// - `pool`: Reference to shared [`DbPool`].
/// - `region`: Optional client ISO country code.
///
/// # Returns
/// - `Ok(String)` containing the 3-letter currency code.
///
/// # Errors
/// - Returns [`AppError::ShouldNotBeHappening`] on unexpected database failure.
pub(crate) async fn get_default_currency(
    pool: &DbPool,
    region: Option<&str>,
) -> Result<String, AppError> {
    if let Ok(family) = get_family(pool).await {
        if !family.currency().is_empty() {
            return Ok(family.currency().to_string());
        }
    }
    Ok(resolve_default_currency(region))
}

/// Fetches the primary household family record.
///
/// Queries the lowest-ID family entity representing the primary household configuration.
///
/// # Execution Model
/// Executes a single-shot atomic `SELECT ... ORDER BY id ASC LIMIT 1` directly against [`DbPool`].
///
/// # Ingress
/// - `pool`: Reference to shared [`DbPool`].
///
/// # Returns
/// - `Ok(FamilyDto)` representing the primary household family.
///
/// # Errors
/// - Returns [`FamilyError::FamilyNotFound`] if no family record exists.
/// - Returns [`AppError::ShouldNotBeHappening`] on query failure.
pub(crate) async fn get_family(pool: &DbPool) -> Result<FamilyDto, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, family_name, currency, created_at
        FROM families
        ORDER BY id ASC
        LIMIT 1;
        "#,
    )
    .fetch_optional(pool)
    .await
    .db_context("FAMILY.GET_FAMILY.QUERY")?;

    match row {
        Some(r) => Ok(FamilyDto::new(
            r.get("id"),
            r.get::<String, _>("family_name"),
            r.get::<String, _>("currency"),
            r.get("created_at"),
        )),
        None => Err(FamilyError::FamilyNotFound {
            action: "FAMILY.GET_FAMILY.NOT_FOUND",
        }
        .into()),
    }
}

/// Updates the family record display name and/or base currency.
///
/// Fetches the existing family record, merges incoming modifications, and persists
/// changes in a single atomic SQL statement with unique name collision checking.
///
/// # Execution Model
/// Executes a single atomic `UPDATE` directly against [`DbPool`]. Engine-level SQLite
/// constraint violations are classified via [`is_unique_violation`].
///
/// # Ingress
/// - `pool`: Reference to shared [`DbPool`].
/// - `name`: Optional validated [`FamilyName`] Value Object.
/// - `currency`: Optional validated [`CurrencyCode`] Value Object.
///
/// # Returns
/// - `Ok(FamilyDto)` representing the updated household family.
///
/// # Errors
/// - Returns [`FamilyError::FamilyNotFound`] if no family record exists.
/// - Returns [`FamilyError::FamilyAlreadyExists`] if the updated family name collides with another family.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn update_family(
    pool: &DbPool,
    name: Option<FamilyName>,
    currency: Option<CurrencyCode>,
) -> Result<FamilyDto, AppError> {
    let existing = get_family(pool).await?;

    let new_name = match name {
        Some(n) => n.into_inner(),
        None => existing.family_name().to_string(),
    };

    let new_currency = match currency {
        Some(c) => c.into_inner(),
        None => existing.currency().to_string(),
    };

    let now = now_epoch_secs();
    let res = sqlx::query(
        r#"
        UPDATE families
        SET family_name = ?, currency = ?, updated_at = ?
        WHERE id = ?;
        "#,
    )
    .bind(&new_name)
    .bind(&new_currency)
    .bind(now)
    .bind(existing.id())
    .execute(pool)
    .await;

    match res {
        Ok(_) => Ok(FamilyDto::new(
            existing.id(),
            new_name,
            new_currency,
            existing.created_at(),
        )),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::FamilyAlreadyExists {
                    action: "FAMILY.UPDATE_FAMILY.ALREADY_EXISTS",
                    family_name: new_name,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_FAMILY.EXECUTE", err))
            }
        }
    }
}

/// Assembles the complete family details (family, members, accounts) in a single workflow.
///
/// Queries family metadata, the full member roster, and all owned depository and revolving
/// financial accounts, constructing a composite [`FamilyDetailsDto`].
///
/// # Execution Model
/// Executes 3 read queries (`families`, `members`, `accounts`) against [`DbPool`], mapping
/// account rows into polymorphic [`AccountDto`] variants (bank accounts and credit cards).
///
/// # Ingress
/// - `pool`: Reference to shared [`DbPool`].
///
/// # Returns
/// - `Ok(FamilyDetailsDto)` aggregating household, members, and accounts.
///
/// # Errors
/// - Returns [`FamilyError::FamilyNotFound`] if the primary family does not exist.
/// - Returns [`AppError::ShouldNotBeHappening`] on database execution failure.
pub(crate) async fn get_family_details(pool: &DbPool) -> Result<FamilyDetailsDto, AppError> {
    let family = get_family(pool).await?;

    let member_rows = sqlx::query(
        r#"
        SELECT id, family_id, member_name, created_at
        FROM members
        WHERE family_id = ?
        ORDER BY id ASC;
        "#,
    )
    .bind(family.id())
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
        SELECT id, family_id, owner_member_id, type, currency, bank_name, last4,
               account_name, available_balance_cents, credit_limit_cents,
               available_cents, created_at
        FROM accounts
        WHERE family_id = ?
        ORDER BY id ASC;
        "#,
    )
    .bind(family.id())
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
                r.get::<String, _>("currency"),
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
                r.get::<String, _>("currency"),
                r.get::<String, _>("bank_name"),
                r.get::<String, _>("account_name"),
                r.get::<String, _>("last4"),
                limit,
                available,
                r.get("created_at"),
            )));
        }
    }

    Ok(FamilyDetailsDto::new(family, members, accounts))
}

/// Seeds initial family, members, and accounts if the families table is empty.
///
/// Idempotently checks if any family record exists. If empty, creates "The Miller Family"
/// with 3 members (Sarah, David, Emma) and sample bank/credit accounts within an explicit
/// transaction block.
///
/// # Execution Model
/// Executes a scalar count query, followed by an explicit transaction (`pool.begin()`)
/// performing sequential inserts with dedicated action tokens and committing atomically.
///
/// # Ingress
/// - `pool`: Reference to shared [`SqlitePool`].
///
/// # Returns
/// - `Ok(())` on successful seeding or if already seeded.
///
/// # Errors
/// - Returns [`AppError::ShouldNotBeHappening`] on database transaction failure.
pub(crate) async fn seed_default_family(pool: &SqlitePool) -> Result<(), AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM families;")
        .fetch_one(pool)
        .await
        .db_context("FAMILY.SEED.COUNT_CHECK")?;

    if count > 0 {
        return Ok(());
    }

    let mut tx = pool.begin().await.db_context("FAMILY.SEED.TX_BEGIN")?;
    let now = now_epoch_secs();
    let initial_currency = resolve_default_currency(None);

    let family_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO families (family_name, currency, created_at, updated_at)
        VALUES ('The Miller Family', ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(&initial_currency)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .db_context("FAMILY.SEED.INSERT_FAMILY")?;

    // Insert initial members
    let sarah_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO members (family_id, member_name, created_at, updated_at)
        VALUES (?, 'Sarah Miller', ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .db_context("FAMILY.SEED.INSERT_SARAH")?;

    let david_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO members (family_id, member_name, created_at, updated_at)
        VALUES (?, 'David Miller', ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .db_context("FAMILY.SEED.INSERT_DAVID")?;

    let emma_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO members (family_id, member_name, created_at, updated_at)
        VALUES (?, 'Emma Miller', ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .db_context("FAMILY.SEED.INSERT_EMMA")?;

    // Insert accounts for Sarah
    sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', ?, 'Chase', '4821', 'Total Checking', 845025, ?, ?);
        "#,
    )
    .bind(family_id)
    .bind(sarah_id)
    .bind(&initial_currency)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .db_context("FAMILY.SEED.SARAH_BANK1")?;

    sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, credit_limit_cents, available_cents, created_at, updated_at
        ) VALUES (?, ?, 'credit_card', ?, 'Chase', '5561', 'Sapphire Preferred', 2000000, 1785000, ?, ?);
        "#,
    )
    .bind(family_id)
    .bind(sarah_id)
    .bind(&initial_currency)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .db_context("FAMILY.SEED.SARAH_CARD1")?;

    // Insert accounts for David
    sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', ?, 'Wells Fargo', '1140', 'Everyday Checking', 320000, ?, ?);
        "#,
    )
    .bind(family_id)
    .bind(david_id)
    .bind(&initial_currency)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .db_context("FAMILY.SEED.DAVID_BANK1")?;

    // Insert account for Emma
    sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', ?, 'Charles Schwab', '3309', 'Investor Checking', 125000, ?, ?);
        "#,
    )
    .bind(family_id)
    .bind(emma_id)
    .bind(&initial_currency)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .db_context("FAMILY.SEED.EMMA_BANK1")?;

    tx.commit().await.db_context("FAMILY.SEED.TX_COMMIT")?;

    Ok(())
}
