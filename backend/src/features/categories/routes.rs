//! Category taxonomy and hierarchy configuration REST route handlers.
//!
//! Exposes HTTP endpoints under `/config` for exploring the 3-tier category hierarchy,
//! retrieving curated palette colors, authoring categories and subcategories, and resetting
//! defaults. Read-only endpoints allow public access for dashboards and visual breakdowns,
//! while administrative mutations require authenticated sessions.
//!
//! # Architecture & CQS Design
//! - **Unified Route Namespace**: Mounted under `/config`, providing canonical entry points
//!   like `GET /api/v1/config/hierarchy` and `POST /api/v1/config/hierarchy/reset` alongside
//!   granular taxonomy resources (`/config/categories/*`).
//! - **Command-Query Separation (CQS)**: Mutation and reset endpoints return lean acknowledgement
//!   envelopes (`ApiResponse<()>`) or created items rather than duplicating expensive hierarchy queries,
//!   eliminating read amplification across high-frequency write paths.
//! - **Strict Error Envelopes**: Handlers delegate persistence to atomic database routines,
//!   mapping domain validation failures and constraint collisions to structured error envelopes.

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
///
/// Canonical route: `GET /api/v1/config/hierarchy`
/// Aliases: `GET /api/v1/config/hierarchies`, `GET /api/v1/config/categories/hierarchy`
///
/// Requires an authenticated session.
///
/// # Ingress
/// - `State(state)`: Injected application state containing the database pool.
/// - `_user`: Authenticated operator session context.
///
/// # Returns
/// - `Ok(Json(ApiResponse<CategoryHierarchyResponse>))`: 200 OK with complete hierarchy and palette.
/// - `Err(AppError)`: Database error if query fails, or 401 Unauthorized if unauthenticated.
async fn get_hierarchy(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ApiResponse<CategoryHierarchyResponse>>, AppError> {
    let hierarchy = db::fetch_hierarchy(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), hierarchy)))
}

/// Retrieves all available palette colors from the database.
///
/// `GET /api/v1/config/categories/colors`
///
/// Requires an authenticated session.
/// Returns the full list of selectable palette colors ordered by display sort sequence.
///
/// # Ingress
/// - `State(state)`: Injected application state containing the database pool.
/// - `_user`: Authenticated operator session context.
///
/// # Returns
/// - `Ok(Json(ApiResponse<Vec<ColorItem>>))`: 200 OK with list of available colors.
/// - `Err(AppError)`: Database error if query fails, or 401 Unauthorized if unauthenticated.
async fn get_colors(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ApiResponse<Vec<ColorItem>>>, AppError> {
    let colors = db::fetch_colors(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), colors)))
}

