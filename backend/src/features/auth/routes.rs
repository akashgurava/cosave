use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    db,
    models::{LoginRequest, RegisterRequest, Role, UserDto},
    security::{
        create_session_cookie, generate_token, hash_password, remove_session_cookie,
        verify_password, AuthUser, SESSION_DURATION_SECS,
    },
};
use crate::core::{
    response::{ApiResponse, Code, Status},
    state::AppState,
};

/// Registers a new user. The first registered user is automatically assigned the Admin role.
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>) {
    let name = payload.name.trim().to_string();
    if name.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            jar,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None::<UserDto>,
            )),
        );
    }

    if payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            jar,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None::<UserDto>,
            )),
        );
    }

    match db::find_user_by_name(&state.db, &name).await {
        Ok(Some(_)) => {
            tracing::warn!(name = %name, "Registration conflict: user already exists");
            return (
                StatusCode::CONFLICT,
                jar,
                Json(ApiResponse::err(
                    Code::conflict(),
                    Status::user_exists(),
                    None::<UserDto>,
                )),
            );
        }
        Err(err) => {
            tracing::error!(error = %err, "Database error querying existing user during registration");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None::<UserDto>,
                )),
            );
        }
        Ok(None) => {}
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!(error = %err, "Argon2 password hashing failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None::<UserDto>,
                )),
            );
        }
    };

    // First user is Admin; subsequent users are Members
    let role = match db::count_users(&state.db).await {
        Ok(0) => Role::Admin,
        Ok(_) => Role::Member,
        Err(err) => {
            tracing::error!(error = %err, "Database error counting users for role assignment");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None::<UserDto>,
                )),
            );
        }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let user_id = generate_token();

    if let Err(err) = db::create_user(&state.db, &user_id, &name, &password_hash, role, now).await {
        tracing::error!(error = %err, "Database error inserting new user");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            jar,
            Json(ApiResponse::err(
                Code::internal_error(),
                Status::internal_error(),
                None::<UserDto>,
            )),
        );
    }

    let session_id = generate_token();
    let expires_at = now + SESSION_DURATION_SECS;

    if let Err(err) = db::create_session(&state.db, &session_id, &user_id, expires_at, now).await {
        tracing::error!(error = %err, "Database error creating session for registered user");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            jar,
            Json(ApiResponse::err(
                Code::internal_error(),
                Status::internal_error(),
                None::<UserDto>,
            )),
        );
    }

    tracing::info!(user_id = %user_id, name = %name, role = %role.as_str(), "Registered and logged in new user");

    let cookie = create_session_cookie(session_id);
    let user_dto = UserDto {
        id: user_id,
        name,
        role,
        created_at: now,
    };

    (
        StatusCode::CREATED,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user_dto))),
    )
}

/// Authenticates with username and password, issuing a session cookie.
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, CookieJar, Json<ApiResponse<Option<UserDto>>>) {
    let name = payload.name.trim();

    let user = match db::find_user_by_name(&state.db, name).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!(name = %name, "Login failed: user not found");
            return (
                StatusCode::UNAUTHORIZED,
                jar,
                Json(ApiResponse::err(
                    Code::unauthorized(),
                    Status::invalid_credentials(),
                    None::<UserDto>,
                )),
            );
        }
        Err(err) => {
            tracing::error!(error = %err, "Database error looking up user during login");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                jar,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None::<UserDto>,
                )),
            );
        }
    };

    if !verify_password(&payload.password, &user.password_hash) {
        tracing::warn!(user_id = %user.id, name = %user.name, "Login failed: invalid password");
        return (
            StatusCode::UNAUTHORIZED,
            jar,
            Json(ApiResponse::err(
                Code::unauthorized(),
                Status::invalid_credentials(),
                None::<UserDto>,
            )),
        );
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let session_id = generate_token();
    let expires_at = now + SESSION_DURATION_SECS;

    if let Err(err) = db::create_session(&state.db, &session_id, &user.id, expires_at, now).await {
        tracing::error!(error = %err, "Database error creating session during login");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            jar,
            Json(ApiResponse::err(
                Code::internal_error(),
                Status::internal_error(),
                None::<UserDto>,
            )),
        );
    }

    tracing::info!(user_id = %user.id, name = %user.name, "User logged in successfully");

    let cookie = create_session_cookie(session_id);
    let user_dto = user.to_dto();

    (
        StatusCode::OK,
        jar.add(cookie),
        Json(ApiResponse::ok(Status::ok(), Some(user_dto))),
    )
}

/// Revokes the current session and clears the session cookie.
async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    use super::security::SESSION_COOKIE_NAME;
    if let Some(token) = jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string()) {
        if let Err(err) = db::delete_session(&state.db, &token).await {
            tracing::error!(error = %err, "Database error deleting session on logout");
        } else {
            tracing::info!("Revoked session on logout");
        }
    }

    let remove_cookie = remove_session_cookie();
    (
        StatusCode::OK,
        jar.add(remove_cookie),
        Json(ApiResponse::ok(Status::ok(), ())),
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
