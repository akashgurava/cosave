use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::core::{
    response::{ApiResponse, Status},
    state::AppState,
};

#[derive(Serialize)]
struct HealthData {}

/// Basic health check endpoint returning service status.
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(Status::healthy(), HealthData {})),
    )
}

/// Builds the `/health` route with lightweight request tracing.
pub(crate) fn router() -> Router<AppState> {
    // Only log request entry for health pings to keep logs readable.
    let health_trace = TraceLayer::new_for_http().on_response(()).on_eos(());

    Router::new().route("/health", get(health_check).layer(health_trace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_health_response_serialization() {
        let resp = ApiResponse::ok(Status::healthy(), HealthData {});
        let serialized = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            serialized,
            json!({
                "code": 0,
                "status": "HEALTHY",
                "data": {}
            })
        );
    }
}
