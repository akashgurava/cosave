# Backend Testing Standards & Conventions

Authoritative standards for writing integration, unit, and regression tests across the Rust Axum backend.

---

## 1. Core Philosophy & The 3-Tier Testing Pyramid

Testing in CoSave guarantees extreme fidelity, fast feedback, and zero vanity tests. Tests verify real domain invariants, database integrity, and consumer HTTP wire contracts.

Every backend test belongs to one of three strictly delineated tiers:

```
                    ▲
                   / \
                  /   \
                 / Tier\
                /   3   \     Black-Box HTTP Route Tests (`backend/tests/api/`)
               /─────────\    Mandatory for EVERY endpoint. 3-axis matrix.
              /   Tier    \   Direct DB & Transaction Tests (`src/features/*/db/`)
             /     2       \  Mandatory for transactions, cascades, views. Banned for CRUD.
            /───────────────\ Pure Domain Unit Tests (`src/features/*/models.rs`)
           /     Tier 1      \ Mandatory for Value Objects, domain math, invariants.
          /───────────────────\
```

1. **High Signal-to-Noise Ratio**: Every test must prove a non-trivial failure mode or contract invariant. Tests that merely exercise language syntax, compiler derives, or trivial getters are strictly banned.
2. **Fastest Execution Seam First**: Test domain rules and calculations in pure memory (Tier 1, microseconds). Test transaction rollback and cascading mechanics directly against SQLite (Tier 2, milliseconds). Reserve HTTP routing and wire-serialization assertions for true end-to-end integration (Tier 3).
3. **Black-Box Wire JSON Verification**: Route tests execute against the compiled Axum router via `tower::Service::oneshot`. Direct handler invocation (`create_type(...).await`) and deserialization back into internal Rust structs (`ApiResponse<T>`) are strictly forbidden.

---

## 2. Directory Layout & Compilation Seams

```text
backend/
├── src/
│   ├── core/
│   │   └── test_utils.rs          # TestApp harness & test db pool provisioning
│   └── features/
│       └── categories/
│           ├── models.rs          # [Tier 1 Unit Tests] in #[cfg(test)] mod tests
│           ├── db/
│           │   ├── categories.rs
│           │   ├── transaction_types.rs
│           │   └── tests.rs       # [Tier 2 DB Tests] in #[cfg(test)] mod tests
│           ├── routes.rs          # Clean Axum handlers (~100-200 lines, ZERO test bloat)
│           ├── error.rs
│           └── mod.rs
└── tests/                         # [Tier 3 Route Integration Tests]
    └── api/                       # Single test binary for fast compilation
        ├── main.rs                # Router integration runner
        ├── auth_routes.rs         # 3-axis black-box tests for /api/v1/auth
        └── category_routes.rs     # 3-axis black-box tests for /api/v1/config/hierarchy & categories
```

- **Tier 1 & Tier 2 stay co-located**: Kept inside feature modules under `#[cfg(test)] mod tests`. This provides immediate access to private Value Object constructors and internal database queries without exposing them across module boundaries.
- **Tier 3 moves to `backend/tests/api/`**: Keeps `routes.rs` clean and uncluttered. It compiles as an external crate consumer, physically preventing developers or agents from bypassing the HTTP stack or calling handlers directly.
- **Single Test Binary**: All integration tests in `backend/tests/api/` are modules registered under `backend/tests/api/main.rs`. This prevents Cargo from compiling multiple integration test binaries, keeping test linking times sub-second.

---

## 3. Tier Specifications & Mandatory Thresholds

### Tier 1: Pure Domain Unit Tests (`models.rs`)
* **Location**: Co-located in `models.rs` (`#[cfg(test)] mod tests`).
* **When Mandatory**: Whenever a Value Object (`try_new`), domain calculation (e.g. `AmountCents` arithmetic/overflow/rounding), or domain state machine exists.
* **Scope**: Boundary parsing (empty strings, whitespace trimming, string length boundaries, valid/invalid formats, arithmetic overflow).
* **Execution**: Pure in-memory (no database, no network, no HTTP router). Microsecond execution.

