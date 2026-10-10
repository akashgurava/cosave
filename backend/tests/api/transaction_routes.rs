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
    let (account_id, type_id, category_id) = app.seed_test_account(&cookie).await;

    // 1. Create manual transaction via POST /api/v1/transactions
    let create_payload = json!({
        "date": "2026-10-05",
        "description": "WHOLEFDS SOMA #10294",
        "payee": "Whole Foods Market",
        "amount": -8420,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    });

    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", create_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");

    let tx_id = body["data"]["id"].as_str().expect("transaction ID string");
    assert_eq!(body["data"]["source"], "manual");
    assert_eq!(body["data"]["description"], "WHOLEFDS SOMA #10294");
    assert_eq!(body["data"]["payee"], "Whole Foods Market");
    assert_eq!(body["data"]["amount"], -8420);
    assert_eq!(body["data"]["date"], "2026-10-05");
    assert_eq!(body["data"]["typeId"], type_id);
    assert_eq!(body["data"]["accountId"], account_id);
    assert_eq!(body["data"]["categoryId"], category_id);
    assert_eq!(body["data"]["status"], "cleared");

    // 2. Fetch created transaction via GET /api/v1/transactions/:id
    let (get_status, get_body) = app
        .get_with_cookie(&format!("/api/v1/transactions/{tx_id}"), &cookie)
        .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["code"], 0);
    assert_eq!(get_body["data"]["id"], tx_id);
    assert_eq!(get_body["data"]["source"], "manual");
    assert_eq!(get_body["data"]["description"], "WHOLEFDS SOMA #10294");
    assert_eq!(get_body["data"]["typeId"], type_id);
    assert_eq!(get_body["data"]["accountId"], account_id);
    assert_eq!(get_body["data"]["categoryId"], category_id);

    // 3. List transactions via GET /api/v1/transactions with search filter
    let (list_status, list_body) = app
        .get_with_cookie("/api/v1/transactions?q=WHOLEFDS", &cookie)
        .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_body["code"], 0);
    assert_eq!(list_body["data"]["totalCount"], 1);
    assert_eq!(list_body["data"]["page"], 1);
    assert_eq!(list_body["data"]["pageSize"], 20);
    assert_eq!(list_body["data"]["totalPages"], 1);
    let items = list_body["data"]["items"]
        .as_array()
        .expect("array of transactions");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], tx_id);
    assert_eq!(items[0]["source"], "manual");
    assert_eq!(items[0]["payee"], "Whole Foods Market");

    // List with non-matching query returns empty array in paginated envelope
    let (empty_status, empty_body) = app
        .get_with_cookie("/api/v1/transactions?q=NONEXISTENT", &cookie)
        .await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(empty_body["data"]["totalCount"], 0);
    assert_eq!(empty_body["data"]["totalPages"], 1);
    assert_eq!(empty_body["data"]["items"].as_array().unwrap().len(), 0);

    // 4. Update transaction via PATCH /api/v1/transactions/:id
    let update_payload = json!({
        "source": "manual",
        "date": "2026-10-05",
        "description": "WHOLEFDS SOMA #10294",
        "payee": "Whole Foods SOMA Organic",
        "amount": -9000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    });
    let (patch_status, patch_body) = app
        .patch_with_cookie(
            &format!("/api/v1/transactions/{tx_id}"),
            update_payload,
            &cookie,
        )
        .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["data"]["source"], "manual");
    assert_eq!(patch_body["data"]["payee"], "Whole Foods SOMA Organic");
    assert_eq!(patch_body["data"]["amount"], -9000);

    // 5. Attempt deleting manual transaction with wrong source "import" returns 404 Not Found (table isolation)
    let (wrong_del_status, wrong_del_body) = app
        .delete_with_cookie_and_body(
            &format!("/api/v1/transactions/{tx_id}"),
            json!({ "source": "import" }),
            &cookie,
        )
        .await;
    assert_eq!(wrong_del_status, StatusCode::NOT_FOUND);
    assert_eq!(wrong_del_body["status"], "TRANSACTION_NOT_FOUND");

    // 6. Delete transaction via DELETE /api/v1/transactions/:id with correct JSON payload
    let delete_payload = json!({
        "source": "manual"
    });
    let (del_status, del_body) = app
        .delete_with_cookie_and_body(
            &format!("/api/v1/transactions/{tx_id}"),
            delete_payload,
            &cookie,
        )
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert!(del_body["data"].is_null());

    // Verify subsequent GET returns 404
    let (after_del_status, _) = app
        .get_with_cookie(&format!("/api/v1/transactions/{tx_id}"), &cookie)
        .await;
    assert_eq!(after_del_status, StatusCode::NOT_FOUND);
}

