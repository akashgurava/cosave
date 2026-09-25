# Backend Error Standards & Conventions

Standards for error creation, propagation, database failure mapping, and API response formatting across the Rust Axum backend.

---

## 1. Core Architecture

Backend errors operate on a two-tier hierarchy:

1. **Feature Errors (`features/<feature>/error.rs`)**: Domain failures specific to a feature (e.g., `CategoryError::TypeAlreadyExists`, `AuthError::InvalidCredentials`).
2. **Application Error (`core/error.rs`)**: The root `AppError` enum that wraps all feature errors via manual `From` implementations (`From<AuthError>`, `From<CategoryError>`) and owns infrastructure failures:
   - `InitSchema { action, table, source }`: Startup DDL migrations.
   - `ShouldNotBeHappening { action, reason }`: Runtime invariant breaks and unexpected database query/pool failures.

Route handlers and database functions return `Result<T, AppError>`. Error propagation relies exclusively on `?`.

---

## 2. Invariants

1. **Error Token Single Source of Truth (SSOT)**: The screaming snake case error token (e.g. `"INIT_SCHEMA_ERROR"`, `"INVALID_USERNAME"`, `"TYPE_ALREADY_EXISTS"`) is defined in exactly one place: `self.code() -> &'static str`. Deriving `thiserror::Error` with string format attributes (`#[error("...")]`) is strictly forbidden. Searching the screaming code with `rg` resolves to its single definition in `self.code()`.
2. **Action Uniqueness & Taxonomy (`FEATURE.WORKFLOW.STEP[.BRANCH]`)**: Every error instantiation, validation failure, and query execution must carry a **globally unique compile-time action string**. Reusing an umbrella action string (e.g. `CONFIG.CATEGORIES.RESOLVE_COLOR` across multiple validation checks or `AUTH.REGISTER` across username, password, and session failures) is strictly prohibited. Searching any action string with `rg` must pinpoint the exact source code line and failure point with zero ambiguity.
3. **Manual `Display` & `Error` Trait Implementation**:
   - `std::fmt::Display` is implemented manually. Every variant message begins with `{code}. ACTION: {action}` where `let code = self.code();`.
   - `std::error::Error` is implemented manually, returning `source` references (`Some(source)`) for wrapped errors and `None` otherwise.
4. **Feature Error Structural Symmetry**: Every feature error enum follows the exact same 6-part anatomy:
   - Rigid enum definition with `action: &'static str` on every variant.
   - `self.action() -> &'static str` accessor.
   - `self.code() -> &'static str` SSOT definition of the token.
   - `impl fmt::Display` prefixing `{code}. ACTION: {action}`.
   - `impl std::error::Error` for standard error propagation.
   - `impl IntoResponse` mapping to status code, `ApiResponse<ErrorPayload>`, and unified tracing log.
