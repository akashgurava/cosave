use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};

use crate::{
    core::{
        error::AppError,
        response::{ApiResponse, Status},
        state::AppState,
    },
    features::{
        auth::AuthUser,
        categories::{
            db,
            models::{
                CategoryHierarchyResponse, CategoryItem, CreateCategoryRequest,
                CreateSubcategoryRequest, CreateTypeRequest, SubcategoryItem, TransactionTypeItem,
                UpdateNameRequest, UpdateTypeColorRequest,
            },
        },
    },
};

/// Retrieves the complete transaction type, category, and subcategory hierarchy.
/// Publicly accessible to allow visitors to view categories and the Sankey graph.
async fn get_hierarchy(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Creates a new transaction type.
async fn create_type(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateTypeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TransactionTypeItem>>), AppError> {
    let created = db::create_type(&state.db, payload).await?;
    tracing::info!(
        user_id = %user.0.id,
        type_id = %created.id,
        type_name = %created.name,
        "created transaction type"
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
    db::update_type_color(&state.db, &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, type_id = %id, "updated transaction type color");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a transaction type and cascades deletion to categories and subcategories.
async fn delete_type(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_type(&state.db, &id).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, type_id = %id, "deleted transaction type");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Creates a new category under a transaction type.
async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryItem>>), AppError> {
    let created = db::create_category(&state.db, payload).await?;
    tracing::info!(
        user_id = %user.0.id,
        category_id = %created.id,
        category_name = %created.name,
        "created category"
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
    db::update_category_name(&state.db, &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, category_id = %id, "updated category name");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a category and cascades to its subcategories.
async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_category(&state.db, &id).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, category_id = %id, "deleted category");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Creates a new subcategory under a category.
async fn create_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSubcategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SubcategoryItem>>), AppError> {
    let created = db::create_subcategory(&state.db, payload).await?;
    tracing::info!(
        user_id = %user.0.id,
        subcategory_id = %created.id,
        subcategory_name = %created.name,
        "created subcategory"
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
    db::update_subcategory_name(&state.db, &id, payload).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, subcategory_id = %id, "updated subcategory name");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Deletes a subcategory.
async fn delete_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    db::delete_subcategory(&state.db, &id).await?;
    let hierarchy = db::fetch_hierarchy(&state.db).await?;
    tracing::info!(user_id = %user.0.id, subcategory_id = %id, "deleted subcategory");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Resets all categories back to system defaults.
async fn reset_defaults(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    let hierarchy = db::reset_defaults(&state.db).await?;
    tracing::info!(user_id = %user.0.id, "reset categories to defaults");
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_hierarchy).post(create_category))
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
    use crate::{
        core::{db::init_db, response::Code},
        features::auth::models::User,
    };

    fn test_user() -> AuthUser {
        AuthUser(User {
            id: "user-test-1".to_string(),
            name: "testuser".to_string(),
            password_hash: "hash".to_string(),
            role: "admin".to_string(),
            created_at: 0,
            updated_at: 0,
        })
    }

    #[tokio::test]
    async fn test_get_hierarchy_returns_seeded_defaults() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        let res = get_hierarchy(State(state)).await.unwrap();
        assert_eq!(res.0.code, Code::Zero);
        assert_eq!(res.0.status, Status::Ok);

        let hierarchy = res.0.data;
        assert_eq!(hierarchy.types.len(), 4);
        assert_eq!(hierarchy.categories.len(), 8);

        let total_subs: usize = hierarchy
            .categories
            .iter()
            .map(|c| c.subcategories.len())
            .sum();
        assert_eq!(total_subs, 14);
    }

    #[tokio::test]
    async fn test_type_crud_lifecycle() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Create type
        let create_req = CreateTypeRequest {
            name: "Crypto".to_string(),
            color: "#8b5cf6".to_string(),
        };
        let (status, res) = create_type(State(state.clone()), test_user(), Json(create_req))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let created_type = res.0.data;
        assert_eq!(created_type.name, "Crypto");
        assert_eq!(created_type.color, "#8b5cf6");

        // Update color
        let update_req = UpdateTypeColorRequest {
            color: "#a855f7".to_string(),
        };
        let res = update_type_color(
            State(state.clone()),
            test_user(),
            Path(created_type.id.clone()),
            Json(update_req),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status, Status::Ok);

        // Delete type
        let res = delete_type(
            State(state.clone()),
            test_user(),
            Path(created_type.id.clone()),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status, Status::Ok);
    }

    #[tokio::test]
    async fn test_category_and_subcategory_crud() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Create category under Income
        let cat_req = CreateCategoryRequest {
            type_name: "Income".to_string(),
            name: "Consulting".to_string(),
        };
        let (status, res) = create_category(State(state.clone()), test_user(), Json(cat_req))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let category = res.0.data;
        assert_eq!(category.name, "Consulting");
        assert_eq!(category.type_name, "Income");

        // Create subcategory
        let sub_req = CreateSubcategoryRequest {
            category_id: category.id.clone(),
            name: "Tech Advisory".to_string(),
        };
        let (sub_status, sub_res) =
            create_subcategory(State(state.clone()), test_user(), Json(sub_req))
                .await
                .unwrap();
        assert_eq!(sub_status, StatusCode::CREATED);
        let subcategory = sub_res.0.data;
        assert_eq!(subcategory.name, "Tech Advisory");

        // Rename subcategory
        let update_sub = UpdateNameRequest {
            name: "Enterprise Architecture".to_string(),
        };
        let res = update_subcategory(
            State(state.clone()),
            test_user(),
            Path(subcategory.id.clone()),
            Json(update_sub),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status, Status::Ok);

        // Delete subcategory
        let res = delete_subcategory(
            State(state.clone()),
            test_user(),
            Path(subcategory.id.clone()),
        )
        .await
        .unwrap();
        assert_eq!(res.0.status, Status::Ok);

        // Delete category
        let res = delete_category(State(state.clone()), test_user(), Path(category.id.clone()))
            .await
            .unwrap();
        assert_eq!(res.0.status, Status::Ok);
    }

    #[tokio::test]
    async fn test_conflict_on_duplicate_creation() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Duplicate type name "Income"
        let create_req = CreateTypeRequest {
            name: "Income".to_string(),
            color: "#10b981".to_string(),
        };
        let err = create_type(State(state.clone()), test_user(), Json(create_req))
            .await
            .unwrap_err();
        match err {
            AppError::Conflict(_) => {}
            other => panic!("expected AppError::Conflict, got {other:?}"),
        }

        // Duplicate category name under Expense: "Housing"
        let cat_req = CreateCategoryRequest {
            type_name: "Expense".to_string(),
            name: "Housing".to_string(),
        };
        let cat_err = create_category(State(state.clone()), test_user(), Json(cat_req))
            .await
            .unwrap_err();
        match cat_err {
            AppError::Conflict(_) => {}
            other => panic!("expected AppError::Conflict, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_reset_defaults_restores_hierarchy() {
        let db = init_db("sqlite::memory:").await.unwrap();
        db::seed_default_categories(&db).await.unwrap();
        let state = AppState::new(db);

        // Delete all types
        let hierarchy = db::fetch_hierarchy(&state.db).await.unwrap();
        for t in hierarchy.types {
            let _ = delete_type(State(state.clone()), test_user(), Path(t.id))
                .await
                .unwrap();
        }

        let cleared = db::fetch_hierarchy(&state.db).await.unwrap();
        assert_eq!(cleared.types.len(), 0);

        // Reset defaults
        let res = reset_defaults(State(state.clone()), test_user())
            .await
            .unwrap();
        assert_eq!(res.0.data.types.len(), 4);
        assert_eq!(res.0.data.categories.len(), 8);
    }
}
