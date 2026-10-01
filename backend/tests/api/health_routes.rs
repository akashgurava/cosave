use axum::http::StatusCode;
use serde_json::json;

use super::TestApp;

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

#[tokio::test]
async fn test_api_not_found_fallback() {
    let app = TestApp::new().await;
    let (status, body) = app.get("/api/v1/non_existent_endpoint").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        body,
        json!({
            "code": 404,
            "status": "NOT_FOUND",
            "data": {
                "action": "APP.ROUTER.API_NOT_FOUND",
                "message": "The requested API endpoint was not found."
            }
        })
    );
}
