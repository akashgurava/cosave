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
    (
        StatusCode::OK,
        jar.add(remove_session_cookie()),
        Json(ApiResponse::ok(Status::ok(), Some(()))),
    )
}

/// Returns the currently authenticated user's profile.
async fn me(user: AuthUser) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(Status::ok(), user.into_user().to_dto())),
    )
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
    use super::*;

    use crate::core::{init_db, Code};

    use super::super::models::Role;
    use super::super::security::SESSION_COOKIE_NAME;

    #[tokio::test]
    async fn test_register_first_user_is_admin_and_second_is_member() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let payload1 = RegisterRequest::new("firstuser", "password123");

        let (status1, jar1, response1) = register(State(state.clone()), jar, Json(payload1))
            .await
            .unwrap();
        assert_eq!(status1, StatusCode::CREATED);
        assert!(jar1.get(SESSION_COOKIE_NAME).is_some());
        assert_eq!(response1.0.code(), Code::Zero);
        assert_eq!(response1.0.status(), Status::Ok);

        let user1 = response1.0.into_data().unwrap();
        assert_eq!(user1.name(), "firstuser");
        assert_eq!(user1.role(), Role::Admin);

        let payload2 = RegisterRequest::new("seconduser", "password456");
        let (status2, _, response2) = register(State(state.clone()), jar1, Json(payload2))
            .await
            .unwrap();
        assert_eq!(status2, StatusCode::CREATED);
        let user2 = response2.0.into_data().unwrap();
        assert_eq!(user2.name(), "seconduser");
        assert_eq!(user2.role(), Role::Member);
    }

    #[tokio::test]
    async fn test_duplicate_username_conflict() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let payload1 = RegisterRequest::new("testuser", "password123");
        let (status1, _, _) = register(State(state.clone()), jar.clone(), Json(payload1))
            .await
            .unwrap();
        assert_eq!(status1, StatusCode::CREATED);

        let payload2 = RegisterRequest::new("TESTUSER", "newpassword");
        let err2 = register(State(state), jar, Json(payload2))
            .await
            .unwrap_err();
        assert_eq!(err2.code(), "USER_EXISTS");
        let res2 = err2.into_response();
        assert_eq!(res2.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_login_credentials_verification() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let reg = RegisterRequest::new("testuser", "secretpassword");
        let _ = register(State(state.clone()), jar.clone(), Json(reg))
            .await
            .unwrap();

        let login_wrong = LoginRequest::new("testuser", "wrong");
        let err_wrong = login(State(state.clone()), jar.clone(), Json(login_wrong))
            .await
            .unwrap_err();
        assert_eq!(err_wrong.code(), "INVALID_CREDENTIALS");
        let res_wrong = err_wrong.into_response();
        assert_eq!(res_wrong.status(), StatusCode::UNAUTHORIZED);

        let login_ok = LoginRequest::new("testuser", "secretpassword");
        let (status_ok, jar_ok, res_ok) = login(State(state), jar, Json(login_ok)).await.unwrap();
        assert_eq!(status_ok, StatusCode::OK);
        assert!(jar_ok.get(SESSION_COOKIE_NAME).is_some());
        assert_eq!(res_ok.0.into_data().unwrap().name(), "testuser");
    }
}
