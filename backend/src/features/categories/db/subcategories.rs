use crate::{
    core::{db::DbPool, db_err, error::AppError, DbResultExt},
    features::categories::{
        db::util::{generate_token, is_unique_violation, now_epoch_secs},
        error::CategoryError,
        models::{CreateSubcategoryRequest, SubcategoryItem, UpdateNameRequest},
    },
};

/// Atomically creates a new subcategory under an existing category.
pub(crate) async fn create_subcategory(
    pool: &DbPool,
    payload: CreateSubcategoryRequest,
) -> Result<SubcategoryItem, AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.CREATE_SUBCATEGORY";
    let category_id = payload.category_id.trim();
    let name = payload.name.trim().to_string();

    if category_id.is_empty() || name.is_empty() {
        return Err(CategoryError::EmptySubcategoryName { action: ACTION }.into());
    }

    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.BEGIN_TRANSACTION")?;

    let cat_exists: Option<(String,)> = sqlx::query_as("SELECT id FROM categories WHERE id = ?")
        .bind(category_id)
        .fetch_optional(&mut *tx)
        .await
        .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.QUERY_PARENT_CATEGORY")?;

    if cat_exists.is_none() {
        return Err(CategoryError::CategoryNotFound {
            action: ACTION,
            id: category_id.to_string(),
        }
        .into());
    }

    let max_sort: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sort_order) FROM subcategories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(&mut *tx)
            .await
            .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.QUERY_MAX_SORT")?;
    let next_sort = max_sort.0.unwrap_or(0) + 1;

    let id = format!("sub-{}", &generate_token()[..10]);
    let now = now_epoch_secs();

    let insert_res = sqlx::query(
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
    .execute(&mut *tx)
    .await;

    match insert_res {
        Ok(_) => {
            tx.commit()
                .await
                .db_context("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.COMMIT_TRANSACTION")?;
            Ok(SubcategoryItem { id, name })
        }
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::SubcategoryAlreadyExists {
                    action: ACTION,
                    name,
                }
                .into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_SUBCATEGORY.INSERT", err))
            }
        }
    }
}

/// Updates the name of an existing subcategory.
pub(crate) async fn update_subcategory_name(
    pool: &DbPool,
    id: &str,
    payload: UpdateNameRequest,
) -> Result<(), AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.UPDATE_SUBCATEGORY_NAME";
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(CategoryError::EmptySubcategoryName { action: ACTION }.into());
    }

    let now = now_epoch_secs();
    let res = sqlx::query("UPDATE subcategories SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await;

    match res {
        Ok(exec) => {
            if exec.rows_affected() == 0 {
                Err(CategoryError::SubcategoryNotFound {
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
                Err(CategoryError::SubcategoryAlreadyExists {
                    action: ACTION,
                    name,
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

/// Deletes a subcategory.
pub(crate) async fn delete_subcategory(pool: &DbPool, id: &str) -> Result<(), AppError> {
    const ACTION: &str = "CONFIG.CATEGORIES.DELETE_SUBCATEGORY";
    let res = sqlx::query("DELETE FROM subcategories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .db_context("CONFIG.CATEGORIES.DELETE_SUBCATEGORY.EXECUTE")?;

    if res.rows_affected() == 0 {
        Err(CategoryError::SubcategoryNotFound {
            action: ACTION,
            id: id.to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
