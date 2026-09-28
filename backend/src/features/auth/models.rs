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
pub(crate) struct User {
    id: String,
    username: String,
    password_hash: String,
    role: String,
    created_at: i64,
    #[sqlx(rename = "updated_at")]
    _updated_at: i64,
}

impl User {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub(crate) fn role_enum(&self) -> Role {
        Role::from_str(&self.role)
    }

    pub(crate) fn to_dto(&self) -> UserDto {
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
pub(crate) struct UserDto {
    id: String,
    username: String,
    role: Role,
    created_at: i64,
}

impl UserDto {
    pub(crate) fn new(
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

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn role(&self) -> Role {
        self.role
    }
}

/// Registration request payload.
#[derive(Deserialize)]
pub(crate) struct RegisterRequest {
    username: String,
    password: String,
}

impl RegisterRequest {
    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }
}

/// Login request payload.
#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

impl LoginRequest {
    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }
}