5. **Granular Action Isolation & Unique Tokens**: Never execute multiple queries, statements, migrations, or error instantiations under a single action token. Every distinct SQL execution, table creation, index creation, transaction boundary, and validation failure must have its own dedicated call with its own unique action string.
6. **Structured API Envelope**: Failure responses return `ApiResponse<ErrorPayload>` where `data` is `{ "action": "...", "message": "..." }`, `status` is `self.code()`, and `code` is the HTTP status number.
7. **No Leaked SQL or Credentials**: Client error payloads must never leak raw SQL queries, engine internals, or sensitive credentials. Raw SQL and database errors are captured in `%self` server tracing logs; client envelopes receive clean, actionable `ErrorPayload` messages.
8. **Strict Credential Naming (`username`)**: Error variant fields and authentication logic must strictly name credential identifiers `username`. Never use `user_name`, `name`, or `user` for credentials (`name` is reserved strictly for a `Member`'s display name).
9. **Unified Logging**: `into_response()` emits `tracing::error!(action, code, %self)` for 5xx errors and `tracing::warn!(action, code, %self)` for 4xx errors. Do not duplicate log calls inside individual match arms.

---

## 3. Canonical Patterns

### Core App Error: `core/error.rs`

```rust
use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::features::{auth::AuthError, categories::CategoryError};

use super::response::{ApiResponse, Code, ErrorPayload, Status};

/// Central application error type.
#[derive(Debug)]
pub enum AppError {
    Auth(AuthError),
    Category(CategoryError),
    InitSchema {
        action: &'static str,
        table: &'static str,
        source: sqlx::Error,
    },
    ShouldNotBeHappening {
        action: &'static str,
        reason: String,
    },
}

impl AppError {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.action(),
            Self::Category(err) => err.action(),
            Self::InitSchema { action, .. } => action,
            Self::ShouldNotBeHappening { action, .. } => action,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Auth(err) => err.code(),
            Self::Category(err) => err.code(),
            Self::InitSchema { .. } => "INIT_SCHEMA_ERROR",
            Self::ShouldNotBeHappening { .. } => "SHOULD_NOT_BE_HAPPENING",
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Auth(err) => write!(f, "{err}"),
            Self::Category(err) => write!(f, "{err}"),
            Self::InitSchema {
                action,
                table,
                source,
            } => {
                write!(f, "{code}. ACTION: {action}. TABLE: {table}. ERROR: {source}")
            }
            Self::ShouldNotBeHappening { action, reason } => {
                write!(f, "{code}. ACTION: {action}. REASON: {reason}")
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Auth(err) => Some(err),
            Self::Category(err) => Some(err),
            Self::InitSchema { source, .. } => Some(source),
            Self::ShouldNotBeHappening { .. } => None,
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}

impl From<CategoryError> for AppError {
    fn from(err: CategoryError) -> Self {
        Self::Category(err)
    }
}
```

### Feature Error: `features/<feature>/error.rs`

```rust
use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

#[derive(Debug)]
pub enum FeatureError {
    EntityNotFound {
        action: &'static str,
        id: String,
    },
    EntityAlreadyExists {
        action: &'static str,
        name: String,
    },
}

impl FeatureError {
    pub fn action(&self) -> &'static str {
        match self {
            Self::EntityNotFound { action, .. } => action,
            Self::EntityAlreadyExists { action, .. } => action,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::EntityNotFound { .. } => "ENTITY_NOT_FOUND",
            Self::EntityAlreadyExists { .. } => "ENTITY_ALREADY_EXISTS",
        }
    }
}

impl fmt::Display for FeatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::EntityNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. ID: '{id}'")
            }
            Self::EntityAlreadyExists { action, name } => {
                write!(f, "{code}. ACTION: {action}. Name: '{name}'")
            }
        }
    }
}

impl Error for FeatureError {}

impl IntoResponse for FeatureError {
    fn into_response(self) -> Response {
        let action = self.action();
        let code_str = self.code();

        let (status_code, code, message) = match &self {
            Self::EntityNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Entity with id '{id}' was not found."),
            ),
            Self::EntityAlreadyExists { name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Entity '{name}' already exists."),
            ),
        };

        if status_code.is_server_error() {
            tracing::error!(action = action, code = code_str, error = %self, "request failed");
        } else {
            tracing::warn!(action = action, code = code_str, error = %self, "client error");
        }

        let body = Json(ApiResponse::err(
            code,
            Status::custom(code_str),
            ErrorPayload::new(action, message),
        ));
        (status_code, body).into_response()
    }
}
```

### Database Helpers: `core/db.rs`

```rust
/// Extension trait for mapping `sqlx::Error` into `AppError::ShouldNotBeHappening`.
pub(crate) trait DbResultExt<T> {
    fn db_context(self, action: &'static str) -> Result<T, AppError>;
}

impl<T> DbResultExt<T> for Result<T, sqlx::Error> {
    fn db_context(self, action: &'static str) -> Result<T, AppError> {
        self.map_err(|e| AppError::ShouldNotBeHappening {
            action,
            reason: format!("database error: {e}"),
        })
    }
}

/// Helper function to convert a `sqlx::Error` into an `AppError::ShouldNotBeHappening`.
pub(crate) fn db_err(action: &'static str, err: sqlx::Error) -> AppError {
    AppError::ShouldNotBeHappening {
        action,
        reason: format!("database error: {err}"),
    }
}

/// Isolated helper for executing individual schema DDL statements.
pub(crate) async fn create_db_object(
    action: &'static str,
    table: &'static str,
    pool: &DbPool,
    sql: &str,
) -> Result<(), AppError> {
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::InitSchema { action, table, source: e })?;
    Ok(())
}
```

### Schema Initialization (Separate Statement Calls)

```rust
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), AppError> {
    // 1. Table creation has its own action and table tag
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE",
        "categories",
        pool,
        r#"CREATE TABLE IF NOT EXISTS categories (...);"#,
    ).await?;

    // 2. Index creation is a separate call with its own action
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX_TYPE_ID",
        "categories",
        pool,
        "CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);",
    ).await?;

    Ok(())
}
```

### Runtime Queries & Transactions

```rust
// 1. Read query with granular action context
let user = sqlx::query_as::<_, User>("SELECT ...")
    .bind(token)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_SESSION.QUERY")?;

// 2. Transaction boundaries with distinct actions
let mut tx = pool.begin().await.db_context("CONFIG.CATEGORIES.CREATE_TYPE.BEGIN_TRANSACTION")?;

let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM ...")
    .fetch_one(&mut *tx)
    .await
    .db_context("CONFIG.CATEGORIES.CREATE_TYPE.QUERY_MAX_SORT")?;

let insert_res = sqlx::query("INSERT INTO ...").execute(&mut *tx).await;

match insert_res {
    Ok(_) => {
        tx.commit().await.db_context("CONFIG.CATEGORIES.CREATE_TYPE.COMMIT_TRANSACTION")?;
        Ok(...)
    }
    Err(err) => {
        if is_unique_violation(&err) {
            Err(CategoryError::TypeAlreadyExists {
                action: "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS",
                name,
            }.into())
        } else {
            Err(db_err("CONFIG.CATEGORIES.CREATE_TYPE.INSERT", err))
        }
    }
}
```

---

## 4. Rigidity Contrast

### Anti-Pattern ❌ (Duplicated Code Strings, thiserror Derives, Bundled Queries, Reused Actions)

```rust
// AVOID: Duplicating the code token in #[error] and in self.code()
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("INVALID_USERNAME. ACTION: {action}...")]
    InvalidUsername { action: &'static str, username: String },
}
impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidUsername { .. } => "INVALID_USERNAME", // DUPLICATION!
        }
    }
}

// AVOID: Reusing an umbrella action token across multiple failure sites
const ACTION: &str = "AUTH.REGISTER";
if username.len() < 3 {
    return Err(AuthError::InvalidUsername { action: ACTION, .. }.into()); // AMBIGUOUS!
}
if password.len() < 6 {
    return Err(AuthError::InvalidPassword { action: ACTION, .. }.into()); // AMBIGUOUS!
}

// AVOID: Generic string, no screaming code, no action taxonomy
return Err(AppError::Internal("failed to save user".into()));

// AVOID: Bundling multiple queries into a single ambiguous action
let user = sqlx::query(...).fetch_one(pool).await.map_err(|e| AppError::ShouldNotBeHappening {
    action: "DB_ERROR",
    reason: e.to_string(),
})?;

// AVOID: Combining table and index DDL into a single create_db_object call
create_db_object("INIT", "categories", pool, "CREATE TABLE ...; CREATE INDEX ...;").await?;
```

### Canonical Pattern ✅ (Strict Single Source of Truth & Globally Unique Actions)

```rust
// PREFER: Single source of truth in self.code() with manual Display prefixing
#[derive(Debug)]
pub enum AuthError {
    InvalidUsername { action: &'static str, username: String },
}
impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidUsername { .. } => "INVALID_USERNAME", // SINGLE DEFINITION
        }
    }
}
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::InvalidUsername { action, username } => {
                write!(f, "{code}. ACTION: {action}. Username: '{username}'")
            }
        }
    }
}

// PREFER: Globally unique action tokens pinpointing the exact failure site
if username.len() < 3 {
    return Err(AuthError::InvalidUsername {
        action: "AUTH.REGISTER.USERNAME_LEN",
        username,
        min_len: 3,
    }.into());
}
if password.len() < 6 {
    return Err(AuthError::InvalidPassword {
        action: "AUTH.REGISTER.PASSWORD_LEN",
        min_len: 6,
    }.into());
}

// PREFER: Dedicated db_context for each query execution
let user = sqlx::query_as::<_, User>(...)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_SESSION.QUERY")?;

// PREFER: Separate create_db_object call for each table and index
create_db_object("CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE", "categories", pool, "...").await?;
create_db_object("CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX", "categories", pool, "...").await?;
```
