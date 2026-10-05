//! Database connectivity, connection pooling, and DDL execution.
//!
//! Provides the shared asynchronous SQLite connection pool represented by [`DbPool`],
//! configured with write-ahead logging (WAL) and foreign key enforcement. The [`init_db`]
//! function ensures underlying storage directories exist on disk before connecting and
//! bootstrapping core system tables. Runtime database errors are converted into structured
//! application errors tagged with compile-time action identifiers.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use sqlx::{
    error::ErrorKind,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Executor, Pool, Sqlite, Transaction,
};

use super::{init_core_schema, AppError};

/// Default maximum concurrent SQLite connections allowed in the pool.
const DEFAULT_MAX_DB_CONNECTIONS: u32 = 10;

/// Shared thread-safe SQLite connection pool across Axum handlers and database operations.
pub type DbPool = Pool<Sqlite>;

/// Extension trait for ergonomic mapping of `sqlx::Error` into `AppError::ShouldNotBeHappening`.
///
/// Attaches a globally unique compile-time action token to runtime database failures,
/// preserving failure context without leaking raw SQL to client envelopes.
pub(crate) trait DbResultExt<T> {
    /// Maps an underlying database error into `AppError::ShouldNotBeHappening` tagged with `action`.
    fn db_context(self, action: &'static str) -> Result<T, AppError>;
}

impl<T> DbResultExt<T> for Result<T, sqlx::Error> {
    fn db_context(self, action: &'static str) -> Result<T, AppError> {
        self.map_err(|e| AppError::ShouldNotBeHappening {
            action,
            reason: format!("database error: {e}"),
        })
    }
}

/// Converts a `sqlx::Error` into `AppError::ShouldNotBeHappening` tagged with a compile-time action token.
///
/// Use this in fallback branches, match arms, or closures where the result has already been unwrapped
/// or destructured and a direct error return is required.
pub(crate) fn db_err(action: &'static str, err: sqlx::Error) -> AppError {
    AppError::ShouldNotBeHappening {
        action,
        reason: format!("database error: {err}"),
    }
}

/// Initializes the SQLite connection pool, ensures filesystem directories exist, and bootstraps core schema.
///
/// # Behavior
/// 1. If `database_url` is a file-based path (`sqlite://<path>`), ensures the parent directory exists on disk.
/// 2. Configures SQLite pragmas: `create_if_missing`, WAL journal mode (`SqliteJournalMode::Wal`), and `foreign_keys(true)`.
/// 3. Provisions an asynchronous connection pool bounded to `DEFAULT_MAX_DB_CONNECTIONS`.
/// 4. Executes `init_core_schema` inside an atomic transaction to bootstrap system metadata tables (`app_meta`).
/// 5. Emits structured informational log `CORE.INIT_DB.POOL_READY`.
///
/// # Errors
/// Returns [`AppError::ShouldNotBeHappening`] with action `CORE.INIT_DB.PARSE_OPTIONS` if the URL is invalid,
/// or `CORE.INIT_DB.CONNECT` if connection pool establishment fails.
pub async fn init_db(database_url: &str) -> Result<DbPool, AppError> {
    // If using a file-based sqlite URL, ensure parent directory exists
    if let Some(file_path) = database_url.strip_prefix("sqlite://") {
        let clean_path = file_path.split('?').next().unwrap_or(file_path);
        if clean_path != ":memory:" {
            if let Some(parent) = Path::new(clean_path).parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    let _ = fs::create_dir_all(parent);
                }
            }
        }
    }

    let options = SqliteConnectOptions::from_str(database_url)
        .db_context("CORE.INIT_DB.PARSE_OPTIONS")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(DEFAULT_MAX_DB_CONNECTIONS)
        .connect_with(options)
        .await
        .db_context("CORE.INIT_DB.CONNECT")?;

    let mut tx = pool.begin().await.db_context("CORE.INIT_DB.TX_BEGIN")?;
    init_core_schema(&mut tx).await?;
    tx.commit().await.db_context("CORE.INIT_DB.TX_COMMIT")?;

    tracing::info!(
        database_url = %database_url,
        "CORE.INIT_DB.POOL_READY. Initialized SQLite connection pool with WAL journal mode"
    );

    Ok(pool)
}

