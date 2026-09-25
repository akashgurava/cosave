# Backend Testing Standards & Conventions

Authoritative standards for writing integration, contract, and regression tests across the Rust Axum backend.

---

## 1. Core Principles & Architecture

Backend tests verify the user experience and external API contract as a true black box. Tests execute against the compiled Axum router using in-process HTTP requests via `tower::Service::oneshot`.

1. **Direct Handler Invocations are Banned**: Calling Rust handler functions directly (`create_type(State(...), ...).await`) is strictly forbidden in route tests. Direct calls bypass path parsing, auth extractors (`AuthUser`), cookie handling (`CookieJar`), middleware, JSON serialization, and error mapping.
2. **Assertions Target Wire JSON (`serde_json::Value`)**: Tests assert against raw JSON representations rather than deserializing into Rust structs. This guarantees that field naming, casing, envelope wrapping, and error structures match the frontend client contract.
3. **Isolated In-Memory Databases**: Every test instance uses an isolated `sqlite::memory:` database with full schema initialization (`init_schemas(&pool)`), preventing cross-test state leakage.
4. **Ergonomic `TestApp` Harness**: Tests use `crate::core::test_utils::TestApp` to execute HTTP calls cleanly without manual boilerplate.

---

## 2. The Mandatory 3-Axis Test Matrix

Every API endpoint must provide automated test coverage along three orthogonal axes:

| Axis                                 | Focus                                 | Required Checks                                                                                                                                                                                                                            |
| :----------------------------------- | :------------------------------------ | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Axis 1: Happy Path & UX**          | Successful workflow                   | - Expected HTTP status (200 OK, 201 Created)<br>- Outer envelope: `{ "code": 0, "status": "OK", "data": ... }`<br>- Response cookies (e.g. `Set-Cookie` on auth routes)<br>- Verification of database persistence on subsequent reads      |
| **Axis 2: Domain Validation**        | Business rule rejections              | - HTTP status 400 Bad Request or 409 Conflict<br>- Envelope: `{ "code": <HTTP_STATUS>, "status": "<CODE>", "data": { "action": "<UNIQUE_ACTION>", "message": "<ACTIONABLE_USER_STRING>" } }`<br>- Exact error token and action token match |
| **Axis 3: Auth & Security Boundary** | Unauthenticated & unauthorized access | - HTTP status 401 Unauthorized or 403 Forbidden<br>- Missing or invalid token yields expected `AUTH.EXTRACT_USER.*` action<br>- Zero leaks of raw SQL, database drivers, or server stack traces                                            |

---

## 3. Test Harness Anatomy: `TestApp`

Located in `backend/src/core/test_utils.rs` (compiled under `#[cfg(test)]`):

```rust
let app = TestApp::new().await;

// 1. Unauthenticated request
let (status, body) = app.get("/api/v1/health").await;
assert_eq!(status, StatusCode::OK);
assert_eq!(body["status"], "HEALTHY");

// 2. Authenticated request with session cookie
let (status, body) = app
    .post_with_cookie(
        "/api/v1/categories/types",
        json!({ "name": "Investments", "color": "#10b981" }),
        &session_cookie,
    )
    .await;
assert_eq!(status, StatusCode::CREATED);
assert_eq!(body["data"]["name"], "Investments");
```

---

## 4. Rigidity Contrast

### Anti-Pattern ❌ (Direct Handler Calls, Struct Unpacking, Trivial String Tests)

```rust
// AVOID: Calling handler functions directly in tests
let (status, res) = create_type(State(state.clone()), test_user(), Json(req)).await.unwrap();
assert_eq!(status, StatusCode::CREATED);
let data = res.0.into_data(); // Bypasses JSON serialization!
assert_eq!(data.name(), "Crypto");

// AVOID: Testing dummy serializer functions instead of hitting the endpoint
let resp = ApiResponse::ok(Status::healthy(), HealthData {});
let serialized = serde_json::to_value(&resp).unwrap();
assert_eq!(serialized["status"], "HEALTHY");
```

### Canonical Pattern ✅ (Black-Box HTTP via `TestApp` & Wire-JSON Inspection)

```rust
// PREFER: Complete black-box HTTP request through the router
let app = TestApp::new().await;
let admin_cookie = app.login_as_admin().await;

let (status, body) = app
    .post_with_cookie(
        "/api/v1/categories/types",
        json!({ "name": "Crypto", "color": "#8b5cf6" }),
        &admin_cookie,
    )
    .await;

// Assert HTTP Status & Outer ApiResponse Envelope
assert_eq!(status, StatusCode::CREATED);
assert_eq!(body["code"], 0);
assert_eq!(body["status"], "OK");
assert_eq!(body["data"]["name"], "Crypto");
assert_eq!(body["data"]["color"], "#8b5cf6");

// Assert Domain Validation & Error Envelope
let (err_status, err_body) = app
    .post_with_cookie(
        "/api/v1/categories/types",
        json!({ "name": "Crypto", "color": "#8b5cf6" }),
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
assert!(err_body["data"]["message"]
    .as_str()
    .unwrap()
    .contains("already exists"));

// Assert Auth Boundary
let (unauth_status, unauth_body) = app
    .post(
        "/api/v1/categories/types",
        json!({ "name": "Unauthorized Type" }),
    )
    .await;

assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
assert_eq!(unauth_body["status"], "UNAUTHENTICATED");
assert_eq!(
    unauth_body["data"]["action"],
    "AUTH.EXTRACT_USER.MISSING_TOKEN"
);
```
