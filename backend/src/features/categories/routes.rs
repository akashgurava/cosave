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
    use super::*;

    use crate::core::{init_db, Code};
    use crate::features::auth::User;

    fn test_user() -> AuthUser {
        AuthUser::new(User::new("user-test-1", "testuser", "hash", "admin", 0, 0))
    }

    #[tokio::test]
    async fn test_get_hierarchy_returns_seeded_defaults() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        let res = get_hierarchy(State(state)).await.unwrap();
        assert_eq!(res.0.code(), Code::Zero);
        assert_eq!(res.0.status(), Status::Ok);

        let hierarchy = res.0.into_data();
        assert_eq!(hierarchy.types().len(), 4);
        assert_eq!(hierarchy.categories().len(), 8);
        assert_eq!(hierarchy.colors().len(), 12);

        let total_subs: usize = hierarchy
            .categories()
            .iter()
            .map(|c| c.subcategories().len())
            .sum();
        assert_eq!(total_subs, 14);
    }

    #[tokio::test]
    async fn test_type_crud_lifecycle() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Create type
        let create_req = CreateTypeRequest::new("Crypto", Some("#8b5cf6"), None);
        let (status, res) = create_type(State(state.clone()), test_user(), Json(create_req))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let created_type = res.0.into_data();
        assert_eq!(created_type.name(), "Crypto");
        assert_eq!(created_type.color(), "#8b5cf6");

        // Update color to another valid seeded palette color (Blue #3b82f6)
        let update_req = UpdateTypeColorRequest::new(Some("#3b82f6"), None);
        let res = update_type_color(
            State(state.clone()),
            test_user(),
            Path(created_type.id().to_string()),
            Json(update_req),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status(), Status::Ok);

        // Verify color persistence across subsequent hierarchy fetch
        let hierarchy = get_hierarchy(State(state.clone()))
            .await
            .unwrap()
            .0
            .into_data();
        let crypto_type = hierarchy
            .types()
            .iter()
            .find(|t| t.name() == "Crypto")
            .unwrap();
        assert_eq!(crypto_type.color(), "#3b82f6");

        // Delete type
        let res = delete_type(
            State(state.clone()),
            test_user(),
            Path(created_type.id().to_string()),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_category_and_subcategory_crud() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Create category under Income
        let cat_req = CreateCategoryRequest::new("Income", "Consulting");
        let (status, res) = create_category(State(state.clone()), test_user(), Json(cat_req))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let category = res.0.into_data();
        assert_eq!(category.name(), "Consulting");
        assert_eq!(category.type_name(), "Income");

        // Create subcategory
        let sub_req = CreateSubcategoryRequest::new(category.id(), "Tech Advisory");
        let (sub_status, sub_res) =
            create_subcategory(State(state.clone()), test_user(), Json(sub_req))
                .await
                .unwrap();
        assert_eq!(sub_status, StatusCode::CREATED);
        let subcategory = sub_res.0.into_data();
        assert_eq!(subcategory.name(), "Tech Advisory");

        // Rename subcategory
        let update_sub = UpdateNameRequest::new("Enterprise Architecture");
        let res = update_subcategory(
            State(state.clone()),
            test_user(),
            Path(subcategory.id().to_string()),
            Json(update_sub),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status(), Status::Ok);

        // Delete subcategory
        let res = delete_subcategory(
            State(state.clone()),
            test_user(),
            Path(subcategory.id().to_string()),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status(), Status::Ok);

        // Delete category
        let res = delete_category(
            State(state.clone()),
            test_user(),
            Path(category.id().to_string()),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_conflict_on_duplicate_creation() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Duplicate type name "Income"
        let create_req = CreateTypeRequest::new("Income", Some("#10b981"), None);
        let err = create_type(State(state.clone()), test_user(), Json(create_req))
            .await
            .unwrap_err();
        match err {
            AppError::Category(crate::features::categories::CategoryError::TypeAlreadyExists {
                ..
            }) => {}
            other => panic!("expected CategoryError::TypeAlreadyExists, got {other:?}"),
        }

        // Duplicate category name under Expense: "Housing"
        let cat_req = CreateCategoryRequest::new("Expense", "Housing");
        let cat_err = create_category(State(state.clone()), test_user(), Json(cat_req))
            .await
            .unwrap_err();
        match cat_err {
            AppError::Category(
                crate::features::categories::CategoryError::CategoryAlreadyExists { .. },
            ) => {}
            other => panic!("expected CategoryError::CategoryAlreadyExists, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_reset_defaults_restores_hierarchy() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Delete all types
        let hierarchy = db::fetch_hierarchy(state.db()).await.unwrap();
        for t in hierarchy.types() {
            let _ = delete_type(State(state.clone()), test_user(), Path(t.id().to_string()))
                .await
                .unwrap();
        }

        let cleared = db::fetch_hierarchy(state.db()).await.unwrap();
        assert_eq!(cleared.types().len(), 0);

        // Reset defaults
        let res = reset_defaults(State(state.clone()), test_user())
            .await
            .unwrap();
        assert_eq!(res.0.data().types().len(), 4);
        assert_eq!(res.0.data().categories().len(), 8);
        assert_eq!(res.0.data().colors().len(), 12);
    }
}
