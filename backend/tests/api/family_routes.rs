//! Tier 3 black-box HTTP route tests for family, member, and account endpoints.
//!
//! Executes requests through the Axum router via [`TestApp`] asserting raw JSON wire envelopes
//! across the 3-axis test matrix:
//! - **Axis 1 (Happy Path & Core Workflows)**: Family details, currency resolution, member CRUD,
//!   bank account creation/updates, credit card creation/updates, and account deletion.
//! - **Axis 2 (Domain Validation & Error Contracts)**: Empty names, invalid currencies, invalid last4,
//!   negative balances/limits, missing resources (404), and unique name conflicts (409).
//! - **Axis 3 (Auth Boundary & Security)**: Missing authentication cookies (401 Unauthorized)
//!   on all mutation endpoints.

use axum::http::StatusCode;
use serde_json::json;

use super::TestApp;

// ============================================================================
// Axis 1: Happy Path & Core Workflows
// ============================================================================

#[tokio::test]
async fn test_get_family_details_and_currency_defaults() {
    let app = TestApp::new().await;

    // 1. Fetch details
    let (status, body) = app.get("/api/v1/config/family").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");

    let family = &body["data"]["family"];
    assert_eq!(family["family_name"], "The Miller Family");
    assert_eq!(family["currency"], "INR");

    let members = body["data"]["members"].as_array().expect("members array");
    assert_eq!(members.len(), 3);
    assert_eq!(members[0]["member_name"], "Sarah Miller");

    let accounts = body["data"]["accounts"].as_array().expect("accounts array");
    assert_eq!(accounts.len(), 4);

    // Verify bank account structure
    let bank = accounts
        .iter()
        .find(|a| a["type"] == "bank_account")
        .expect("bank account present");
    assert_eq!(bank["bank_name"], "Chase");
    assert_eq!(bank["account_name"], "Total Checking");
    assert_eq!(bank["last4"], "4821");
    assert_eq!(bank["available_balance_cents"], 845025);

    // Verify credit card structure and computed outstanding balance
    let credit = accounts
        .iter()
        .find(|a| a["type"] == "credit_card")
        .expect("credit card present");
    assert_eq!(credit["bank_name"], "Chase");
    assert_eq!(credit["card_name"], "Sapphire Preferred");
    assert_eq!(credit["last4"], "5561");
    assert_eq!(credit["credit_limit_cents"], 2000000);
    assert_eq!(credit["available_cents"], 1785000);
    assert_eq!(credit["outstanding_cents"], 215000);

    // 2. Fetch default currency (canonical and alias)
    let (curr_status, curr_body) = app.get("/api/v1/config/currency/default").await;
    assert_eq!(curr_status, StatusCode::OK);
    assert_eq!(curr_body["data"]["currency"], "INR");

    let (alias_status, alias_body) = app.get("/api/v1/config/family/currency/default").await;
    assert_eq!(alias_status, StatusCode::OK);
    assert_eq!(alias_body["data"]["currency"], "INR");
}

#[tokio::test]
async fn test_family_update_and_currency_change() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({
                "family_name": "The Smith Family",
                "currency": "EUR"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["family_name"], "The Smith Family");
    assert_eq!(body["data"]["currency"], "EUR");

    // Overview reflects updated family
    let (ov_status, ov_body) = app.get("/api/v1/config/family").await;
    assert_eq!(ov_status, StatusCode::OK);
    assert_eq!(ov_body["data"]["family"]["family_name"], "The Smith Family");
    assert_eq!(ov_body["data"]["family"]["currency"], "EUR");

    // Default currency now returns EUR
    let (_, curr_body) = app.get("/api/v1/config/currency/default").await;
    assert_eq!(curr_body["data"]["currency"], "EUR");
}

