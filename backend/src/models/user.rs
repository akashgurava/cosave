use serde::{Deserialize, Serialize};

/// System role for an authenticated user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Role {
    Admin,
    Member,
}

impl Role {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "admin" => Self::Admin,
            _ => Self::Member,
        }
    }
}

/// Internal user database entity.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: Option<String>,
    pub(crate) password_hash: String,
    pub(crate) role: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

impl User {
    pub(crate) fn role_enum(&self) -> Role {
        Role::from_str(&self.role)
    }

    pub(crate) fn to_dto(&self) -> UserDto {
        UserDto {
            id: self.id.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            role: self.role_enum(),
            created_at: self.created_at,
        }
    }
}

/// Safe public user representation returned in API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UserDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: Option<String>,
    pub(crate) role: Role,
    pub(crate) created_at: i64,
}

/// Active user session entity stored in SQLite.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct Session {
    pub(crate) id: String,
    pub(crate) user_id: String,
    pub(crate) expires_at: i64,
    pub(crate) created_at: i64,
}
