use std::fs;
use std::path::Path;
use std::str::FromStr;

use sqlx::{
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
/// 3. Provisions an asynchronous connection pool bounded to [`DEFAULT_MAX_DB_CONNECTIONS`].
/// 4. Executes [`init_core_schema`] inside an atomic transaction to bootstrap system metadata tables (`app_meta`).
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
