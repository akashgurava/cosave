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
        seed_default_categories(pool).await?;
    }

    Ok(())
}

/// Seeds the default 4 types, 8 categories, and 14 subcategories.
pub(crate) async fn seed_default_categories(pool: &DbPool) -> Result<(), sqlx::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    tracing::info!("seeding default transaction types, categories, and subcategories");

    let mut tx = pool.begin().await?;

    // 4 Types
    let types = [
        ("type-income", "Income", "#10b981", 1),
        ("type-expense", "Expense", "#f43f5e", 2),
        ("type-transfer", "Transfer", "#71717a", 3),
        ("type-invest", "Invest", "#3b82f6", 4),
    ];

    for (id, name, color, sort) in types {
        sqlx::query(
            r#"
            INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(color)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 8 Categories: (id, type_id, name, sort)
    let categories = [
        ("cat-inc-salary", "type-income", "Salary", 1),
        ("cat-inc-freelance", "type-income", "Freelance", 2),
        ("cat-exp-housing", "type-expense", "Housing", 1),
        ("cat-exp-food", "type-expense", "Food", 2),
        ("cat-exp-transport", "type-expense", "Transport", 3),
        ("cat-exp-personal", "type-expense", "Personal", 4),
        ("cat-trf-internal", "type-transfer", "Internal", 1),
        ("cat-inv-equities", "type-invest", "Equities", 1),
    ];

    for (id, type_id, name, sort) in categories {
        sqlx::query(
            r#"
            INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(type_id)
        .bind(name)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 14 Subcategories: (id, category_id, name, sort)
    let subcategories = [
        // Salary
        ("sub-inc-primary", "cat-inc-salary", "Primary Employer", 1),
        ("sub-inc-bonus", "cat-inc-salary", "Bonus", 2),
        // Freelance
        ("sub-inc-consulting", "cat-inc-freelance", "Consulting", 1),
        ("sub-inc-retainers", "cat-inc-freelance", "Retainers", 2),
        // Housing
        ("sub-exp-rent", "cat-exp-housing", "Rent", 1),
        ("sub-exp-utilities", "cat-exp-housing", "Utilities", 2),
        // Food
        ("sub-exp-groceries", "cat-exp-food", "Groceries", 1),
        ("sub-exp-dining", "cat-exp-food", "Dining Out", 2),
        // Transport
        ("sub-exp-fuel", "cat-exp-transport", "Fuel", 1),
        ("sub-exp-transit", "cat-exp-transport", "Public Transit", 2),
        // Personal
        ("sub-exp-fitness", "cat-exp-personal", "Gym & Fitness", 1),
        // Internal
        (
            "sub-trf-checking",
            "cat-trf-internal",
            "Checking to Savings",
            1,
        ),
        ("sub-trf-emergency", "cat-trf-internal", "Emergency Fund", 2),
        // Equities
        ("sub-inv-etf", "cat-inv-equities", "Index ETFs", 1),
    ];

    for (id, cat_id, name, sort) in subcategories {
        sqlx::query(
            r#"
            INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(cat_id)
        .bind(name)
        .bind(sort)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Atomically resets categories to default 4 types, 8 categories, and 14 subcategories.
pub(crate) async fn reset_default_categories(pool: &DbPool) -> Result<(), sqlx::Error> {
    tracing::info!("resetting categories to default configuration");
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM transaction_types")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    seed_default_categories(pool).await
}
