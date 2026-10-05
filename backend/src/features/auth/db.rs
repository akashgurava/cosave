//! Authentication persistence layer for user accounts and active sessions.
//!
//! Handles SQLite table initialization for user identities and session tokens,
//! enforcing cascade deletion so removing a user automatically purges their sessions.
//! Workflows manage atomic first-user registration with initial administrator promotion,
//! credential lookup for login verification, and token revocation on logout.
//! Queries are scoped to guarantee reliable identity resolution and session lifecycle management.

use sqlx::{Executor, Sqlite, Transaction};

use crate::core::{create_db_object, now_epoch_secs, AppError, DbPool, DbResultExt};

use super::error::AuthError;
use super::models::{LoginRequest, RawPassword, RegisterRequest, Role, User, UserDto, Username};
use super::security::{generate_token, hash_password, verify_password, SESSION_DURATION_SECS};

/// Initializes the authentication schema for user credentials and active sessions.
///
/// Provisions the security tables supporting user identity management, Argon2id credential
/// verification, and time-bounded session token lifecycles.
///
/// # Domain Rules & Referential Integrity
/// - **Session Cascade**: Deleting a user cascades to all their active session tokens (`ON DELETE CASCADE`),
///   preventing orphaned authentication contexts.
/// - **Credential Uniqueness**: Usernames are strictly unique (`UNIQUE(username)`).
/// - **Session Lookup Indexes**: Dedicated indexes on `sessions(user_id)` and `sessions(expires_at)`
///   enable fast session invalidation upon logout and rapid expiry pruning.
///
/// # Execution & Idempotency
/// - Executes atomically within the caller-provided [`Transaction`].
/// - Idempotent across restarts using `CREATE ... IF NOT EXISTS` DDL.
/// - Executed via [`create_db_object`] with dedicated action tokens (`AUTH.INIT_SCHEMA.*`).
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL statement fails.
pub(crate) async fn init_auth_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), AppError> {
    create_db_object(
        "AUTH.INIT_SCHEMA.USERS_TABLE",
        "users",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
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
        tx,
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
        tx,
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);",
    )
    .await?;

    create_db_object(
        "AUTH.INIT_SCHEMA.SESSIONS_INDEX_EXPIRES_AT",
        "sessions",
        tx,
        "CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);",
    )
    .await?;

    Ok(())
}

/// Checks if a user already exists with the given username (case-insensitive).
async fn find_user_by_username(pool: &DbPool, username: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE LOWER(username) = LOWER(?) LIMIT 1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_USERNAME.QUERY")?;

    Ok(user)
}

/// Finds an active user session by token, ensuring the session has not expired.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `token`: Inbound session token from cookie or Bearer header.
/// - `now`: Current UTC epoch seconds.
///
/// # Returns
/// - `Ok(Some(User))` if a matching session exists and `expires_at > now`.
/// - `Ok(None)` if no session matches or the session has expired.
pub(super) async fn find_user_by_session_token(
    pool: &DbPool,
    token: &str,
    now: i64,
) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.username, u.password_hash, u.role, u.created_at, u.updated_at
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

