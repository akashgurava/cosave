use serde::{Deserialize, Serialize};

use super::error::AuthError;

/// System role for an authenticated user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Role {
    Admin,
    Member,
}

impl Role {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    pub(super) fn from_str(s: &str) -> Self {
        match s {
            "admin" => Self::Admin,
            _ => Self::Member,
        }
    }
}

/// Internal user database entity.
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
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub(super) fn role_enum(&self) -> Role {
        Role::from_str(&self.role)
    }

    pub(super) fn to_dto(&self) -> UserDto {
        UserDto {
            id: self.id.clone(),
            username: self.username.clone(),
            role: self.role_enum(),
            created_at: self.created_at,
        }
    }
}

/// Safe public user representation returned in API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct UserDto {
    id: String,
    username: String,
    role: Role,
    created_at: i64,
}

impl UserDto {
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

    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn username(&self) -> &str {
        &self.username
    }

    pub(super) fn role(&self) -> Role {
        self.role
    }
}

/// Registration request payload.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegisterRequest {
    username: String,
    password: String,
}

impl RegisterRequest {
    pub(super) fn username(&self) -> &str {
        &self.username
    }

    pub(super) fn password(&self) -> &str {
        &self.password
    }
}

/// Login request payload.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

impl LoginRequest {
    pub(super) fn username(&self) -> &str {
        &self.username
    }

    pub(super) fn password(&self) -> &str {
        &self.password
    }
}

/// Validated username value object ("Parse, Don't Validate").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Username(String);

impl Username {
    pub(super) const MIN_LEN: usize = 3;

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

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated raw password value object ("Parse, Don't Validate").
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawPassword(String);

impl RawPassword {
    pub(super) const MIN_LEN: usize = 6;

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

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation() {
        assert!(Username::try_new("alice", "TEST.USERNAME").is_ok());
        assert_eq!(
            Username::try_new("  bob  ", "TEST.USERNAME")
                .unwrap()
                .as_str(),
            "bob"
        );
        assert!(Username::try_new("al", "TEST.USERNAME").is_err());
        assert!(Username::try_new("", "TEST.USERNAME").is_err());
        assert!(Username::try_new("   ", "TEST.USERNAME").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(RawPassword::try_new("123456", "TEST.PASSWORD").is_ok());
        assert!(RawPassword::try_new("password", "TEST.PASSWORD").is_ok());
        assert!(RawPassword::try_new("12345", "TEST.PASSWORD").is_err());
        assert!(RawPassword::try_new("", "TEST.PASSWORD").is_err());
    }
}
