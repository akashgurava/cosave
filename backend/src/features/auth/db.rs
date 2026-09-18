use super::models::{Role, User};
use crate::core::db::DbPool;

pub(crate) async fn find_user_by_name(
    pool: &DbPool,
    name: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE name = ? COLLATE NOCASE")
        .bind(name)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn find_user_by_session_token(
    pool: &DbPool,
    token: &str,
    now: i64,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.name, u.password_hash, u.role, u.created_at, u.updated_at
        FROM users u
        INNER JOIN sessions s ON u.id = s.user_id
        WHERE s.id = ? AND s.expires_at > ?
        "#,
    )
    .bind(token)
    .bind(now)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn count_users(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub(crate) async fn create_user(
    pool: &DbPool,
    id: &str,
    name: &str,
    password_hash: &str,
    role: Role,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (id, name, password_hash, role, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(password_hash)
    .bind(role.as_str())
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn create_session(
    pool: &DbPool,
    id: &str,
    user_id: &str,
    expires_at: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn delete_session(pool: &DbPool, token: &str) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}
