use serde::{Serialize, Serializer};

/// Standard numeric status code returned in API envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Zero,
    BadRequest,
    Unauthorized,
    NotFound,
    Conflict,
    InternalError,
}

impl Code {
    pub const fn zero() -> Self {
        Self::Zero
    }

    pub const fn bad_request() -> Self {
        Self::BadRequest
    }

    pub const fn unauthorized() -> Self {
        Self::Unauthorized
    }

    pub const fn not_found() -> Self {
        Self::NotFound
    }

    pub const fn conflict() -> Self {
        Self::Conflict
    }

    pub const fn internal_error() -> Self {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Healthy,
    Ok,
    Custom(&'static str),
}

impl Status {
    pub const fn healthy() -> Self {
        Self::Healthy
    }

    pub const fn ok() -> Self {
        Self::Ok
    }

    pub const fn custom(s: &'static str) -> Self {
        Self::Custom(s)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Ok => "OK",
            Self::Custom(s) => s,
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Structured error payload returned in API error responses.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    action: &'static str,
    message: String,
}

impl ErrorPayload {
    pub fn new(action: &'static str, message: impl Into<String>) -> Self {
        Self {
            action,
            message: message.into(),
        }
    }
}

/// Standard response envelope for all JSON endpoints.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    code: Code,
    status: Status,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(status: Status, data: T) -> Self {
        Self {
            code: Code::zero(),
            status,
            data,
        }
    }

    pub fn err(code: Code, status: Status, data: T) -> Self {
        Self { code, status, data }
    }
}