#[tokio::test]
async fn test_member_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Create member (test singular alias /api/v1/config/member)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/member",
            json!({
                "family_id": 1,
                "member_name": "Lucas Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    let member_id = body["data"]["id"].as_i64().expect("member id");
    assert_eq!(body["data"]["member_name"], "Lucas Miller");

    // 2. Update member (test plural /api/v1/config/members/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/members/{member_id}"),
            json!({
                "member_name": "Lucas J. Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["member_name"], "Lucas J. Miller");

    // 3. Delete member (test singular /api/v1/config/member/{id})
    let (del_status, del_body) = app
        .delete_with_cookie(&format!("/api/v1/config/member/{member_id}"), &cookie)
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert_eq!(del_body["status"], "OK");

    // 4. Deleting non-existent member returns 404
    let (del_404_status, del_404_body) = app
        .delete_with_cookie(&format!("/api/v1/config/members/{member_id}"), &cookie)
        .await;
    assert_eq!(del_404_status, StatusCode::NOT_FOUND);
    assert_eq!(del_404_body["status"], "MEMBER_NOT_FOUND");
}

#[tokio::test]
async fn test_bank_account_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // Find Sarah's member ID
    let (_, overview) = app.get("/api/v1/config/family").await;
    let sarah_id = overview["data"]["members"][0]["id"]
        .as_i64()
        .expect("sarah id");

    // 1. Create bank account (test singular /api/v1/config/account/bank)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/account/bank",
            json!({
                "family_id": 1,
                "owner_member_id": sarah_id,
                "currency": "INR",
                "bank_name": "HSBC",
                "account_name": "Premier Savings",
                "last4": "9912",
                "available_balance_cents": 500000
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["type"], "bank_account");
    assert_eq!(body["data"]["currency"], "INR");
    assert_eq!(body["data"]["bank_name"], "HSBC");
    assert_eq!(body["data"]["account_name"], "Premier Savings");
    assert_eq!(body["data"]["last4"], "9912");
    assert_eq!(body["data"]["available_balance_cents"], 500000);
    let account_id = body["data"]["id"].as_i64().expect("account id");

    // 2. Update bank account (test plural /api/v1/config/accounts/bank/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/accounts/bank/{account_id}"),
            json!({
                "currency": "GBP",
                "bank_name": "HSBC UK",
                "account_name": "Global Savings",
                "last4": "9912",
                "available_balance_cents": 750000
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["currency"], "GBP");
    assert_eq!(up_body["data"]["bank_name"], "HSBC UK");
    assert_eq!(up_body["data"]["account_name"], "Global Savings");
    assert_eq!(up_body["data"]["available_balance_cents"], 750000);

    // 3. Delete bank account (test singular /api/v1/config/account/{id})
    let (del_status, del_body) = app
        .delete_with_cookie(&format!("/api/v1/config/account/{account_id}"), &cookie)
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert_eq!(del_body["status"], "OK");

    // 4. Deleting non-existent account returns 404
    let (del_404_status, del_404_body) = app
        .delete_with_cookie(&format!("/api/v1/config/accounts/{account_id}"), &cookie)
        .await;
    assert_eq!(del_404_status, StatusCode::NOT_FOUND);
    assert_eq!(del_404_body["status"], "ACCOUNT_NOT_FOUND");
}

#[tokio::test]
async fn test_credit_card_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // Find Sarah's member ID
    let (_, overview) = app.get("/api/v1/config/family").await;
    let sarah_id = overview["data"]["members"][0]["id"]
        .as_i64()
        .expect("sarah id");

    // 1. Create credit card (test plural /api/v1/config/accounts/credit)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "family_id": 1,
                "owner_member_id": sarah_id,
                "currency": "USD",
                "bank_name": "American Express",
                "card_name": "Gold Card",
                "last4": "1004",
                "credit_limit_cents": 1000000,
                "available_cents": 800000
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["type"], "credit_card");
    assert_eq!(body["data"]["currency"], "USD");
    assert_eq!(body["data"]["credit_limit_cents"], 1000000);
    assert_eq!(body["data"]["available_cents"], 800000);
    assert_eq!(body["data"]["outstanding_cents"], 200000);
    let card_id = body["data"]["id"].as_i64().expect("card id");

    // 2. Update credit card (test singular /api/v1/config/account/credit/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/account/credit/{card_id}"),
            json!({
                "currency": "USD",
                "bank_name": "American Express",
                "card_name": "Platinum Card",
                "last4": "1004",
                "credit_limit_cents": 2000000,
                "available_cents": 1500000
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["card_name"], "Platinum Card");
    assert_eq!(up_body["data"]["credit_limit_cents"], 2000000);
    assert_eq!(up_body["data"]["available_cents"], 1500000);
    assert_eq!(up_body["data"]["outstanding_cents"], 500000);

    // 3. Delete credit card (test plural /api/v1/config/accounts/{id})
    let (del_status, del_body) = app
        .delete_with_cookie(&format!("/api/v1/config/accounts/{card_id}"), &cookie)
        .await;
    assert_eq!(del_status, StatusCode::OK);
    assert_eq!(del_body["status"], "OK");
}

// ============================================================================
// Axis 2: Domain Validation & Constraint Errors
// ============================================================================

#[tokio::test]
async fn test_family_domain_validation_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Empty family name
    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({
                "family_name": "   "
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "EMPTY_FAMILY_NAME");

    // 2. Invalid currency code
    let (curr_status, curr_body) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({
                "currency": "US"
            }),
            &cookie,
        )
        .await;
    assert_eq!(curr_status, StatusCode::BAD_REQUEST);
    assert_eq!(curr_body["status"], "INVALID_CURRENCY");

    // 3. Empty member name
    let (m_status, m_body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "family_id": 1,
                "member_name": ""
            }),
            &cookie,
        )
        .await;
    assert_eq!(m_status, StatusCode::BAD_REQUEST);
    assert_eq!(m_body["status"], "EMPTY_MEMBER_NAME");

    // 4. Invalid last 4 digits
    let (acc_status, acc_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "family_id": 1,
                "owner_member_id": 1,
                "currency": "INR",
                "bank_name": "Bank",
                "account_name": "Checking",
                "last4": "123",
                "available_balance_cents": 1000
            }),
            &cookie,
        )
        .await;
    assert_eq!(acc_status, StatusCode::BAD_REQUEST);
    assert_eq!(acc_body["status"], "INVALID_LAST4");

    // 5. Negative monetary amount
    let (neg_status, neg_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "family_id": 1,
                "owner_member_id": 1,
                "currency": "INR",
                "bank_name": "Bank",
                "account_name": "Checking",
                "last4": "1234",
                "available_balance_cents": -500
            }),
            &cookie,
        )
        .await;
    assert_eq!(neg_status, StatusCode::BAD_REQUEST);
    assert_eq!(neg_body["status"], "NEGATIVE_AMOUNT");

    // 6. Owner member not found
    let (own_status, own_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "family_id": 1,
                "owner_member_id": 99999,
                "currency": "INR",
                "bank_name": "Bank",
                "account_name": "Checking",
                "last4": "1234",
                "available_balance_cents": 1000
            }),
            &cookie,
        )
        .await;
    assert_eq!(own_status, StatusCode::NOT_FOUND);
    assert_eq!(own_body["status"], "MEMBER_NOT_FOUND");
}

