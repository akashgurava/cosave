//! Root transaction type persistence, color mapping, and cascade deletion.
//!
//! Manages top-level cashflow classifications such as Income, Expense, and Transfer.
//! Database routines handle atomic type creation, palette color association, display
//! sort ordering, and cascading deletion of associated child categories. Operations
//! run inside transactions to keep taxonomy definitions consistent.

use sqlx::Executor;

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};
use crate::features::categories::models::{
    CreateTypeRequest, TransactionTypeItem, TypeName, UpdateTypeColorRequest,
};
use crate::features::categories::CategoryError;

/// Atomically creates a new transaction type within an isolated database transaction.
///
/// Validates the type name Value Object, inserts into `transaction_types` with the provided
/// palette `color_id`, computes the next sequential `sort_order`, and commits.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateTypeRequest`] containing name and color ID.
///
/// # Returns
/// - `Ok(TransactionTypeItem)` representing the newly created type with ID and resolved color.
///
/// # Errors
/// - Returns [`CategoryError::EmptyTypeName`] if the name fails Value Object validation.
/// - Returns [`CategoryError::ColorNotFound`] if `color_id` does not exist in `colors`.
/// - Returns [`CategoryError::TypeAlreadyExists`] if a type with the same name already exists.
pub(in crate::features::categories) async fn create_type(
    pool: &DbPool,
    payload: CreateTypeRequest,
) -> Result<TransactionTypeItem, AppError> {
    let name = TypeName::try_new(payload.name(), "CONFIG.CATEGORIES.CREATE_TYPE.EMPTY_NAME")?;
    let color_id = payload.color_id();
    let now = now_epoch_secs();

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_TYPE.BEGIN_TRANSACTION")?;

    let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
        .fetch_one(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_TYPE.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let insert_res = tx
        .execute(
            sqlx::query(
                r#"
            INSERT INTO transaction_types (name, color_id, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            )
            .bind(name.as_str())
            .bind(color_id)
            .bind(next_sort)
            .bind(now)
            .bind(now),
        )
        .await;

    let raw_name = name.into_inner();
    match insert_res {
        Ok(exec_res) => {
            let id = exec_res.last_insert_rowid();
            let (color_hex,): (String,) = sqlx::query_as("SELECT hex FROM colors WHERE id = ?")
                .bind(color_id)
                .fetch_one(&mut *tx)
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_TYPE.FETCH_COLOR_HEX")?;

            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_TYPE.COMMIT_TRANSACTION")?;
            Ok(TransactionTypeItem::new(id, raw_name, color_hex, color_id))
        }
        Err(err) => {
            if is_foreign_key_violation(&err) {
                Err(CategoryError::ColorNotFound {
                    action: "CONFIG.CATEGORIES.CREATE_TYPE.COLOR_NOT_FOUND",
                    id: color_id,
                }
                .into())
            } else if is_unique_violation(&err) {
                Err(CategoryError::TypeAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS",
                    name: raw_name,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_TYPE.INSERT", err))
            }
        }
    }
}

/// Updates the visual display color of an existing transaction type.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target transaction type.
/// - `payload`: Inbound [`UpdateTypeColorRequest`] containing the target palette color ID.
///
/// # Errors
/// - Returns [`CategoryError::TypeNotFound`] if no row exists with the specified `id`.
/// - Returns [`CategoryError::ColorNotFound`] if `color_id` does not exist in `colors`.
pub(in crate::features::categories) async fn update_type_color(
    pool: &DbPool,
    id: i64,
    payload: UpdateTypeColorRequest,
) -> Result<(), AppError> {
    let color_id = payload.color_id();
    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE transaction_types SET color_id = ?, updated_at = ? WHERE id = ?")
        .bind(color_id)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    match res {
        Ok(query_res) => {
            if query_res.rows_affected() == 0 {
                Err(CategoryError::TypeNotFound {
                    action: "CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.TYPE_NOT_FOUND",
                    id: id.to_string(),
                }
                .into())
            } else {
                Ok(())
            }
        }
        Err(err) => {
            if is_foreign_key_violation(&err) {
                Err(CategoryError::ColorNotFound {
                    action: "CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.COLOR_NOT_FOUND",
                    id: color_id,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.EXECUTE", err))
            }
        }
    }
}

/// Deletes a transaction type by primary key, automatically cascading deletions to child categories.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target transaction type.
///
/// # Errors
/// - Returns [`CategoryError::TypeNotFound`] if no transaction type with `id` exists.
/// - Returns [`AppError::ShouldNotBeHappening`] if deletion query execution fails.
pub(in crate::features::categories) async fn delete_type(
    pool: &DbPool,
    id: i64,
) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM transaction_types WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.DELETE_TYPE.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::TypeNotFound {
            action: "CONFIG.CATEGORIES.DELETE_TYPE.TYPE_NOT_FOUND",
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