/// Creates a new transaction type.
///
/// `POST /api/v1/config/categories/types`
///
/// Requires authentication. Validates unique type naming, associates the palette color ID,
/// and inserts the new top-level classification tier.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`CreateTypeRequest`] containing name and color ID.
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<TransactionTypeItem>)))`: 201 Created with new type.
/// - `Err(AppError)`: 400 Bad Request if name invalid, 404 if color ID not found, 409 if type exists.
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
///
/// `PATCH /api/v1/config/categories/types/{id}/color`
///
/// Requires authentication. Updates the palette color reference of an existing transaction type.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Transaction type identifier.
/// - `Json(payload)`: Validated [`UpdateTypeColorRequest`] containing palette color ID.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with confirmation.
/// - `Err(AppError)`: 404 if type or color ID not found.
async fn update_type_color(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTypeColorRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::update_type_color(state.db(), id, payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        type_id = %id,
        "CONFIG.CATEGORIES.ROUTE.UPDATE_TYPE_COLOR. Transaction type color updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Deletes a transaction type and cascades deletion to categories and subcategories.
///
/// `DELETE /api/v1/config/categories/types/{id}`
///
/// Requires authentication. Removes the target transaction type and cascades deletion
/// to all nested categories and subcategories.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Identifier of transaction type to remove.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with deletion confirmation.
/// - `Err(AppError)`: 404 Not Found if type does not exist.
async fn delete_type(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_type(state.db(), id).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        type_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_TYPE. Transaction type deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Creates a new category under a transaction type.
///
/// `POST /api/v1/config/categories`
///
/// Requires authentication. Resolves target parent type, verifies name uniqueness within
/// that type, and inserts the new mid-level category entity.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`CreateCategoryRequest`] containing target type and category name.
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<CategoryItem>)))`: 201 Created with new category.
/// - `Err(AppError)`: 400 Bad Request if empty, 404 if type not found, 409 if category exists.
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
///
/// `PATCH /api/v1/config/categories/{id}`
///
/// Requires authentication. Validates non-empty name, ensures uniqueness within the same parent
/// type, and updates the category record. Returns the renamed category representation.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Category identifier.
/// - `Json(payload)`: Validated [`UpdateNameRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<CategoryItem>))`: 200 OK with updated category.
/// - `Err(AppError)`: 400 if empty, 404 if not found, 409 if name already exists under type.
async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateNameRequest>,
) -> Result<Json<ApiResponse<CategoryItem>>, AppError> {
    let updated = db::update_category_name(state.db(), id, payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        category_id = %id,
        category_name = %updated.name(),
        "CONFIG.CATEGORIES.ROUTE.UPDATE_CATEGORY. Category name updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Deletes a category and cascades to its subcategories.
///
/// `DELETE /api/v1/config/categories/{id}`
///
/// Requires authentication. Deletes the target category and cascades removal to all
/// child subcategories.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Category identifier.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with deletion confirmation.
/// - `Err(AppError)`: 404 Not Found if category does not exist.
async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_category(state.db(), id).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        category_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_CATEGORY. Category deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Creates a new subcategory under a category.
///
/// `POST /api/v1/config/categories/subcategories`
///
/// Requires authentication. Verifies parent category existence, ensures subcategory name
/// is unique within that category, and inserts the leaf subcategory record.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`CreateSubcategoryRequest`].
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<SubcategoryItem>)))`: 201 Created with new subcategory.
/// - `Err(AppError)`: 400 if empty, 404 if category not found, 409 if subcategory exists.
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
///
/// `PATCH /api/v1/config/categories/subcategories/{id}`
///
/// Requires authentication. Validates non-empty name, ensures uniqueness within the same
/// parent category, and updates the subcategory name. Returns the renamed subcategory representation.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Subcategory identifier.
/// - `Json(payload)`: Validated [`UpdateNameRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<SubcategoryItem>))`: 200 OK with updated subcategory.
/// - `Err(AppError)`: 400 if empty, 404 if not found, 409 if subcategory exists under parent.
async fn update_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateNameRequest>,
) -> Result<Json<ApiResponse<SubcategoryItem>>, AppError> {
    let updated = db::update_subcategory_name(state.db(), id, payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        subcategory_id = %id,
        subcategory_name = %updated.name(),
        "CONFIG.CATEGORIES.ROUTE.UPDATE_SUBCATEGORY. Subcategory name updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Deletes a subcategory.
///
/// `DELETE /api/v1/config/categories/subcategories/{id}`
///
/// Requires authentication. Deletes the leaf subcategory record.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Subcategory identifier.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with deletion confirmation.
/// - `Err(AppError)`: 404 Not Found if subcategory does not exist.
async fn delete_subcategory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_subcategory(state.db(), id).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        subcategory_id = %id,
        "CONFIG.CATEGORIES.ROUTE.DELETE_SUBCATEGORY. Subcategory deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Resets all categories, transaction types, and palette colors back to system defaults.
///
/// Canonical route: `POST /api/v1/config/hierarchy/reset`
/// Aliases: `POST /api/v1/config/hierarchies/reset`, `POST /api/v1/config/categories/reset`
///
/// Requires authentication. Atomically clears user-modified categories, types, and colors,
/// re-seeding canonical defaults from the embedded JSON template within a single transaction.
/// In accordance with Command-Query Separation (CQS), returns `ApiResponse<()>` without
/// fetching the updated hierarchy. Callers fetch `GET /api/v1/config/hierarchy` when needed.
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with restoration confirmation.
/// - `Err(AppError)`: Database error if transaction fails.
async fn reset_defaults(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::reset_defaults(state.db()).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        "CONFIG.CATEGORIES.ROUTE.RESET_DEFAULTS. Categories reset to defaults"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Configures and returns the Axum router for category and hierarchy endpoints.
pub(super) fn router() -> Router<AppState> {
    Router::new()
        // Hierarchy query routes
        .route("/hierarchy", get(get_hierarchy))
        .route("/hierarchies", get(get_hierarchy))
        .route("/categories/hierarchy", get(get_hierarchy))
        // Hierarchy reset routes
        .route("/hierarchy/reset", post(reset_defaults))
        .route("/hierarchies/reset", post(reset_defaults))
        .route("/categories/reset", post(reset_defaults))
        // Palette colors
        .route("/categories/colors", get(get_colors))
        // Types
        .route("/categories/types", post(create_type))
        .route("/categories/types/{id}", delete(delete_type))
        .route("/categories/types/{id}/color", patch(update_type_color))
        // Categories
        .route("/categories", post(create_category))
        .route(
            "/categories/{id}",
            patch(update_category).delete(delete_category),
        )
        // Subcategories
        .route("/categories/subcategories", post(create_subcategory))
        .route(
            "/categories/subcategories/{id}",
            patch(update_subcategory).delete(delete_subcategory),
        )
}
