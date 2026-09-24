# Unified Error Architecture and Propagation

Backend errors roll up transparently from feature-scoped enums into a central `AppError`, with compile-time action paths and screaming error identifiers surfacing structured `{ action, message }` payloads to API clients.

## Context & Decision

CoSave requires strict error searchability across server logs, developer tooling, and user interfaces, with specific, actionable failure diagnostics.

We established a strict, two-tier error model:
1. **Feature-Scoped Domain Errors**: Features define discrete error types (e.g. `AuthError`, `CategoryError`) in `features/<feature>/error.rs` using `thiserror`. All variants define `action: &'static str`, non-sensitive domain fields, and implement `self.action()` and `self.code() -> &'static str`.
2. **Central Application Rollup**: `core::error::AppError` wraps feature errors transparently via `#[error(transparent)] <Feature>(#[from] <Feature>Error)` and directly owns infrastructure failures (`InitSchema` for startup DDL migrations, `ShouldNotBeHappening` for runtime invariant breaks and unexpected database query/pool failures).
3. **Structured API Envelope**: `ApiResponse<ErrorPayload>` wraps errors as `data: { action, message }` with a screaming status identifier (`status: self.code()`). Specific, actionable messages are returned to the client in `ErrorPayload.message`, while full diagnostic error chains are logged to `tracing` (warn for 4xx, error for 5xx). Raw SQL queries and database internals are never leaked to clients.
4. **Action Hierarchy & Granular Query Context**: Function scopes declare compile-time action paths (`const ACTION: &str = "FEATURE.WORKFLOW[.STEP]"`) passed directly into error constructors. Queries are never bundled under vague umbrella actions; each distinct database call uses `DbResultExt` (`.db_context(action)`) or `create_db_object(action, table, ...)` with its own unique action token.

## Consequences

- Route handlers collapse into concise `Result<impl IntoResponse, AppError>` signatures relying on `?`.
- Single-keyword searchability (`rg "CATEGORY_ALREADY_EXISTS"` or `rg "CONFIG.CATEGORIES.CREATE_TYPE"`) is guaranteed across logs, UI, and backend code.
- Infrastructure and database failures are unified into `ShouldNotBeHappening` without artificial distinction between transaction vs query errors.
- Frontend API layers unpack `{ action, message }` directly into `ApiError` properties for badges and user messaging.
