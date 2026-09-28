# Backend Ingress, Security & State Standards

Authoritative standards for HTTP request ingestion, input hygiene, three-tier model separation, actor-scoped database queries, authorization guards, and immutable runtime application configuration.

---

## 1. Core Architecture

The backend establishes rigid security, ingress, and runtime boundaries to protect against payload tampering, horizontal privilege escalation, data leakage, and configuration drift:

1. **Request DTO Strictness & Input Hygiene**: All deserialized request structs must enforce `#[serde(deny_unknown_fields)]`. Extraneous fields, typos, or stale client properties are rejected with 400 Bad Request. Incoming strings must be trimmed of surrounding whitespace before parsing into domain Value Objects. JSON deserialization rejections map cleanly to standard `ApiResponse` error envelopes with actionable messages, never leaking raw parser internals.
2. **Three-Tier Model Separation**: Data structures are strictly separated into three distinct layers:
   - **Wire DTOs** (`models.rs`): HTTP wire representations (`*Request`, `*Response`) with serialization annotations.
   - **Domain Entities & Value Objects** (`models.rs` or domain modules): Business objects holding validated Value Objects. Domain entities never derive `sqlx::FromRow`.
   - **Database Projections / Rows** (`db/*.rs`): Physical SQLite schema representations deriving `sqlx::FromRow`.
   - Raw database rows are never returned directly to API responses.
3. **Resource Authorization & Mandatory Query Scoping**: In tenant-, family-, or user-scoped domains, all SELECT, UPDATE, and DELETE queries must explicitly scope the resource owner in the `WHERE` clause (e.g. `WHERE id = ? AND family_id = ?`). Unscoped queries are strictly forbidden. If a resource exists under another tenant, queries return 0 rows, triggering a standard 404 Not Found error without leaking whether the resource exists.
4. **Role & Capability Guards**: Handlers requiring elevated privileges must verify the authenticated actor's permissions (e.g. `user.require_admin(ACTION)?`) before executing business or database logic.
5. **Startup Configuration SSOT & Immutable AppState**: Environment variables are validated and loaded once into a strongly-typed `AppConfig` struct during server startup. Calling `std::env::var` or reading environment variables at runtime inside route handlers, domain logic, or database operations is strictly forbidden. `AppState` wraps read-only references to `Arc<AppConfig>` and `DbPool`, ensuring state immutability.

---

## 2. Invariants

1. **Deny Unknown Fields**: Every struct annotated with `#[derive(Deserialize)]` representing an HTTP request body must include `#[serde(deny_unknown_fields)]`.
2. **Whitespace Trimming**: Handlers must trim user input strings before passing them to Value Object constructors (`try_new(raw.trim(), ACTION)`).
3. **No Database Row Leakage**: Structs deriving `sqlx::FromRow` are internal to `db/` or private feature helpers. They must never derive `Serialize` for direct return in `ApiResponse<T>`. All response payloads must use dedicated Response DTOs.
4. **Mandatory Query Owner Scoping**: All mutation queries modifying user- or family-owned resources must include the owner's identifier in the `WHERE` clause:
   - `UPDATE table SET ... WHERE id = ? AND family_id = ?;`
   - `DELETE FROM table WHERE id = ? AND family_id = ?;`
   - Bypassing the owner constraint is strictly forbidden.
5. **Zero Resource Existence Leakage**: When a query targeting an owned resource matches 0 rows, handlers must return `NotFound` (HTTP 404). Handlers must never return `Forbidden` (HTTP 403) for an item ID lookup, as this reveals that the resource ID exists under another tenant.
6. **Zero Runtime Environment Access**: `std::env::var` calls are prohibited outside the startup boot sequence in `main.rs` / `core::config`. All configuration parameters must be accessed via `state.config()`.

---

## 3. Canonical Patterns

### Canonical Request DTO with Deny Unknown Fields

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateCategoryRequest {
    name: String,
    type_id: String,
}