/// Executes an isolated DDL statement (e.g. `CREATE TABLE`, `CREATE INDEX`) within a transaction tagged with dedicated metadata.
///
/// Statement execution is performed via `tx.execute(sqlx::query(sql))` on an active transaction handle,
/// mapping any DDL failure into [`AppError::InitSchema`] while capturing the exact action token,
/// target table name, and underlying `sqlx::Error`.
pub(crate) async fn create_db_object(
    action: &'static str,
    table: &'static str,
    tx: &mut Transaction<'_, Sqlite>,
    sql: &str,
) -> Result<(), AppError> {
    tx.execute(sqlx::query(sql))
        .await
        .map_err(|e| AppError::InitSchema {
            action,
            table,
            source: e,
        })?;

    Ok(())
}

/// Inspects a [`sqlx::Error`] to determine whether it was caused by a unique constraint or primary key violation.
///
/// Evaluates the underlying database error against [`ErrorKind::UniqueViolation`], native
/// [`sqlx::error::DatabaseError::is_unique_violation`], and fallback SQLite error message patterns
/// for compound `UNIQUE` or `PRIMARY KEY` conflicts.
///
/// # Ingress
/// - `err`: Reference to the underlying [`sqlx::Error`] returned by query execution.
///
/// # Returns
/// - `true` if the error originates from a duplicate key or unique constraint violation.
/// - `false` otherwise.
pub(crate) fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.kind() == ErrorKind::UniqueViolation || db_err.is_unique_violation() {
            return true;
        }
        let msg = db_err.message().to_lowercase();
        if msg.contains("unique") || msg.contains("primary key") {
            return true;
        }
    }
    false
}

