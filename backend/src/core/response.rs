use axum::{http::StatusCode, Json};
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
    NotFound,
    Custom(&'static str),
}

impl Status {
    pub const fn healthy() -> Self {
        Self::Healthy
    }

    pub const fn ok() -> Self {
        Self::Ok
    }

    pub const fn not_found() -> Self {
        Self::NotFound
    }

    pub const fn custom(s: &'static str) -> Self {
        Self::Custom(s)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Ok => "OK",
            Self::NotFound => "NOT_FOUND",
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

    pub fn action(&self) -> &'static str {
        self.action
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn into_message(self) -> String {
        self.message
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

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn into_parts(self) -> (Code, Status, T) {
        (self.code, self.status, self.data)
    }
}

/// Canonical fallback handler for unmatched API routes returning standard error envelope.
pub async fn api_not_found() -> (StatusCode, Json<ApiResponse<ErrorPayload>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::err(
            Code::not_found(),
            Status::not_found(),
            ErrorPayload::new(
                "APP.ROUTER.API_NOT_FOUND",
                "The requested API endpoint was not found.",
            ),
        )),
    )
}

/// Canonical fallback handler for unmatched root routes when running in API-only mode.
pub async fn api_only_root_fallback() -> (StatusCode, Json<ApiResponse<ErrorPayload>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::err(
            Code::not_found(),
            Status::not_found(),
            ErrorPayload::new(
                "APP.ROUTER.API_ONLY_NOT_FOUND",
                "Endpoint not found. Note: Server is running in API-only mode.",
            ),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_as_str() {
        assert_eq!(Status::healthy().as_str(), "HEALTHY");
        assert_eq!(Status::ok().as_str(), "OK");
        assert_eq!(Status::not_found().as_str(), "NOT_FOUND");
        assert_eq!(Status::custom("MY_STATUS").as_str(), "MY_STATUS");
    }

    #[test]
    fn test_code_as_i32() {
        assert_eq!(Code::zero().as_i32(), 0);
        assert_eq!(Code::bad_request().as_i32(), 400);
        assert_eq!(Code::unauthorized().as_i32(), 401);
        assert_eq!(Code::not_found().as_i32(), 404);
        assert_eq!(Code::conflict().as_i32(), 409);
        assert_eq!(Code::internal_error().as_i32(), 500);
    }

    #[test]
    fn test_error_payload_accessors() {
        let payload = ErrorPayload::new("MY.ACTION", "Something failed");
        assert_eq!(payload.action(), "MY.ACTION");
        assert_eq!(payload.message(), "Something failed");
        assert_eq!(payload.into_message(), "Something failed");
    }

    #[test]
    fn test_api_response_accessors() {
        let resp = ApiResponse::ok(Status::ok(), "test_data".to_string());
        assert_eq!(resp.code(), Code::zero());
        assert_eq!(resp.status(), Status::ok());
        assert_eq!(resp.data(), "test_data");

        let (code, status, data) = resp.into_parts();
        assert_eq!(code, Code::zero());
        assert_eq!(status, Status::ok());
        assert_eq!(data, "test_data");

        let err_resp = ApiResponse::err(Code::bad_request(), Status::custom("BAD_REQUEST"), 123);
        assert_eq!(err_resp.code(), Code::bad_request());
        assert_eq!(err_resp.into_data(), 123);
    }
}
