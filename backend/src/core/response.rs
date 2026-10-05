//! Standard REST API response envelopes, error payloads, and status codes.
//!
//! All HTTP endpoints format their responses using the standardized [`ApiResponse`] container,
//! pairing a typed [`Code`] and descriptive [`Status`] with the output payload. Successful requests
//! return data directly, while failures return an [`ErrorPayload`] containing a compile-time action
//! identifier and a clean user-facing message. This predictable wire structure simplifies decoding
//! and consistent error handling across the web frontend.

use axum::{http::StatusCode, Json};
use serde::{Serialize, Serializer};

/// Standard numeric status code returned in API envelopes.
///
/// Serializes to an integer: `0` for success, or the corresponding HTTP status number
/// (e.g. `400`, `401`, `404`, `409`, `500`) for errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    /// Success indicator (`0`).
    Zero,
    /// Client bad request error (`400`).
    BadRequest,
    /// Authentication or session failure (`401`).
    Unauthorized,
    /// Resource not found (`404`).
    NotFound,
    /// State conflict or constraint violation (`409`).
    Conflict,
    /// Internal server error (`500`).
    InternalError,
}

impl Code {
    /// Constructs a success code (`0`).
    pub const fn zero() -> Self {
        Self::Zero
    }

    /// Constructs a bad request code (`400`).
    pub const fn bad_request() -> Self {
        Self::BadRequest
    }

    /// Constructs an unauthorized code (`401`).
    pub const fn unauthorized() -> Self {
        Self::Unauthorized
    }

    /// Constructs a not found code (`404`).
    pub const fn not_found() -> Self {
        Self::NotFound
    }

    /// Constructs a conflict code (`409`).
    pub const fn conflict() -> Self {
        Self::Conflict
    }

    /// Constructs an internal server error code (`500`).
    pub const fn internal_error() -> Self {
        Self::InternalError
    }

    /// Resolves the integer representation for wire serialization.
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
///
/// Serializes to an uppercase string slice (e.g. `"OK"`, `"HEALTHY"`, `"NOT_FOUND"`, or feature error tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Liveness indicator (`"HEALTHY"`).
    Healthy,
    /// General success status (`"OK"`).
    Ok,
    /// Resource not found status (`"NOT_FOUND"`).
    NotFound,
    /// Feature-defined screaming snake_case error token.
    Custom(&'static str),
}

impl Status {
    /// Constructs a healthy status token (`"HEALTHY"`).
    pub const fn healthy() -> Self {
        Self::Healthy
    }

    /// Constructs a standard success status token (`"OK"`).
    pub const fn ok() -> Self {
        Self::Ok
    }

    /// Constructs a not found status token (`"NOT_FOUND"`).
    pub const fn not_found() -> Self {
        Self::NotFound
    }

    /// Constructs a custom screaming snake_case status token.
    pub const fn custom(s: &'static str) -> Self {
        Self::Custom(s)
    }

    /// Returns the static string slice representation.
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

/// Structured error payload returned in the `data` field of API error responses.
///
/// Encapsulates a compile-time action identifier pinpointing the exact failure site
/// and a user-facing explanation message.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    action: &'static str,
    message: String,
}

impl ErrorPayload {
    /// Constructs a new [`ErrorPayload`] with an action token and message.
    pub fn new(action: &'static str, message: impl Into<String>) -> Self {
        Self {
            action,
            message: message.into(),
        }
    }

    /// Returns the compile-time action token pinpointing the failure point.
    pub fn action(&self) -> &'static str {
        self.action
    }

    /// Returns the user-facing error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Consumes the payload, returning the owned error message.
    pub fn into_message(self) -> String {
        self.message
    }
}

/// Standard response envelope for all JSON API endpoints.
///
/// Encapsulates numeric status code, alphanumeric status token, and typed response data.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    code: Code,
    status: Status,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Constructs a success response envelope with `code = 0` and the provided status token and data.
    pub fn ok(status: Status, data: T) -> Self {
        Self {
            code: Code::zero(),
            status,
            data,
        }
    }

    /// Constructs an error response envelope with an explicit error code, status token, and error payload.
    pub fn err(code: Code, status: Status, data: T) -> Self {
        Self { code, status, data }
    }

    /// Returns the numeric status code.
    pub fn code(&self) -> Code {
        self.code
    }

    /// Returns the alphanumeric status token.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Returns a borrowed reference to the inner response data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Consumes the envelope, returning the inner data.
    pub fn into_data(self) -> T {
        self.data
    }

    /// Consumes the envelope, destructuring into its constituent parts: `(code, status, data)`.
    pub fn into_parts(self) -> (Code, Status, T) {
        (self.code, self.status, self.data)
    }
}

/// Canonical fallback route handler for unmatched `/api/v1/*` requests.
///
/// # Endpoint Contract
/// - **Method / Path**: `ANY /api/v1/*` (catch-all fallback)
/// - **Route Tag**: Core Router Fallback
///
/// # Security & Access Control
/// - **Auth Requirement**: Public (unauthenticated)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Global (unscoped)
///
/// # Ingress (Inputs)
/// - **State**: None
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None
/// - **Cookies**: None
/// - **Request Body**: None
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: None (always returns 404)
/// - **Response Cookies**: None
/// - **Response Headers**: `Content-Type: application/json`
/// - **Response Body**: [`ApiResponse<ErrorPayload>`] with action `APP.ROUTER.API_NOT_FOUND`
/// - **Database Mutations**: None
///
/// # Failure Contract (Errors)
/// - `404 NOT_FOUND`: Emits [`Code::not_found`], [`Status::not_found`], and message `"The requested API endpoint was not found."`.
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

/// Canonical fallback route handler for unmatched root requests when running in API-only mode.
///
/// # Endpoint Contract
/// - **Method / Path**: `ANY /*` (root fallback when static SPA hosting is disabled)
/// - **Route Tag**: Core Server Fallback
///
/// # Security & Access Control
/// - **Auth Requirement**: Public (unauthenticated)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Global (unscoped)
///
/// # Ingress (Inputs)
/// - **State**: None
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None
/// - **Cookies**: None
/// - **Request Body**: None
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: None (always returns 404)
/// - **Response Cookies**: None
/// - **Response Headers**: `Content-Type: application/json`
/// - **Response Body**: [`ApiResponse<ErrorPayload>`] with action `APP.ROUTER.API_ONLY_NOT_FOUND`
/// - **Database Mutations**: None
///
/// # Failure Contract (Errors)
/// - `404 NOT_FOUND`: Emits [`Code::not_found`], [`Status::not_found`], and message informing caller of API-only mode.
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
