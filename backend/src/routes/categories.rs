use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    auth::{generate_token, AuthUser},
    db::DbPool,
    models::category::{
        CategoryHierarchyResponse, CategoryHierarchyRow, CategoryItem, CreateCategoryRequest,
        CreateSubcategoryRequest, CreateTypeRequest, SubcategoryItem, TransactionTypeItem,
        UpdateNameRequest, UpdateTypeColorRequest,
    },
    response::{ApiResponse, Code, Status},
    state::AppState,
};

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

async fn fetch_hierarchy(pool: &DbPool) -> Result<CategoryHierarchyResponse, sqlx::Error> {
    let rows: Vec<CategoryHierarchyRow> = sqlx::query_as(
        r#"
        SELECT
            type_id,
            type_name,
            type_color,
            type_sort_order,
            category_id,
            category_name,
            category_sort_order,
            subcategory_id,
            subcategory_name,
            subcategory_sort_order
        FROM v_category_hierarchy
        ORDER BY type_sort_order, type_name, category_sort_order, category_name, subcategory_sort_order, subcategory_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut types: Vec<TransactionTypeItem> = Vec::new();
    let mut categories: Vec<CategoryItem> = Vec::new();

    for row in rows {
        if !types.iter().any(|t| t.id == row.type_id) {
            types.push(TransactionTypeItem {
                id: row.type_id.clone(),
                name: row.type_name.clone(),
                color: row.type_color.clone(),
            });
        }

        if let (Some(cat_id), Some(cat_name)) = (row.category_id, row.category_name) {
            if let Some(cat) = categories.iter_mut().find(|c| c.id == cat_id) {
                if let (Some(sub_id), Some(sub_name)) = (row.subcategory_id, row.subcategory_name) {
                    if !cat.subcategories.iter().any(|s| s.id == sub_id) {
                        cat.subcategories.push(SubcategoryItem {
                            id: sub_id,
                            name: sub_name,
                        });
                    }
                }
            } else {
                let mut subcategories = Vec::new();
                if let (Some(sub_id), Some(sub_name)) = (row.subcategory_id, row.subcategory_name) {
                    subcategories.push(SubcategoryItem {
                        id: sub_id,
                        name: sub_name,
                    });
                }
                categories.push(CategoryItem {
                    id: cat_id,
                    name: cat_name,
                    type_name: row.type_name.clone(),
                    subcategories,
                });
            }
        }
    }

    Ok(CategoryHierarchyResponse { types, categories })
}

/// Retrieves the complete transaction type, category, and subcategory hierarchy.
async fn get_hierarchy(
    State(state): State<AppState>,
    _user: AuthUser,
) -> (
    StatusCode,
    Json<ApiResponse<Option<CategoryHierarchyResponse>>>,
) {
    match fetch_hierarchy(&state.db).await {
        Ok(hierarchy) => (
            StatusCode::OK,
            Json(ApiResponse::ok(Status::ok(), Some(hierarchy))),
        ),
        Err(err) => {
            tracing::error!(error = %err, "failed to query category hierarchy");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Creates a new transaction type.
async fn create_type(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(payload): Json<CreateTypeRequest>,
) -> (StatusCode, Json<ApiResponse<Option<TransactionTypeItem>>>) {
    let name = payload.name.trim().to_string();
    let color = payload.color.trim().to_string();

    if name.is_empty() || color.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    let id = format!("type-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let max_sort: Result<(Option<i64>,), _> =
        sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
            .fetch_one(&state.db)
            .await;
    let next_sort = max_sort.map(|r| r.0.unwrap_or(0) + 1).unwrap_or(1);

    let res = sqlx::query(
        r#"
        INSERT INTO transaction_types (id, name, color, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&color)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => {
            tracing::info!(
                user_id = %_user.0.id,
                type_id = %id,
                type_name = %name,
                "created transaction type"
            );
            (
                StatusCode::CREATED,
                Json(ApiResponse::ok(
                    Status::ok(),
                    Some(TransactionTypeItem { id, name, color }),
                )),
            )
        }
        Err(err) => {
            tracing::error!(error = %err, type_name = %name, "failed to insert transaction type");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Updates the color of a transaction type.
async fn update_type_color(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTypeColorRequest>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let color = payload.color.trim().to_string();
    if color.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE transaction_types SET color = ?, updated_at = ? WHERE id = ?")
        .bind(&color)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                type_id = %id,
                color = %color,
                "updated transaction type color"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, type_id = %id, "failed to update transaction type color");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Deletes a transaction type and cascades deletion to categories and subcategories.
async fn delete_type(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let res = sqlx::query("DELETE FROM transaction_types WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                type_id = %id,
                "deleted transaction type and cascaded child categories"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, type_id = %id, "failed to delete transaction type");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Creates a new category under a transaction type.
async fn create_category(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> (StatusCode, Json<ApiResponse<Option<CategoryItem>>>) {
    let type_name_or_id = payload.type_name.trim();
    let name = payload.name.trim().to_string();

    if type_name_or_id.is_empty() || name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    // Lookup transaction type by ID or name
    let type_row: Result<Option<(String, String)>, _> = sqlx::query_as(
        "SELECT id, name FROM transaction_types WHERE id = ? OR name = ? COLLATE NOCASE LIMIT 1",
    )
    .bind(type_name_or_id)
    .bind(type_name_or_id)
    .fetch_optional(&state.db)
    .await;

    let (type_id, canonical_type_name) = match type_row {
        Ok(Some((tid, tname))) => (tid, tname),
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::err(
                    Code::not_found(),
                    Status::not_found(),
                    None,
                )),
            );
        }
        Err(err) => {
            tracing::error!(error = %err, query = %type_name_or_id, "failed to lookup transaction type");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            );
        }
    };

    let id = format!("cat-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let max_sort: Result<(Option<i64>,), _> =
        sqlx::query_as("SELECT MAX(sort_order) FROM categories WHERE type_id = ?")
            .bind(&type_id)
            .fetch_one(&state.db)
            .await;
    let next_sort = max_sort.map(|r| r.0.unwrap_or(0) + 1).unwrap_or(1);

    let res = sqlx::query(
        r#"
        INSERT INTO categories (id, type_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&type_id)
    .bind(&name)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => {
            tracing::info!(
                user_id = %_user.0.id,
                category_id = %id,
                category_name = %name,
                type_name = %canonical_type_name,
                "created category"
            );
            (
                StatusCode::CREATED,
                Json(ApiResponse::ok(
                    Status::ok(),
                    Some(CategoryItem {
                        id,
                        name,
                        type_name: canonical_type_name,
                        subcategories: Vec::new(),
                    }),
                )),
            )
        }
        Err(err) => {
            tracing::error!(error = %err, category_name = %name, type_id = %type_id, "failed to insert category");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Updates the name of a category.
async fn update_category(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNameRequest>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE categories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                category_id = %id,
                name = %name,
                "updated category name"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, category_id = %id, "failed to update category name");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Deletes a category and cascades to its subcategories.
async fn delete_category(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                category_id = %id,
                "deleted category and cascaded subcategories"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, category_id = %id, "failed to delete category");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Creates a new subcategory under a category.
async fn create_subcategory(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(payload): Json<CreateSubcategoryRequest>,
) -> (StatusCode, Json<ApiResponse<Option<SubcategoryItem>>>) {
    let category_id = payload.category_id.trim();
    let name = payload.name.trim().to_string();

    if category_id.is_empty() || name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    let cat_exists: Result<Option<(String,)>, _> =
        sqlx::query_as("SELECT id FROM categories WHERE id = ?")
            .bind(category_id)
            .fetch_optional(&state.db)
            .await;

    match cat_exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::err(
                    Code::not_found(),
                    Status::not_found(),
                    None,
                )),
            );
        }
        Err(err) => {
            tracing::error!(error = %err, category_id = %category_id, "failed to verify category existence");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            );
        }
    }

    let id = format!("sub-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let max_sort: Result<(Option<i64>,), _> =
        sqlx::query_as("SELECT MAX(sort_order) FROM subcategories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(&state.db)
            .await;
    let next_sort = max_sort.map(|r| r.0.unwrap_or(0) + 1).unwrap_or(1);

    let res = sqlx::query(
        r#"
        INSERT INTO subcategories (id, category_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(category_id)
    .bind(&name)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => {
            tracing::info!(
                user_id = %_user.0.id,
                subcategory_id = %id,
                subcategory_name = %name,
                category_id = %category_id,
                "created subcategory"
            );
            (
                StatusCode::CREATED,
                Json(ApiResponse::ok(
                    Status::ok(),
                    Some(SubcategoryItem { id, name }),
                )),
            )
        }
        Err(err) => {
            tracing::error!(error = %err, subcategory_name = %name, category_id = %category_id, "failed to insert subcategory");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Updates the name of a subcategory.
async fn update_subcategory(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNameRequest>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(
                Code::bad_request(),
                Status::bad_request(),
                None,
            )),
        );
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE subcategories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                subcategory_id = %id,
                name = %name,
                "updated subcategory name"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, subcategory_id = %id, "failed to update subcategory name");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Deletes a subcategory.
async fn delete_subcategory(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Option<()>>>) {
    let res = sqlx::query("DELETE FROM subcategories WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            tracing::info!(
                user_id = %_user.0.id,
                subcategory_id = %id,
                "deleted subcategory"
            );
            (
                StatusCode::OK,
                Json(ApiResponse::ok(Status::ok(), Some(()))),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(
                Code::not_found(),
                Status::not_found(),
                None,
            )),
        ),
        Err(err) => {
            tracing::error!(error = %err, subcategory_id = %id, "failed to delete subcategory");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Resets the category hierarchy to the default configuration.
async fn reset_defaults(
    State(state): State<AppState>,
    _user: AuthUser,
) -> (
    StatusCode,
    Json<ApiResponse<Option<CategoryHierarchyResponse>>>,
) {
    if let Err(err) = crate::db::reset_default_categories(&state.db).await {
        tracing::error!(error = %err, "failed to reset default categories in database");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(
                Code::internal_error(),
                Status::internal_error(),
                None,
            )),
        );
    }

    tracing::info!(user_id = %_user.0.id, "user reset categories to default configuration");

    match fetch_hierarchy(&state.db).await {
        Ok(hierarchy) => (
            StatusCode::OK,
            Json(ApiResponse::ok(Status::ok(), Some(hierarchy))),
        ),
        Err(err) => {
            tracing::error!(error = %err, "failed to query category hierarchy after default reset");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(
                    Code::internal_error(),
                    Status::internal_error(),
                    None,
                )),
            )
        }
    }
}

/// Builds and returns the `/categories` router.
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
    use crate::{db::init_db, models::user::User};

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
        let state = AppState::new(db);

        let (status, res) = get_hierarchy(State(state), test_user()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(res.0.code, Code::Zero);
        assert_eq!(res.0.status, Status::Ok);

        let hierarchy = res.0.data.expect("hierarchy present");
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
        let state = AppState::new(db);

        // Create type
        let create_req = CreateTypeRequest {
            name: "Crypto".to_string(),
            color: "#8b5cf6".to_string(),
        };
        let (status, res) = create_type(State(state.clone()), test_user(), Json(create_req)).await;
        assert_eq!(status, StatusCode::CREATED);
        let created_type = res.0.data.unwrap();
        assert_eq!(created_type.name, "Crypto");
        assert_eq!(created_type.color, "#8b5cf6");

        // Update color
        let update_req = UpdateTypeColorRequest {
            color: "#a855f7".to_string(),
        };
        let (update_status, _) = update_type_color(
            State(state.clone()),
            test_user(),
            Path(created_type.id.clone()),
            Json(update_req),
        )
        .await;
        assert_eq!(update_status, StatusCode::OK);

        // Delete type
        let (del_status, _) = delete_type(
            State(state.clone()),
            test_user(),
            Path(created_type.id.clone()),
        )
        .await;
        assert_eq!(del_status, StatusCode::OK);

        // Deleting non-existent type returns 404
        let (del_status_404, _) =
            delete_type(State(state), test_user(), Path(created_type.id)).await;
        assert_eq!(del_status_404, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_category_and_subcategory_crud() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);

        // Create category under "Income"
        let cat_req = CreateCategoryRequest {
            type_name: "Income".to_string(),
            name: "Dividends".to_string(),
        };
        let (status, res) = create_category(State(state.clone()), test_user(), Json(cat_req)).await;
        assert_eq!(status, StatusCode::CREATED);
        let cat = res.0.data.unwrap();
        assert_eq!(cat.name, "Dividends");
        assert_eq!(cat.type_name, "Income");

        // Create subcategory
        let sub_req = CreateSubcategoryRequest {
            category_id: cat.id.clone(),
            name: "Quarterly Payouts".to_string(),
        };
        let (sub_status, sub_res) =
            create_subcategory(State(state.clone()), test_user(), Json(sub_req)).await;
        assert_eq!(sub_status, StatusCode::CREATED);
        let sub = sub_res.0.data.unwrap();
        assert_eq!(sub.name, "Quarterly Payouts");

        // Update subcategory name
        let sub_rename = UpdateNameRequest {
            name: "Monthly Dividends".to_string(),
        };
        let (ren_status, _) = update_subcategory(
            State(state.clone()),
            test_user(),
            Path(sub.id.clone()),
            Json(sub_rename),
        )
        .await;
        assert_eq!(ren_status, StatusCode::OK);

        // Delete subcategory
        let (del_sub_status, _) =
            delete_subcategory(State(state.clone()), test_user(), Path(sub.id)).await;
        assert_eq!(del_sub_status, StatusCode::OK);

        // Delete category
        let (del_cat_status, _) = delete_category(State(state), test_user(), Path(cat.id)).await;
        assert_eq!(del_cat_status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_reset_defaults_restores_hierarchy() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let state = AppState::new(db);

        // Clear a type
        let (del_status, _) = delete_type(
            State(state.clone()),
            test_user(),
            Path("type-income".to_string()),
        )
        .await;
        assert_eq!(del_status, StatusCode::OK);

        let (_, mid_res) = get_hierarchy(State(state.clone()), test_user()).await;
        assert_eq!(mid_res.0.data.unwrap().types.len(), 3);

        // Reset
        let (reset_status, reset_res) = reset_defaults(State(state), test_user()).await;
        assert_eq!(reset_status, StatusCode::OK);
        let fresh = reset_res.0.data.unwrap();
        assert_eq!(fresh.types.len(), 4);
        assert_eq!(fresh.categories.len(), 8);
        let total_subs: usize = fresh.categories.iter().map(|c| c.subcategories.len()).sum();
        assert_eq!(total_subs, 14);
    }
}
