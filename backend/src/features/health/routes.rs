use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::core::{ApiResponse, AppState, Status};

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
    use axum::http::StatusCode;
    use serde_json::json;

    use crate::core::TestApp;

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = TestApp::new().await;
        let (status, body) = app.get("/api/v1/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body,
            json!({
                "code": 0,
                "status": "HEALTHY",
                "data": {}
            })
        );
    }
}
