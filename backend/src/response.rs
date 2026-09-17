use serde::{Serialize, Serializer};

/// Standard numeric status code returned in API envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    Zero,
    BadRequest,
    Unauthorized,
    NotFound,
    Conflict,
    InternalError,
}

#[allow(dead_code)]
impl Code {
    pub(crate) const fn zero() -> Self {
        Self::Zero
    }

    pub(crate) const fn bad_request() -> Self {
        Self::BadRequest
    }

    pub(crate) const fn unauthorized() -> Self {
        Self::Unauthorized
    }

    pub(crate) const fn not_found() -> Self {
        Self::NotFound
    }

    pub(crate) const fn conflict() -> Self {
        Self::Conflict
    }

    pub(crate) const fn internal_error() -> Self {
        Self::InternalError
    }

    fn as_i32(&self) -> i32 {
        match self {
            Self::Zero => 0,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::InternalError => 500,
        }
    }
}

impl Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.as_i32())
    }
}

/// Standardized status strings serialized in SCREAMING_SNAKE_CASE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Status {
    Healthy,
    Ok,
    BadRequest,
    Unauthenticated,
    InvalidCredentials,
    UserExists,
    NotFound,
    InternalError,
}

#[allow(dead_code)]
impl Status {
    pub(crate) const fn healthy() -> Self {
        Self::Healthy
    }

    pub(crate) const fn ok() -> Self {
        Self::Ok
    }

    pub(crate) const fn bad_request() -> Self {
        Self::BadRequest
    }

    pub(crate) const fn unauthenticated() -> Self {
        Self::Unauthenticated
    }

    pub(crate) const fn invalid_credentials() -> Self {
        Self::InvalidCredentials
    }

    pub(crate) const fn user_exists() -> Self {
        Self::UserExists
    }

    pub(crate) const fn not_found() -> Self {
        Self::NotFound
    }

    pub(crate) const fn internal_error() -> Self {
        Self::InternalError
    }

    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Ok => "OK",
            Self::BadRequest => "BAD_REQUEST",
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::UserExists => "USER_EXISTS",
            Self::NotFound => "NOT_FOUND",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

/// Standard response envelope for all JSON endpoints.
#[derive(Debug, Serialize)]
pub(crate) struct ApiResponse<T: Serialize> {
    pub(crate) code: Code,
    pub(crate) status: Status,
    pub(crate) data: T,
}

#[allow(dead_code)]
impl<T: Serialize> ApiResponse<T> {
    pub(crate) fn new(code: Code, status: Status, data: T) -> Self {
        Self { code, status, data }
    }

    pub(crate) fn ok(status: Status, data: T) -> Self {
        Self {
            code: Code::zero(),
            status,
            data,
        }
    }

    pub(crate) fn err(code: Code, status: Status, data: T) -> Self {
        Self { code, status, data }
    }
}
