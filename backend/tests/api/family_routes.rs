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
    assert!(
        family.is_null(),
        "family starts blank before user creates it"
    );

    let members = body["data"]["members"].as_array().expect("members array");
    assert_eq!(members.len(), 0, "members start blank");

    let accounts = body["data"]["accounts"].as_array().expect("accounts array");
    assert_eq!(accounts.len(), 0, "accounts start blank");

    let currencies = body["data"]["currencies"]
        .as_array()
        .expect("currencies array");
    assert_eq!(
        currencies.len(),
        20,
        "20 currencies loaded from default_currency.json"
    );

    // 2. Fetch default currency (canonical and alias)
    let (curr_status, curr_body) = app.get("/api/v1/config/currency/default").await;
    assert_eq!(curr_status, StatusCode::OK);
    assert_eq!(curr_body["data"]["currency"], "USD");

    let (alias_status, alias_body) = app.get("/api/v1/config/family/currency/default").await;
    assert_eq!(alias_status, StatusCode::OK);
    assert_eq!(alias_body["data"]["currency"], "USD");

    // 3. Test regional resolution
    let (in_status, in_body) = app.get("/api/v1/config/currency/default?region=IN").await;
    assert_eq!(in_status, StatusCode::OK);
    assert_eq!(in_body["data"]["currency"], "INR");

    // 4. Test currencies endpoint (canonical and alias)
    let (currs_status, currs_body) = app.get("/api/v1/config/currencies").await;
    assert_eq!(currs_status, StatusCode::OK);
    assert_eq!(currs_body["data"].as_array().unwrap().len(), 20);

    let (fam_currs_status, fam_currs_body) = app.get("/api/v1/config/family/currencies").await;
    assert_eq!(fam_currs_status, StatusCode::OK);
    assert_eq!(fam_currs_body["data"].as_array().unwrap().len(), 20);
}

#[tokio::test]
async fn test_family_update_and_currency_change() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({
                "familyName": "The Smith Family",
                "currencyId": 2
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["familyName"], "The Smith Family");
    assert_eq!(body["data"]["currencyId"], 2);

    // Overview reflects updated family
    let (ov_status, ov_body) = app.get("/api/v1/config/family").await;
    assert_eq!(ov_status, StatusCode::OK);
    assert_eq!(ov_body["data"]["family"]["familyName"], "The Smith Family");
    assert_eq!(ov_body["data"]["family"]["currencyId"], 2);

    // Default currency now returns EUR
    let (_, curr_body) = app.get("/api/v1/config/currency/default").await;
    assert_eq!(curr_body["data"]["currency"], "EUR");
}

