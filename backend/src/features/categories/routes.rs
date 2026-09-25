use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};

use crate::core::{ApiResponse, AppError, AppState, Status};
use crate::features::auth::AuthUser;

use super::db;
use super::models::{
    CategoryHierarchyResponse, CategoryItem, ColorItem, CreateCategoryRequest,
    CreateSubcategoryRequest, CreateTypeRequest, SubcategoryItem, TransactionTypeItem,
    UpdateNameRequest, UpdateTypeColorRequest,
};

/// Retrieves the complete transaction type, category, and subcategory hierarchy.
/// Publicly accessible to allow visitors to view categories and the Sankey graph.
async fn get_hierarchy(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Retrieves all available palette colors from the database.
async fn get_colors(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ColorItem>>>, AppError> {
    let colors = db::fetch_colors(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), colors)))
}

/// Creates a new transaction type.
async fn create_type(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateTypeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TransactionTypeItem>>), AppError> {
    let created = db::create_type(state.db(), payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        type_id = %created.id(),
        type_name = %created.name(),
        "CONFIG.CATEGORIES.ROUTE.CREATE_TYPE. Transaction type created"
    );
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates the color of a transaction type.
async fn update_type_color(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTypeColorRequest>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::update_type_color(state.db(), &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        type_id = %id,
        "CONFIG.CATEGORIES.ROUTE.UPDATE_TYPE_COLOR. Transaction type color updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a transaction type and cascades deletion to categories and subcategories.
async fn delete_type(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_type(state.db(), &id).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        type_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_TYPE. Transaction type deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Creates a new category under a transaction type.
async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryItem>>), AppError> {
    let created = db::create_category(state.db(), payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        category_id = %created.id(),
        category_name = %created.name(),
        "CONFIG.CATEGORIES.ROUTE.CREATE_CATEGORY. Category created"
    );
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates the name of a category.
async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNameRequest>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::update_category_name(state.db(), &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        category_id = %id,
        "CONFIG.CATEGORIES.ROUTE.UPDATE_CATEGORY. Category name updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a category and cascades to its subcategories.
async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_category(state.db(), &id).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        category_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_CATEGORY. Category deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Creates a new subcategory under a category.
async fn create_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSubcategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SubcategoryItem>>), AppError> {
    let created = db::create_subcategory(state.db(), payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        subcategory_id = %created.id(),
        subcategory_name = %created.name(),
        "CONFIG.CATEGORIES.ROUTE.CREATE_SUBCATEGORY. Subcategory created"
    );
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates the name of a subcategory.
async fn update_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNameRequest>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::update_subcategory_name(state.db(), &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        subcategory_id = %id,
        "CONFIG.CATEGORIES.ROUTE.UPDATE_SUBCATEGORY. Subcategory name updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a subcategory.
async fn delete_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_subcategory(state.db(), &id).await?;
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        subcategory_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_SUBCATEGORY. Subcategory deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Resets all categories back to system defaults.
async fn reset_defaults(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    let hierarchy = db::reset_defaults(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        "CONFIG.CATEGORIES.ROUTE.RESET_DEFAULTS. Categories reset to defaults"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_hierarchy).post(create_category))
        .route("/colors", get(get_colors))
        .route("/types", post(create_type))
        .route("/types/{id}", delete(delete_type))
        .route("/types/{id}/color", patch(update_type_color))
        .route("/{id}", patch(update_category).delete(delete_category))
        .route("/subcategories", post(create_subcategory))
        .route(
            "/subcategories/{id}",
            patch(update_subcategory).delete(delete_subcategory),
        )
        .route("/reset", post(reset_defaults))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{json, Value};

    use crate::core::TestApp;

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

        let type_id = body["data"]["id"].as_str().expect("type id string");

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

        let cat_id = body["data"]["id"].as_str().expect("category id");

        // 2. Create subcategory under Consulting
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

        let sub_id = sub_body["data"]["id"].as_str().expect("subcategory id");

        // 3. Rename subcategory
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

        // 4. Verify renamed subcategory in hierarchy
        let (status, hierarchy) = app.get("/api/v1/categories").await;
        assert_eq!(status, StatusCode::OK);
        let category = hierarchy["data"]["categories"]
            .as_array()
            .expect("categories array")
            .iter()
            .find(|c| c["id"] == cat_id)
            .expect("Consulting category should exist");
        let renamed = category["subcategories"]
            .as_array()
            .expect("subcategories array")
            .iter()
            .find(|s| s["id"] == sub_id)
            .expect("renamed subcategory should exist");
        assert_eq!(renamed["name"], "Enterprise Architecture");

        // 5. Delete subcategory
        let (status, del_sub_body) = app
            .delete_with_cookie(
                &format!("/api/v1/categories/subcategories/{sub_id}"),
                &cookie,
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(del_sub_body["code"], 0);

        // 6. Delete category
        let (status, del_cat_body) = app
            .delete_with_cookie(&format!("/api/v1/categories/{cat_id}"), &cookie)
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(del_cat_body["code"], 0);

        // 7. Verify category is gone from hierarchy
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
            let id = t["id"].as_str().expect("type id");
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
                json!({ "category_id": "dummy", "name": "Sub" }),
            ),
            (
                "PATCH",
                "/api/v1/categories/types/dummy-id/color",
                json!({ "color": "#8b5cf6" }),
            ),
            (
                "PATCH",
                "/api/v1/categories/dummy-id",
                json!({ "name": "New Name" }),
            ),
            (
                "PATCH",
                "/api/v1/categories/subcategories/dummy-id",
                json!({ "name": "New Name" }),
            ),
            ("DELETE", "/api/v1/categories/types/dummy-id", json!({})),
            ("DELETE", "/api/v1/categories/dummy-id", json!({})),
            (
                "DELETE",
                "/api/v1/categories/subcategories/dummy-id",
                json!({}),
            ),
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
}
