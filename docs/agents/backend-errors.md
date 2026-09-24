# Backend Error Standards & Conventions

Standards for error creation, propagation, database failure mapping, and API response formatting across the Rust Axum backend.

---

## 1. Core Architecture

Backend errors operate on a two-tier hierarchy:
1. **Feature Errors (`features/<feature>/error.rs`)**: Domain failures specific to a feature (e.g., `CategoryError::TypeAlreadyExists`, `AuthError::InvalidCredentials`).
2. **Application Error (`core/error.rs`)**: The root `AppError` enum that wraps all feature errors via `#[from]` and owns infrastructure failures:
   - `InitSchema { action, table, source }`: Startup DDL migrations.
   - `ShouldNotBeHappening { action, reason }`: Runtime invariant breaks and unexpected database query/pool failures.

Route handlers and database functions return `Result<T, AppError>`. Error propagation relies exclusively on `?`.

---

## 2. Invariants

1. **Single-Identifier Rigidity**: Error types are rigidly typed, feature-scoped, and identifiable by a single unique `SCREAMING_SNAKE_CASE` token returned via `self.code() -> &'static str`. The token strictly mirrors the variant name in PascalCase. Searching this single keyword with `rg` must pinpoint the enum definition, the call site, and the log line with zero ambiguity.
2. **Action Hierarchy (`FEATURE.WORKFLOW[.STEP]`)**: Every error variant carries a compile-time `action: &'static str` field. The taxonomy is 3-tiered: `FEATURE` (e.g. `AUTH`, `CONFIG.CATEGORIES`), `WORKFLOW` (e.g. `REGISTER_USER`, `CREATE_TYPE`), and granular `STEP` (e.g. `FIND_EXISTING_USER`, `QUERY_MAX_SORT`, `COMMIT_TRANSACTION`).
3. **Granular Action Isolation (No Query Bundling)**: Never execute multiple queries, statements, or migrations under a single action token. Every distinct SQL execution, table creation, index creation, transaction boundary, and sort calculation must have its own dedicated call with its own unique action string.
4. **Structured API Envelope**: Failure responses return `ApiResponse<ErrorPayload>` where `data` is `{ "action": "...", "message": "..." }`, `status` is `self.code()`, and `code` is the HTTP status number.
5. **No Leaked SQL or Credentials**: Client error payloads must never leak raw SQL queries, engine internals, or sensitive credentials. Raw SQL and database errors are captured in `%self` server tracing logs; client envelopes receive clean, actionable `ErrorPayload` messages.
6. **Strict Credential Naming (`username`)**: Error variant fields and authentication logic must strictly name credential identifiers `username`. Never use `user_name`, `name`, or `user` for credentials (`name` is reserved strictly for a `Member`'s display name).
7. **Unified Logging**: `into_response()` emits `tracing::error!(action, code, %self)` for 5xx errors and `tracing::warn!(action, code, %self)` for 4xx errors. Do not duplicate log calls inside individual match arms.

---

## 3. Canonical Patterns

### Core App Error: `core/error.rs`

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;
use crate::core::response::{ApiResponse, Code, ErrorPayload, Status};
use crate::features::{auth::AuthError, categories::CategoryError};

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Category(#[from] CategoryError),

    #[error("INIT_SCHEMA_ERROR. ACTION: {action}. TABLE: {table}. ERROR: {source}")]
    InitSchema {
        action: &'static str,
        table: &'static str,
        #[source]
        source: sqlx::Error,
    },

    #[error("SHOULD_NOT_BE_HAPPENING. ACTION: {action}. REASON: {reason}")]
    ShouldNotBeHappening {
        action: &'static str,
        reason: String,
    },
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

/// Helper function to convert a `sqlx::Error` into `AppError::ShouldNotBeHappening`.
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
            Err(CategoryError::TypeAlreadyExists { action: ACTION, name }.into())
        } else {
            Err(db_err("CONFIG.CATEGORIES.CREATE_TYPE.INSERT", err))
        }
    }
}
```

---

## 4. Rigidity Contrast

### Anti-Pattern ❌ (Vague Strings or Bundled Queries)

```rust
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

### Canonical Pattern ✅ (Strict Rigidity & Granular Actions)

```rust
// PREFER: Single screaming identifier, compile-time action path
return Err(CategoryError::TypeAlreadyExists {
    action: ACTION,
    name,
}.into());

// PREFER: Dedicated db_context for each query execution
let user = sqlx::query_as::<_, User>(...)
    .fetch_optional(pool)
    .await
    .db_context("AUTH.FIND_USER_BY_SESSION.QUERY")?;

// PREFER: Separate create_db_object call for each table and index
create_db_object("CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE", "categories", pool, "...").await?;
create_db_object("CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX", "categories", pool, "...").await?;
```
