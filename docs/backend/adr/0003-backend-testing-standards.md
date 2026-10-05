# Backend Testing Strategy: 3-Tier Architecture, Physical Seam Separation, and Anti-Pattern Rubric

Backend tests in CoSave are structured into a 3-tier testing pyramid: (1) Pure Domain Unit Tests (`models.rs`), (2) Direct Database & Transaction Tests (`db/tests.rs` or `db.rs`), and (3) Black-Box HTTP Route Integration Tests (`backend/tests/api/main.rs`), enforcing strict mandatory-when-present thresholds and explicitly outlawing vanity tests.

## Context & Decision

Previously, backend integration tests were codified solely around black-box HTTP route tests directly embedded inside `routes.rs`. While this successfully eliminated direct handler invocation (`create_type(...).await`), confining all tests to `routes.rs` introduced severe architectural friction:

1. **Implementation File Bloat**: Route files swelled to 800+ lines, burying core routing and HTTP mapping logic under hundreds of lines of test fixtures and assertions.
2. **Missing Granular Verification**:
   - Pure domain invariants (Value Object parsing, boundary conditions, whitespace trimming, integer currency overflow) were either tested slowly and indirectly through HTTP requests or omitted entirely.
   - Persistence and transactional invariants (multi-statement transaction rollbacks on failure, `ON DELETE CASCADE` referential integrity, and complex view aggregations) were coupled to HTTP endpoints rather than tested directly against the database engine.
3. **Vanity Testing Risks**: Without explicit guardrails, tests risked degrading into low-value vanity checks—such as asserting standard `serde` serialization roundtrips, testing trivial struct field getters, or duplicating identical CRUD operations across both database and route layers.

We established a comprehensive, high-signal 3-tier testing standard:

### 1. The 3-Tier Testing Pyramid

Every test must belong to one of three well-defined tiers, each governed by strict mandatory thresholds:

| Tier | Focus & Location | Mandatory Threshold | Forbidden Anti-Patterns |
| :--- | :--- | :--- | :--- |
| **Tier 1: Pure Domain Unit Tests** | Pure business logic, Value Objects, and domain math in `models.rs` (`#[cfg(test)] mod tests`). Pure in-memory (0 DB, 0 HTTP, microsecond execution). | Mandatory whenever a Value Object (`try_new`), domain calculation (`AmountMinorUnits` overflow/rounding), or domain state machine exists. | ❌ Testing trivial struct getters or public fields.<br>❌ Testing serde derive roundtrips or string formatting.<br>❌ Testing compiler/macro derivations (`Clone`, `Debug`). |
| **Tier 2: Direct Database & Transaction Tests** | Database queries, transactions, cascades, and views in `db/tests.rs` or `db.rs` (`#[cfg(test)] mod tests`). Direct execution against an isolated `DbPool`. | Mandatory for multi-statement write transactions (`pool.begin()`) verifying atomic rollback on failure, `ON DELETE CASCADE\|RESTRICT` constraints, and complex CTEs/views. | ❌ Duplicate CRUD tests: testing basic single-row `INSERT` or `SELECT` already verified by route tests.<br>❌ Testing trivial SQL syntax checked by SQLx. |
| **Tier 3: Black-Box HTTP Route Tests** | Router, middleware, wire JSON envelopes, and status codes in `backend/tests/api/main.rs`. Black-box HTTP execution via `TestApp.oneshot()`. | Mandatory for every endpoint without exception, adhering strictly to the 3-axis matrix. | ❌ Calling handler functions directly in memory.<br>❌ Deserializing responses into Rust structs (`ApiResponse<T>`) instead of inspecting raw `serde_json::Value`. |

### 2. Physical Seam Separation & Single Test Binary

To prevent route handler bloat while keeping test linking times sub-second:
- **Tier 1 and Tier 2** remain co-located inside the feature module as unit tests (`#[cfg(test)] mod tests`), preserving direct access to private Value Object constructors and internal database query functions without relaxing module visibility.
- **Tier 3 (Route Tests)** is physically separated into `backend/tests/api/` (e.g., `category_routes.rs`, `auth_routes.rs`), compiled through a single integration test runner (`backend/tests/api/main.rs`). This physical seam provides compiler-level guarantees: tests cannot accidentally call internal handler functions or inspect private feature state.

### 3. Mandatory 3-Axis Route Matrix

Every endpoint tested under Tier 3 must satisfy three orthogonal axes:
- **Axis 1 (Happy Path & User Experience)**: Valid input returns expected HTTP status (200/201), valid JSON envelope `{ "code": 0, "status": "OK", "data": ... }`, required cookie headers, and verifies persisted database state.
- **Axis 2 (Domain Validation & User Messaging)**: Invalid or conflicting input returns exact 400/409 HTTP status, screaming error code, unique action token, and an actionable user message in `{ "code", "status", "data": { "action", "message" } }`.
- **Axis 3 (Auth & Boundary Security)**: Protected routes reject missing or expired session cookies with 401 Unauthorized, returning expected auth error tokens without leaking internal database or engine details.

### 4. Test Harness & In-Memory Isolation (`TestApp`)

All database and route tests execute against an isolated `sqlite::memory:` database pool with complete schema migrations (`init_schemas(&pool)`). Boilerplate is unified in `crate::core::test_utils::TestApp`:
- `TestApp::new().await`: Provisions isolated test app and HTTP client.
- `TestApp::new_db_pool().await` / `app.db()`: Exposes an isolated `DbPool` for direct Tier 2 database and transaction tests.

## Consequences

- Route files remain concise and readable (~100-200 lines), stripped of test boilerplate.
- Domain rules, edge cases, and arithmetic overflows are exhaustively tested at microsecond speeds.
- Multi-step write transactions explicitly verify rollback safety when intermediate operations fail.
- Low-value vanity testing (serde roundtrips, getters, duplicate CRUD) is permanently banned across the engineering team and automated agents.
- Tier 3 route tests maintain a true black-box HTTP compilation boundary.
