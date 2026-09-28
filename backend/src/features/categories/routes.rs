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
