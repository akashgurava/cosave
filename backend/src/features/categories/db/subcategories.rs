//! Leaf subcategory persistence, naming updates, and deletion.
//!
//! Manages granular subcategories scoped under mid-level categories, such as Rent, Groceries,
//! or Dining Out. Persistence routines verify parent category existence, enforce unique names
//! within the parent scope, and automatically assign sequential sort positions.
//! Deleting a subcategory cleanly removes the leaf node without impacting the broader hierarchy.

use sqlx::Executor;

use crate::core::{db_err, is_unique_violation, now_epoch_secs, AppError, DbPool, DbResultExt};
use crate::features::categories::models::{
    CreateSubcategoryRequest, SubcategoryItem, SubcategoryName, UpdateNameRequest,
};
use crate::features::categories::CategoryError;

/// Atomically creates a new subcategory scoped under an existing category.
///
/// Verifies the parent category exists, validates the subcategory name Value Object, computes
/// the next sequential `sort_order`, inserts into `subcategories`, and commits the transaction.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateSubcategoryRequest`] containing parent category ID and subcategory name.
///
/// # Returns
/// - `Ok(SubcategoryItem)` representing the newly created subcategory with ID and name.
///
/// # Errors
/// - Returns [`CategoryError::EmptySubcategoryName`] if the name fails Value Object validation.
/// - Returns [`CategoryError::CategoryNotFound`] if the parent category does not exist.
/// - Returns [`CategoryError::SubcategoryAlreadyExists`] if a subcategory with the same name exists under this category.
/// - Returns [`AppError`] on database transaction failure.
pub(in crate::features::categories) async fn create_subcategory(
    pool: &DbPool,
    payload: CreateSubcategoryRequest,
) -> Result<SubcategoryItem, AppError> {
    let name = SubcategoryName::try_new(
        payload.name(),
        "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.EMPTY_NAME",
    )?;

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.BEGIN_TRANSACTION")?;

    let cat_exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM categories WHERE id = ?")
        .bind(payload.category_id())
        .fetch_optional(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.QUERY_PARENT_CATEGORY")?;

    if cat_exists.is_none() {
        return Err(CategoryError::CategoryNotFound {
            action: "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.PARENT_NOT_FOUND",
            id: payload.category_id().to_string(),
        }
        .into());
    }

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM subcategories WHERE category_id = ?")
            .bind(payload.category_id())
            .fetch_one(&mut *tx)
            .await
            .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let now = now_epoch_secs();

    let insert_res = tx
        .execute(
            sqlx::query(
                r#"
            INSERT INTO subcategories (category_id, name, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            )
            .bind(payload.category_id())
            .bind(name.as_str())
            .bind(next_sort)
            .bind(now)
            .bind(now),
        )
        .await;

    let raw_name = name.into_inner();
    match insert_res {
        Ok(exec_res) => {
            let id = exec_res.last_insert_rowid();
            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.COMMIT_TRANSACTION")?;
            Ok(SubcategoryItem::new(id, raw_name))
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::SubcategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_SUBCATEGORY.ALREADY_EXISTS",
                    name: raw_name,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.INSERT", err))
            }
        }
    }
}

/// Updates the name of an existing subcategory by primary key.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target subcategory.
/// - `payload`: Inbound [`UpdateNameRequest`] containing the new subcategory name.
///
/// # Returns
/// - `Ok(SubcategoryItem)` representing the renamed subcategory with ID and updated name.
///
/// # Errors
/// - Returns [`CategoryError::EmptySubcategoryName`] if the name fails Value Object validation.
/// - Returns [`CategoryError::SubcategoryNotFound`] if no subcategory exists with `id`.
/// - Returns [`CategoryError::SubcategoryAlreadyExists`] if another subcategory under the same parent has this name.
/// - Returns [`AppError::ShouldNotBeHappening`] if update execution fails.
pub(in crate::features::categories) async fn update_subcategory_name(
    pool: &DbPool,
    id: i64,
    payload: UpdateNameRequest,
) -> Result<SubcategoryItem, AppError> {
    let name = SubcategoryName::try_new(
        payload.name(),
        "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.EMPTY_NAME",
    )?;

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE subcategories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(name.as_str())
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    let raw_name = name.into_inner();
    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(CategoryError::SubcategoryNotFound {
                    action: "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.SUBCATEGORY_NOT_FOUND",
                    id: id.to_string(),
                }
                .into())
            } else {
                Ok(SubcategoryItem::new(id, raw_name))
            }
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::SubcategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.ALREADY_EXISTS",
                    name: raw_name,
                }
                .into())
            } else {
                Err(db_err(
                    "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME.EXECUTE",
                    err,
                ))
            }
        }
    }
}

/// Deletes a subcategory by primary key.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target subcategory.
///
/// # Errors
/// - Returns [`CategoryError::SubcategoryNotFound`] if no subcategory with `id` exists.
/// - Returns [`AppError::ShouldNotBeHappening`] if deletion query execution fails.
pub(in crate::features::categories) async fn delete_subcategory(
    pool: &DbPool,
    id: i64,
) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM subcategories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.DELETE_SUBCATEGORY.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::SubcategoryNotFound {
            action: "CONFIG.CATEGORIES.DELETE_SUBCATEGORY.SUBCATEGORY_NOT_FOUND",
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
