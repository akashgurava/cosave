use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;

use crate::core::{ApiResponse, AppError, AppState, Status};

use super::db;
use super::models::{LoginRequest, RegisterRequest, UserDto};
use super::security::{
    create_session_cookie, remove_session_cookie, AuthUser, SESSION_COOKIE_NAME,
};

/// Registers a new user. The first registered user is automatically assigned the Admin role.
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>), AppError> {
    let (user, token) = db::register_user(state.db(), payload).await?;
    let cookie = create_session_cookie(token);
    tracing::debug!(
        user_id = %user.id(),
        username = %user.name(),
        role = %user.role().as_str(),
        "AUTH.ROUTE.REGISTER. User registered successfully"
    );
    Ok((
        StatusCode::CREATED,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user))),
    ))
}

/// Authenticates with username and password, issuing a session cookie.
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>), AppError> {
    let (user, token) = db::authenticate_user(state.db(), payload).await?;
    let cookie = create_session_cookie(token);
    tracing::debug!(
        user_id = %user.id(),
        username = %user.name(),
        "AUTH.ROUTE.LOGIN. User authenticated successfully"
    );
    Ok((
        StatusCode::OK,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user))),
    ))
}

/// Logs out the user by clearing the session cookie and deleting the session from SQLite.
async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<()>>>) {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let _ = db::logout(state.db(), cookie.value()).await;
    }
    tracing::debug!("AUTH.ROUTE.LOGOUT. User logged out successfully");
    (
        StatusCode::OK,
        jar.add(remove_session_cookie()),
        Json(ApiResponse::ok(Status::ok(), Some(()))),
    )
}

/// Returns the currently authenticated user's profile.
async fn me(user: AuthUser) -> impl IntoResponse {
    let dto = user.into_user().to_dto();
    tracing::debug!(
        user_id = %dto.id(),
        username = %dto.name(),
        "AUTH.ROUTE.ME. User profile retrieved"
    );
    (StatusCode::OK, Json(ApiResponse::ok(Status::ok(), dto)))
}

/// Builds and returns the `/auth` router.
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[cfg(test)]
mod tests {
    use axum::http::{header, StatusCode};
    use serde_json::json;

    use crate::core::TestApp;

    // =========================================================================
    // Axis 1: Happy Path & User Experience (Registration, Login, Me, Logout)
    // =========================================================================