### Tier 2: Direct Database & Transaction Tests (`db/tests.rs` or `db.rs`)
* **Location**: Co-located in `db/tests.rs` or `db.rs` (`#[cfg(test)] mod tests`).
* **When Mandatory**:
  - Multi-statement write transactions (`pool.begin()`) verifying atomic rollback when an intermediate step fails.
  - Foreign key cascading behaviors (`ON DELETE CASCADE` vs `ON DELETE RESTRICT`).
  - Complex hierarchical queries, recursive CTEs, or SQLite join views (e.g. `v_category_hierarchy`).
  - Concurrency conflicts or unique constraint collision semantics at the database engine level.
* **Scope**: Executed directly against an isolated in-memory `DbPool`. Calls internal DB queries with realistic data.
* **Banned**: Writing redundant direct DB tests for basic single-row `INSERT`, `UPDATE`, or `SELECT` operations that are already tested in Tier 3 Route tests.

### Tier 3: Black-Box HTTP Route Integration Tests (`backend/tests/api/`)
* **Location**: `backend/tests/api/<feature>_routes.rs` compiled through `backend/tests/api/main.rs`.
* **When Mandatory**: Every endpoint without exception.
* **Scope**: The mandatory 3-axis matrix via `TestApp.oneshot()`:
  - **Axis 1 (Happy Path & UX)**: Valid request -> 200/201, wire JSON envelope `{ code: 0, status: "OK", data: ... }`, cookies, and verified persistence on subsequent reads.
  - **Axis 2 (Domain Validation & User Messaging)**: Invalid/conflicting input -> 400/409, `{ code, status, data: { action, message } }`.
  - **Axis 3 (Auth & Boundary Security)**: Missing/expired session -> 401/403, zero leak of SQL or internal traces.

---

## 4. Canonical Code Patterns

### Canonical Tier 1: Value Object Boundary Tests (`models.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_name_validation() {
        // Valid name with surrounding whitespace is trimmed
        let name = CategoryName::try_new("  Groceries  ", "TEST.NAME").expect("valid name");
        assert_eq!(name.as_str(), "Groceries");

        // Empty or whitespace-only is rejected with exact action
        let err = CategoryName::try_new("   ", "TEST.EMPTY").unwrap_err();
        assert_eq!(err.action(), "TEST.EMPTY");
        assert_eq!(err.code(), "EMPTY_CATEGORY_NAME");

        // Length boundary checks
        let exact_max = "a".repeat(64);
        assert!(CategoryName::try_new(exact_max, "TEST.MAX").is_ok());

        let too_long = "a".repeat(65);
        let err = CategoryName::try_new(too_long, "TEST.TOO_LONG").unwrap_err();
        assert_eq!(err.code(), "CATEGORY_NAME_TOO_LONG");
    }
}
```

### Canonical Tier 2: Transaction Rollback & Cascade Tests (`db/tests.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_utils::TestApp;

    #[tokio::test]
    async fn test_transaction_rolls_back_on_step_failure() {
        let pool = TestApp::new_db_pool().await;

        // Verify that if step 2 fails in a multi-write workflow, step 1 is rolled back
        let mut tx = pool.begin().await.expect("begin transaction");
        insert_category_item(&mut tx, "cat_temp", "Temporary").await.expect("step 1 succeeds");

        // Simulate failure on step 2
        let failure: Result<(), _> = Err("simulated failure");
        if failure.is_err() {
            tx.rollback().await.expect("rollback transaction");
        }

        // Assert that step 1 was NOT persisted
        let found = find_category_by_id(&pool, "cat_temp").await.expect("query succeeds");
        assert!(found.is_none(), "rolled back category must not exist in database");
    }

    #[tokio::test]
    async fn test_foreign_key_cascade_deletion() {
        let pool = TestApp::new_db_pool().await;

        // Create parent type and child category
        create_type(&pool, "typ_1", "Expense").await.unwrap();
        create_category(&pool, "cat_1", "typ_1", "Food").await.unwrap();

        // Delete parent type -> foreign key cascade must delete child category
        delete_type(&pool, "typ_1").await.unwrap();

        let category = find_category_by_id(&pool, "cat_1").await.unwrap();
        assert!(category.is_none(), "child category must be deleted by cascade");
    }
}
```

### Canonical Tier 3: Black-Box HTTP Route Tests (`backend/tests/api/`)

```rust
use axum::http::StatusCode;
use cosave::TestApp;
use serde_json::json;

