//! Authentication domain entities, wire Data Transfer Objects (DTOs), and Value Objects.
//!
//! # Three-Tier Model Architecture
//! - **Database Entity (`User`)**: Internal SQLite representation mapped from the `users` table
//!   via `sqlx::FromRow`. Never serialized to API clients.
//! - **Wire DTOs (`RegisterRequest`, `LoginRequest`, `UserDto`)**: Strict JSON boundary representations
//!   with `deny_unknown_fields` on ingress and safe public fields on egress.
//! - **Domain Value Objects (`Username`, `RawPassword`, `Role`)**: Encapsulated newtypes enforcing
//!   "Parse, Don't Validate" domain invariants, boundary lengths, and whitespace trimming.

use serde::{Deserialize, Serialize};

use super::error::AuthError;

/// Authorization role assigned to a user within the system.
///
/// Controls endpoint access boundaries and administrative capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Role {
    /// Full administrative privileges across settings, user management, and configuration.
    Admin,
    /// Standard household member with personal and family-level operational access.
    Member,
}

impl Role {
    /// Returns the static lowercase string representation of the role.
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    /// Parses a database string value into a [`Role`], defaulting to [`Role::Member`] if unrecognized.
    pub(super) fn from_str(s: &str) -> Self {
        match s {
            "admin" => Self::Admin,
            _ => Self::Member,
        }
    }
}

/// Internal database entity representing a row in the `users` table.
///
/// Encapsulates persistent credentials, authorization role, and timestamps.
/// Never serialized directly into wire responses.
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct User {
    id: String,
    username: String,
    password_hash: String,
    role: String,
    created_at: i64,
    #[sqlx(rename = "updated_at")]
    _updated_at: i64,
}

impl User {
    /// Returns the unique user identifier string (`usr-...`).
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    /// Returns the Argon2id password hash for verification.
    pub(super) fn password_hash(&self) -> &str {
        &self.password_hash
    }

    /// Resolves the user's persisted role string into a [`Role`].
    pub(super) fn role_enum(&self) -> Role {
        Role::from_str(&self.role)
    }

    /// Maps the internal database row into a safe, client-facing [`UserDto`].
    pub(super) fn to_dto(&self) -> UserDto {
        UserDto {
            id: self.id.clone(),
            username: self.username.clone(),
            role: self.role_enum(),
            created_at: self.created_at,
        }
    }
}

/// Public presentation DTO returned in API responses.
///
/// Exposes non-sensitive user identity, assigned role, and account creation timestamp.
/// Password hashes and sensitive internal metadata are strictly excluded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct UserDto {
    id: String,
    username: String,
    role: Role,
    created_at: i64,
}

impl UserDto {
    /// Constructs a new [`UserDto`] instance.
    pub(super) fn new(
        id: impl Into<String>,
        username: impl Into<String>,
        role: Role,
        created_at: i64,
    ) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            role,
            created_at,
        }
    }

    /// Returns the unique user identifier string (`usr-...`).
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    /// Returns the user's unique username.
    pub(super) fn username(&self) -> &str {
        &self.username
    }

    /// Returns the user's assigned [`Role`].
    pub(super) fn role(&self) -> Role {
        self.role
    }
}

/// Inbound wire request payload for user registration (`POST /api/v1/auth/register`).
///
/// Derives `Deserialize` with `deny_unknown_fields` to reject unrecognized JSON properties.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegisterRequest {
    username: String,
    password: String,
}

impl RegisterRequest {
    /// Constructs a new [`RegisterRequest`] instance for testing.
    #[cfg(test)]
    pub(super) fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Returns the inbound username string reference.
    pub(super) fn username(&self) -> &str {
        &self.username
    }

    /// Returns the inbound raw password string reference.
    pub(super) fn password(&self) -> &str {
        &self.password
    }
}

/// Inbound wire request payload for credential authentication (`POST /api/v1/auth/login`).
///
/// Derives `Deserialize` with `deny_unknown_fields` to reject unrecognized JSON properties.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

impl LoginRequest {
    /// Returns the inbound username string reference.
    pub(super) fn username(&self) -> &str {
        &self.username
    }

    /// Returns the inbound raw password string reference.
    pub(super) fn password(&self) -> &str {
        &self.password
    }
}

/// Validated username domain Value Object ("Parse, Don't Validate").
///
/// Enforces non-empty string, trimming, and a minimum length constraint of [`Username::MIN_LEN`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Username(String);

impl Username {
    /// Minimum required character length for usernames.
    pub(super) const MIN_LEN: usize = 3;

    /// Validates, trims, and constructs a new [`Username`] Value Object.
    ///
    /// # Errors
    /// Returns [`AuthError::InvalidUsername`] if trimmed length is less than [`MIN_LEN`].
    pub(super) fn try_new(raw: impl Into<String>, action: &'static str) -> Result<Self, AuthError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.len() < Self::MIN_LEN {
            return Err(AuthError::InvalidUsername {
                action,
                username: trimmed,
                min_len: Self::MIN_LEN,
            });
        }
        Ok(Self(trimmed))
    }

    /// Returns the inner trimmed username as a string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the Value Object and returns the owned username string.
    pub(super) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated raw password domain Value Object ("Parse, Don't Validate").
///
/// Enforces a minimum length constraint of [`RawPassword::MIN_LEN`] prior to Argon2id hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawPassword(String);

impl RawPassword {
    /// Minimum required character length for raw passwords.
    pub(super) const MIN_LEN: usize = 6;

    /// Validates and constructs a new [`RawPassword`] Value Object.
    ///
    /// # Errors
    /// Returns [`AuthError::InvalidPassword`] if character length is less than [`MIN_LEN`].
    pub(super) fn try_new(raw: impl Into<String>, action: &'static str) -> Result<Self, AuthError> {
        let raw = raw.into();
        if raw.len() < Self::MIN_LEN {
            return Err(AuthError::InvalidPassword {
                action,
                min_len: Self::MIN_LEN,
            });
        }
        Ok(Self(raw))
    }

    /// Returns the inner raw password as a string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_parsing() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("member"), Role::Member);
        assert_eq!(Role::from_str("arbitrary_unknown_role"), Role::Member);
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Member.as_str(), "member");
    }

    #[test]
    fn test_username_validation() {
        // Valid exact min length (3)
        assert!(Username::try_new("abc", "TEST.USERNAME").is_ok());
        // Trimming surrounding whitespace
        assert_eq!(
            Username::try_new("  bob  ", "TEST.USERNAME")
                .unwrap()
                .as_str(),
            "bob"
        );
        // Length 2 fails boundary check
        let err = Username::try_new("ab", "TEST.USERNAME").unwrap_err();
        assert_eq!(err.code(), "INVALID_USERNAME");
        assert_eq!(err.action(), "TEST.USERNAME");

        // Empty and whitespace-only fail
        assert!(Username::try_new("", "TEST.USERNAME").is_err());
        assert!(Username::try_new("   ", "TEST.USERNAME").is_err());
    }

    #[test]
    fn test_password_validation() {
        // Valid exact min length (6)
        assert!(RawPassword::try_new("123456", "TEST.PASSWORD").is_ok());
        assert!(RawPassword::try_new("password", "TEST.PASSWORD").is_ok());

        // Length 5 fails boundary check
        let err = RawPassword::try_new("12345", "TEST.PASSWORD").unwrap_err();
        assert_eq!(err.code(), "INVALID_PASSWORD");
        assert_eq!(err.action(), "TEST.PASSWORD");

        // Empty fails
        assert!(RawPassword::try_new("", "TEST.PASSWORD").is_err());
    }
}
