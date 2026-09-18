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

    tracing::info!(
        database_url = %database_url,
        "initialized SQLite connection pool with WAL journal mode"
    );

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

        CREATE TABLE IF NOT EXISTS transaction_types (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            color TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY NOT NULL,
            type_id TEXT NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, name)
        );

        CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);

        CREATE TABLE IF NOT EXISTS subcategories (
            id TEXT PRIMARY KEY NOT NULL,
            category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(category_id, name)
        );

        CREATE INDEX IF NOT EXISTS idx_subcategories_category_id ON subcategories(category_id);

        CREATE VIEW IF NOT EXISTS v_category_hierarchy AS
        SELECT
            t.id AS type_id,
            t.name AS type_name,
            t.color AS type_color,
            t.sort_order AS type_sort_order,
            c.id AS category_id,
            c.name AS category_name,
            c.sort_order AS category_sort_order,
            s.id AS subcategory_id,
            s.name AS subcategory_name,
            s.sort_order AS subcategory_sort_order
        FROM transaction_types t
        LEFT JOIN categories c ON c.type_id = t.id
        LEFT JOIN subcategories s ON s.category_id = c.id
        ORDER BY t.sort_order, t.name, c.sort_order, c.name, s.sort_order, s.name;
        "#,
    )
    .execute(pool)
    .await?;

    // Seed default categories if table is empty
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_types")
        .fetch_one(pool)
        .await?;
    if count.0 == 0 {
        crate::features::categories::db::seed_default_categories(pool).await?;
    }

    Ok(())
}
