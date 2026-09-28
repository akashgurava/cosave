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
