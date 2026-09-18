use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;

use super::{
    db,
    models::{LoginRequest, RegisterRequest, UserDto},
    security::{create_session_cookie, remove_session_cookie, AuthUser, SESSION_COOKIE_NAME},
};
use crate::core::{
    error::AppError,
    response::{ApiResponse, Code, Status},
    state::AppState,
};

/// Registers a new user. The first registered user is automatically assigned the Admin role.
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>) {
    match db::register_user(&state.db, payload).await {
        Ok((user, token)) => {
            let cookie = create_session_cookie(token);
            (
                StatusCode::CREATED,
                jar.add(cookie),
                Json(ApiResponse::ok(Status::ok(), Some(user))),
            )
        }
        Err(AppError::BadRequest(_)) => (
            StatusCode::BAD_REQUEST,
            jar,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        ),
        Err(AppError::UserExists) => (
            StatusCode::CONFLICT,
            jar,
            Json(ApiResponse::err(
                Code::conflict(),
                Status::user_exists(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, "Registration failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Authenticates with username and password, issuing a session cookie.
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>) {
    match db::authenticate_user(&state.db, payload).await {
        Ok((user, token)) => {
            let cookie = create_session_cookie(token);
            (
                StatusCode::OK,
                jar.add(cookie),
                Json(ApiResponse::ok(Status::ok(), Some(user))),
            )
        }
        Err(AppError::InvalidCredentials) => (
            StatusCode::UNAUTHORIZED,
            jar,
            Json(ApiResponse::err(
                Code::unauthorized(),
                Status::invalid_credentials(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, "Login failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Logs out the user by clearing the session cookie and deleting the session from SQLite.
async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<()>>>) {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let _ = db::logout(&state.db, cookie.value()).await;
    }
    (
        StatusCode::OK,
        jar.remove(remove_session_cookie()),
        Json(ApiResponse::ok(Status::ok(), Some(()))),
    )
}

/// Returns the currently authenticated user's profile.
async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(Status::ok(), user.to_dto())),
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
    use super::super::models::Role;
    use super::super::security::SESSION_COOKIE_NAME;
    use super::*;
    use crate::core::db::init_db;

    #[tokio::test]
    async fn test_register_first_user_is_admin_and_second_is_member() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let payload1 = RegisterRequest {
            name: "firstuser".to_string(),
            password: "password123".to_string(),
        };

        let (status1, jar1, response1) = register(State(state.clone()), jar, Json(payload1)).await;
        assert_eq!(status1, StatusCode::CREATED);
        assert!(jar1.get(SESSION_COOKIE_NAME).is_some());
        assert_eq!(response1.0.code, Code::Zero);
        assert_eq!(response1.0.status, Status::Ok);

        let user1 = response1.0.data.unwrap();
        assert_eq!(user1.name, "firstuser");
        assert_eq!(user1.role, Role::Admin);

        let payload2 = RegisterRequest {
            name: "seconduser".to_string(),
            password: "password456".to_string(),
        };
        let (status2, _, response2) = register(State(state.clone()), jar1, Json(payload2)).await;
        assert_eq!(status2, StatusCode::CREATED);
        let user2 = response2.0.data.unwrap();
        assert_eq!(user2.name, "seconduser");
        assert_eq!(user2.role, Role::Member);
    }

    #[tokio::test]
    async fn test_duplicate_username_conflict() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let payload1 = RegisterRequest {
            name: "testuser".to_string(),
            password: "password123".to_string(),
        };
        let (status1, _, _) = register(State(state.clone()), jar.clone(), Json(payload1)).await;
        assert_eq!(status1, StatusCode::CREATED);

        let payload2 = RegisterRequest {
            name: "TESTUSER".to_string(), // case insensitive
            password: "newpassword".to_string(),
        };
        let (status2, _, res2) = register(State(state), jar, Json(payload2)).await;
        assert_eq!(status2, StatusCode::CONFLICT);
        assert_eq!(res2.0.status, Status::UserExists);
    }

    #[tokio::test]
    async fn test_login_credentials_verification() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);
        let jar = CookieJar::new();

        let reg = RegisterRequest {
            name: "testuser".to_string(),
            password: "secretpassword".to_string(),
        };
        let _ = register(State(state.clone()), jar.clone(), Json(reg)).await;

        let login_wrong = LoginRequest {
            name: "testuser".to_string(),
            password: "wrong".to_string(),
        };
        let (status_wrong, _, res_wrong) =
            login(State(state.clone()), jar.clone(), Json(login_wrong)).await;
        assert_eq!(status_wrong, StatusCode::UNAUTHORIZED);
        assert_eq!(res_wrong.0.status, Status::InvalidCredentials);

        let login_ok = LoginRequest {
            name: "testuser".to_string(),
            password: "secretpassword".to_string(),
        };
        let (status_ok, jar_ok, res_ok) = login(State(state), jar, Json(login_ok)).await;
        assert_eq!(status_ok, StatusCode::OK);
        assert!(jar_ok.get(SESSION_COOKIE_NAME).is_some());
        assert_eq!(res_ok.0.data.unwrap().name, "testuser");
    }
}
