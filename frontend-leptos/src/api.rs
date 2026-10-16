//! Standard REST API response envelopes and client transport models.
//!
//! Mirrors `backend/src/core/response.rs` for deserializing standard wire envelopes.

use serde::Deserialize;

/// Standard generic response envelope for all JSON API endpoints.
///
/// Encapsulates numeric status code, status token, and typed response data payload.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    code: i64,
    status: String,
    data: T,
}

impl<T> ApiResponse<T> {
    /// Returns the numeric response code (`0` for success).
    pub(crate) fn code(&self) -> i64 {
        self.code
    }

    /// Returns the status token (e.g. `"OK"`, `"HEALTHY"`).
    pub(crate) fn status(&self) -> &str {
        &self.status
    }

    /// Returns a reference to the inner response payload.
    pub(crate) fn data(&self) -> &T {
        &self.data
    }

    /// Returns `true` strictly when the response indicates success (`code == 0`).
    pub(crate) fn is_ok(&self) -> bool {
        self.code == 0
    }
}
