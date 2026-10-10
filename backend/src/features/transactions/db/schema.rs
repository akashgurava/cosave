//! Database schema initialization and table creation for transactions, staging, manual entries, and lineage sources.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Initializes the transaction domain schema for statement ingestion, manual entries, master ledger, and lineage sources.
///
/// Provisions the relational tables supporting the parallel two-stream ingestion architecture:
/// 1. `statement_imports`: Batch metadata tracking for uploaded statement files.
/// 2. `raw_statement_rows`: 100% unbiased raw statement line storage as JSON key-value mappings.
/// 3. `staging_transactions`: Normalized statement rows (1:1 shared PK with raw rows) for deduplication.
/// 4. `manual_transactions`: User-entered transactions and manual override snapshots.
/// 5. `transaction_sources`: Primary transaction identity registry sitting on the union of staging and manual streams.
/// 6. `transactions`: Pure master family ledger sharing primary key 1:1 with `transaction_sources(id)`.
///
/// # Domain Rules & Referential Integrity
/// - **Tenant Isolation**: `statement_imports`, `manual_transactions`, and `transaction_sources` cascade delete if their parent family is deleted (`ON DELETE CASCADE`).
/// - **Preserved Financial Reality**: Deleting an account, member, type, category, or subcategory sets foreign keys to `NULL` on `transactions` (`ON DELETE SET NULL`), ensuring historical financial records are never destroyed.
/// - **Authoritative Identity Registry**: `transaction_sources` generates the authoritative transaction ID and acts as the Single Source of Truth (SSOT) via `source_type` ('import' vs 'manual').
/// - **Clean Ledger Separation**: `transactions.id` references `transaction_sources.id` directly, keeping the ledger pure and fast for queries, filters, and charts.
///
/// # Execution & Idempotency
/// - Executes atomically within the caller-provided [`Transaction`].
/// - Idempotent across restarts using `CREATE ... IF NOT EXISTS` DDL.
/// - Executed via [`create_db_object`] with dedicated action tokens (`TRANSACTIONS.INIT_SCHEMA.*`).
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL statement fails.
pub(crate) async fn init_transaction_schema(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), AppError> {
    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.STATEMENT_IMPORTS_TABLE",
        "statement_imports",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS statement_imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
            filename TEXT NOT NULL,
            file_hash TEXT NOT NULL,
            file_format TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_STATEMENT_IMPORTS_ACCOUNT_ID",
        "statement_imports",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_statement_imports_account_id ON statement_imports(account_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_STATEMENT_IMPORTS_FAMILY_ID",
        "statement_imports",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_statement_imports_family_id ON statement_imports(family_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.RAW_STATEMENT_ROWS_TABLE",
        "raw_statement_rows",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS raw_statement_rows (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            import_id INTEGER NOT NULL REFERENCES statement_imports(id) ON DELETE CASCADE,
            row_index INTEGER NOT NULL,
            raw_payload TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_RAW_STATEMENT_ROWS_IMPORT_ID",
        "raw_statement_rows",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_raw_statement_rows_import_id ON raw_statement_rows(import_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.STAGING_TRANSACTIONS_TABLE",
        "staging_transactions",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS staging_transactions (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES raw_statement_rows(id) ON DELETE CASCADE,
            date INTEGER NOT NULL,
            amount INTEGER NOT NULL,
            description TEXT NOT NULL,
            balance INTEGER,
            ref_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            error_message TEXT,
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_STAGING_TRANSACTIONS_REF_ID",
        "staging_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_staging_transactions_ref_id ON staging_transactions(ref_id) WHERE ref_id IS NOT NULL;
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_STAGING_TRANSACTIONS_MATCH",
        "staging_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_staging_transactions_match ON staging_transactions(date, amount);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.MANUAL_TRANSACTIONS_TABLE",
        "manual_transactions",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS manual_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
            member_id INTEGER REFERENCES members(id) ON DELETE SET NULL,
            type_id INTEGER REFERENCES transaction_types(id) ON DELETE SET NULL,
            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            subcategory_id INTEGER REFERENCES subcategories(id) ON DELETE SET NULL,
            amount INTEGER NOT NULL,
            date INTEGER NOT NULL,
            description TEXT NOT NULL,
            payee TEXT,
            notes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_MANUAL_TRANSACTIONS_FAMILY_ID",
        "manual_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_manual_transactions_family_id ON manual_transactions(family_id, date DESC);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_MANUAL_TRANSACTIONS_ACCOUNT_ID",
        "manual_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_manual_transactions_account_id ON manual_transactions(account_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.TRANSACTION_SOURCES_TABLE",
        "transaction_sources",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS transaction_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            source_type TEXT NOT NULL,
            staging_id INTEGER REFERENCES staging_transactions(id) ON DELETE SET NULL,
            manual_id INTEGER REFERENCES manual_transactions(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_TRANSACTION_SOURCES_FAMILY_ID",
        "transaction_sources",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_transaction_sources_family_id ON transaction_sources(family_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_TRANSACTION_SOURCES_STAGING_ID",
        "transaction_sources",
        tx,
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_transaction_sources_staging_id ON transaction_sources(staging_id) WHERE staging_id IS NOT NULL;
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_TRANSACTION_SOURCES_MANUAL_ID",
        "transaction_sources",
        tx,
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_transaction_sources_manual_id ON transaction_sources(manual_id) WHERE manual_id IS NOT NULL;
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.TRANSACTIONS_TABLE",
        "transactions",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES transaction_sources(id) ON DELETE CASCADE,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
            member_id INTEGER REFERENCES members(id) ON DELETE SET NULL,
            type_id INTEGER REFERENCES transaction_types(id) ON DELETE SET NULL,
            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            subcategory_id INTEGER REFERENCES subcategories(id) ON DELETE SET NULL,
            amount INTEGER NOT NULL,
            date INTEGER NOT NULL,
            description TEXT NOT NULL,
            payee TEXT,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'cleared',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_TRANSACTIONS_FAMILY_DATE",
        "transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_transactions_family_date ON transactions(family_id, date DESC);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_TRANSACTIONS_ACCOUNT_ID",
        "transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_transactions_account_id ON transactions(account_id);
        "#,
    )
    .await?;

    Ok(())
}
