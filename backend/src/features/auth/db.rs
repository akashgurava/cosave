use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    models::{LoginRequest, RegisterRequest, Role, User, UserDto},
    security::{generate_token, hash_password, verify_password, SESSION_DURATION_SECS},
};
use crate::core::{db::DbPool, error::AppError};

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Creates auth domain tables and indices.
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), sqlx::Error> {
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

/// Looks up an active user by session token.
pub(crate) async fn find_user_by_session_token(
    pool: &DbPool,
    token: &str,
    now: i64,
) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
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
    .await?;

    Ok(user)
}

/// Atomically registers a new user, hashes password, assigns role (first user Admin), and creates session.
pub(crate) async fn register_user(
    pool: &DbPool,
    payload: RegisterRequest,
) -> Result<(UserDto, String), AppError> {
    let name = payload.name.trim().to_string();
    if name.len() < 2 {
        return Err(AppError::BadRequest(
            "Username must be at least 2 characters".to_string(),
        ));
    }

    if payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    let existing: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE name = ? COLLATE NOCASE")
            .bind(&name)
            .fetch_optional(pool)
            .await?;

    if existing.is_some() {
        return Err(AppError::UserExists);
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|e| AppError::Internal(format!("Argon2 hashing error: {e}")))?;

    let mut tx = pool.begin().await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;
    let role = if count.0 == 0 {
        Role::Admin
    } else {
        Role::Member
    };

    let now = now_epoch_secs();
    let user_id = generate_token();

    sqlx::query(
        r#"
        INSERT INTO users (id, name, password_hash, role, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&name)
    .bind(&password_hash)
    .bind(role.as_str())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    let session_token = generate_token();
    let expires_at = now + SESSION_DURATION_SECS;

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&session_token)
    .bind(&user_id)
    .bind(expires_at)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let user_dto = UserDto {
        id: user_id,
        name,
        role,
        created_at: now,
    };

    Ok((user_dto, session_token))
}

/// Verifies credentials and creates a new authenticated session.
pub(crate) async fn authenticate_user(
    pool: &DbPool,
    payload: LoginRequest,
) -> Result<(UserDto, String), AppError> {
    let name = payload.name.trim();

    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE name = ? COLLATE NOCASE")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::InvalidCredentials),
    };

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::InvalidCredentials);
    }

    let now = now_epoch_secs();
    let session_token = generate_token();
    let expires_at = now + SESSION_DURATION_SECS;

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&session_token)
    .bind(&user.id)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;

    Ok((user.to_dto(), session_token))
}

/// Deletes an active session on logout.
pub(crate) async fn logout(pool: &DbPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}
