//! Authentication HTTP route handlers and endpoint routing.
//!
//! Mounts REST endpoints under `/auth` for user registration, login credential verification,
//! session revocation, and current user profile inspection. Handlers parse inbound requests,
//! manage secure HTTP-only cookies alongside bearer tokens, and invoke underlying persistence
//! workflows. Responses are returned in standard API envelopes, returning clear status messages to clients.

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

/// Registers a new user and issues an active session cookie.
///
/// If no users currently exist in the database, the newly created account is automatically
/// assigned the Admin role; subsequent accounts are assigned the Member role.
///
/// # Endpoint Contract
/// - **Method / Path**: `POST /api/v1/auth/register`
/// - **Route Tag**: Auth / Registration
///
/// # Security & Access Control
/// - **Auth Requirement**: Public (unauthenticated)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Global (unscoped)
///
/// # Ingress (Inputs)
/// - **State**: [`AppState`] providing the shared SQLite connection pool
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None required
/// - **Cookies**: Inbound [`CookieJar`] (retains existing cookies and appends session cookie)
/// - **Request Body**: `Json<RegisterRequest>` with strict `deny_unknown_fields` (`username`, `password`)
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: `201 Created`
/// - **Response Cookies**: Sets `cosave_session` cookie (`HttpOnly`, `SameSite=Lax`, `Path=/`, 30-day expiration)
/// - **Response Headers**: `Content-Type: application/json`, `Set-Cookie`
/// - **Response Body**: [`ApiResponse<Option<UserDto>>`] with status [`Status::ok`] (`code: 0, status: "OK", data: UserDto`)
/// - **Database Mutations**: Transactionally inserts a row into `users` and an active session row into `sessions`
///
/// # Failure Contract (Errors)
/// - `400 Bad Request`: [`AuthError::InvalidUsername`] if username fails minimum length (3) or
///   [`AuthError::InvalidPassword`] if password fails minimum length (6).
/// - `409 Conflict`: [`AuthError::UserAlreadyExists`] if the username is already taken.
/// - `500 Internal Server Error`: [`AppError::ShouldNotBeHappening`] on database transaction failure.
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>), AppError> {
    let (user, token) = db::register_user(state.db(), payload).await?;
    let cookie = create_session_cookie(token);
    tracing::debug!(
        user_id = %user.id(),
        username = %user.username(),
        role = %user.role().as_str(),
        "AUTH.ROUTE.REGISTER. User registered successfully"
    );
    Ok((
        StatusCode::CREATED,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user))),
    ))
}

/// Authenticates credentials with username and password, issuing an active session cookie.
///
/// # Endpoint Contract
/// - **Method / Path**: `POST /api/v1/auth/login`
/// - **Route Tag**: Auth / Credential Authentication
///
/// # Security & Access Control
/// - **Auth Requirement**: Public (unauthenticated)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Global (unscoped)
///
/// # Ingress (Inputs)
/// - **State**: [`AppState`] providing the shared SQLite connection pool
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None required
/// - **Cookies**: Inbound [`CookieJar`] (retains existing cookies and appends session cookie)
/// - **Request Body**: `Json<LoginRequest>` with strict `deny_unknown_fields` (`username`, `password`)
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: `200 OK`
/// - **Response Cookies**: Sets `cosave_session` cookie (`HttpOnly`, `SameSite=Lax`, `Path=/`, 30-day expiration)
/// - **Response Headers**: `Content-Type: application/json`, `Set-Cookie`
/// - **Response Body**: [`ApiResponse<Option<UserDto>>`] with status [`Status::ok`] (`code: 0, status: "OK", data: UserDto`)
/// - **Database Mutations**: Inserts a new active session row into `sessions`
///
/// # Failure Contract (Errors)
/// - `401 Unauthorized`: [`AuthError::InvalidCredentials`] if the username is empty, not found, or the password hash does not match.
/// - `500 Internal Server Error`: [`AppError::ShouldNotBeHappening`] on database query failure.
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>), AppError> {
    let (user, token) = db::authenticate_user(state.db(), payload).await?;
    let cookie = create_session_cookie(token);
    tracing::debug!(
        user_id = %user.id(),
        username = %user.username(),
        "AUTH.ROUTE.LOGIN. User authenticated successfully"
    );
    Ok((
        StatusCode::OK,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user))),
    ))
}

/// Logs out the user by clearing the session cookie and deleting the session from SQLite.
///
/// # Endpoint Contract
/// - **Method / Path**: `POST /api/v1/auth/logout`
/// - **Route Tag**: Auth / Session Revocation
///
/// # Security & Access Control
/// - **Auth Requirement**: Tolerant (operates cleanly whether a valid session cookie exists or not)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Session-scoped
///
/// # Ingress (Inputs)
/// - **State**: [`AppState`] providing the shared SQLite connection pool
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None required
/// - **Cookies**: Inbound [`CookieJar`] checked for `cosave_session` cookie
/// - **Request Body**: None
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: `200 OK`
/// - **Response Cookies**: Overwrites `cosave_session` with an expired tombstone cookie (`Max-Age=0`)
/// - **Response Headers**: `Content-Type: application/json`, `Set-Cookie`
/// - **Response Body**: [`ApiResponse<Option<()>>`] with status [`Status::ok`] (`code: 0, status: "OK", data: null`)
/// - **Database Mutations**: Deletes the matching row from `sessions` table in SQLite
///
/// # Failure Contract (Errors)
/// - Infallible handler; missing or already-expired cookies are ignored and the cookie is unconditionally cleared.
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
///
/// Resolves the user identity via the [`AuthUser`] extractor, which verifies the active session
/// token supplied via either the `cosave_session` cookie or `Authorization: Bearer <token>` header.
///
/// # Endpoint Contract
/// - **Method / Path**: `GET /api/v1/auth/me`
/// - **Route Tag**: Auth / Identity Profile
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated (valid, unexpired session token required)
/// - **Role Authorization**: Member or Admin
/// - **Resource Scoping**: User-scoped (caller's own profile)
///
/// # Ingress (Inputs)
/// - **State**: Implicitly resolved by [`AuthUser`] extractor via [`AppState`]
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: Optional `Authorization: Bearer <token>`
/// - **Cookies**: Optional `cosave_session=<token>`
/// - **Request Body**: None
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: `200 OK`
/// - **Response Cookies**: None
/// - **Response Headers**: `Content-Type: application/json`
/// - **Response Body**: [`ApiResponse<UserDto>`] with status [`Status::ok`] (`code: 0, status: "OK", data: UserDto`)
/// - **Database Mutations**: None (read-only query)
///
/// # Failure Contract (Errors)
/// - `401 Unauthorized`: [`AuthError::Unauthenticated`] if no valid session token is found or the session has expired.
async fn me(user: AuthUser) -> impl IntoResponse {
    let dto = user.into_user().to_dto();
    tracing::debug!(
        user_id = %dto.id(),
        username = %dto.username(),
        "AUTH.ROUTE.ME. User profile retrieved"
    );
    (StatusCode::OK, Json(ApiResponse::ok(Status::ok(), dto)))
}

/// Builds and returns the `/auth` feature router.
///
/// # Mounted Routes
/// - `POST /register`: Registers a new user ([`register`]).
/// - `POST /login`: Authenticates user credentials ([`login`]).
/// - `POST /logout`: Revokes active session ([`logout`]).
/// - `GET /me`: Inspects authenticated profile ([`me`]).
pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}
