use crate::{
    core::{db::DbPool, db_err, error::AppError, DbResultExt},
    features::categories::{
        db::util::{generate_token, is_unique_violation, now_epoch_secs},
        error::CategoryError,
        models::{CategoryItem, CreateCategoryRequest, UpdateNameRequest},
    },
};

/// Atomically creates a new category under a transaction type.
pub(crate) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.CREATE_CATEGORY";
    let type_name_or_id = payload.type_name.trim();
    let name = payload.name.trim().to_string();

    if type_name_or_id.is_empty() || name.is_empty() {
        return Err(CategoryError::EmptyCategoryName { action: ACTION }.into());
    }

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.BEGIN_TRANSACTION")?;

    let parent_type: Option<(String, String)> = sqlx::query_as(
        "SELECT id, name FROM transaction_types WHERE id = ? OR name = ? COLLATE NOCASE LIMIT 1",
    )
    .bind(type_name_or_id)
    .bind(type_name_or_id)
    .fetch_optional(&mut *tx)
    .await
    .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.QUERY_PARENT_TYPE")?;

    let (type_id, canonical_type_name) = match parent_type {
        Some((tid, tname)) => (tid, tname),
        None => {
            return Err(CategoryError::TypeNotFound {
                action: ACTION,
                id: type_name_or_id.to_string(),
            }
            .into())
        }
    };

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM categories WHERE type_id = ?")
            .bind(&type_id)
            .fetch_one(&mut *tx)
            .await
            .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("cat-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
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
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.COMMIT_TRANSACTION")?;
            Ok(CategoryItem {
                id,
                name,
                type_name: canonical_type_name,
                subcategories: Vec::new(),
            })
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: ACTION,
                    name,
                    type_name: canonical_type_name,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_CATEGORY.INSERT", err))
            }
        }
    }
}

/// Updates the name of an existing category.
pub(crate) async fn update_category_name(
    pool: &DbPool,
    id: &str,
    payload: UpdateNameRequest,
) -> Result<(), AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME";
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(CategoryError::EmptyCategoryName { action: ACTION }.into());
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE categories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(CategoryError::CategoryNotFound {
                    action: ACTION,
                    id: id.to_string(),
                }
                .into())
            } else {
                Ok(())
            }
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: ACTION,
                    name,
                    type_name: String::new(),
                }
                .into())
            } else {
                Err(db_err(
                    "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.EXECUTE",
                    err,
                ))
            }
        }
    }
}

/// Deletes a category and cascades to its subcategories.
pub(crate) async fn delete_category(pool: &DbPool, id: &str) -> Result<(), AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.DELETE_CATEGORY";
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.DELETE_CATEGORY.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::CategoryNotFound {
            action: ACTION,
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