#[tokio::test]
async fn test_member_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 0. Ensure family exists
    app.patch_with_cookie(
        "/api/v1/config/family",
        json!({
            "familyName": "My Family",
            "currencyId": 1
        }),
        &cookie,
    )
    .await;

    // 1. Create member (test singular alias /api/v1/config/member)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/member",
            json!({
                "familyId": 1,
                "memberName": "Lucas Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    let member_id = body["data"]["id"].as_i64().expect("member id");
    assert_eq!(body["data"]["memberName"], "Lucas Miller");

    // 2. Update member (test plural /api/v1/config/members/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/members/{member_id}"),
            json!({
                "memberName": "Lucas J. Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["memberName"], "Lucas J. Miller");

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

    // Ensure family exists
    app.patch_with_cookie(
        "/api/v1/config/family",
        json!({
            "familyName": "My Family",
            "currencyId": 1
        }),
        &cookie,
    )
    .await;

    // Create Sarah member first
    let (_, m_body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    let sarah_id = m_body["data"]["id"].as_i64().expect("sarah id");

    // 1. Create bank account (test singular /api/v1/config/account/bank)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/account/bank",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 4,
                "bankName": "HSBC",
                "accountName": "Premier Savings",
                "last4": "9912",
                "availableBalanceCents": 500000
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["type"], "bank_account");
    assert_eq!(body["data"]["currencyId"], 4);
    assert_eq!(body["data"]["bankName"], "HSBC");
    assert_eq!(body["data"]["accountName"], "Premier Savings");
    assert_eq!(body["data"]["last4"], "9912");
    assert_eq!(body["data"]["availableBalanceCents"], 500000);
    let account_id = body["data"]["id"].as_i64().expect("account id");

    // 2. Update bank account (test plural /api/v1/config/accounts/bank/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/accounts/bank/{account_id}"),
            json!({
                "currencyId": 3,
                "bankName": "HSBC UK",
                "accountName": "Global Savings",
                "last4": "9912",
                "availableBalanceCents": 750000
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["currencyId"], 3);
    assert_eq!(up_body["data"]["bankName"], "HSBC UK");
    assert_eq!(up_body["data"]["accountName"], "Global Savings");
    assert_eq!(up_body["data"]["availableBalanceCents"], 750000);

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

    // Ensure family exists
    app.patch_with_cookie(
        "/api/v1/config/family",
        json!({
            "familyName": "My Family",
            "currencyId": 1
        }),
        &cookie,
    )
    .await;

    // Create Sarah member first
    let (_, m_body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    let sarah_id = m_body["data"]["id"].as_i64().expect("sarah id");

    // 1. Create credit card (test plural /api/v1/config/accounts/credit)
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "American Express",
                "cardName": "Gold Card",
                "last4": "1004",
                "creditLimitCents": 1000000,
                "availableCents": 800000
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["type"], "credit_card");
    assert_eq!(body["data"]["currencyId"], 1);
    assert_eq!(body["data"]["creditLimitCents"], 1000000);
    assert_eq!(body["data"]["availableCents"], 800000);
    assert_eq!(body["data"]["outstandingCents"], 200000);
    let card_id = body["data"]["id"].as_i64().expect("card id");

    // 2. Update credit card (test singular /api/v1/config/account/credit/{id})
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/account/credit/{card_id}"),
            json!({
                "currencyId": 1,
                "bankName": "American Express",
                "cardName": "Platinum Card",
                "last4": "1004",
                "creditLimitCents": 2000000,
                "availableCents": 1500000
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::OK);
    assert_eq!(up_body["data"]["currencyId"], 1);
    assert_eq!(up_body["data"]["cardName"], "Platinum Card");
    assert_eq!(up_body["data"]["creditLimitCents"], 2000000);
    assert_eq!(up_body["data"]["availableCents"], 1500000);
    assert_eq!(up_body["data"]["outstandingCents"], 500000);

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
                "familyName": "   ",
                "currencyId": 1
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["status"], "EMPTY_FAMILY_NAME");

    // 2. Missing mandatory currencyId
    let (curr_status, _) = app
        .patch_with_cookie(
            "/api/v1/config/family",
            json!({
                "familyName": "Valid Family"
            }),
            &cookie,
        )
        .await;
    assert_eq!(curr_status, StatusCode::UNPROCESSABLE_ENTITY);

    // 3. Empty member name
    let (m_status, m_body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": ""
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
                "familyId": 1,
                "ownerMemberId": 1,
                "currencyId": 4,
                "bankName": "Bank",
                "accountName": "Checking",
                "last4": "123",
                "availableBalanceCents": 1000
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
                "familyId": 1,
                "ownerMemberId": 1,
                "currencyId": 4,
                "bankName": "Bank",
                "accountName": "Checking",
                "last4": "1234",
                "availableBalanceCents": -500
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
                "familyId": 1,
                "ownerMemberId": 99999,
                "currencyId": 4,
                "bankName": "Bank",
                "accountName": "Checking",
                "last4": "1234",
                "availableBalanceCents": 1000
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

    // Ensure family exists
    app.patch_with_cookie(
        "/api/v1/config/family",
        json!({
            "familyName": "My Family",
            "currencyId": 1
        }),
        &cookie,
    )
    .await;

    // Create Sarah Miller and David Miller first
    let (_, sarah_res) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    let sarah_id = sarah_res["data"]["id"].as_i64().unwrap();

    let (_, david_res) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "David Miller"
            }),
            &cookie,
        )
        .await;
    let david_id = david_res["data"]["id"].as_i64().unwrap();

    // 1. Member duplicate name conflict in the same family
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["status"], "MEMBER_ALREADY_EXISTS");

    // 2. Member update to an existing name conflict
    let (up_status, up_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/members/{david_id}"),
            json!({
                "memberName": "Sarah Miller"
            }),
            &cookie,
        )
        .await;
    assert_eq!(up_status, StatusCode::CONFLICT);
    assert_eq!(up_body["status"], "MEMBER_ALREADY_EXISTS");

    // Create Sarah's Total Checking bank account
    let _ = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Chase",
                "accountName": "Total Checking",
                "last4": "4821",
                "availableBalanceCents": 1000
            }),
            &cookie,
        )
        .await;

    // 3. Bank account duplicate name conflict for same member (Sarah already has "Total Checking")
    let (acc_status, acc_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Chase",
                "accountName": "Total Checking",
                "last4": "1234",
                "availableBalanceCents": 1000
            }),
            &cookie,
        )
        .await;
    assert_eq!(acc_status, StatusCode::CONFLICT);
    assert_eq!(acc_body["status"], "ACCOUNT_ALREADY_EXISTS");

    // Create Sarah's Sapphire Preferred credit card
    let _ = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Chase",
                "cardName": "Sapphire Preferred",
                "last4": "5561",
                "creditLimitCents": 100000,
                "availableCents": 50000
            }),
            &cookie,
        )
        .await;

    // 4. Credit card duplicate name conflict for same member (Sarah already has "Sapphire Preferred" at Chase)
    let (card_status, card_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Chase",
                "cardName": "Sapphire Preferred",
                "last4": "5555",
                "creditLimitCents": 100000,
                "availableCents": 50000
            }),
            &cookie,
        )
        .await;
    assert_eq!(card_status, StatusCode::CONFLICT);
    assert_eq!(card_body["status"], "ACCOUNT_ALREADY_EXISTS");

    // 5. Creating account with same name at a DIFFERENT bank for the same member succeeds
    let (diff_bank_status, diff_bank_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/bank",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Wells Fargo",
                "accountName": "Total Checking",
                "last4": "7788",
                "availableBalanceCents": 2000
            }),
            &cookie,
        )
        .await;
    assert_eq!(diff_bank_status, StatusCode::CREATED);
    assert_eq!(diff_bank_body["data"]["bankName"], "Wells Fargo");
    assert_eq!(diff_bank_body["data"]["accountName"], "Total Checking");

    // 6. Creating credit card with same name at a DIFFERENT bank for the same member succeeds
    let (diff_card_status, diff_card_body) = app
        .post_with_cookie(
            "/api/v1/config/accounts/credit",
            json!({
                "familyId": 1,
                "ownerMemberId": sarah_id,
                "currencyId": 1,
                "bankName": "Barclays",
                "cardName": "Sapphire Preferred",
                "last4": "3344",
                "creditLimitCents": 200000,
                "availableCents": 100000
            }),
            &cookie,
        )
        .await;
    assert_eq!(diff_card_status, StatusCode::CREATED);
    assert_eq!(diff_card_body["data"]["bankName"], "Barclays");
    assert_eq!(diff_card_body["data"]["cardName"], "Sapphire Preferred");
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
                "familyName": "Hacked Family"
            }),
        )
        .await;
    assert_eq!(p_status, StatusCode::UNAUTHORIZED);
    assert_eq!(p_body["status"], "UNAUTHENTICATED");

    let (m_status, _, _) = app
        .post(
            "/api/v1/config/members",
            json!({
                "familyId": 1,
                "memberName": "Intruder"
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
                "familyId": 1,
                "ownerMemberId": 1,
                "currencyId": 4,
                "bankName": "Bank",
                "accountName": "Checking",
                "last4": "1234",
                "availableBalanceCents": 1000
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
            json!({ "familyName": "Fake" }),
            "cosave_session=forged_token_value",
        )
        .await;
    assert_eq!(forged_status, StatusCode::UNAUTHORIZED);
    assert_eq!(forged_body["status"], "UNAUTHENTICATED");
}
