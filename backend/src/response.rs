use serde::{Serialize, Serializer};

/// Standard numeric status code returned in API envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    Zero,
}

#[allow(dead_code)]
impl Code {
    pub(crate) const fn zero() -> Self {
        Self::Zero
    }

    fn as_i32(&self) -> i32 {
        match self {
            Self::Zero => 0,
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
    #[allow(dead_code)]
    Ok,
}

#[allow(dead_code)]
impl Status {
    pub(crate) const fn healthy() -> Self {
        Self::Healthy
    }

    pub(crate) const fn ok() -> Self {
        Self::Ok
    }

    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Ok => "OK",
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
}
