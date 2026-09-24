use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::{fs, path::Path, str::FromStr};

use crate::core::NewAppError;

pub(crate) type DbPool = Pool<Sqlite>;

/// Initializes the SQLite connection pool and creates tables if not present.
pub(crate) async fn init_db(database_url: &str) -> Result<DbPool, NewAppError> {
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
        // TODO: change to new
        .unwrap()
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        // TODO: change to new
        .unwrap();

    tracing::info!(
        database_url = %database_url,
        "initialized SQLite connection pool with WAL journal mode"
    );

    crate::features::init_schemas(&pool).await?;

    Ok(pool)
}

pub(crate) async fn create_db_object(
    task: String,
    object_name: String,
    pool: &DbPool,
    sql: &str,
) -> Result<(), NewAppError> {
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| NewAppError::init_schema(task, object_name, e))?;

    Ok(())
}