#[tokio::test]
async fn test_create_category_3_axis_matrix() {
    let app = TestApp::new().await;
    let admin_cookie = app.login_as_admin().await;

    // Axis 1: Happy Path & User Experience (with DB persistence check)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({ "name": "Crypto", "color_id": 6 }),
            &admin_cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");
    assert_eq!(body["data"]["name"], "Crypto");

    // Axis 2: Domain Validation & User Messaging (Conflict)
    let (err_status, err_body) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({ "name": "Crypto", "color_id": 6 }),
            &admin_cookie,
        )
        .await;
    assert_eq!(err_status, StatusCode::CONFLICT);
    assert_eq!(err_body["code"], 409);
    assert_eq!(err_body["status"], "TYPE_ALREADY_EXISTS");
    assert_eq!(
        err_body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS"
    );

    // Axis 3: Auth & Boundary Security
    let (unauth_status, unauth_body) = app
        .post(
            "/api/v1/config/categories/types",
            json!({ "name": "Unauthorized", "color_id": 6 }),
        )
        .await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unauth_body["status"], "UNAUTHENTICATED");
    assert_eq!(
        unauth_body["data"]["action"],
        "AUTH.EXTRACT_USER.MISSING_TOKEN"
    );
}
```

---

## 5. Strict Anti-Pattern Rubric (Banned Tests)

| Forbidden Anti-Pattern | Why It Is Banned | Correct Alternative |
| :--- | :--- | :--- |
| **Serde Derive Roundtrips**<br>`assert_eq!(serde_json::to_string(&item), "...")` | Testing whether third-party macro `serde` works is a vanity test that adds zero domain confidence. | Test the real wire API contract in Tier 3 Route tests via `app.oneshot()`. |
| **String Formatting & Interpolation**<br>`assert_eq!(format!("{}", item), "...")` or `assert_eq!(item.to_string(), "...")` | Asserts trivial string template concatenation on internal types without verifying business logic or domain invariants. | Test actual boundary parsing and validation in Tier 1 Value Objects, or HTTP wire envelopes in Tier 3. |
| **Trivial Getters & Constructors**<br>`assert_eq!(item.id(), "123")` | Asserts standard Rust struct memory layout without business logic or invariant enforcement. | Test fallible Value Object constructors (`try_new`) that validate domain rules. |
| **Testing Macro Derivations**<br>`assert_eq!(item.clone(), item)` | Verifies the Rust standard library `Clone` or `Debug` derive, which is guaranteed by the compiler. | Omit completely. |
| **Direct Handler Invocation**<br>`create_type(State(...), req).await` | Bypasses HTTP routing, path extractors, auth middleware, cookie jars, and error response envelopes. | Execute through Axum via `app.post_with_cookie()` or `TestApp.oneshot()`. |
| **Deserializing into Internal Structs**<br>`let resp: ApiResponse<Category> = ...` | Masks missing wire fields, casing discrepancies, or nullability mismatch in the external contract. | Assert directly against raw `serde_json::Value` in Tier 3 tests. |
| **Duplicate CRUD DB Tests**<br>Direct DB test asserting basic `insert_user()` and `find_user()` | Duplicates the identical query paths already fully exercised by Tier 3 route tests. | Reserve Tier 2 DB tests strictly for multi-step transactions, rollbacks, and cascades. |

---

## 6. Test Harness: `TestApp`

Located in `crate::core::test_utils`:

```rust
// 1. Provision an isolated in-memory HTTP app (Tier 3)
let app = TestApp::new().await;
let admin_cookie = app.login_as_admin().await;
let (status, body) = app.get("/api/v1/config/hierarchy").await;

// 2. Provision an isolated in-memory DbPool (Tier 2)
let pool = TestApp::new_db_pool().await;
```
