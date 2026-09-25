# Unified Error Architecture and Propagation

Backend errors roll up transparently from feature-scoped enums into a central `AppError`, with compile-time action paths and screaming error identifiers surfacing structured `{ action, message }` payloads to API clients.

## Context & Decision

CoSave requires strict error searchability across server logs, developer tooling, and user interfaces, with specific, actionable failure diagnostics. Error tokens must never be duplicated between macro attributes and string returns.

We established a strict, two-tier error model:

1. **Error Token SSOT & Feature-Scoped Domain Errors**: Features define discrete error types (e.g. `AuthError`, `CategoryError`) in `features/<feature>/error.rs`. Every variant defines `action: &'static str` and non-sensitive domain fields. `self.code() -> &'static str` is the absolute Single Source of Truth for screaming error tokens. Deriving `thiserror::Error` or duplicating error code strings in format macros is forbidden.
2. **Manual `Display` & `Error` Trait Implementation**: Enums implement `std::fmt::Display` manually, formatting every variant message with `{code}. ACTION: {action}` as the uniform prefix where `let code = self.code();`. `std::error::Error` is implemented manually, returning `source` references (`Some(source)`) for wrapped errors and `None` otherwise.
3. **Central Application Rollup**: `core::error::AppError` wraps feature errors via manual `From` implementations (`From<AuthError>`, `From<CategoryError>`) and directly owns infrastructure failures (`InitSchema` for startup DDL migrations, `ShouldNotBeHappening` for runtime invariant breaks and unexpected database query/pool failures).
4. **Structured API Envelope**: `ApiResponse<ErrorPayload>` wraps errors as `data: { action, message }` with a screaming status identifier (`status: self.code()`). Specific, actionable messages are returned to the client in `ErrorPayload.message`, while full diagnostic error chains are logged to `tracing` (warn for 4xx, error for 5xx). Raw SQL queries and database internals are never leaked to clients.
5. **Action Uniqueness & Granular Failure Context**: Every error instantiation, validation check, database query, and transaction boundary carries a globally unique compile-time action path (`FEATURE.WORKFLOW.STEP[.BRANCH]`). Reusing umbrella action tokens across multiple distinct failure sites or branches is strictly forbidden; searching any action string with `rg` pinpoints the single exact failure site.
6. **Authoritative Error Logging & Standalone Log Identifiers**: Axum's `into_response()` serves as the sole, authoritative server logging sink for errors, emitting structured `tracing::warn!` or `tracing::error!` records with action, code, and source chains directly to the terminal. Call sites never double-log errors before returning `Err(...)`. Standalone log statements (informational, startup, lifecycle, debug where no error is returned) carry dedicated, globally unique compile-time action tokens prefixed as `{ACTION}. {message}`.

## Consequences

- Route handlers collapse into concise `Result<impl IntoResponse, AppError>` signatures relying on `?`.
- Single-keyword searchability (`rg "CATEGORY_ALREADY_EXISTS"` or `rg "CONFIG.CATEGORIES.CREATE_CATEGORY.ALREADY_EXISTS"`) is guaranteed across logs, UI, and backend code.
- Zero token ambiguity: every failure path logs a globally unique action token.
- Zero duplicate error logs: errors are logged strictly once at the `into_response()` boundary.
- Zero error string duplication: tokens exist in exactly one line of source code.
- Infrastructure and database failures are unified into `ShouldNotBeHappening` without artificial distinction between transaction vs query errors.
- Frontend API layers unpack `{ action, message }` directly into `ApiError` properties for badges and user messaging.