#[tokio::test]
async fn test_family_conflict_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Member duplicate name conflict in the same family
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "family_id": 1,
                "member_name": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["status"], "MEMBER_ALREADY_EXISTS");

    // 2. Member update to an existing name conflict
    let (_, overview) = app.get("/api/v1/config/family").await;
    let david_id = overview["data"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["member_name"] == "David Miller")
        .unwrap()["id"]
        .as_i64()
        .unwrap();

    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/members/{david_id}"),
            json!({
                "member_name": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::CONFLICT);
    assert_eq!(up_body["status"], "MEMBER_ALREADY_EXISTS");

    // 3. Bank account duplicate name conflict for same member (Sarah already has "Total Checking")
    let sarah_id = overview["data"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["member_name"] == "Sarah Miller")
        .unwrap()["id"]
        .as_i64()
        .unwrap();

    let (acc_status, acc_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "family_id": 1,
                "owner_member_id": sarah_id,
                "currency": "INR",
                "bank_name": "Chase",
                "account_name": "Total Checking",
                "last4": "1234",
                "available_balance_cents": 1000
            }),
            &cookie,
        )
        .await;
    assert_eq!(acc_status, StatusCode::CONFLICT);
    assert_eq!(acc_body["status"], "ACCOUNT_ALREADY_EXISTS");

    // 4. Credit card duplicate name conflict for same member (Sarah already has "Sapphire Preferred")
    let (card_status, card_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "family_id": 1,
                "owner_member_id": sarah_id,
                "currency": "USD",
                "bank_name": "Chase",
                "card_name": "Sapphire Preferred",
                "last4": "5555",
                "credit_limit_cents": 100000,
                "available_cents": 50000
            }),
            &cookie,
        )
        .await;
    assert_eq!(card_status, StatusCode::CONFLICT);
    assert_eq!(card_body["status"], "ACCOUNT_ALREADY_EXISTS");
}

// ============================================================================
// Axis 3: Auth Boundary & Session Enforcement
// ============================================================================

#[tokio::test]
async fn test_family_auth_boundary_rejections() {
    let app = TestApp::new().await;

    // 1. Unauthenticated mutation attempts must return 401 Unauthorized
    let (p_status, p_body) = app
        .patch(
            "/api/v1/config/family",
            json!({
                "family_name": "Hacked Family"
            }),
        )
        .await;
    assert_eq!(p_status, StatusCode::UNAUTHORIZED);
    assert_eq!(p_body["status"], "UNAUTHENTICATED");

    let (m_status, _, _) = app
        .post(
            "/api/v1/config/members",
            json!({
                "family_id": 1,
                "member_name": "Intruder"
            }),
        )
        .await;
    assert_eq!(m_status, StatusCode::UNAUTHORIZED);

    let (del_status, _) = app.delete("/api/v1/config/members/1").await;
    assert_eq!(del_status, StatusCode::UNAUTHORIZED);

    let (b_status, _, _) = app
        .post(
            "/api/v1/config/accounts/bank",
            json!({
                "family_id": 1,
                "owner_member_id": 1,
                "currency": "INR",
                "bank_name": "Bank",
                "account_name": "Checking",
                "last4": "1234",
                "available_balance_cents": 1000
            }),
        )
        .await;
    assert_eq!(b_status, StatusCode::UNAUTHORIZED);

    let (del_acc_status, _) = app.delete("/api/v1/config/accounts/1").await;
    assert_eq!(del_acc_status, StatusCode::UNAUTHORIZED);

    // 2. Forged cookie rejection
    let (forged_status, forged_body) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({ "family_name": "Fake" }),
            "cosave_session=forged_token_value",
        )
        .await;
    assert_eq!(forged_status, StatusCode::UNAUTHORIZED);
    assert_eq!(forged_body["status"], "UNAUTHENTICATED");
}