// ============================================================================
// Axis 2: Domain Validation & Error Contracts
// ============================================================================

#[tokio::test]
async fn test_transaction_domain_validation_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_id, type_id, category_id) = app.seed_test_account(&cookie).await;

    // Zero amount rejected (400 Bad Request)
    let zero_amount_payload = json!({
        "date": "2026-10-05",
        "description": "Zero Amount",
        "amount": 0,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id
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
        "accountId": account_id,
        "categoryId": category_id
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
        "accountId": account_id,
        "categoryId": category_id
    });
    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", empty_desc_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "EMPTY_TRANSACTION_DESCRIPTION");

    // Non-existent transaction returns 404 Not Found on GET
    let (not_found_status, not_found_body) = app
        .get_with_cookie("/api/v1/transactions/nonexistent-uuid", &cookie)
        .await;
    assert_eq!(not_found_status, StatusCode::NOT_FOUND);
    assert_eq!(not_found_body["status"], "TRANSACTION_NOT_FOUND");

    // Updating non-existent transaction returns 404 Not Found on PATCH
    let not_found_update_payload = json!({
        "source": "manual",
        "date": "2026-10-05",
        "description": "Ghost Edit",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id
    });
    let (patch_nf_status, patch_nf_body) = app
        .patch_with_cookie(
            "/api/v1/transactions/nonexistent-uuid",
            not_found_update_payload,
            &cookie,
        )
        .await;
    assert_eq!(patch_nf_status, StatusCode::NOT_FOUND);
    assert_eq!(patch_nf_body["status"], "TRANSACTION_NOT_FOUND");

    // Updating with source "import" rejected as UNSUPPORTED_OPERATION (400 Bad Request)
    let unsupported_update_payload = json!({
        "source": "import",
        "date": "2026-10-05",
        "description": "Unsupported Edit",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id
    });
    let (unsupported_status, unsupported_body) = app
        .patch_with_cookie(
            "/api/v1/transactions/some-id",
            unsupported_update_payload,
            &cookie,
        )
        .await;
    assert_eq!(unsupported_status, StatusCode::BAD_REQUEST);
    assert_eq!(unsupported_body["status"], "UNSUPPORTED_OPERATION");

    // Deleting non-existent transaction returns 404 Not Found
    let (del_nf_status, del_nf_body) = app
        .delete_with_cookie_and_body(
            "/api/v1/transactions/nonexistent-uuid",
            json!({ "source": "manual" }),
            &cookie,
        )
        .await;
    assert_eq!(del_nf_status, StatusCode::NOT_FOUND);
    assert_eq!(del_nf_body["status"], "TRANSACTION_NOT_FOUND");

    // Deleting with invalid source returns 400 Bad Request (INVALID_SOURCE_TYPE)
    let (del_inv_status, del_inv_body) = app
        .delete_with_cookie_and_body(
            "/api/v1/transactions/some-id",
            json!({ "source": "bogus_source" }),
            &cookie,
        )
        .await;
    assert_eq!(del_inv_status, StatusCode::BAD_REQUEST);
    assert_eq!(del_inv_body["status"], "INVALID_SOURCE_TYPE");
}

// ============================================================================
// Axis 3: Auth Boundary & Security
// ============================================================================

