use sqlx::Executor;

use crate::core::{create_db_object, now_epoch_secs, AppError, DbPool, DbResultExt};

/// Initializes core system metadata tables.
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

/// Retrieves a metadata value by key, returning `None` if not set.
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

/// Upserts a metadata key-value pair within an existing transaction.
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
