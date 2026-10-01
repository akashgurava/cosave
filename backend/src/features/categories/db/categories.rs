//! Mid-level category persistence, naming mutations, and cascade deletion.
//!
//! Manages grouping categories scoped under root transaction types, such as Housing, Food,
//! or Transportation. Routines enforce unique category names within each parent type,
//! maintain sequential display ordering, and handle updates and cascading deletions.
//! All mutations execute within transactions tagged with compile-time action identifiers.

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};
use crate::features::categories::models::{
    CategoryItem, CategoryName, CreateCategoryRequest, UpdateNameRequest,
};
use crate::features::categories::CategoryError;

/// Creates a new category scoped under a parent transaction type.
///
/// Validates the category name Value Object, inserts into `categories` calculating
/// the next sort order in a single atomic SQL statement, and returns the created category.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateCategoryRequest`] containing target type ID and category name.
///
/// # Returns
/// - `Ok(CategoryItem)` representing the newly created category with ID.
///
/// # Errors
/// - Returns [`CategoryError::EmptyCategoryName`] if category name fails Value Object validation.
/// - Returns [`CategoryError::TypeNotFound`] if the parent transaction type does not exist.
/// - Returns [`CategoryError::CategoryAlreadyExists`] if a category with the same name exists under this type.
/// - Returns [`AppError`] on database execution failure.
pub(in crate::features::categories) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> {
    let name = CategoryName::try_new(
        payload.name(),
        "CONFIG.CATEGORIES.CREATE_CATEGORY.EMPTY_NAME",
    )?;
    let type_id = payload.type_id();

    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query(
        r#"
        INSERT INTO categories (type_id, name, sort_order, created_at, updated_at)
        VALUES (?, ?, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM categories WHERE type_id = ?), ?, ?)
        "#,
    )
    .bind(type_id)
    .bind(&raw_name)
    .bind(type_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await;

    match res {
        Ok(exec) => {
            let id = exec.last_insert_rowid();
            Ok(CategoryItem::new(id, raw_name))
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_CATEGORY.ALREADY_EXISTS",
                    name: raw_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(CategoryError::TypeNotFound {
                    action: "CONFIG.CATEGORIES.CREATE_CATEGORY.TYPE_NOT_FOUND",
                    id: type_id.to_string(),
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_CATEGORY.INSERT", err))
            }
        }
    }
}

/// Updates the name of an existing category by primary key.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target category.
/// - `payload`: Inbound [`UpdateNameRequest`] containing the new category name.
///
/// # Returns
/// - `Ok(CategoryItem)` representing the renamed category with ID and updated name.
///
/// # Errors
/// - Returns [`CategoryError::EmptyCategoryName`] if the name fails Value Object validation.
/// - Returns [`CategoryError::CategoryNotFound`] if no category exists with `id`.
/// - Returns [`CategoryError::CategoryAlreadyExists`] if another category under the same type has this name.
/// - Returns [`AppError::ShouldNotBeHappening`] if update execution fails.
pub(in crate::features::categories) async fn update_category_name(
    pool: &DbPool,
    id: i64,
    payload: UpdateNameRequest,
) -> Result<CategoryItem, AppError> {
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
                Ok(CategoryItem::new(id, raw_name))
            }
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.UPDATE_CATEGORY_NAME.ALREADY_EXISTS",
                    name: raw_name,
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

/// Deletes a category by primary key, automatically cascading deletions to child subcategories.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target category.
///
/// # Errors
/// - Returns [`CategoryError::CategoryNotFound`] if no category with `id` exists.
/// - Returns [`AppError::ShouldNotBeHappening`] if deletion query execution fails.
pub(in crate::features::categories) async fn delete_category(
    pool: &DbPool,
    id: i64,
) -> Result<(), AppError> {
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
