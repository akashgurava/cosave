use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};
use time::Duration;

use crate::{
    models::user::User,
    response::{ApiResponse, Code, Status},
    state::AppState,
};

pub(crate) const SESSION_COOKIE_NAME: &str = "cosave_session";
pub(crate) const SESSION_DURATION_SECS: i64 = 30 * 24 * 3600; // 30 days

/// Hashes a plaintext password using Argon2id with a cryptographically secure random salt.
pub(crate) fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Verifies a plaintext password against an Argon2 hash string.
pub(crate) fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Generates a cryptographically secure random 256-bit hex session token.
pub(crate) fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Creates an HttpOnly, Lax, Path="/" session cookie with a 30-day lifetime.
pub(crate) fn create_session_cookie(token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(Some(Duration::seconds(SESSION_DURATION_SECS)));
    cookie
}

/// Creates an expired cookie to clear the active session on logout.
pub(crate) fn remove_session_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(Some(Duration::seconds(0)));
    cookie
}

/// Rejection response returned when route authentication fails.
pub(crate) enum AuthRejection {
    Unauthenticated,
    InternalError,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::err(
                    Code::unauthorized(),
                    Status::unauthenticated(),
                    (),
                )),
            )
                .into_response(),
            Self::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    (),
                )),
            )
                .into_response(),
        }
    }
}

/// Axum extractor that requires an authenticated user via cookie or Bearer header.
pub(crate) struct AuthUser(pub(crate) User);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthRejection::InternalError)?;

        let token = jar
            .get(SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .or_else(|| {
                parts
                    .headers
                    .get(AUTHORIZATION)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("Bearer "))
                    .map(|s| s.trim().to_string())
            });

        let Some(token) = token else {
            return Err(AuthRejection::Unauthenticated);
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.name, u.password_hash, u.role, u.created_at, u.updated_at
            FROM users u
            INNER JOIN sessions s ON u.id = s.user_id
            WHERE s.id = ? AND s.expires_at > ?
            "#,
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|_| AuthRejection::InternalError)?;

        match user {
            Some(u) => Ok(AuthUser(u)),
            None => Err(AuthRejection::Unauthenticated),
        }
    }
}

/// Axum extractor that extracts the authenticated user if present, or `None` if anonymous.
#[allow(dead_code)]
pub(crate) struct OptionalAuthUser(pub(crate) Option<User>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(AuthUser(user)) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("Hashing failed");
        assert!(verify_password(password, &hash));
        assert!(!verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_token_generation_length() {
        let token1 = generate_token();
        let token2 = generate_token();
        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);
        assert_ne!(token1, token2);
    }
}