impl CreateCategoryRequest {
    pub(crate) fn into_parts(self) -> (String, String) {
        (self.name, self.type_id)
    }
}
```

### Canonical Three-Tier Model Transformation Flow

```rust
// 1. Ingress: Handler receives Wire Request DTO
async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), AppError> {
    let (raw_name, type_id) = payload.into_parts();

    // 2. Domain Validation: Parse into Value Object
    let category_name = CategoryName::try_new(
        raw_name,
        "CONFIG.CATEGORIES.ROUTE.CREATE_CATEGORY.PARSE_NAME"
    )?;

    // 3. Database Execution: DB helper returns internal DB projection/entity
    let created_entity = db::insert_category(
        state.db(),
        user.family_id(),
        &type_id,
        category_name,
    ).await?;

    // 4. Egress: Transform Domain Entity into Wire Response DTO
    let response_dto = CategoryResponse::from_entity(created_entity);

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), response_dto)),
    ))
}
```

### Canonical Actor-Scoped Mutation Query

```rust
pub(crate) async fn delete_category(
    pool: &DbPool,
    family_id: &str,
    category_id: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM categories WHERE id = ? AND family_id = ?"
    )
    .bind(category_id)
    .bind(family_id)
    .execute(pool)
    .await
    .db_context("CONFIG.CATEGORIES.DELETE_CATEGORY.EXECUTE")?;

    if result.rows_affected() == 0 {
        return Err(CategoryError::CategoryNotFound {
            action: "CONFIG.CATEGORIES.DELETE_CATEGORY.NOT_FOUND",
            id: category_id.to_string(),
        }.into());
    }

    Ok(())
}
```

### Canonical Role Verification Guard

```rust
impl AuthUser {
    pub(crate) fn require_role(
        &self,
        required: Role,
        action: &'static str,
    ) -> Result<(), AuthError> {
        if self.role() != required {
            return Err(AuthError::ForbiddenRole {
                action,
                required: required.as_str(),
                actual: self.role().as_str(),
            });
        }
        Ok(())
    }

    pub(crate) fn require_admin(&self, action: &'static str) -> Result<(), AuthError> {
        self.require_role(Role::Admin, action)
    }
}
```

### Canonical Immutable Application Configuration

```rust
// In core/config.rs: Loaded once during startup boot
#[derive(Debug, Clone)]
pub struct AppConfig {
    database_url: String,
    port: u16,
    environment: Environment,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/cosave.db".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5171);
        let environment = match std::env::var("ENV").as_deref() {
            Ok("PROD") => Environment::Production,
            _ => Environment::Development,
        };

        Ok(Self {
            database_url,
            port,
            environment,
        })
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
```

---

## 4. Anti-Patterns

### Anti-Pattern 1: Permissive Request Deserialization

```rust
// ❌ Anti-Pattern: Allows unknown or misspelled fields, ignoring client errors
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateCategoryPayload {
    name: Option<String>,
}

// ✅ Canonical: Strictly rejects any unexpected fields
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateCategoryPayload {
    name: Option<String>,
}
```

### Anti-Pattern 2: Unscoped Database Mutation

```rust
// ❌ Anti-Pattern: Anyone with a valid category_id can delete another family's category
DELETE FROM categories WHERE id = ?;

// ✅ Canonical: Scoped strictly to the authenticated family/tenant
DELETE FROM categories WHERE id = ? AND family_id = ?;
```

### Anti-Pattern 3: Direct Row Serialization to Response

```rust
// ❌ Anti-Pattern: Exposing physical SQLite row directly as external API JSON
#[derive(Debug, Serialize, sqlx::FromRow)]
pub(crate) struct CategoryDbRow {
    id: String,
    type_id: String,
    password_hash: Option<String>, // Accidental data leak!
}

// ✅ Canonical: Decoupled Response DTO constructed from domain model
#[derive(Debug, Serialize)]
pub(crate) struct CategoryResponse {
    id: String,
    name: String,
}
```

### Anti-Pattern 4: Runtime Environment Lookups

```rust
// ❌ Anti-Pattern: Handler queries OS environment on every request
async fn export_data() -> Result<..., AppError> {
    let secret = std::env::var("ENCRYPTION_KEY").unwrap(); // Can panic or fail dynamically!
}

// ✅ Canonical: Configuration loaded once at boot and accessed via AppState
async fn export_data(State(state): State<AppState>) -> Result<..., AppError> {
    let key = state.config().encryption_key();
}
```

---

## 5. Checklist for Agents

Before completing any ingress, authorization, or configuration task:

- [ ] All request body structs derive `#[derive(Deserialize)]` with `#[serde(deny_unknown_fields)]`.
- [ ] User-supplied strings are trimmed before passing into Value Object constructors.
- [ ] Three-tier model separation is maintained: Request/Response DTOs $\ne$ Domain Entities $\ne$ DB Rows.
- [ ] Structs deriving `sqlx::FromRow` are never serialized directly as API responses.
- [ ] All queries for user/family-owned data include the owner identifier (`family_id` / `user_id`) in `WHERE`.
- [ ] Cross-tenant access attempts return 404 Not Found (zero existence leakage).
- [ ] Privileged endpoints invoke `user.require_admin(...)` or equivalent capability checks before database logic.
- [ ] Zero calls to `std::env::var` exist outside startup configuration loading in `main.rs` / `core::config`.
