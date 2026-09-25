use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

use crate::core::{init_db, AppState};

/// Lightweight in-process test harness for black-box HTTP verification against Axum.
pub(crate) struct TestApp {
    router: Router,
}

impl TestApp {
    /// Creates a fresh in-memory database with migrations and category default seeds.
    pub(crate) async fn new() -> Self {
        let pool = init_db("sqlite::memory:")
            .await
            .expect("Failed to initialize test SQLite in-memory database");
        crate::features::init_schemas(&pool)
            .await
            .expect("Failed to run schema migrations in test database");
        crate::features::init_features(&pool)
            .await
            .expect("Failed to seed feature defaults in test database");

        let state = AppState::new(pool);
        let router = Router::new().nest("/api/v1", crate::features::router().with_state(state));

        Self { router }
    }

    /// Creates a fresh in-memory database with migrations but without seeding defaults.
    pub(crate) async fn new_unseeded() -> Self {
        let pool = init_db("sqlite::memory:")
            .await
            .expect("Failed to initialize test SQLite in-memory database");
        crate::features::init_schemas(&pool)
            .await
            .expect("Failed to run schema migrations in test database");

        let state = AppState::new(pool);
        let router = Router::new().nest("/api/v1", crate::features::router().with_state(state));

        Self { router }
    }

    /// Sends a raw HTTP request into the router and returns status, headers, and parsed JSON.
    pub(crate) async fn request(&self, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
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
    pub(crate) async fn get(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .expect("Failed to build GET request");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for GET requests with a session cookie.
    pub(crate) async fn get_with_cookie(&self, uri: &str, cookie: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("Failed to build GET request with cookie");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for POST requests with JSON payload.
    pub(crate) async fn post(&self, uri: &str, body: Value) -> (StatusCode, HeaderMap, Value) {
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
    pub(crate) async fn post_with_cookie(
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
    pub(crate) async fn patch(&self, uri: &str, body: Value) -> (StatusCode, Value) {
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
    pub(crate) async fn patch_with_cookie(
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
    pub(crate) async fn delete(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .body(Body::empty())
            .expect("Failed to build DELETE request");

        let (status, _, json) = self.request(req).await;
        (status, json)
    }

    /// Helper for DELETE requests with session cookie.
    pub(crate) async fn delete_with_cookie(&self, uri: &str, cookie: &str) -> (StatusCode, Value) {
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
    pub(crate) async fn login_as_admin(&self) -> String {
        let (status, headers, body) = self
            .post(
                "/api/v1/auth/register",
                serde_json::json!({
                    "name": "admin_test",
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
}
