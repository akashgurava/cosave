use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    models::{LoginRequest, RegisterRequest, Role, User, UserDto},
    security::{generate_token, hash_password, verify_password, SESSION_DURATION_SECS},
};
use crate::{
    core::{create_db_object, db::DbPool, error::AppError, DbResultExt},
    features::auth::AuthError,
};

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Creates auth domain tables and indices.
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), AppError> {
    create_db_object(
        "AUTH.INIT_SCHEMA.USERS_TABLE",
        "users",
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "AUTH.INIT_SCHEMA.SESSIONS_TABLE",
        "sessions",
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "AUTH.INIT_SCHEMA.SESSIONS_INDEX_USER_ID",
        "sessions",
        pool,
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);",
    )
    .await?;

    create_db_object(
        "AUTH.INIT_SCHEMA.SESSIONS_INDEX_EXPIRES_AT",
        "sessions",
        pool,
        "CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);",
    )
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
    .await
    .db_context("AUTH.FIND_USER_BY_SESSION.QUERY")?;

    Ok(user)
}

/// Atomically registers a new user, hashes password, assigns role (first user Admin), and creates session.
pub(crate) async fn register_user(
    pool: &DbPool,
    payload: RegisterRequest,
) -> Result<(UserDto, String), AppError> {
    const ACTION: &str = "AUTH.REGISTER_USER";
    let username = payload.name.trim().to_string();
    if username.len() < 2 {
        return Err(AuthError::InvalidUsername {
            action: ACTION,
            username,
            min_len: 2,
        }
        .into());
    }

    if payload.password.len() < 6 {
        return Err(AuthError::InvalidPassword {
            action: ACTION,
            min_len: 6,
        }
        .into());
    }

    let existing: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE name = ? COLLATE NOCASE")
            .bind(&username)
            .fetch_optional(pool)
            .await
            .db_context("AUTH.REGISTER_USER.FIND_EXISTING_USER")?;

    if existing.is_some() {
        return Err(AuthError::UserExists {
            action: ACTION,
            username,
        }
        .into());
    }

    let password_hash = hash_password(&payload.password)?;

    let mut tx = pool
        .begin()
        .await
        .db_context("AUTH.REGISTER_USER.BEGIN_TRANSACTION")?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .db_context("AUTH.REGISTER_USER.COUNT_USERS")?;

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
    .bind(&username)
    .bind(&password_hash)
    .bind(role.as_str())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| AuthError::InsertNewUserError {
        action: ACTION,
        username: username.clone(),
        source: e,
    })?;

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
    .await
    .map_err(|e| AuthError::InsertNewSessionError {
        action: ACTION,
        user_id: user_id.clone(),
        source: e,
    })?;

    tx.commit()
        .await
        .db_context("AUTH.REGISTER_USER.COMMIT_TRANSACTION")?;

    let user_dto = UserDto {
        id: user_id,
        name: username,
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
    const ACTION: &str = "AUTH.AUTHENTICATE_USER";
    let username = payload.name.trim();

    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE name = ? COLLATE NOCASE")
        .bind(username)
        .fetch_optional(pool)
        .await
        .db_context("AUTH.AUTHENTICATE_USER.FIND_USER")?;

    let user = match user {
        Some(u) => u,
        None => return Err(AuthError::InvalidCredentials { action: ACTION }.into()),
    };

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials { action: ACTION }.into());
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
    .await
    .map_err(|e| AuthError::InsertNewSessionError {
        action: ACTION,
        user_id: user.id.clone(),
        source: e,
    })?;

    Ok((user.to_dto(), session_token))
}

/// Deletes an active session on logout.
pub(crate) async fn logout(pool: &DbPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(token)
        .execute(pool)
        .await
        .db_context("AUTH.LOGOUT.DELETE_SESSION")?;
    Ok(())
}
