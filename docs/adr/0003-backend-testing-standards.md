# Backend API Testing Standards and Black-Box HTTP Verification

Backend integration tests must execute as black-box HTTP requests through Axum's router using `app.oneshot(Request)`, strictly asserting wire-format JSON response envelopes (`serde_json::Value`) and enforcing a mandatory 3-axis test matrix across every endpoint.

## Context & Decision

Prior backend tests directly invoked Rust handler functions in-memory (`create_type(State(...), ...).await`) and asserted against internal Rust structs. This bypassed HTTP routing, path extraction, cookie handling, middleware, JSON serialization, and error translation, testing trivial struct unpacking rather than the real API contract or user experience.

We established a strict, consumer-driven backend testing standard:

1. **Strict HTTP Execution Seam (`app.oneshot(Request)`)**: Route tests are banned from calling Rust handler functions directly. All tests must construct standard HTTP requests (`http::Request`), pass them through the compiled Axum router via `tower::Service::oneshot`, and inspect raw HTTP response status codes, `Set-Cookie` headers, and wire response bytes.
2. **Wire-Format JSON Envelope Assertions (`serde_json::Value`)**: Response bodies must be parsed into raw `serde_json::Value` to strictly assert the external API contract consumed by the frontend and client:
   - Success responses must verify `{ "code": 0, "status": "OK", "data": <PAYLOAD> }`.
   - Error responses must verify `{ "code": <HTTP_STATUS>, "status": "<SCREAMING_ERROR_CODE>", "data": { "action": "<UNIQUE_ACTION>", "message": "<USER_FACING_MESSAGE>" } }`.
   - Deserializing responses back into internal Rust structs (`ApiResponse<T>`) is forbidden in route tests.
3. **Mandatory 3-Axis Test Matrix**: Every endpoint must be covered by three orthogonal axes:
   - **Axis 1 (Happy Path & User Experience)**: Valid input returns expected HTTP status (200/201), valid JSON envelope, required cookie headers (e.g. `Set-Cookie` on auth routes), and verifies persisted database state.
   - **Axis 2 (Domain Validation & User Messaging)**: Invalid or conflicting input returns exact 400/409 HTTP status, screaming error code, unique action token, and a clear, actionable user message.
   - **Axis 3 (Auth & Boundary Security)**: Protected routes reject missing or expired session cookies with 401 Unauthorized, returning expected auth error tokens without leaking internal database or engine details.
4. **Isolated Per-Test In-Memory SQLite**: Every test suite runs against an isolated `sqlite::memory:` database pool with complete schema migrations (`init_schemas(&pool)`), guaranteeing zero test state contamination and deterministic parallel execution.
5. **Ergonomic Test Harness (`TestApp`)**: Boilerplate for constructing HTTP requests, headers, and parsing JSON bodies is unified into an ergonomic `TestApp` harness located in `crate::core::test_utils`, providing semantic client methods (`get`, `post`, `patch`, `delete`, `with_cookie`).

## Consequences

- Tests mirror exact client network traffic and user experience with zero runtime mock drift.
- Breaking changes to JSON wire casing, field nullability, cookie flags, or error payloads fail immediately at test time.
- Direct handler testing anti-patterns are eliminated codebase-wide.
- Route tests run fast in-memory (<1 second for the full suite) without spinning up network ports or sockets.