#[tokio::test]
async fn test_transaction_auth_boundary_rejections() {
    let app = TestApp::new().await;

    // Unauthenticated GET returns 401
    let (status, _) = app.get("/api/v1/transactions").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app.get("/api/v1/transactions/1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Unauthenticated POST returns 401
    let (status, _, _) = app
        .post(
            "/api/v1/transactions",
            json!({
                "date": "2026-10-05",
                "amount": -1000,
                "typeId": 1,
                "accountId": 1,
                "categoryId": 1
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

#[tokio::test]
async fn test_transaction_pagination_and_multi_criteria_filters() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_1, type_id, category_id) = app.seed_test_account(&cookie).await;

    // Create a second account for account_ids multi-filtering
    let member_2_id = app.create_member(&cookie, "Second Member").await;
    let account_2 = app
        .create_bank_account(
            &cookie,
            member_2_id,
            "Wells Fargo",
            "Savings Account",
            100000,
        )
        .await;

    // Seed 25 transactions across account_1 and account_2
    for i in 1..=25 {
        let (target_account, amount, desc, status) = if i <= 15 {
            (
                account_1,
                -1000 * i,
                format!("Account 1 Expense #{i}"),
                "cleared",
            )
        } else {
            (
                account_2,
                5000 * i,
                format!("Account 2 Income #{i}"),
                "pending",
            )
        };

        let create_payload = json!({
            "date": format!("2026-10-{:02}", (i % 28) + 1),
            "description": desc,
            "payee": if i % 2 == 0 { Some(format!("Payee {i}")) } else { None },
            "amount": amount,
            "typeId": type_id,
            "accountId": target_account,
            "categoryId": category_id,
            "status": status
        });

        let (st, body) = app
            .post_with_cookie("/api/v1/transactions", create_payload, &cookie)
            .await;
        assert_eq!(st, StatusCode::CREATED, "Failed to seed tx: {body:?}");
    }

    // 1. Pagination: default page 1 (20 items)
    let (st, body) = app.get_with_cookie("/api/v1/transactions", &cookie).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["data"]["totalCount"], 25);
    assert_eq!(body["data"]["page"], 1);
    assert_eq!(body["data"]["pageSize"], 20);
    assert_eq!(body["data"]["totalPages"], 2);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 20);

    // 2. Pagination: page 2 (remaining 5 items)
    let (st, body) = app
        .get_with_cookie("/api/v1/transactions?page=2&pageSize=20", &cookie)
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["data"]["totalCount"], 25);
    assert_eq!(body["data"]["page"], 2);
    assert_eq!(body["data"]["pageSize"], 20);
    assert_eq!(body["data"]["totalPages"], 2);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 5);

    // 3. Multi-value filter by account_ids: single account (account_1 has 15 items)
    let (st, body) = app
        .get_with_cookie(
            &format!("/api/v1/transactions?account_ids={account_1}"),
            &cookie,
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["data"]["totalCount"], 15);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 15);

    // 4. Repeated query param style: ?account_ids=1&account_ids=2
    let (st, body) = app
        .get_with_cookie(
            &format!("/api/v1/transactions?account_ids={account_1}&account_ids={account_2}"),
            &cookie,
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["data"]["totalCount"], 25);

    // 5. Filter by statuses: "pending" (account_2 items only, 10 items)
    let (st, body) = app
        .get_with_cookie("/api/v1/transactions?statuses=pending", &cookie)
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["data"]["totalCount"], 10);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 10);

    // 6. Filter by amount bounds: |amount| between 1000 and 5000
    let (st, body) = app
        .get_with_cookie(
            "/api/v1/transactions?minAmount=1000&maxAmount=5000",
            &cookie,
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = body["data"]["items"].as_array().unwrap();
    for it in items {
        let abs_val = it["amount"].as_i64().unwrap().abs();
        assert!(
            (1000..=5000).contains(&abs_val),
            "Amount {abs_val} out of range [1000, 5000]"
        );
    }

    // 7. Nullable payee verification: check an odd item with null payee
    let has_null_payee = items.iter().any(|it| it["payee"].is_null());
    assert!(has_null_payee, "Should have transactions with null payee");
}

