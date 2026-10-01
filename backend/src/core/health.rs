//! System health check endpoint and lightweight tracing router.
//!
//! Provides an infallible liveness probe mounted at `/api/v1/health` (or `/health` directly)
//! for container orchestrators, load balancers, and external uptime monitors.

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use super::{ApiResponse, AppState, Status};

/// Payload data returned by the health check endpoint.
#[derive(Serialize)]
struct HealthData {}

/// Verifies that the HTTP server process is live and responsive.
///
/// # Endpoint Contract
/// - **Method / Path**: `GET /api/v1/health`
/// - **Route Tag**: Core Health & Liveness
///
/// # Security & Access Control
/// - **Auth Requirement**: Public (unauthenticated)
/// - **Role Authorization**: Public
/// - **Resource Scoping**: Global (unscoped)
///
/// # Ingress (Inputs)
/// - **State**: None (infallible, statelessly responds to incoming pings)
/// - **Path Parameters**: None
/// - **Query Parameters**: None
/// - **Headers**: None
/// - **Cookies**: None
/// - **Request Body**: None
///
/// # Egress (Outputs & Side Effects)
/// - **Success Status**: `200 OK`
/// - **Response Cookies**: None
/// - **Response Headers**: `Content-Type: application/json`
/// - **Response Body**: [`ApiResponse<HealthData>`] with status [`Status::healthy`] (`code: 0, status: "HEALTHY", data: {}`)
/// - **Database Mutations**: None (read-only liveness check)
///
/// # Failure Contract (Errors)
/// - Infallible endpoint; always returns HTTP 200 with healthy envelope.
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(Status::healthy(), HealthData {})),
    )
}

/// Builds the `/health` route with lightweight request tracing.
///
/// Configures a dedicated [`TraceLayer`] that silences verbose response and end-of-stream
/// events to prevent log pollution from frequent health checks.
pub(crate) fn router() -> Router<AppState> {
    // Only log request entry for health pings to keep logs readable.
    let health_trace = TraceLayer::new_for_http().on_response(()).on_eos(());

    Router::new().route("/health", get(health_check).layer(health_trace))
}
