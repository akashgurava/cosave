use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use rand::RngCore;
use time::Duration;

use crate::core::{AppError, AppState};

use super::db;
use super::error::AuthError;
use super::models::User;

pub(crate) const SESSION_COOKIE_NAME: &str = "cosave_session";
pub(crate) const SESSION_DURATION_SECS: i64 = 30 * 24 * 3600; // 30 days

/// Hashes a plaintext password using Argon2id with a cryptographically secure random salt.
pub(crate) fn hash_password(password: &str) -> Result<String, AppError> {
    const ACTION: &str = "AUTH.HASH_PASSWORD";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::ShouldNotBeHappening {
            action: ACTION,
            reason: format!("argon2 hashing failed: {e}"),
        })
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

/// Axum extractor that requires an authenticated user via cookie or Bearer header.
pub(crate) struct AuthUser(User);

impl AuthUser {
    #[cfg(test)]
    pub(crate) fn new(user: User) -> Self {
        Self(user)
    }

    pub(crate) fn user_id(&self) -> &str {
        self.0.id()
    }

    pub(crate) fn into_user(self) -> User {
        self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::ShouldNotBeHappening {
                action: "AUTH.EXTRACT_USER.PARSE_COOKIES",
                reason: "failed parsing cookie jar from request parts".to_string(),
            })?;

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
            return Err(AuthError::Unauthenticated {
                action: "AUTH.EXTRACT_USER",
            }
            .into());
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let user = db::find_user_by_session_token(app_state.db(), &token, now).await?;

        match user {
            Some(u) => Ok(AuthUser(u)),
            None => {
                tracing::debug!("Unauthenticated request: session token invalid or expired");
                Err(AuthError::Unauthenticated {
                    action: "AUTH.EXTRACT_USER.VALIDATE_TOKEN",
                }
                .into())
            }
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
