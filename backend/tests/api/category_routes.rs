use axum::http::StatusCode;
use serde_json::{json, Value};

use super::TestApp;

#[tokio::test]
async fn test_get_hierarchy_and_colors() {
    let app = TestApp::new().await;

    let (status, body) = app.get("/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");

    let types = body["data"]["types"].as_array().expect("types array");
    let categories = body["data"]["categories"]
        .as_array()
        .expect("categories array");
    let colors = body["data"]["colors"].as_array().expect("colors array");

    assert_eq!(types.len(), 4);
    assert_eq!(categories.len(), 8);
    assert_eq!(colors.len(), 12);

    let total_subcategories: usize = categories
        .iter()
        .map(|c| c["subcategories"].as_array().map(|s| s.len()).unwrap_or(0))
        .sum();
    assert_eq!(total_subcategories, 14);

    let (status, colors_body) = app.get("/api/v1/categories/colors").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(colors_body["code"], 0);
    assert_eq!(colors_body["status"], "OK");
    assert_eq!(
        colors_body["data"].as_array().expect("colors array").len(),
        12
    );
}

#[tokio::test]
async fn test_type_crud_lifecycle() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Create a new transaction type
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "Crypto",
                "color": "#8b5cf6"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");
    assert_eq!(body["data"]["name"], "Crypto");
    assert_eq!(body["data"]["color"], "#8b5cf6");

    let type_id = body["data"]["id"].as_i64().expect("type id integer");

    // 2. Update type color (to Blue #3b82f6)
    let (status, update_body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/types/{type_id}/color"),
            json!({ "color": "#3b82f6" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(update_body["code"], 0);
    assert_eq!(update_body["status"], "OK");

    // 3. Verify color persistence across GET hierarchy
    let (status, hierarchy) = app.get("/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let crypto_type = hierarchy["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .find(|t| t["id"] == type_id)
        .expect("created type should exist");
    assert_eq!(crypto_type["name"], "Crypto");
    assert_eq!(crypto_type["color"], "#3b82f6");

    // 4. Delete transaction type
    let (status, delete_body) = app
        .delete_with_cookie(&format!("/api/v1/categories/types/{type_id}"), &cookie)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(delete_body["code"], 0);
    assert_eq!(delete_body["status"], "OK");

    // 5. Verify deletion in subsequent hierarchy fetch
    let (status, hierarchy_after) = app.get("/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let exists = hierarchy_after["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .any(|t| t["id"] == type_id);
    assert!(!exists, "deleted type must not exist in hierarchy");
}

#[tokio::test]
async fn test_category_and_subcategory_crud() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Create category under Income
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories",
            json!({
                "type_name": "Income",
                "name": "Consulting"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");
    assert_eq!(body["data"]["name"], "Consulting");
    assert_eq!(body["data"]["type"], "Income");

    let cat_id = body["data"]["id"].as_i64().expect("category id");

    // 2. Rename category to Advisory Services
    let (status, rename_cat_body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/{cat_id}"),
            json!({ "name": "Advisory Services" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rename_cat_body["code"], 0);
    assert_eq!(rename_cat_body["status"], "OK");

    // Verify renamed category in hierarchy
    let (status, hierarchy) = app.get("/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let category = hierarchy["data"]["categories"]
        .as_array()
        .expect("categories array")
        .iter()
        .find(|c| c["id"] == cat_id)
        .expect("Advisory Services category should exist");
    assert_eq!(category["name"], "Advisory Services");

    // 3. Create subcategory under Advisory Services
    let (status, sub_body) = app
        .post_with_cookie(
            "/api/v1/categories/subcategories",
            json!({
                "category_id": cat_id,
                "name": "Tech Advisory"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(sub_body["code"], 0);
    assert_eq!(sub_body["status"], "OK");
    assert_eq!(sub_body["data"]["name"], "Tech Advisory");

    let sub_id = sub_body["data"]["id"].as_i64().expect("subcategory id");

    // 4. Rename subcategory
    let (status, patch_body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/subcategories/{sub_id}"),
            json!({ "name": "Enterprise Architecture" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patch_body["code"], 0);
    assert_eq!(patch_body["status"], "OK");

    // 5. Verify renamed subcategory in hierarchy
    let (status, hierarchy) = app.get("/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let category = hierarchy["data"]["categories"]
        .as_array()
        .expect("categories array")
        .iter()
        .find(|c| c["id"] == cat_id)
        .expect("Advisory Services category should exist");
    let renamed = category["subcategories"]
        .as_array()
        .expect("subcategories array")
        .iter()
        .find(|s| s["id"] == sub_id)
        .expect("renamed subcategory should exist");
    assert_eq!(renamed["name"], "Enterprise Architecture");

    // 6. Delete subcategory
    let (status, del_sub_body) = app
        .delete_with_cookie(
            &format!("/api/v1/categories/subcategories/{sub_id}"),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(del_sub_body["code"], 0);

    // 7. Delete category
    let (status, del_cat_body) = app
        .delete_with_cookie(&format!("/api/v1/categories/{cat_id}"), &cookie)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(del_cat_body["code"], 0);

    // 8. Verify category is gone from hierarchy
    let (_, hierarchy_after) = app.get("/api/v1/categories").await;
    let exists = hierarchy_after["data"]["categories"]
        .as_array()
        .expect("categories array")
        .iter()
        .any(|c| c["id"] == cat_id);
    assert!(!exists, "deleted category must not exist in hierarchy");
}

#[tokio::test]
async fn test_reset_defaults_restores_hierarchy() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // Fetch hierarchy and delete all types
    let (_, hierarchy) = app.get("/api/v1/categories").await;
    let types = hierarchy["data"]["types"].as_array().expect("types array");
    for t in types {
        let id = t["id"].as_i64().expect("type id");
        let (status, _) = app
            .delete_with_cookie(&format!("/api/v1/categories/types/{id}"), &cookie)
            .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Verify hierarchy is cleared
    let (_, cleared) = app.get("/api/v1/categories").await;
    assert_eq!(cleared["data"]["types"].as_array().unwrap().len(), 0);

    // Call reset endpoint
    let (status, reset_body) = app
        .post_with_cookie("/api/v1/categories/reset", json!({}), &cookie)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reset_body["code"], 0);
    assert_eq!(reset_body["status"], "OK");
    assert_eq!(reset_body["data"]["types"].as_array().unwrap().len(), 4);
    assert_eq!(
        reset_body["data"]["categories"].as_array().unwrap().len(),
        8
    );
    assert_eq!(reset_body["data"]["colors"].as_array().unwrap().len(), 12);
}

#[tokio::test]
async fn test_category_validation_and_conflict_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Duplicate type name "Income" -> 409 Conflict
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "Income",
                "color": "#10b981"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 409);
    assert_eq!(body["status"], "TYPE_ALREADY_EXISTS");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS"
    );
    assert!(body["data"]["message"]
        .as_str()
        .unwrap()
        .contains("Transaction type 'Income' already exists."));

    // 2. Duplicate category name "Housing" under "Expense" -> 409 Conflict
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories",
            json!({
                "type_name": "Expense",
                "name": "Housing"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 409);
    assert_eq!(body["status"], "CATEGORY_ALREADY_EXISTS");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_CATEGORY.ALREADY_EXISTS"
    );
    assert!(body["data"]["message"]
        .as_str()
        .unwrap()
        .contains("Category 'Housing' already exists under type 'Expense'."));

    // 3. Empty type name -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "   ",
                "color": "#10b981"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_TYPE_NAME");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_TYPE.EMPTY_NAME"
    );

    // 4. Empty category name -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories",
            json!({
                "type_name": "Expense",
                "name": "   "
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_CATEGORY_NAME");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_CATEGORY.EMPTY_NAME"
    );

    // 5. Non-existent parent type -> 404 Not Found
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories",
            json!({
                "type_name": "NonExistentType",
                "name": "Some Category"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "TYPE_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_CATEGORY.TYPE_NOT_FOUND"
    );

    // 6. Unrecognized color -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "Forex",
                "color": "#123456"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "UNRECOGNIZED_COLOR");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.RESOLVE_COLOR.UNRECOGNIZED_COLOR"
    );

    // 7. Empty subcategory name -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/subcategories",
            json!({
                "category_id": 999999,
                "name": "   "
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_SUBCATEGORY_NAME");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.EMPTY_NAME"
    );

    // 8. Non-existent parent category when creating subcategory -> 404 Not Found
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/subcategories",
            json!({
                "category_id": 999999,
                "name": "New Sub"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "CATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.PARENT_NOT_FOUND"
    );

    // 9. Color ID not found in palette -> 404 Not Found
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "Bonds",
                "color_id": 99999
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "COLOR_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.RESOLVE_COLOR.COLOR_ID_NOT_FOUND"
    );

    // 10. Duplicate subcategory name under same category -> 409 Conflict
    let (_, hierarchy) = app.get("/api/v1/categories").await;
    let housing_cat = hierarchy["data"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Housing")
        .unwrap();
    let housing_id = housing_cat["id"].as_i64().unwrap();

    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/subcategories",
            json!({
                "category_id": housing_id,
                "name": "Rent & Mortgage"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 409);
    assert_eq!(body["status"], "SUBCATEGORY_ALREADY_EXISTS");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.ALREADY_EXISTS"
    );
    assert!(body["data"]["message"]
        .as_str()
        .unwrap()
        .contains("Subcategory 'Rent & Mortgage' already exists under this category."));

    // 11. Unknown fields in request body -> rejected (serde deny_unknown_fields)
    let (status, _) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "Bonds",
                "color": "#10b981",
                "unexpected_extra_field": true
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 12. Missing color on type creation -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "NoColorType"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "MISSING_COLOR");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.RESOLVE_COLOR.MISSING_COLOR"
    );

    // 13. Empty color string on type creation -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({
                "name": "EmptyColorType",
                "color": "   "
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_COLOR");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.RESOLVE_COLOR.EMPTY_COLOR"
    );

    // 14. Rename category with whitespace/empty name -> 400 Bad Request
    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/{housing_id}"),
            json!({ "name": "   " }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_CATEGORY_NAME");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.EMPTY_NAME"
    );

    // 15. Rename subcategory with whitespace/empty name -> 400 Bad Request
    let sub_id = housing_cat["subcategories"].as_array().unwrap()[0]["id"]
        .as_i64()
        .unwrap();
    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/subcategories/{sub_id}"),
            json!({ "name": "   " }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 400);
    assert_eq!(body["status"], "EMPTY_SUBCATEGORY_NAME");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.EMPTY_NAME"
    );

    // 16. Non-existent mutations & deletions -> 404 Not Found
    // 16a. Non-existent type update color
    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/categories/types/999999/color",
            json!({ "color": "#10b981" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "TYPE_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.TYPE_NOT_FOUND"
    );

    // 16b. Non-existent type delete
    let (status, body) = app
        .delete_with_cookie("/api/v1/categories/types/999999", &cookie)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "TYPE_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.DELETE_TYPE.TYPE_NOT_FOUND"
    );

    // 16c. Non-existent category rename
    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/categories/999999",
            json!({ "name": "Nonexistent" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "CATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.CATEGORY_NOT_FOUND"
    );

    // 16d. Non-existent category delete
    let (status, body) = app
        .delete_with_cookie("/api/v1/categories/999999", &cookie)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "CATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.DELETE_CATEGORY.CATEGORY_NOT_FOUND"
    );

    // 16e. Non-existent subcategory rename
    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/categories/subcategories/999999",
            json!({ "name": "Nonexistent" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "SUBCATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.SUBCATEGORY_NOT_FOUND"
    );

    // 16f. Non-existent subcategory delete
    let (status, body) = app
        .delete_with_cookie("/api/v1/categories/subcategories/999999", &cookie)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "SUBCATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.DELETE_SUBCATEGORY.SUBCATEGORY_NOT_FOUND"
    );

    // 17. Rename category to already existing name under same type -> 409 Conflict
    let food_cat = hierarchy["data"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Food & Dining")
        .unwrap();
    let food_id = food_cat["id"].as_i64().unwrap();

    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/{food_id}"),
            json!({ "name": "Housing" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 409);
    assert_eq!(body["status"], "CATEGORY_ALREADY_EXISTS");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.ALREADY_EXISTS"
    );

    // 18. Rename subcategory to already existing name under same category -> 409 Conflict
    let subcategories = housing_cat["subcategories"].as_array().unwrap();
    let first_sub_name = subcategories[0]["name"].as_str().unwrap();
    let second_sub_id = subcategories[1]["id"].as_i64().unwrap();

    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/categories/subcategories/{second_sub_id}"),
            json!({ "name": first_sub_name }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 409);
    assert_eq!(body["status"], "SUBCATEGORY_ALREADY_EXISTS");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.ALREADY_EXISTS"
    );
}

#[tokio::test]
async fn test_category_unauthenticated_rejections() {
    let app = TestApp::new().await;

    let endpoints: Vec<(&str, &str, Value)> = vec![
        (
            "POST",
            "/api/v1/categories/types",
            json!({ "name": "Crypto", "color": "#8b5cf6" }),
        ),
        (
            "POST",
            "/api/v1/categories",
            json!({ "type_name": "Income", "name": "Bonus" }),
        ),
        (
            "POST",
            "/api/v1/categories/subcategories",
            json!({ "category_id": 1, "name": "Sub" }),
        ),
        (
            "PATCH",
            "/api/v1/categories/types/1/color",
            json!({ "color": "#8b5cf6" }),
        ),
        (
            "PATCH",
            "/api/v1/categories/1",
            json!({ "name": "New Name" }),
        ),
        (
            "PATCH",
            "/api/v1/categories/subcategories/1",
            json!({ "name": "New Name" }),
        ),
        ("DELETE", "/api/v1/categories/types/1", json!({})),
        ("DELETE", "/api/v1/categories/1", json!({})),
        ("DELETE", "/api/v1/categories/subcategories/1", json!({})),
        ("POST", "/api/v1/categories/reset", json!({})),
    ];

    for (method, uri, payload) in endpoints {
        let (status, body) = match method {
            "POST" => {
                let (status, _, body) = app.post(uri, payload).await;
                (status, body)
            }
            "PATCH" => app.patch(uri, payload).await,
            "DELETE" => app.delete(uri).await,
            _ => unreachable!(),
        };

        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "endpoint {method} {uri} should require auth"
        );
        assert_eq!(body["code"], 401);
        assert_eq!(body["status"], "UNAUTHENTICATED");
        assert_eq!(body["data"]["action"], "AUTH.EXTRACT_USER.MISSING_TOKEN");
        assert_eq!(
            body["data"]["message"],
            "Authentication required. Please sign in to access this resource."
        );
    }
}

#[tokio::test]
async fn test_category_invalid_session_rejection() {
    let app = TestApp::new().await;

    let (status, body) = app
        .post_with_cookie(
            "/api/v1/categories/types",
            json!({ "name": "Crypto", "color": "#8b5cf6" }),
            "cosave_session=forged_invalid_session_token_12345",
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], 401);
    assert_eq!(body["status"], "UNAUTHENTICATED");
    assert_eq!(body["data"]["action"], "AUTH.EXTRACT_USER.VALIDATE_TOKEN");
}
