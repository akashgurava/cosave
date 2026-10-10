use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

use cosave::{init_db, init_features, init_schemas, router, AppConfig, AppState, DbPool};

/// Lightweight in-process test harness for black-box HTTP verification against Axum.
pub struct TestApp {
    router: Router,
    db: DbPool,
}

impl TestApp {
    /// Creates a fresh in-memory database with migrations and category default seeds.
    pub async fn new() -> Self {
        let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
            .await
            .expect("Failed to initialize test SQLite in-memory database");
        init_schemas(&pool)
            .await
            .expect("Failed to run schema migrations in test database");
        init_features(&pool)
            .await
            .expect("Failed to seed feature defaults in test database");

        let state = AppState::for_test(pool.clone());
        let router = Router::new().nest("/api/v1", router().with_state(state));

        Self { router, db: pool }
    }

    /// Creates a fresh in-memory database with migrations but without seeding defaults.
    pub async fn new_unseeded() -> Self {
        let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
            .await
            .expect("Failed to initialize test SQLite in-memory database");
        init_schemas(&pool)
            .await
            .expect("Failed to run schema migrations in test database");

        let state = AppState::for_test(pool.clone());
        let router = Router::new().nest("/api/v1", router().with_state(state));

        Self { router, db: pool }
    }

    /// Returns a reference to the underlying test SQLite connection pool.
    pub fn db(&self) -> &DbPool {
        &self.db
    }

    /// Sends a raw HTTP request into the router and returns status, headers, and parsed JSON.
    pub async fn request(&self, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("Router oneshot execution failed");

        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body bytes");

        let json: Value = if body_bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
                Value::String(String::from_utf8_lossy(&body_bytes).into_owned())
            })
        };

        (status, headers, json)
    }

    /// Helper for GET requests.
    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .expect("Failed to build GET request");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for GET requests with a session cookie.
    pub async fn get_with_cookie(&self, uri: &str, cookie: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("Failed to build GET request with cookie");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for GET requests with an Authorization: Bearer token.
    pub async fn get_with_bearer(&self, uri: &str, token: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .expect("Failed to build GET request with Bearer token");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for POST requests with JSON payload.
    pub async fn post(&self, uri: &str, body: Value) -> (StatusCode, HeaderMap, Value) {
        let body_str = serde_json::to_string(&body).expect("Failed to serialize body");
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body_str))
            .expect("Failed to build POST request");

        self.request(req).await
    }

    /// Helper for POST requests with JSON payload and session cookie.
    pub async fn post_with_cookie(
        &self,
        uri: &str,
        body: Value,
        cookie: &str,
    ) -> (StatusCode, Value) {
        let body_str = serde_json::to_string(&body).expect("Failed to serialize body");
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie)
            .body(Body::from(body_str))
            .expect("Failed to build POST request with cookie");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for PATCH requests with JSON payload.
    pub async fn patch(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        let body_str = serde_json::to_string(&body).expect("Failed to serialize body");
        let req = Request::builder()
            .method(Method::PATCH)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body_str))
            .expect("Failed to build PATCH request");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for PATCH requests with JSON payload and session cookie.
    pub async fn patch_with_cookie(
        &self,
        uri: &str,
        body: Value,
        cookie: &str,
    ) -> (StatusCode, Value) {
        let body_str = serde_json::to_string(&body).expect("Failed to serialize body");
        let req = Request::builder()
            .method(Method::PATCH)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie)
            .body(Body::from(body_str))
            .expect("Failed to build PATCH request with cookie");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for DELETE requests without cookie.
    pub async fn delete(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .body(Body::empty())
            .expect("Failed to build DELETE request");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for DELETE requests with session cookie.
    pub async fn delete_with_cookie(&self, uri: &str, cookie: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("Failed to build DELETE request with cookie");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Creates an admin user and returns the `cosave_session=<token>` cookie string.
    pub async fn login_as_admin(&self) -> String {
        let (status, headers, body) = self
            .post(
                "/api/v1/auth/register",
                serde_json::json!({
                    "username": "admin_test",
                    "password": "Password123!"
                }),
            )
            .await;

        assert_eq!(status, StatusCode::CREATED, "Admin setup failed: {body:?}");

        headers
            .get(header::SET_COOKIE)
            .and_then(|h| h.to_str().ok())
            .map(|c| c.split(';').next().unwrap_or("").to_string())
            .expect("Set-Cookie header missing from register response")
    }

    /// Creates a member via `POST /api/v1/config/member` and returns the generated member ID.
    pub async fn create_member(&self, cookie: &str, member_name: &str) -> i64 {
        let (status, body) = self
            .post_with_cookie(
                "/api/v1/config/member",
                serde_json::json!({
                    "familyId": 1,
                    "memberName": member_name
                }),
                cookie,
            )
            .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "Create member failed: {body:?}"
        );
        body["data"]["id"].as_i64().expect("member ID")
    }

    /// Creates a bank account via `POST /api/v1/config/account/bank` and returns the account ID.
    pub async fn create_bank_account(
        &self,
        cookie: &str,
        owner_member_id: i64,
        bank_name: &str,
        account_name: &str,
        available_balance: i64,
    ) -> i64 {
        let (status, body) = self
            .post_with_cookie(
                "/api/v1/config/account/bank",
                serde_json::json!({
                    "familyId": 1,
                    "ownerMemberId": owner_member_id,
                    "currencyId": 1,
                    "bankName": bank_name,
                    "accountName": account_name,
                    "last4": "1234",
                    "availableBalance": available_balance
                }),
                cookie,
            )
            .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "Create bank account failed: {body:?}"
        );
        body["data"]["id"].as_i64().expect("account ID")
    }

    /// Provisions a standard test family instrument (member + checking account + type ID) via HTTP.
    pub async fn seed_test_account(&self, cookie: &str) -> (i64, i64) {
        // Ensure family exists with currency 1
        self.post_with_cookie(
            "/api/v1/config/family",
            serde_json::json!({
                "familyName": "Test Family",
                "currencyId": 1
            }),
            cookie,
        )
        .await;

        let member_id = self.create_member(cookie, "Test Member").await;
        let account_id = self
            .create_bank_account(cookie, member_id, "Chase", "Checking", 500_000)
            .await;

        // Fetch hierarchy to resolve first transaction type ID
        let (status, body) = self
            .get_with_cookie("/api/v1/config/hierarchy", cookie)
            .await;
        assert_eq!(status, StatusCode::OK);
        let type_id = body["data"]["types"][0]["id"]
            .as_i64()
            .expect("transaction type ID from hierarchy");

        (account_id, type_id)
    }
}
