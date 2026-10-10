//! Database schema initialization and table creation for transactions, staging, manual entries, and unified ledger view.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Initializes the transaction domain schema for statement ingestion, manual entries, and unified master ledger view.
///
/// Provisions the relational tables and views supporting the parallel two-stream ingestion architecture:
/// 1. `statement_imports`: Batch metadata tracking for uploaded statement files.
/// 2. `raw_statement_rows`: 100% unbiased raw statement line storage as JSON key-value mappings.
/// 3. `staging_transactions`: Normalized statement rows (1:1 shared PK with raw rows) for deduplication.
/// 4. `imported_transactions`: Finalized ingested transactions promoted from staging.
/// 5. `manual_transactions`: User-entered manual transactions.
/// 6. `transactions`: Unified SQL VIEW projecting both manual and imported streams via `UNION ALL`.
///
/// # Domain Rules & Referential Integrity
/// - **Tenant Isolation**: `statement_imports`, `manual_transactions`, and `imported_transactions` cascade delete if their parent family is deleted (`ON DELETE CASCADE`).
/// - **Account Attachment**: Transactions are strictly anchored to accounts (`account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE`), so deleting an account cascades to all associated transactions.
/// - **Preserved Taxonomy**: Foreign keys to types and categories use `ON DELETE RESTRICT` preventing destruction of referenced hierarchy.
/// - **Zero Redundancy View**: `transactions` is a unified SQL view projecting `source` ('manual' vs 'import') along with all transaction columns.
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
            id TEXT PRIMARY KEY NOT NULL,
            raw_row_id INTEGER REFERENCES raw_statement_rows(id) ON DELETE CASCADE,
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
        "TRANSACTIONS.INIT_SCHEMA.IMPORTED_TRANSACTIONS_TABLE",
        "imported_transactions",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS imported_transactions (
            id TEXT PRIMARY KEY NOT NULL REFERENCES staging_transactions(id) ON DELETE CASCADE,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            type_id INTEGER NOT NULL REFERENCES transaction_types(id) ON DELETE RESTRICT,
            category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
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
        "TRANSACTIONS.INIT_SCHEMA.IDX_IMPORTED_TRANSACTIONS_FAMILY_DATE",
        "imported_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_imported_transactions_family_date ON imported_transactions(family_id, date DESC);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.IDX_IMPORTED_TRANSACTIONS_ACCOUNT_ID",
        "imported_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_imported_transactions_account_id ON imported_transactions(account_id);
        "#,
    )
    .await?;

    create_db_object(
        "TRANSACTIONS.INIT_SCHEMA.MANUAL_TRANSACTIONS_TABLE",
        "manual_transactions",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS manual_transactions (
            id TEXT PRIMARY KEY NOT NULL,
            family_id INTEGER NOT NULL REFERENCES families(id) ON DELETE CASCADE,
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            type_id INTEGER NOT NULL REFERENCES transaction_types(id) ON DELETE RESTRICT,
            category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
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
        "TRANSACTIONS.INIT_SCHEMA.IDX_MANUAL_TRANSACTIONS_FAMILY_DATE",
        "manual_transactions",
        tx,
        r#"
        CREATE INDEX IF NOT EXISTS idx_manual_transactions_family_date ON manual_transactions(family_id, date DESC);
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
        "TRANSACTIONS.INIT_SCHEMA.TRANSACTIONS_VIEW",
        "transactions",
        tx,
        r#"
        CREATE VIEW IF NOT EXISTS transactions AS
        SELECT 
            id,
            'manual' AS source,
            family_id,
            account_id,
            type_id,
            category_id,
            subcategory_id,
            amount,
            date,
            description,
            payee,
            notes,
            status,
            created_at,
            updated_at
        FROM manual_transactions

        UNION ALL

        SELECT 
            id,
            'import' AS source,
            family_id,
            account_id,
            type_id,
            category_id,
            subcategory_id,
            amount,
            date,
            description,
            payee,
            notes,
            status,
            created_at,
            updated_at
        FROM imported_transactions;
        "#,
    )
    .await?;

    Ok(())
}
