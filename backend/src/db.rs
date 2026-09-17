use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::{fs, path::Path, str::FromStr};

pub(crate) type DbPool = Pool<Sqlite>;

/// Initializes the SQLite connection pool and creates tables if not present.
pub(crate) async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
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

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    init_schema(&pool).await?;

    Ok(pool)
}

/// Runs initial table and index creation migrations.
async fn init_schema(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
