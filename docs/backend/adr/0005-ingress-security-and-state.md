# Ingress DTO Strictness, Three-Tier Model Separation, Actor-Scoped Queries, and Immutable Runtime Configuration

Incoming HTTP payloads enforce strict schema conformance with `deny_unknown_fields` and sanitization, domain models enforce a strict three-tier separation from wire DTOs and database rows, database queries mandate actor/tenant scoping to prevent horizontal privilege escalation, and runtime application configuration is validated once at startup into an immutable state container.

## Context & Decision

Allowing loose JSON request parsing, mixing database row representations with HTTP response payloads, running unscoped database mutations, and calling environment variables dynamically at runtime introduce severe security, integrity, and operational liabilities.

We established four architectural standards for ingress, authorization, and runtime configuration:

1. **Request DTO Strictness & Payload Sanitization**: All deserialized request payloads must annotate `#[serde(deny_unknown_fields)]` to reject unexpected parameters, outdated client schemas, and silent parameter pollution. Incoming strings must be trimmed of surrounding whitespace before parsing into domain Value Objects. Axum JSON deserialization rejections must map to standard `ApiResponse` error envelopes with HTTP status 400 Bad Request and actionable user error messages, never leaking raw Rust compiler types or internal stack traces.
2. **Three-Tier Model Separation**: Data structures are strictly separated into three distinct tiers:
   - **Wire DTOs** (`models.rs`): Serialized HTTP contracts (`*Request`, `*Response`) matching client wire formats.
   - **Domain Entities & Value Objects** (`models.rs` or domain submodules): Encapsulated business entities composed of validated Value Objects. Domain entities never derive `sqlx::FromRow`.
   - **Database Projections / Rows** (`db/*.rs`): Physical SQLite schema representations deriving `sqlx::FromRow`. Direct return of raw database rows to HTTP responses is strictly forbidden.
3. **Mandatory Query Scoping & Zero Existence Leakage**: In tenant-, family-, or user-scoped domains, all SELECT, UPDATE, and DELETE queries must explicitly include the actor's scope (`family_id` or `user_id`) in their `WHERE` clause. Unscoped queries (e.g. `DELETE FROM categories WHERE id = ?`) are strictly prohibited. When an item exists under another tenant, queries must return 0 rows and trigger standard 404 Not Found errors without leaking whether the resource exists under a different tenant. Handlers requiring elevated privileges must execute explicit role checks before database operations.
4. **Startup Configuration SSOT & Immutable AppState**: All environment variables and configuration settings (`COSAVE_DATABASE_URL`, `COSAVE_PORT`, `COSAVE_ENV`) are validated and parsed into an immutable `AppConfig` struct strictly once at server startup. Calling `std::env::var` or inspecting the environment at runtime inside route handlers, domain logic, or database helpers is strictly forbidden. `AppState` wraps read-only references to `Arc<AppConfig>` and `DbPool`, guaranteeing state immutability across request lifecycles.

## Consequences

- API request contracts are strictly enforced, eliminating subtle bugs caused by misspelled client parameters.
- Internal database schema changes cannot inadvertently alter external JSON API contracts, as DTOs and database rows are decoupled.
- Horizontal privilege escalation is structurally prevented by query-level actor scoping.
- Unauthorized callers cannot probe for the existence of private resources belonging to other users or families.
- Environment misconfigurations fail fast at startup rather than triggering unexpected failures during runtime request handling.
