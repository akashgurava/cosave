//! Core system metadata management and schema initialization.
//!
//! Owns the `app_meta` table which stores application-level key-value configuration,
//! schema flags, and declarative seed state. Timestamps are tracked in UTC epoch seconds
//! via [`now_epoch_secs`].

use sqlx::Executor;

use super::{create_db_object, now_epoch_secs, AppError, DbPool, DbResultExt};

/// Initializes the core metadata database schema.
///
/// # Database Objects Created
/// - **Tables**:
///   - `app_meta`: System key-value metadata store (`key TEXT PRIMARY KEY NOT NULL`, `value TEXT NOT NULL`, `updated_at INTEGER NOT NULL`).
/// - **Indexes**:
///   - None (primary key index on `key` is maintained implicitly by SQLite).
/// - **Views / Triggers**:
///   - None.
///
/// # Invariants
/// - Uses idempotent `CREATE TABLE IF NOT EXISTS` DDL.
/// - Executed strictly via [`create_db_object`] with action `CORE.META.INIT_SCHEMA.APP_META_TABLE`.
/// - `updated_at` stores UTC epoch seconds sourced from [`now_epoch_secs`].
///
/// # Errors
/// Returns [`AppError::InitSchema`] if table creation DDL fails.
pub(crate) async fn init_core_schema(pool: &DbPool) -> Result<(), AppError> {
    create_db_object(
        "CORE.META.INIT_SCHEMA.APP_META_TABLE",
        "app_meta",
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await
}

/// Retrieves a metadata value by its unique key.
///
/// # Ingress
/// - `action`: Globally unique compile-time action token provided by the caller (`FEATURE.WORKFLOW.STEP`).
/// - `pool`: Reference to the shared [`DbPool`].
/// - `key`: Unique metadata identifier string.
///
/// # Returns
/// - `Ok(Some(value))` if the key exists.
/// - `Ok(None)` if the key has not been set.
///
/// # Errors
/// Returns [`AppError::ShouldNotBeHappening`] with the caller's `action` if the database query fails.
pub(crate) async fn get_meta(
    action: &'static str,
    pool: &DbPool,
    key: &str,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM app_meta WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .db_context(action)?;

    Ok(row.map(|r| r.0))
}

/// Upserts a metadata key-value pair within an existing atomic transaction.
///
/// If the `key` already exists in `app_meta`, updates its `value` and refreshes `updated_at`
/// with the current UTC epoch seconds. If not, inserts a new row.
///
/// # Ingress
/// - `action`: Globally unique compile-time action token provided by the caller (`FEATURE.WORKFLOW.STEP`).
/// - `tx`: Active SQLite transaction handle (`&mut Transaction<'_, Sqlite>`).
/// - `key`: Metadata identifier string.
/// - `value`: New metadata string value.
///
/// # Errors
/// Returns [`AppError::ShouldNotBeHappening`] with the caller's `action` if query execution fails.
pub(crate) async fn set_meta_tx(
    action: &'static str,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    let now = now_epoch_secs();
    tx.execute(
        sqlx::query(
            r#"
            INSERT INTO app_meta (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now),
    )
    .await
    .db_context(action)?;

    Ok(())
}