    #[tokio::test]
    async fn test_register_admin_and_member_lifecycle() {
        let app = TestApp::new_unseeded().await;

        // 1. First registered user automatically becomes Admin
        let (status1, headers1, body1) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "firstuser", "password": "password123" }),
            )
            .await;

        assert_eq!(status1, StatusCode::CREATED);
        assert_eq!(body1["code"], 0);
        assert_eq!(body1["status"], "OK");
        assert_eq!(body1["data"]["name"], "firstuser");
        assert_eq!(body1["data"]["role"], "admin");

        // Verify session cookie header issuance
        let cookie1 = headers1
            .get(header::SET_COOKIE)
            .and_then(|h| h.to_str().ok())
            .expect("Set-Cookie header missing on register");
        assert!(cookie1.contains("cosave_session="));

        // 2. Second registered user receives Member role
        let (status2, _, body2) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "seconduser", "password": "password456" }),
            )
            .await;

        assert_eq!(status2, StatusCode::CREATED);
        assert_eq!(body2["code"], 0);
        assert_eq!(body2["status"], "OK");
        assert_eq!(body2["data"]["name"], "seconduser");
        assert_eq!(body2["data"]["role"], "member");
    }

    #[tokio::test]
    async fn test_login_and_logout_lifecycle() {
        let app = TestApp::new_unseeded().await;

        // Register user
        let _ = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "auth_flow_user", "password": "securepassword123" }),
            )
            .await;

        // Login
        let (login_status, login_headers, login_body) = app
            .post(
                "/api/v1/auth/login",
                json!({ "name": "auth_flow_user", "password": "securepassword123" }),
            )
            .await;

        assert_eq!(login_status, StatusCode::OK);
        assert_eq!(login_body["code"], 0);
        assert_eq!(login_body["status"], "OK");
        assert_eq!(login_body["data"]["name"], "auth_flow_user");

        let raw_cookie = login_headers
            .get(header::SET_COOKIE)
            .and_then(|h| h.to_str().ok())
            .expect("Set-Cookie missing on login");
        let session_cookie = raw_cookie.split(';').next().unwrap_or("").to_string();

        // Verify profile access with cookie
        let (me_status, me_body) = app
            .get_with_cookie("/api/v1/auth/me", &session_cookie)
            .await;
        assert_eq!(me_status, StatusCode::OK);
        assert_eq!(me_body["data"]["name"], "auth_flow_user");

        // Logout
        let (logout_status, logout_headers, logout_body) = app
            .request(
                axum::http::Request::builder()
                    .method(axum::http::Method::POST)
                    .uri("/api/v1/auth/logout")
                    .header(header::COOKIE, &session_cookie)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await;

        assert_eq!(logout_status, StatusCode::OK);
        assert_eq!(logout_body["status"], "OK");

        let logout_cookie = logout_headers
            .get(header::SET_COOKIE)
            .and_then(|h| h.to_str().ok())
            .expect("Set-Cookie missing on logout");
        assert!(logout_cookie.contains("Max-Age=0"));

        // Subsequent /me call with deleted session must fail
        let (post_logout_status, post_logout_body) = app
            .get_with_cookie("/api/v1/auth/me", &session_cookie)
            .await;
        assert_eq!(post_logout_status, StatusCode::UNAUTHORIZED);
        assert_eq!(post_logout_body["status"], "UNAUTHENTICATED");
        assert_eq!(
            post_logout_body["data"]["action"],
            "AUTH.EXTRACT_USER.VALIDATE_TOKEN"
        );
    }

    // =========================================================================
    // Axis 2: Domain Validation & Actionable Error Envelopes
    // =========================================================================

    #[tokio::test]
    async fn test_register_validation_and_conflict_errors() {
        let app = TestApp::new_unseeded().await;

        // 1. Short username (<3 chars)
        let (status_u, _, body_u) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "ab", "password": "password123" }),
            )
            .await;
        assert_eq!(status_u, StatusCode::BAD_REQUEST);
        assert_eq!(body_u["code"], 400);
        assert_eq!(body_u["status"], "INVALID_USERNAME");
        assert_eq!(body_u["data"]["action"], "AUTH.REGISTER.USERNAME_LEN");
        assert!(body_u["data"]["message"]
            .as_str()
            .unwrap()
            .contains("at least 3 characters"));

        // 2. Short password (<6 chars)
        let (status_p, _, body_p) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "validuser", "password": "123" }),
            )
            .await;
        assert_eq!(status_p, StatusCode::BAD_REQUEST);
        assert_eq!(body_p["code"], 400);
        assert_eq!(body_p["status"], "INVALID_PASSWORD");
        assert_eq!(body_p["data"]["action"], "AUTH.REGISTER.PASSWORD_LEN");
        assert!(body_p["data"]["message"]
            .as_str()
            .unwrap()
            .contains("at least 6 characters"));

        // 3. Duplicate username conflict (case-insensitive)
        let (status_reg, _, _) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "alice", "password": "password123" }),
            )
            .await;
        assert_eq!(status_reg, StatusCode::CREATED);

        let (status_dup, _, body_dup) = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "ALICE", "password": "newpassword123" }),
            )
            .await;
        assert_eq!(status_dup, StatusCode::CONFLICT);
        assert_eq!(body_dup["code"], 409);
        assert_eq!(body_dup["status"], "USER_EXISTS");
        assert_eq!(body_dup["data"]["action"], "AUTH.REGISTER.CHECK_EXISTING");
        assert!(body_dup["data"]["message"]
            .as_str()
            .unwrap()
            .contains("already exists"));
    }

    #[tokio::test]
    async fn test_login_failure_diagnostics() {
        let app = TestApp::new_unseeded().await;

        let _ = app
            .post(
                "/api/v1/auth/register",
                json!({ "name": "bob", "password": "password123" }),
            )
            .await;

        // 1. Non-existent username
        let (status_ghost, _, body_ghost) = app
            .post(
                "/api/v1/auth/login",
                json!({ "name": "ghost_user", "password": "password123" }),
            )
            .await;
        assert_eq!(status_ghost, StatusCode::UNAUTHORIZED);
        assert_eq!(body_ghost["code"], 401);
        assert_eq!(body_ghost["status"], "INVALID_CREDENTIALS");
        assert_eq!(body_ghost["data"]["action"], "AUTH.LOGIN.FIND_USER");

        // 2. Wrong password
        let (status_wrong, _, body_wrong) = app
            .post(
                "/api/v1/auth/login",
                json!({ "name": "bob", "password": "incorrect_password" }),
            )
            .await;
        assert_eq!(status_wrong, StatusCode::UNAUTHORIZED);
        assert_eq!(body_wrong["code"], 401);
        assert_eq!(body_wrong["status"], "INVALID_CREDENTIALS");
        assert_eq!(body_wrong["data"]["action"], "AUTH.LOGIN.VERIFY_PASSWORD");

        // 3. Empty username
        let (status_empty, _, body_empty) = app
            .post(
                "/api/v1/auth/login",
                json!({ "name": "", "password": "password123" }),
            )
            .await;
        assert_eq!(status_empty, StatusCode::UNAUTHORIZED);
        assert_eq!(body_empty["code"], 401);
        assert_eq!(body_empty["status"], "INVALID_CREDENTIALS");
        assert_eq!(body_empty["data"]["action"], "AUTH.LOGIN.USERNAME_EMPTY");
    }

    // =========================================================================
    // Axis 3: Auth & Security Boundary
    // =========================================================================

    #[tokio::test]
    async fn test_auth_boundary_rejections() {
        let app = TestApp::new_unseeded().await;

        // 1. Missing session cookie on protected endpoint
        let (status_missing, body_missing) = app.get("/api/v1/auth/me").await;
        assert_eq!(status_missing, StatusCode::UNAUTHORIZED);
        assert_eq!(body_missing["code"], 401);
        assert_eq!(body_missing["status"], "UNAUTHENTICATED");
        assert_eq!(
            body_missing["data"]["action"],
            "AUTH.EXTRACT_USER.MISSING_TOKEN"
        );

        // 2. Invalid session token on protected endpoint
        let (status_invalid, body_invalid) = app
            .get_with_cookie("/api/v1/auth/me", "cosave_session=non_existent_token_12345")
            .await;
        assert_eq!(status_invalid, StatusCode::UNAUTHORIZED);
        assert_eq!(body_invalid["code"], 401);
        assert_eq!(body_invalid["status"], "UNAUTHENTICATED");
        assert_eq!(
            body_invalid["data"]["action"],
            "AUTH.EXTRACT_USER.VALIDATE_TOKEN"
        );
    }
}
