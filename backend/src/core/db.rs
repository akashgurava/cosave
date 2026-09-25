use std::fs;
use std::path::Path;
use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};

use super::error::AppError;

pub type DbPool = Pool<Sqlite>;

/// Extension trait for mapping `sqlx::Error` into `AppError::ShouldNotBeHappening`.
pub(crate) trait DbResultExt<T> {
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

/// Helper function to convert a `sqlx::Error` into an `AppError::ShouldNotBeHappening`.
pub(crate) fn db_err(action: &'static str, err: sqlx::Error) -> AppError {
    AppError::ShouldNotBeHappening {
        action,
        reason: format!("database error: {err}"),
    }
}

/// Initializes the SQLite connection pool and creates tables if not present.
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
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .db_context("CORE.INIT_DB.CONNECT")?;

    tracing::info!(
        database_url = %database_url,
        "initialized SQLite connection pool with WAL journal mode"
    );

    crate::features::init_schemas(&pool).await?;

    Ok(pool)
}

pub(crate) async fn create_db_object(
    action: &'static str,
    table: &'static str,
    pool: &DbPool,
    sql: &str,
) -> Result<(), AppError> {
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::InitSchema {
            action,
            table,
            source: e,
        })?;

    Ok(())
}
