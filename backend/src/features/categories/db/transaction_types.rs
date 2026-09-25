use crate::core::{db_err, AppError, DbPool, DbResultExt};

use super::super::error::CategoryError;
use super::super::models::{CreateTypeRequest, TransactionTypeItem, UpdateTypeColorRequest};
use super::colors::resolve_color_id;
use super::util::{generate_token, is_unique_violation, now_epoch_secs};

/// Atomically creates a new transaction type.
pub(crate) async fn create_type(
    pool: &DbPool,
    payload: CreateTypeRequest,
) -> Result<TransactionTypeItem, AppError> {
    let name = payload.name().trim().to_string();
    if name.is_empty() {
        return Err(CategoryError::EmptyTypeName {
            action: "CONFIG.CATEGORIES.CREATE_TYPE.EMPTY_NAME",
        }
        .into());
    }

    let (color_id, color_hex) = resolve_color_id(pool, payload.color_id(), payload.color()).await?;

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_TYPE.BEGIN_TRANSACTION")?;

    let max_sort: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM transaction_types")
        .fetch_one(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_TYPE.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("type-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
        r#"
        INSERT INTO transaction_types (id, name, color_id, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(color_id)
    .bind(next_sort)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_TYPE.COMMIT_TRANSACTION")?;
            Ok(TransactionTypeItem::new(id, name, color_hex, color_id))
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::TypeAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS",
                    name,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_TYPE.INSERT", err))
            }
        }
    }
}

/// Updates display color of a transaction type.
pub(crate) async fn update_type_color(
    pool: &DbPool,
    id: &str,
    payload: UpdateTypeColorRequest,
) -> Result<(), AppError> {
    let (color_id, _) = resolve_color_id(pool, payload.color_id(), payload.color()).await?;

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE transaction_types SET color_id = ?, updated_at = ? WHERE id = ?")
        .bind(color_id)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::TypeNotFound {
            action: "CONFIG.CATEGORIES.UPDATE_TYPE_COLOR.TYPE_NOT_FOUND",
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}

/// Deletes a transaction type and cascades to associated categories.
pub(crate) async fn delete_type(pool: &DbPool, id: &str) -> Result<(), AppError> {
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
