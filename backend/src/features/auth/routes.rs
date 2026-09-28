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
        username = %dto.username(),
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
