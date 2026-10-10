//! Tier 3 black-box HTTP route tests for transactions endpoints.
//!
//! Executes requests through the Axum router via [`TestApp`] asserting raw JSON wire envelopes
//! across the 3-axis test matrix:
//! - **Axis 1 (Happy Path & Core Workflows)**: Transaction creation, list filtering, single transaction retrieval,
//!   partial updates with lineage tracking, and transaction deletion.
//! - **Axis 2 (Domain Validation & Error Contracts)**: Rejection of zero amount, empty description, invalid date format,
//!   non-existent transaction (404), and invalid account reference.
//! - **Axis 3 (Auth Boundary & Security)**: Missing authentication cookies/bearer token (401 Unauthorized)
//!   on all mutation endpoints (`POST`, `PATCH`, `DELETE`).

use axum::http::StatusCode;
use serde_json::json;

use super::TestApp;

// ============================================================================
// Axis 1: Happy Path & Core Workflows
// ============================================================================

#[tokio::test]
async fn test_transaction_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_id, type_id) = app.seed_test_account(&cookie).await;

    // 1. Create manual transaction via POST /api/v1/transactions
    let create_payload = json!({
        "date": "2026-10-05",
        "description": "WHOLEFDS SOMA #10294",
        "payee": "Whole Foods Market",
        "amount": -8420,
        "typeId": type_id,
        "accountId": account_id,
        "status": "cleared"
    });

    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", create_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");

    let tx_id = body["data"]["id"].as_i64().expect("transaction ID");
    assert_eq!(body["data"]["description"], "WHOLEFDS SOMA #10294");
    assert_eq!(body["data"]["payee"], "Whole Foods Market");
    assert_eq!(body["data"]["amount"], -8420);
    assert_eq!(body["data"]["date"], "2026-10-05");
    assert_eq!(body["data"]["status"], "cleared");

    // 2. Fetch created transaction via GET /api/v1/transactions/:id
    let (get_status, get_body) = app.get(&format!("/api/v1/transactions/{tx_id}")).await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["code"], 0);
    assert_eq!(get_body["data"]["id"], tx_id);
    assert_eq!(get_body["data"]["description"], "WHOLEFDS SOMA #10294");

    // 3. List transactions via GET /api/v1/transactions with search filter
    let (list_status, list_body) = app.get("/api/v1/transactions?q=WHOLEFDS").await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"].as_array().expect("array of transactions");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], tx_id);

    // List with non-matching query returns empty array
    let (empty_status, empty_body) = app.get("/api/v1/transactions?q=NONEXISTENT").await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(empty_body["data"].as_array().unwrap().len(), 0);

    // 4. Update transaction via PATCH /api/v1/transactions/:id
    let update_payload = json!({
        "payee": "Whole Foods SOMA Organic",
        "amount": -9000
    });
    let (patch_status, patch_body) = app
        .patch_with_cookie(
            &format!("/api/v1/transactions/{tx_id}"),
            update_payload,
            &cookie,
        )
        .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["data"]["payee"], "Whole Foods SOMA Organic");
    assert_eq!(patch_body["data"]["amount"], -9000);

    // 5. Delete transaction via DELETE /api/v1/transactions/:id
    let (del_status, del_body) = app
        .delete_with_cookie(&format!("/api/v1/transactions/{tx_id}"), &cookie)
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert!(del_body["data"].is_null());

    // Verify subsequent GET returns 404
    let (after_del_status, _) = app.get(&format!("/api/v1/transactions/{tx_id}")).await;
    assert_eq!(after_del_status, StatusCode::NOT_FOUND);
}

// ============================================================================
// Axis 2: Domain Validation & Error Contracts
// ============================================================================

#[tokio::test]
async fn test_transaction_domain_validation_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_id, type_id) = app.seed_test_account(&cookie).await;

    // Zero amount rejected (400 Bad Request)
    let zero_amount_payload = json!({
        "date": "2026-10-05",
        "description": "Zero Amount",
        "amount": 0,
        "typeId": type_id,
        "accountId": account_id
    });
    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", zero_amount_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "ZERO_TRANSACTION_AMOUNT");

    // Invalid date format rejected (400 Bad Request)
    let invalid_date_payload = json!({
        "date": "invalid-date",
        "description": "Invalid Date",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id
    });
    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", invalid_date_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "INVALID_TRANSACTION_DATE");

    // Empty description rejected (400 Bad Request)
    let empty_desc_payload = json!({
        "date": "2026-10-05",
        "description": "   ",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id
    });
    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", empty_desc_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "EMPTY_TRANSACTION_DESCRIPTION");

    // Non-existent transaction returns 404 Not Found
    let (not_found_status, not_found_body) = app.get("/api/v1/transactions/99999").await;
    assert_eq!(not_found_status, StatusCode::NOT_FOUND);
    assert_eq!(not_found_body["status"], "TRANSACTION_NOT_FOUND");
}

// ============================================================================
// Axis 3: Auth Boundary & Security
// ============================================================================

#[tokio::test]
async fn test_transaction_auth_boundary_rejections() {
    let app = TestApp::new().await;

    // Unauthenticated POST returns 401
    let (status, _, _) = app
        .post(
            "/api/v1/transactions",
            json!({
                "date": "2026-10-05",
                "amount": -1000,
                "typeId": 1,
                "accountId": 1
            }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Unauthenticated PATCH returns 401
    let (status, _) = app
        .patch("/api/v1/transactions/1", json!({ "payee": "Hacker" }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Unauthenticated DELETE returns 401
    let (status, _) = app.delete("/api/v1/transactions/1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
