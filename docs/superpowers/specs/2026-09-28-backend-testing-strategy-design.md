# Design Spec: 3-Tier Backend Testing Architecture & Split Directory Layout

- **Date**: 2026-09-28
- **Topic**: Backend Testing Strategy, ADR 0003 Amendment, and Documentation
- **Status**: Approved in Brainstorming

---

## 1. Problem Statement & Motivation

Currently, backend tests in CoSave reside almost exclusively as in-module tests inside `backend/src/features/<feature>/routes.rs`. While this successfully enforces black-box HTTP route verification via the 3-axis matrix, it suffers from two major deficiencies:

1. **File Bloat in Implementation Modules**: Route handlers in `routes.rs` are obscured by 600+ lines of test code, making routing and request handling logic difficult to navigate and maintain.
2. **Missing Granular Verification Layers**:
   - **Domain Logic & Value Objects**: Validation rules, boundary cases, and financial math are either tested indirectly through cumbersome HTTP calls or left untested at the unit level.
   - **Database & Transaction Mechanics**: Atomic transaction rollbacks on partial failure, complex recursive CTEs/views (`v_category_hierarchy`), and foreign key cascades (`ON DELETE CASCADE` vs `RESTRICT`) lack dedicated database-level tests.
3. **Risk of Vanity Testing**: Without clear rubrics, developers and agents risk writing low-signal "vanity tests" (e.g. testing serde derive roundtrips, trivial struct getters, or duplicating basic CRUD tests across both DB and Route tiers).

---

## 2. The 3-Tier Testing Architecture

We establish a strictly differentiated 3-tier testing pyramid where each tier has a distinct scope, execution speed, and mandatory threshold.

```
                    ▲
                   / \
                  /   \
                 / Tier\
                /   3   \     Black-Box HTTP Route Tests (`backend/tests/api/`)
               /─────────\    Mandatory for EVERY endpoint.
              /   Tier    \   Direct DB & Transaction Tests (`src/features/*/db/`)
             /     2       \  Mandatory for transactions, cascades, views. Banned for CRUD.
            /───────────────\ Pure Domain Unit Tests (`src/features/*/models.rs`)
           /     Tier 1      \ Mandatory for Value Objects, domain math, invariants.
          /───────────────────\
```

### 2.1 Tier 1: Pure Domain Unit Tests (`models.rs`)
- **Location**: `backend/src/features/<feature>/models.rs` (in `#[cfg(test)] mod tests`).
- **When Mandatory**: Whenever a Value Object (`try_new`), domain calculation (`AmountCents` arithmetic/overflow/rounding), or domain state machine transition exists.
- **Scope**:
  - Boundary input parsing (empty strings, whitespace trimming, string length boundaries, valid/invalid formats).
  - Overflow, underflow, and precision in financial calculations.
  - State machine transition validity.
- **Execution**: Pure in-memory (no database pool, no network, no HTTP router). Execution takes microseconds.
- **Anti-Patterns (Strictly Banned)**:
  - ❌ Testing trivial struct getters or public fields (`assert_eq!(item.id(), "123")`).
  - ❌ Testing serde derive roundtrips or string format interpolations on internal structs.
  - ❌ Testing third-party macro derivations (e.g. asserting that `#[derive(Clone)]` or `#[derive(Debug)]` works).

### 2.2 Tier 2: Direct Database & Transaction Tests (`db/tests.rs` or `db.rs`)
- **Location**: `backend/src/features/<feature>/db/tests.rs` (or `#[cfg(test)] mod tests` in `db.rs`).
- **When Mandatory**:
  - Multi-statement write transactions (`pool.begin()`) to verify atomic rollback on partial failure.
  - Foreign key cascading behaviors (`ON DELETE CASCADE` vs `ON DELETE RESTRICT`).
  - Complex hierarchical queries, recursive CTEs, or SQLite join views (e.g. `v_category_hierarchy`).
  - Concurrency conflicts or unique constraint collision semantics at the database engine level.
- **Scope**: Executed directly against an isolated in-memory `DbPool` using internal feature queries with realistic fixtures.
- **Anti-Patterns (Strictly Banned)**:
  - ❌ Duplicate CRUD tests: Writing a direct DB test for a basic single-row `INSERT`, `UPDATE`, or `SELECT` that is already exercised end-to-end in Tier 3 Route tests.
  - ❌ Testing raw SQL syntax on trivial statements that the compiler/SQLx query macros already verify.

