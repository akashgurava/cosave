use axum::http::StatusCode;
use serde_json::{json, Value};

use super::TestApp;

#[tokio::test]
async fn test_get_hierarchy_and_colors() {
    let app = TestApp::new().await;

    let (status, body) = app.get("/api/v1/config/categories/hierarchy").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");

    let types = body["data"]["types"].as_array().expect("types array");
    let colors = body["data"]["colors"].as_array().expect("colors array");

    assert_eq!(types.len(), 4);
    assert_eq!(colors.len(), 12);

    let total_categories: usize = types
        .iter()
        .map(|t| t["categories"].as_array().map(|c| c.len()).unwrap_or(0))
        .sum();
    assert_eq!(total_categories, 8);

    let total_subcategories: usize = types
        .iter()
        .flat_map(|t| t["categories"].as_array().expect("categories array"))
        .map(|c| c["subcategories"].as_array().map(|s| s.len()).unwrap_or(0))
        .sum();
    assert_eq!(total_subcategories, 14);

    let (status, colors_body) = app.get("/api/v1/config/categories/colors").await;
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
            "/api/v1/config/categories/types",
            json!({
                "name": "Crypto",
                "color_id": 6
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");
    assert_eq!(body["data"]["name"], "Crypto");
    assert_eq!(body["data"]["color"], "#8b5cf6");
    assert_eq!(body["data"]["color_id"], 6);

    let type_id = body["data"]["id"].as_i64().expect("type id integer");

    // 2. Update type color (to Blue #3b82f6 with color_id = 4)
    let (status, update_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/categories/types/{type_id}/color"),
            json!({ "color_id": 4 }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(update_body["code"], 0);
    assert_eq!(update_body["status"], "OK");

    // 3. Verify color persistence across GET hierarchy
    let (status, hierarchy) = app.get("/api/v1/config/categories/hierarchy").await;
    assert_eq!(status, StatusCode::OK);
    let crypto_type = hierarchy["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .find(|t| t["id"] == type_id)
        .expect("created type should exist");
    assert_eq!(crypto_type["name"], "Crypto");
    assert_eq!(crypto_type["color"], "#3b82f6");
    assert_eq!(crypto_type["color_id"], 4);

    // 4. Delete transaction type
    let (status, delete_body) = app
        .delete_with_cookie(
            &format!("/api/v1/config/categories/types/{type_id}"),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(delete_body["code"], 0);
    assert_eq!(delete_body["status"], "OK");

    // 5. Verify deletion in subsequent hierarchy fetch
    let (status, hierarchy_after) = app.get("/api/v1/config/categories/hierarchy").await;
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
            "/api/v1/config/categories",
            json!({
                "type_id": 1,
                "name": "Consulting"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["code"], 0);
    assert_eq!(body["status"], "OK");
    assert_eq!(body["data"]["name"], "Consulting");

    let cat_id = body["data"]["id"].as_i64().expect("category id");

    // 2. Rename category to Advisory Services
    let (status, rename_cat_body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/categories/{cat_id}"),
            json!({ "name": "Advisory Services" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rename_cat_body["code"], 0);
    assert_eq!(rename_cat_body["status"], "OK");

    // Verify renamed category in hierarchy
    let (status, hierarchy) = app.get("/api/v1/config/categories/hierarchy").await;
    assert_eq!(status, StatusCode::OK);
    let category = hierarchy["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .flat_map(|t| t["categories"].as_array().expect("categories array"))
        .find(|c| c["id"] == cat_id)
        .expect("Advisory Services category should exist");
    assert_eq!(category["name"], "Advisory Services");

    // 3. Create subcategory under Advisory Services
    let (status, sub_body) = app
        .post_with_cookie(
            "/api/v1/config/categories/subcategories",
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
            &format!("/api/v1/config/categories/subcategories/{sub_id}"),
            json!({ "name": "Enterprise Architecture" }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patch_body["code"], 0);
    assert_eq!(patch_body["status"], "OK");

    // 5. Verify renamed subcategory in hierarchy
    let (status, hierarchy) = app.get("/api/v1/config/categories/hierarchy").await;
    assert_eq!(status, StatusCode::OK);
    let category = hierarchy["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .flat_map(|t| t["categories"].as_array().expect("categories array"))
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
            &format!("/api/v1/config/categories/subcategories/{sub_id}"),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(del_sub_body["code"], 0);

    // 7. Delete category
    let (status, del_cat_body) = app
        .delete_with_cookie(&format!("/api/v1/config/categories/{cat_id}"), &cookie)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(del_cat_body["code"], 0);

    // 8. Verify category is gone from hierarchy
    let (_, hierarchy_after) = app.get("/api/v1/config/categories/hierarchy").await;
    let exists = hierarchy_after["data"]["types"]
        .as_array()
        .expect("types array")
        .iter()
        .flat_map(|t| t["categories"].as_array().expect("categories array"))
        .any(|c| c["id"] == cat_id);
    assert!(!exists, "deleted category must not exist in hierarchy");
}

#[tokio::test]
async fn test_reset_defaults_restores_hierarchy() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // Fetch hierarchy and delete all types
    let (_, hierarchy) = app.get("/api/v1/config/hierarchy").await;
    let types = hierarchy["data"]["types"].as_array().expect("types array");
    for t in types {
        let id = t["id"].as_i64().expect("type id");
        let (status, del_body) = app
            .delete_with_cookie(&format!("/api/v1/config/categories/types/{id}"), &cookie)
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(del_body["code"], 0);
        assert!(del_body["data"].is_null());
    }

    // Verify hierarchy is cleared
    let (_, cleared) = app.get("/api/v1/config/hierarchy").await;
    assert_eq!(cleared["data"]["types"].as_array().unwrap().len(), 0);

    // Call reset endpoint via canonical POST /api/v1/config/hierarchy/reset
    let (status, reset_body) = app
        .post_with_cookie("/api/v1/config/hierarchy/reset", json!({}), &cookie)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reset_body["code"], 0);
    assert_eq!(reset_body["status"], "OK");
    assert!(
        reset_body["data"].is_null(),
        "reset response data must be null"
    );

    // Verify restored hierarchy via GET /api/v1/config/hierarchy
    let (status, restored) = app.get("/api/v1/config/hierarchy").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["code"], 0);
    assert_eq!(restored["status"], "OK");
    assert_eq!(restored["data"]["types"].as_array().unwrap().len(), 4);
    let total_categories: usize = restored["data"]["types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["categories"].as_array().unwrap().len())
        .sum();
    assert_eq!(total_categories, 8);
    assert_eq!(restored["data"]["colors"].as_array().unwrap().len(), 12);

    // Verify plural alias /api/v1/config/hierarchies/reset
    let (status_alias, reset_alias_body) = app
        .post_with_cookie("/api/v1/config/hierarchies/reset", json!({}), &cookie)
        .await;
    assert_eq!(status_alias, StatusCode::OK);
    assert_eq!(reset_alias_body["code"], 0);
    assert!(reset_alias_body["data"].is_null());

    // Verify plural alias GET /api/v1/config/hierarchies
    let (status_hierarchies, hierarchies_body) = app.get("/api/v1/config/hierarchies").await;
    assert_eq!(status_hierarchies, StatusCode::OK);
    assert_eq!(
        hierarchies_body["data"]["types"].as_array().unwrap().len(),
        4
    );
}

#[tokio::test]
async fn test_category_validation_and_conflict_errors() {
    let app = TestApp::new().await;
    let cookie = app.login_as_admin().await;

    // 1. Duplicate type name "Income" -> 409 Conflict
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({
                "name": "Income",
                "color_id": 1
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
            "/api/v1/config/categories",
            json!({
                "type_id": 2,
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
        .contains("Category 'Housing' already exists under this type."));

    // 3. Empty type name -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({
                "name": "   ",
                "color_id": 1
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
            "/api/v1/config/categories",
            json!({
                "type_id": 2,
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
            "/api/v1/config/categories",
            json!({
                "type_id": 999999,
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

    // 6. Non-existent color ID on type creation -> 404 Not Found
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({
                "name": "Forex",
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
        "CONFIG.CATEGORIES.CREATE_TYPE.COLOR_NOT_FOUND"
    );

    // 7. Empty subcategory name -> 400 Bad Request
    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/subcategories",
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
            "/api/v1/config/categories/subcategories",
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

    // 9. Non-existent color ID on type color update -> 404 Not Found
    let (status, body) = app
        .patch_with_cookie(
            "/api/v1/config/categories/types/1/color",
            json!({
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
        "CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.COLOR_NOT_FOUND"
    );

    // 10. Duplicate subcategory name under same category -> 409 Conflict
    let (_, hierarchy) = app.get("/api/v1/config/categories/hierarchy").await;
    let housing_cat = hierarchy["data"]["types"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["categories"].as_array().unwrap().iter())
        .find(|c| c["name"] == "Housing")
        .unwrap()
        .clone();
    let housing_id = housing_cat["id"].as_i64().unwrap();

    let (status, body) = app
        .post_with_cookie(
            "/api/v1/config/categories/subcategories",
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
            "/api/v1/config/categories/types",
            json!({
                "name": "Bonds",
                "color_id": 1,
                "unexpected_extra_field": true
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 12. Missing color_id on type creation -> 422 Unprocessable Entity
    let (status, _) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({
                "name": "NoColorType"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 13. Invalid color_id type (string instead of int) -> 422 Unprocessable Entity
    let (status, _) = app
        .post_with_cookie(
            "/api/v1/config/categories/types",
            json!({
                "name": "InvalidColorType",
                "color_id": "not_an_int"
            }),
            &cookie,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 14. Rename category with whitespace/empty name -> 400 Bad Request
    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/categories/{housing_id}"),
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
            &format!("/api/v1/config/categories/subcategories/{sub_id}"),
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
            "/api/v1/config/categories/types/999999/color",
            json!({ "color_id": 1 }),
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
        .delete_with_cookie("/api/v1/config/categories/types/999999", &cookie)
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
            "/api/v1/config/categories/999999",
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
        .delete_with_cookie("/api/v1/config/categories/999999", &cookie)
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
            "/api/v1/config/categories/subcategories/999999",
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
        .delete_with_cookie("/api/v1/config/categories/subcategories/999999", &cookie)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(body["status"], "SUBCATEGORY_NOT_FOUND");
    assert_eq!(
        body["data"]["action"],
        "CONFIG.CATEGORIES.DELETE_SUBCATEGORY.SUBCATEGORY_NOT_FOUND"
    );

    // 17. Rename category to already existing name under same type -> 409 Conflict
    let food_cat = hierarchy["data"]["types"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["categories"].as_array().unwrap().iter())
        .find(|c| c["name"] == "Food & Dining")
        .unwrap();
    let food_id = food_cat["id"].as_i64().unwrap();

    let (status, body) = app
        .patch_with_cookie(
            &format!("/api/v1/config/categories/{food_id}"),
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
            &format!("/api/v1/config/categories/subcategories/{second_sub_id}"),
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
            "/api/v1/config/categories/types",
            json!({ "name": "Crypto", "color_id": 1 }),
        ),
        (
            "POST",
            "/api/v1/config/categories",
            json!({ "type_id": 1, "name": "Bonus" }),
        ),
        (
            "POST",
            "/api/v1/config/categories/subcategories",
            json!({ "category_id": 1, "name": "Sub" }),
        ),
        (
            "PATCH",
            "/api/v1/config/categories/types/1/color",
            json!({ "color_id": 1 }),
        ),
        (
            "PATCH",
            "/api/v1/config/categories/1",
            json!({ "name": "New Name" }),
        ),
        (
            "PATCH",
            "/api/v1/config/categories/subcategories/1",
            json!({ "name": "New Name" }),
        ),
        ("DELETE", "/api/v1/config/categories/types/1", json!({})),
        ("DELETE", "/api/v1/config/categories/1", json!({})),
        (
            "DELETE",
            "/api/v1/config/categories/subcategories/1",
            json!({}),
        ),
        ("POST", "/api/v1/config/hierarchy/reset", json!({})),
        ("POST", "/api/v1/config/hierarchies/reset", json!({})),
        ("POST", "/api/v1/config/categories/reset", json!({})),
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
            "/api/v1/config/categories/types",
            json!({ "name": "Crypto", "color_id": 1 }),
            "cosave_session=forged_invalid_session_token_12345",
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], 401);
    assert_eq!(body["status"], "UNAUTHENTICATED");
    assert_eq!(body["data"]["action"], "AUTH.EXTRACT_USER.VALIDATE_TOKEN");
}
