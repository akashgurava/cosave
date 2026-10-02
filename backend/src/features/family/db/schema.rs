//! Database schema initialization and table creation for families, members, and accounts.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Initializes the family domain schema for families, members, and financial accounts.
///
/// Provisions the relational structures supporting multi-member households and their
/// financial instruments (depository bank accounts and credit cards).
///
/// # Domain Rules & Referential Integrity
/// - **Tenant Cascade**: Deleting a family cascades to all its members and accounts (`ON DELETE CASCADE`).
///   Deleting a member cascades to all accounts owned by that member.
/// - **Scoped Uniqueness**:
///   - `family_name` is globally unique.
///   - `member_name` is scoped per family (`UNIQUE(family_id, member_name)`).
///   - `account_name` is scoped per owner member (`UNIQUE(owner_member_id, account_name)`).
/// - **Fast Member Traversal**: Indexed on `members(family_id)` to optimize household lookups.
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
        "FAMILY.INIT_SCHEMA.FAMILIES_TABLE",
        "families",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS families (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_name TEXT UNIQUE NOT NULL,
            currency TEXT NOT NULL,
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
            member_name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(family_id, member_name)
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
            type TEXT NOT NULL,
            currency TEXT NOT NULL,
            bank_name TEXT NOT NULL,
            account_name TEXT NOT NULL,
            last4 TEXT NOT NULL,
            available_balance_cents INTEGER,
            credit_limit_cents INTEGER,
            available_cents INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(owner_member_id, account_name)
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