### 2.3 Tier 3: Black-Box HTTP Route Integration Tests (`backend/tests/api/`)
- **Location**: `backend/tests/api/<feature>_routes.rs` compiled through `backend/tests/api/main.rs`.
- **When Mandatory**: Every endpoint without exception.
- **Scope**: Mandatory 3-axis matrix via `TestApp.oneshot()`:
  - **Axis 1 (Happy Path & UX)**: Valid request -> 200/201, wire JSON envelope `{ code: 0, status: "OK", data: ... }`, cookies, and verified persistence.
  - **Axis 2 (Domain Validation & User Messaging)**: Invalid/conflicting input -> 400/409, `{ code, status, data: { action, message } }`.
  - **Axis 3 (Auth & Boundary Security)**: Missing/expired session -> 401/403, zero leak of SQL or internal traces.
- **Execution**: External integration tests compiled in `backend/tests/api/main.rs`.
- **Anti-Patterns (Strictly Banned)**:
  - ❌ Invoking handler functions directly in-memory (`create_type(...).await`).
  - ❌ Deserializing responses back into internal Rust structs (`ApiResponse<T>`) instead of inspecting raw `serde_json::Value`.

---

## 3. Directory Layout & Compilation Seam

```text
backend/
├── Cargo.toml
├── src/
│   ├── lib.rs                     # Exports TestApp under #[cfg(any(test, feature = "test-utils"))]
│   ├── core/
│   │   ├── mod.rs
│   │   └── test_utils.rs          # TestApp harness & test db pool provisioning
│   └── features/
│       └── categories/
│           ├── models.rs          # [Tier 1 Unit Tests] in #[cfg(test)] mod tests
│           ├── db/
│           │   ├── categories.rs
│           │   ├── transaction_types.rs
│           │   └── tests.rs       # [Tier 2 DB Tests] in #[cfg(test)] mod tests
│           ├── routes.rs          # Clean Axum handlers (~150 lines, ZERO route test bloat)
│           ├── error.rs
│           └── mod.rs
└── tests/                         # [Tier 3 Route Integration Tests]
    └── api/                       # Single test binary for fast compilation
        ├── main.rs                # Router integration runner
        ├── auth_routes.rs         # 3-axis black-box tests for /api/v1/auth
        └── category_routes.rs     # 3-axis black-box tests for /api/v1/categories
```

### 3.1 Single Integration Test Binary
In `backend/Cargo.toml`, register the integration test binary:
```toml
[[test]]
name = "api"
path = "tests/api/main.rs"
```
Inside `backend/tests/api/main.rs`:
```rust
mod auth_routes;
mod category_routes;
```
This prevents Cargo from linking multiple separate binaries for each test file, preserving sub-second test execution speeds.

### 3.2 Harness Support (`TestApp`)
- `TestApp` in `backend/src/core/test_utils.rs` provides both:
  1. Full HTTP testing client: `TestApp::new().await`, `app.get()`, `app.post_with_cookie()`.
  2. Direct database pool provisioning for Tier 2 tests: `TestApp::new_db_pool().await` or `app.db()`.
- `TestApp` is exposed from `cosave` for test binaries via `#[cfg(any(test, feature = "test-utils"))]` in `backend/src/lib.rs`.

---

## 4. Documentation Updates

1. **`docs/adr/0003-backend-testing-standards.md`**:
   - Rename / amend title to: "Backend Testing Strategy: 3-Tier Architecture, Physical Seam Separation, and Anti-Pattern Rubric".
   - Record the decision to establish Tier 1 (Domain Unit), Tier 2 (Direct DB), and Tier 3 (External HTTP Route Integration in `tests/api/`).
   - Codify the mandatory thresholds and explicitly banned vanity test patterns.
2. **`docs/agents/backend-testing.md`**:
   - Update with comprehensive sections for Tier 1, Tier 2, and Tier 3.
   - Add canonical code examples for each tier (Value Object test, Transaction rollback DB test, Route test).
   - Document the Anti-Pattern Rubric (what is forbidden across all tiers).
   - Document the file structure and single integration test binary setup.
3. **`AGENTS.md`**:
   - Update Section 3 (Backend Invariants -> Testing section) to reference the 3-Tier Backend Testing Architecture and the split directory layout.