/// Registers a new user within an atomic database transaction.
///
/// # Behavior
/// 1. Validates username and password Value Objects.
/// 2. Verifies that the username is not already taken (case-insensitive check).
/// 3. Computes the Argon2id password hash before acquiring the database write lock.
/// 4. Opens an explicit database transaction ([`DbPool::begin`]).
/// 5. Counts existing users: if count is 0, the first user receives the [`Role::Admin`] role;
///    otherwise, the user receives the [`Role::Member`] role.
/// 6. Inserts the new user into `users`.
/// 7. Generates a session token and inserts an active session into `sessions`.
/// 8. Commits the transaction atomically.
///
/// # Errors
/// Returns [`AuthError::UserAlreadyExists`] if the username is taken,
/// or [`AppError`] on validation or database failure.
pub(super) async fn register_user(
    pool: &DbPool,
    payload: RegisterRequest,
) -> Result<(UserDto, String), AppError> {
    let username = Username::try_new(payload.username(), "AUTH.REGISTER.USERNAME_LEN")?;
    let password = RawPassword::try_new(payload.password(), "AUTH.REGISTER.PASSWORD_LEN")?;

    let existing = find_user_by_username(pool, username.as_str()).await?;
    if existing.is_some() {
        return Err(AuthError::UserAlreadyExists {
            action: "AUTH.REGISTER.CHECK_EXISTING",
            username: username.into_inner(),
        }
        .into());
    }

    let user_id = format!("usr-{}", &generate_token()[..10]);
    let password_hash = hash_password(password.as_str())?;
    let now = now_epoch_secs();
    let session_token = generate_token();
    let expires_at = now + SESSION_DURATION_SECS;

    let mut tx = pool.begin().await.db_context("AUTH.REGISTER.TX_BEGIN")?;

    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .db_context("AUTH.REGISTER.COUNT_USERS")?;

    let role = if user_count.0 == 0 {
        Role::Admin
    } else {
        Role::Member
    };

    tx.execute(
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, role, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user_id)
        .bind(username.as_str())
        .bind(&password_hash)
        .bind(role.as_str())
        .bind(now)
        .bind(now),
    )
    .await
    .db_context("AUTH.REGISTER.INSERT_USER")?;

    tx.execute(
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, expires_at, created_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&session_token)
        .bind(&user_id)
        .bind(expires_at)
        .bind(now),
    )
    .await
    .db_context("AUTH.REGISTER.INSERT_SESSION")?;

    tx.commit().await.db_context("AUTH.REGISTER.TX_COMMIT")?;

    let user_dto = UserDto::new(user_id, username.into_inner(), role, now);
    Ok((user_dto, session_token))
}

/// Authenticates credentials and returns the user DTO along with a session token.
///
/// # Errors
/// Returns [`AuthError::InvalidCredentials`] if the username does not exist or the password hash does not match.
pub(super) async fn authenticate_user(
    pool: &DbPool,
    payload: LoginRequest,
) -> Result<(UserDto, String), AppError> {
    let username = payload.username().trim();
    if username.is_empty() {
        return Err(AuthError::InvalidCredentials {
            action: "AUTH.LOGIN.USERNAME_EMPTY",
        }
        .into());
    }

    let user = find_user_by_username(pool, username).await?;
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
    .db_context("AUTH.LOGIN.INSERT_SESSION")?;

    Ok((user.to_dto(), session_token))
}

/// Deletes an active session on logout.
pub(super) async fn logout(pool: &DbPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(token)
        .execute(pool)
        .await
        .db_context("AUTH.LOGOUT.DELETE_SESSION")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{init_db, AppConfig};

    async fn setup_auth_test_db() -> DbPool {
        let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
            .await
            .expect("init test sqlite in-memory db");
        let mut tx = pool.begin().await.expect("begin tx");
        init_auth_schema(&mut tx).await.expect("init auth schema");
        tx.commit().await.expect("commit tx");
        pool
    }

    #[tokio::test]
    async fn test_foreign_key_cascade_deleting_user_removes_sessions() {
        let pool = setup_auth_test_db().await;

        let (user_dto, token) =
            register_user(&pool, RegisterRequest::new("cascade_user", "password123"))
                .await
                .expect("register user");

        let now = now_epoch_secs();
        let found_user = find_user_by_session_token(&pool, &token, now)
            .await
            .expect("find session");
        assert!(found_user.is_some());

        // Delete parent user from users table
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_dto.id())
            .execute(&pool)
            .await
            .expect("delete user");

        // Verify SQLite foreign key ON DELETE CASCADE removed the session
        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE id = ?")
            .bind(&token)
            .fetch_one(&pool)
            .await
            .expect("count sessions");
        assert_eq!(
            session_count.0, 0,
            "session must be cascaded on user deletion"
        );
    }

    #[tokio::test]
    async fn test_find_user_by_session_token_rejects_expired_session() {
        let pool = setup_auth_test_db().await;

        let now = now_epoch_secs();
        let user_id = "usr-test-expired";
        let session_token = "expired_token_12345";

        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind("expired_user")
        .bind("hash")
        .bind("member")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert user");

        // Insert session that expired 10 seconds ago
        let expired_at = now - 10;
        sqlx::query(
            "INSERT INTO sessions (id, user_id, expires_at, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(session_token)
        .bind(user_id)
        .bind(expired_at)
        .bind(now - 100)
        .execute(&pool)
        .await
        .expect("insert expired session");

        let found = find_user_by_session_token(&pool, session_token, now)
            .await
            .expect("query session");
        assert!(
            found.is_none(),
            "expired session must not resolve to a user"
        );
    }
}
