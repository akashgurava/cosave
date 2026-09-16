use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::response::{ApiResponse, Status};

#[derive(Serialize)]
struct HealthData {
    service: &'static str,
}

/// Basic health check endpoint returning service status.
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(
            Status::healthy(),
            HealthData { service: "cosave" },
        )),
    )
}

/// Builds the `/health` route with lightweight request tracing.
pub(crate) fn router() -> Router {
    // Only log request entry for health pings to keep logs readable.
    let health_trace = TraceLayer::new_for_http().on_response(()).on_eos(());

    Router::new().route("/health", get(health_check).layer(health_trace))
}
