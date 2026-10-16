//! Database schema initialization and table creation for families, members, and accounts.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Initializes the family domain schema for families, members, and financial accounts.
///
/// Provisions the relational structures supporting multi-member families and their
/// financial instruments (depository bank accounts and credit cards).
///
/// # Domain Rules & Referential Integrity
/// - **Tenant Cascade**: Deleting a family cascades to all its members and accounts (`ON DELETE CASCADE`).
///   Deleting a member cascades to all accounts owned by that member.
/// - **Scoped Uniqueness**:
///   - `name` is globally unique for families (`UNIQUE(name)`).
///   - `name` is scoped per family for members (`UNIQUE(family_id, name)`).
///   - account uniqueness is scoped per owner member and account name (`UNIQUE(owner_member_id, name)`).
/// - **Fast Member Traversal**: Indexed on `members(family_id)` to optimize family lookups.
///
/// # Execution & Idempotency
/// - Executes atomically within the caller-provided [`Transaction`].
/// - Idempotent across restarts using `CREATE ... IF NOT EXISTS` DDL.
/// - Executed via [`create_db_object`] with dedicated action tokens (`FAMILY.INIT_SCHEMA.*`).
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL statement fails.
pub(crate) async fn init_family_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), AppError> {
    create_db_object(
        "FAMILY.INIT_SCHEMA.CURRENCIES_TABLE",
        "currencies",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS currencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            code TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            scale INTEGER NOT NULL,
            sort_order INTEGER NOT NULL,
            region TEXT UNIQUE NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.FAMILIES_TABLE",
        "families",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS families (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT UNIQUE NOT NULL,
            currency_id INTEGER NOT NULL REFERENCES currencies(id) ON DELETE RESTRICT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.MEMBERS_TABLE",
        "members",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS members (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(family_id, name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.IDX_MEMBERS_FAMILY_ID",
        "members",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_members_family_id ON members(family_id);
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.ACCOUNTS_TABLE",
        "accounts",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            owner_member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
            type TEXT NOT NULL CHECK(type IN ('bank_account', 'credit_card')),
            currency_id INTEGER NOT NULL REFERENCES currencies(id) ON DELETE RESTRICT,
            name TEXT NOT NULL,
            available_balance INTEGER,
            credit_limit INTEGER,
            available_credit INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(owner_member_id, name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.IDX_ACCOUNTS_FAMILY_ID",
        "accounts",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_accounts_family_id ON accounts(family_id);
        "#,
    )
    .await?;

    create_db_object(
        "FAMILY.INIT_SCHEMA.IDX_ACCOUNTS_OWNER_ID",
        "accounts",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_accounts_owner_member_id ON accounts(owner_member_id);
        "#,
    )
    .await?;

    Ok(())
}
