use crate::core::{db_err, AppError, DbPool, DbResultExt};

use super::super::error::CategoryError;
use super::super::models::{CategoryItem, CategoryName, CreateCategoryRequest, UpdateNameRequest};
use super::util::{is_unique_violation, now_epoch_secs};

/// Atomically creates a new category under a transaction type.
pub(crate) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> {
    let type_name_or_id = payload.type_name().trim();
    if type_name_or_id.is_empty() {
        return Err(CategoryError::EmptyCategoryName {
            action: "CONFIG.CATEGORIES.CREATE_CATEGORY.EMPTY_NAME",
        }
        .into());
    }
    let name = CategoryName::try_new(
        payload.name(),
        "CONFIG.CATEGORIES.CREATE_CATEGORY.EMPTY_NAME",
    )?;

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.BEGIN_TRANSACTION")?;

    let parent_type: Option<(i64, String)> = if let Ok(parsed_id) = type_name_or_id.parse::<i64>() {
        sqlx::query_as(
            "SELECT id, name FROM transaction_types WHERE id = ? OR name = ? COLLATE NOCASE LIMIT 1",
        )
        .bind(parsed_id)
        .bind(type_name_or_id)
        .fetch_optional(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.QUERY_PARENT_TYPE")?
    } else {
        sqlx::query_as(
            "SELECT id, name FROM transaction_types WHERE name = ? COLLATE NOCASE LIMIT 1",
        )
        .bind(type_name_or_id)
        .fetch_optional(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.QUERY_PARENT_TYPE")?
    };

    let (type_id, canonical_type_name) = match parent_type {
        Some((tid, tname)) => (tid, tname),
        None => {
            return Err(CategoryError::TypeNotFound {
                action: "CONFIG.CATEGORIES.CREATE_CATEGORY.TYPE_NOT_FOUND",
                id: type_name_or_id.to_string(),
            }
            .into())
        }
    };

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM categories WHERE type_id = ?")
            .bind(type_id)
            .fetch_one(&mut *tx)
            .await
            .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO categories (type_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(type_id)
    .bind(name.as_str())
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    let raw_name = name.into_inner();
    match insert_res {
        Ok(exec_res) => {
            let id = exec_res.last_insert_rowid();
            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_CATEGORY.COMMIT_TRANSACTION")?;
            Ok(CategoryItem::new(id, raw_name, canonical_type_name))
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_CATEGORY.ALREADY_EXISTS",
                    name: raw_name,
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
    id: i64,
    payload: UpdateNameRequest,
) -> Result<(), AppError> {
    let name = CategoryName::try_new(
        payload.name(),
        "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.EMPTY_NAME",
    )?;

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE categories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(name.as_str())
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    let raw_name = name.into_inner();
    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(CategoryError::CategoryNotFound {
                    action: "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.CATEGORY_NOT_FOUND",
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
                    action: "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.ALREADY_EXISTS",
                    name: raw_name,
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
pub(crate) async fn delete_category(pool: &DbPool, id: i64) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.DELETE_CATEGORY.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::CategoryNotFound {
            action: "CONFIG.CATEGORIES.DELETE_CATEGORY.CATEGORY_NOT_FOUND",
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