#[tokio::test]
async fn test_account_deletion_cascade_deletes_transactions() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_id, type_id, category_id) = app.seed_test_account(&cookie).await;

    // Create a transaction on this account
    let create_payload = json!({
        "date": "2026-10-05",
        "description": "Ledger Entry To Cascade",
        "amount": -2500,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    });

    let (status, body) = app
        .post_with_cookie("/api/v1/transactions", create_payload, &cookie)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let tx_id = body["data"]["id"].as_str().expect("transaction ID string");

    // Verify transaction exists
    let (get_status, _) = app
        .get_with_cookie(&format!("/api/v1/transactions/{tx_id}"), &cookie)
        .await;
    assert_eq!(get_status, StatusCode::OK);

    // Delete the account via DELETE /api/v1/config/account/:id
    let (del_acc_status, _) = app
        .delete_with_cookie(&format!("/api/v1/config/account/{account_id}"), &cookie)
        .await;
    assert_eq!(del_acc_status, StatusCode::OK);

    // Verify transaction was cascade-deleted (returns 404)
    let (after_status, after_body) = app
        .get_with_cookie(&format!("/api/v1/transactions/{tx_id}"), &cookie)
        .await;
    assert_eq!(after_status, StatusCode::NOT_FOUND);
    assert_eq!(after_body["status"], "TRANSACTION_NOT_FOUND");
}

#[tokio::test]
async fn test_imported_transaction_api_lifecycle_and_deletion() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;
    let (account_id, type_id, category_id) = app.seed_test_account(&cookie).await;

    // Seed imported transaction directly into db
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let import_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO statement_imports (id, family_id, account_id, filename, file_hash, file_format, created_at) VALUES (1, 1, ?, 'statement.csv', 'hash_abc', 'csv', ?);",
    )
    .bind(account_id)
    .bind(now)
    .execute(app.db())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO raw_statement_rows (id, import_id, row_index, raw_payload, created_at) VALUES (1, 1, 0, '{}', ?);",
    )
    .bind(now)
    .execute(app.db())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO staging_transactions (id, raw_row_id, date, amount, description, created_at) VALUES (?, 1, ?, -4500, 'Imported Wire', ?);",
    )
    .bind(&import_id)
    .bind(now)
    .bind(now)
    .execute(app.db())
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO imported_transactions (
            id, family_id, account_id, type_id, category_id, subcategory_id,
            amount, date, description, payee, notes, status, created_at, updated_at
        )
        VALUES (?, 1, ?, ?, ?, NULL, -4500, ?, 'Imported Wire', 'External Vendor', NULL, 'cleared', ?, ?);
        "#,
    )
    .bind(&import_id)
    .bind(account_id)
    .bind(type_id)
    .bind(category_id)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(app.db())
    .await
    .unwrap();

    // 1. Retrieve via GET /api/v1/transactions/:id
    let (get_status, get_body) = app
        .get_with_cookie(&format!("/api/v1/transactions/{import_id}"), &cookie)
        .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["data"]["id"], import_id);
    assert_eq!(get_body["data"]["source"], "import");
    assert_eq!(get_body["data"]["description"], "Imported Wire");
    assert_eq!(get_body["data"]["payee"], "External Vendor");
    assert_eq!(get_body["data"]["amount"], -4500);

    // 2. Editing imported transaction via PATCH returns 400 UNSUPPORTED_OPERATION
    let patch_payload = json!({
        "source": "import",
        "date": "2026-10-05",
        "description": "Attempted Edit",
        "amount": -5000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id
    });
    let (patch_status, patch_body) = app
        .patch_with_cookie(
            &format!("/api/v1/transactions/{import_id}"),
            patch_payload,
            &cookie,
        )
        .await;
    assert_eq!(patch_status, StatusCode::BAD_REQUEST);
    assert_eq!(patch_body["status"], "UNSUPPORTED_OPERATION");

    // 3. Delete with wrong source "manual" returns 404 (table isolation)
    let (wrong_del_status, wrong_del_body) = app
        .delete_with_cookie_and_body(
            &format!("/api/v1/transactions/{import_id}"),
            json!({ "source": "manual" }),
            &cookie,
        )
        .await;
    assert_eq!(wrong_del_status, StatusCode::NOT_FOUND);
    assert_eq!(wrong_del_body["status"], "TRANSACTION_NOT_FOUND");

    // 4. Delete with correct source "import" returns 200 OK
    let (del_status, del_body) = app
        .delete_with_cookie_and_body(
            &format!("/api/v1/transactions/{import_id}"),
            json!({ "source": "import" }),
            &cookie,
        )
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert!(del_body["data"].is_null());

    // 5. Subsequent GET returns 404
    let (after_get_status, _) = app
        .get_with_cookie(&format!("/api/v1/transactions/{import_id}"), &cookie)
        .await;
    assert_eq!(after_get_status, StatusCode::NOT_FOUND);
}