/// Inspects a [`sqlx::Error`] to determine whether it was caused by a foreign key constraint violation.
///
/// Evaluates the underlying database error against [`ErrorKind::ForeignKeyViolation`], native
/// [`sqlx::error::DatabaseError::is_foreign_key_violation`], and fallback SQLite error message patterns
/// for foreign key constraint failures.
///
/// # Ingress
/// - `err`: Reference to the underlying [`sqlx::Error`] returned by query execution.
///
/// # Returns
/// - `true` if the error originates from a foreign key constraint failure.
/// - `false` otherwise.
pub(crate) fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.kind() == ErrorKind::ForeignKeyViolation || db_err.is_foreign_key_violation() {
            return true;
        }
        let msg = db_err.message().to_lowercase();
        if msg.contains("foreign key") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::AppConfig;

    async fn create_test_pool() -> DbPool {
        init_db(AppConfig::IN_MEMORY_DATABASE_URL)
            .await
            .expect("init test sqlite in-memory db")
    }

    #[tokio::test]
    async fn test_is_unique_violation_on_primary_key_conflict() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE test_pk (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_pk table");

        sqlx::query("INSERT INTO test_pk (id, name) VALUES (1, 'Alice');")
            .execute(&pool)
            .await
            .expect("insert first row");

        let err = sqlx::query("INSERT INTO test_pk (id, name) VALUES (1, 'Duplicate Alice');")
            .execute(&pool)
            .await
            .expect_err("insert duplicate primary key must fail");

        assert!(
            is_unique_violation(&err),
            "primary key conflict must be identified as unique violation: {err}"
        );
        assert!(
            !is_foreign_key_violation(&err),
            "primary key conflict must not be identified as foreign key violation"
        );
    }

    #[tokio::test]
    async fn test_is_unique_violation_on_single_column_unique() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE test_unique (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_unique table");

        sqlx::query("INSERT INTO test_unique (email) VALUES ('user@example.com');")
            .execute(&pool)
            .await
            .expect("insert first unique email");

        let err = sqlx::query("INSERT INTO test_unique (email) VALUES ('user@example.com');")
            .execute(&pool)
            .await
            .expect_err("insert duplicate unique email must fail");

        assert!(
            is_unique_violation(&err),
            "unique column conflict must be identified as unique violation: {err}"
        );
        assert!(
            !is_foreign_key_violation(&err),
            "unique column conflict must not be identified as foreign key violation"
        );
    }

    #[tokio::test]
    async fn test_is_unique_violation_on_compound_unique_constraint() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE test_compound_unique (
                parent_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                UNIQUE (parent_id, name)
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_compound_unique table");

        sqlx::query("INSERT INTO test_compound_unique (parent_id, name) VALUES (1, 'Housing');")
            .execute(&pool)
            .await
            .expect("insert first compound unique row");

        // Same name under different parent should succeed
        sqlx::query("INSERT INTO test_compound_unique (parent_id, name) VALUES (2, 'Housing');")
            .execute(&pool)
            .await
            .expect("insert same name under different parent must succeed");

        // Same name under same parent must fail with unique violation
        let err = sqlx::query(
            "INSERT INTO test_compound_unique (parent_id, name) VALUES (1, 'Housing');",
        )
        .execute(&pool)
        .await
        .expect_err("insert duplicate compound unique must fail");

        assert!(
            is_unique_violation(&err),
            "compound unique conflict must be identified as unique violation: {err}"
        );
        assert!(
            !is_foreign_key_violation(&err),
            "compound unique conflict must not be identified as foreign key violation"
        );
    }

    #[tokio::test]
    async fn test_is_foreign_key_violation_on_insert_nonexistent_parent() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE test_parent (
                id INTEGER PRIMARY KEY
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_parent table");

        sqlx::query(
            "CREATE TABLE test_child (
                id INTEGER PRIMARY KEY,
                parent_id INTEGER NOT NULL REFERENCES test_parent(id) ON DELETE CASCADE
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_child table");

        sqlx::query("INSERT INTO test_parent (id) VALUES (10);")
            .execute(&pool)
            .await
            .expect("insert valid parent");

        sqlx::query("INSERT INTO test_child (id, parent_id) VALUES (100, 10);")
            .execute(&pool)
            .await
            .expect("insert valid child referencing parent 10");

        // Insert child referencing non-existent parent 999
        let err = sqlx::query("INSERT INTO test_child (id, parent_id) VALUES (200, 999);")
            .execute(&pool)
            .await
            .expect_err("inserting non-existent parent reference must fail with FK error");

        assert!(
            is_foreign_key_violation(&err),
            "foreign key failure must be identified as foreign key violation: {err}"
        );
        assert!(
            !is_unique_violation(&err),
            "foreign key failure must not be identified as unique violation"
        );
    }

    #[tokio::test]
    async fn test_is_foreign_key_violation_on_delete_restrict() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE test_parent_restrict (
                id INTEGER PRIMARY KEY
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_parent_restrict table");

        sqlx::query(
            "CREATE TABLE test_child_restrict (
                id INTEGER PRIMARY KEY,
                parent_id INTEGER NOT NULL REFERENCES test_parent_restrict(id) ON DELETE RESTRICT
            );",
        )
        .execute(&pool)
        .await
        .expect("create test_child_restrict table");

        sqlx::query("INSERT INTO test_parent_restrict (id) VALUES (1);")
            .execute(&pool)
            .await
            .expect("insert parent");

        sqlx::query("INSERT INTO test_child_restrict (id, parent_id) VALUES (10, 1);")
            .execute(&pool)
            .await
            .expect("insert child referencing parent 1");

        // Deleting parent when child exists with ON DELETE RESTRICT must fail with FK violation
        let err = sqlx::query("DELETE FROM test_parent_restrict WHERE id = 1;")
            .execute(&pool)
            .await
            .expect_err("delete parent with active restrict references must fail");

        assert!(
            is_foreign_key_violation(&err),
            "restricted delete must be identified as foreign key violation: {err}"
        );
        assert!(
            !is_unique_violation(&err),
            "restricted delete must not be identified as unique violation"
        );
    }

    #[tokio::test]
    async fn test_non_constraint_error_returns_false_for_both() {
        let pool = create_test_pool().await;

        // Query non-existent table
        let err = sqlx::query("SELECT * FROM table_that_does_not_exist_xyz;")
            .execute(&pool)
            .await
            .expect_err("querying non-existent table must fail");

        assert!(
            !is_unique_violation(&err),
            "syntax or table missing error is not a unique violation"
        );
        assert!(
            !is_foreign_key_violation(&err),
            "syntax or table missing error is not a foreign key violation"
        );
    }
}
