use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{create_db_object, AppError, DbPool, DbResultExt};

use super::error::AuthError;
use super::models::{LoginRequest, RegisterRequest, Role, User, UserDto};
use super::security::{generate_token, hash_password, verify_password, SESSION_DURATION_SECS};

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

/// Checks if a user already exists with the given username (case-insensitive).
pub(crate) async fn find_user_by_name(pool: &DbPool, name: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, password_hash, role, created_at, updated_at FROM users WHERE LOWER(name) = LOWER(?) LIMIT 1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_NAME.QUERY")?;

    Ok(user)
}

/// Finds an active user session by token, ensuring the session has not expired.
pub(crate) async fn find_user_by_session_token(
    pool: &DbPool,
    token: &str,
    now: i64,
) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.name, u.password_hash, u.role, u.created_at, u.updated_at
        FROM users u
        JOIN sessions s ON s.user_id = u.id
        WHERE s.id = ? AND s.expires_at > ?
        LIMIT 1
        "#,
    )
    .bind(token)
    .bind(now)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_SESSION.QUERY")?;

    Ok(user)
}

/// Registers a new user. The very first user to register receives the `admin` role.
pub(crate) async fn register_user(
    pool: &DbPool,
    payload: RegisterRequest,
) -> Result<(UserDto, String), AppError> {
    let username = payload.name().trim().to_string();
    if username.is_empty() || username.len() < 3 {
        return Err(AuthError::InvalidUsername {
            action: "AUTH.REGISTER.USERNAME_LEN",
            username,
            min_len: 3,
        }
        .into());
    }

    if payload.password().len() < 6 {
        return Err(AuthError::InvalidPassword {
            action: "AUTH.REGISTER.PASSWORD_LEN",
            min_len: 6,
        }
        .into());
    }

    let existing = find_user_by_name(pool, &username).await?;
    if existing.is_some() {
        return Err(AuthError::UserExists {
            action: "AUTH.REGISTER.CHECK_EXISTING",
            username,
        }
        .into());
    }

    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .db_context("AUTH.REGISTER.COUNT_USERS")?;

    let role = if user_count.0 == 0 {
        Role::Admin
    } else {
        Role::Member
    };

    let user_id = format!("usr-{}", &generate_token()[..10]);
    let password_hash = hash_password(payload.password())?;
    let now = now_epoch_secs();

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
    .execute(pool)
    .await
    .db_context("AUTH.REGISTER.INSERT_USER")?;

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
    .execute(pool)
    .await
    .map_err(|e| AuthError::InsertNewSessionError {
        action: "AUTH.REGISTER.INSERT_SESSION",
        user_id: user_id.clone(),
        source: e,
    })?;

    let user_dto = UserDto::new(user_id, username, role, now);
    Ok((user_dto, session_token))
}

/// Authenticates credentials and returns the user DTO along with a session token.
pub(crate) async fn authenticate_user(
    pool: &DbPool,
    payload: LoginRequest,
) -> Result<(UserDto, String), AppError> {
    let username = payload.name().trim();
    if username.is_empty() {
        return Err(AuthError::InvalidCredentials {
            action: "AUTH.LOGIN.USERNAME_EMPTY",
        }
        .into());
    }

    let user = find_user_by_name(pool, username).await?;
    let Some(user) = user else {
        return Err(AuthError::InvalidCredentials {
            action: "AUTH.LOGIN.FIND_USER",
        }
        .into());
    };

    if !verify_password(payload.password(), user.password_hash()) {
        return Err(AuthError::InvalidCredentials {
            action: "AUTH.LOGIN.VERIFY_PASSWORD",
        }
        .into());
    }

    let session_token = generate_token();
    let now = now_epoch_secs();
    let expires_at = now + SESSION_DURATION_SECS;

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&session_token)
    .bind(user.id())
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AuthError::InsertNewSessionError {
        action: "AUTH.LOGIN.INSERT_SESSION",
        user_id: user.id().to_string(),
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
