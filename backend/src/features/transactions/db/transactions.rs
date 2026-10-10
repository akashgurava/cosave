//! Relational database operations for transactions, manual entries, and lineage sources.
//!
//! Encapsulates multi-statement transactional workflows for creating, updating, querying,
//! and deleting financial transactions. Enforces strict family isolation, atomic rollback guarantees,
//! and lineage tracking via `transaction_sources`.

use sqlx::{QueryBuilder, Row, Sqlite};

use crate::core::{is_foreign_key_violation, now_epoch_secs, AppError, DbPool, DbResultExt};

use super::super::error::TransactionError;
use super::super::models::{
    CreateTransactionRequest, PaginatedTransactionsDto, TransactionAmount, TransactionDate,
    TransactionDescription, TransactionDto, TransactionFilterQuery, TransactionPayee,
    TransactionSourceType, TransactionStatus, UpdateTransactionRequest,
};

/// Inserts a new manual transaction into `manual_transactions` and returns the populated [`TransactionDto`].
///
/// Workflow:
/// 1. Validates input domain types.
/// 2. Inserts directly into `manual_transactions` table.
/// 3. Returns the row projected from the unified `transactions` view.
///
/// # Errors
/// Returns [`TransactionError::AccountNotFound`] if the specified account does not exist.
pub(crate) async fn create_manual_transaction(
    pool: &DbPool,
    family_id: i64,
    req: &CreateTransactionRequest,
) -> Result<TransactionDto, AppError> {
    let date = TransactionDate::try_from_iso(req.date(), "TRANSACTION.CREATE_MANUAL.PARSE_DATE")?;
    let amount =
        TransactionAmount::try_new(req.amount(), "TRANSACTION.CREATE_MANUAL.VALIDATE_AMOUNT")?;
    let description = TransactionDescription::new(req.description()).into_inner();
    let payee = TransactionPayee::new(req.payee()).into_inner();
    let status = req
        .status()
        .map(|s| TransactionStatus::try_new(s, "TRANSACTION.CREATE_MANUAL.PARSE_STATUS"))
        .transpose()?
        .unwrap_or(TransactionStatus::Cleared);

    let now = now_epoch_secs();
    let tx_id = uuid::Uuid::new_v4().to_string();

    let insert_result = sqlx::query(
        r#"
        INSERT INTO manual_transactions (
            id, family_id, account_id, type_id, category_id, subcategory_id,
            amount, date, description, payee, notes, status, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(&tx_id)
    .bind(family_id)
    .bind(req.account_id())
    .bind(req.type_id())
    .bind(req.category_id())
    .bind(req.subcategory_id())
    .bind(amount.get())
    .bind(date.epoch_secs())
    .bind(description)
    .bind(&payee)
    .bind(req.notes())
    .bind(status.as_str())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    let transaction_id: String = match insert_result {
        Ok(r) => r.get("id"),
        Err(e) if is_foreign_key_violation(&e) => {
            return Err(TransactionError::AccountNotFound {
                action: "TRANSACTION.CREATE_MANUAL.FK_VIOLATION",
                id: req.account_id(),
            }
            .into());
        }
        Err(e) => return Err(e).db_context("TRANSACTION.CREATE_MANUAL.INSERT_MANUAL")?,
    };

    get_transaction(pool, family_id, &transaction_id)
        .await?
        .ok_or_else(|| AppError::ShouldNotBeHappening {
            action: "TRANSACTION.CREATE_MANUAL.FETCH_AFTER_INSERT",
            reason: format!(
                "transaction with id {transaction_id} not found immediately after creation"
            ),
        })
}

/// Retrieves a single transaction by ID scoped strictly to the family from the unified `transactions` view.
pub(crate) async fn get_transaction(
    pool: &DbPool,
    family_id: i64,
    id: &str,
) -> Result<Option<TransactionDto>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            t.id,
            t.source,
            t.date,
            t.description,
            t.payee,
            t.amount,
            t.type_id,
            t.account_id,
            t.category_id,
            t.subcategory_id,
            t.notes,
            t.status
        FROM transactions t
        WHERE t.id = ? AND t.family_id = ?
        LIMIT 1;
        "#,
    )
    .bind(id)
    .bind(family_id)
    .fetch_optional(pool)
    .await
    .db_context("TRANSACTION.GET.QUERY")?;

    Ok(row.map(|r| {
        let date_epoch: i64 = r.get("date");
        let date_iso = TransactionDate::from_epoch_secs(date_epoch).to_iso_date();

        TransactionDto::new(
            r.get("id"),
            r.get("source"),
            date_iso,
            r.get("description"),
            r.get("payee"),
            r.get("amount"),
            r.get("type_id"),
            r.get("account_id"),
            r.get("category_id"),
            r.get("subcategory_id"),
            r.get("notes"),
            r.get("status"),
        )
    }))
}

/// Applies dynamic filter predicates to a transactions query builder.
fn apply_transaction_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    family_id: i64,
    filters: &'a TransactionFilterQuery,
) {
    builder.push(" WHERE t.family_id = ");
    builder.push_bind(family_id);

    if let Some(q) = filters.query() {
        let pattern = format!("%{}%", q.trim());
        builder.push(" AND (t.description LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR (t.payee IS NOT NULL AND t.payee LIKE ");
        builder.push_bind(pattern);
        builder.push("))");
    }

    if let Some(from_str) = filters.start_date() {
        if let Ok(from_date) = TransactionDate::try_from_iso(from_str, "TRANSACTION.LIST.FROM_DATE")
        {
            builder.push(" AND t.date >= ");
            builder.push_bind(from_date.epoch_secs());
        }
    }

    if let Some(to_str) = filters.to_date() {
        if let Ok(to_date) = TransactionDate::try_from_iso(to_str, "TRANSACTION.LIST.TO_DATE") {
            let end_of_day = to_date.epoch_secs() + 86_399;
            builder.push(" AND t.date <= ");
            builder.push_bind(end_of_day);
        }
    }

    if let Some(min_amt) = filters.amount_min() {
        builder.push(" AND ABS(t.amount) >= ");
        builder.push_bind(min_amt);
    }

    if let Some(max_amt) = filters.amount_max() {
        builder.push(" AND ABS(t.amount) <= ");
        builder.push_bind(max_amt);
    }

    if let Some(account_ids) = filters.account_ids() {
        if !account_ids.is_empty() {
            builder.push(" AND t.account_id IN (");
            let mut separated = builder.separated(", ");
            for aid in account_ids {
                separated.push_bind(aid);
            }
            separated.push_unseparated(")");
        }
    }

    if let Some(type_ids) = filters.type_ids() {
        if !type_ids.is_empty() {
            builder.push(" AND t.type_id IN (");
            let mut separated = builder.separated(", ");
            for tid in type_ids {
                separated.push_bind(tid);
            }
            separated.push_unseparated(")");
        }
    }

    if let Some(category_ids) = filters.category_ids() {
        if !category_ids.is_empty() {
            builder.push(" AND t.category_id IN (");
            let mut separated = builder.separated(", ");
            for cid in category_ids {
                separated.push_bind(cid);
            }
            separated.push_unseparated(")");
        }
    }

    if let Some(subcategory_ids) = filters.subcategory_ids() {
        if !subcategory_ids.is_empty() {
            builder.push(" AND t.subcategory_id IN (");
            let mut separated = builder.separated(", ");
            for scid in subcategory_ids {
                separated.push_bind(scid);
            }
            separated.push_unseparated(")");
        }
    }

    if let Some(statuses) = filters.statuses() {
        if !statuses.is_empty() {
            builder.push(" AND LOWER(t.status) IN (");
            let mut separated = builder.separated(", ");
            for st in statuses {
                separated.push_bind(st.trim().to_ascii_lowercase());
            }
            separated.push_unseparated(")");
        }
    }
}

/// Lists and filters transactions scoped strictly to the family with server-side pagination.
pub(crate) async fn list_transactions(
    pool: &DbPool,
    family_id: i64,
    filters: &TransactionFilterQuery,
) -> Result<PaginatedTransactionsDto, AppError> {
    // 1. Count matching transactions
    let mut count_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT COUNT(*) AS total FROM transactions t");
    apply_transaction_filters(&mut count_builder, family_id, filters);

    let count_query = count_builder.build();
    let count_row = count_query
        .fetch_one(pool)
        .await
        .db_context("TRANSACTION.LIST.COUNT_QUERY")?;
    let total_count: i64 = count_row.get("total");

    let page = filters.page();
    let page_size = filters.page_size();
    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count as u64).div_ceil(page_size as u64) as u32
    };

    // 2. Fetch sliced transactions
    let mut list_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
        SELECT 
            t.id,
            t.source,
            t.date,
            t.description,
            t.payee,
            t.amount,
            t.type_id,
            t.account_id,
            t.category_id,
            t.subcategory_id,
            t.notes,
            t.status
        FROM transactions t
        "#,
    );
    apply_transaction_filters(&mut list_builder, family_id, filters);

    list_builder.push(" ORDER BY t.date DESC, t.id DESC LIMIT ");
    list_builder.push_bind(page_size);
    list_builder.push(" OFFSET ");
    list_builder.push_bind(filters.offset());
    list_builder.push(";");

    let query = list_builder.build();
    let rows = query
        .fetch_all(pool)
        .await
        .db_context("TRANSACTION.LIST.QUERY")?;

    let dtos = rows
        .into_iter()
        .map(|r| {
            let date_epoch: i64 = r.get("date");
            let date_iso = TransactionDate::from_epoch_secs(date_epoch).to_iso_date();

            TransactionDto::new(
                r.get("id"),
                r.get("source"),
                date_iso,
                r.get("description"),
                r.get("payee"),
                r.get("amount"),
                r.get("type_id"),
                r.get("account_id"),
                r.get("category_id"),
                r.get("subcategory_id"),
                r.get("notes"),
                r.get("status"),
            )
        })
        .collect();

    Ok(PaginatedTransactionsDto::new(
        dtos,
        total_count,
        page,
        page_size,
        total_pages,
    ))
}

/// Updates an existing manual transaction with full attribute replacement.
///
/// If `req.source()` is not `"manual"` (e.g. `"import"`), rejects immediately with
/// [`TransactionError::UnsupportedOperation`].
pub(crate) async fn update_transaction(
    pool: &DbPool,
    family_id: i64,
    id: &str,
    req: &UpdateTransactionRequest,
) -> Result<TransactionDto, AppError> {
    if req.source() != "manual" {
        return Err(TransactionError::UnsupportedOperation {
            action: "TRANSACTION.UPDATE.UNSUPPORTED_SOURCE",
            reason: format!(
                "Editing transactions with source '{}' is currently unsupported",
                req.source()
            ),
        }
        .into());
    }

    let date = TransactionDate::try_from_iso(req.date(), "TRANSACTION.UPDATE.PARSE_DATE")?;
    let amount = TransactionAmount::try_new(req.amount(), "TRANSACTION.UPDATE.VALIDATE_AMOUNT")?;
    let description = TransactionDescription::new(req.description()).into_inner();
    let payee = TransactionPayee::new(req.payee()).into_inner();
    let status = req
        .status()
        .map(|s| TransactionStatus::try_new(s, "TRANSACTION.UPDATE.PARSE_STATUS"))
        .transpose()?
        .unwrap_or(TransactionStatus::Cleared);

    let now = now_epoch_secs();

    let result = sqlx::query(
        r#"
        UPDATE manual_transactions SET
            account_id = ?,
            type_id = ?,
            category_id = ?,
            subcategory_id = ?,
            amount = ?,
            date = ?,
            description = ?,
            payee = ?,
            notes = ?,
            status = ?,
            updated_at = ?
        WHERE id = ? AND family_id = ?;
        "#,
    )
    .bind(req.account_id())
    .bind(req.type_id())
    .bind(req.category_id())
    .bind(req.subcategory_id())
    .bind(amount.get())
    .bind(date.epoch_secs())
    .bind(description)
    .bind(&payee)
    .bind(req.notes())
    .bind(status.as_str())
    .bind(now)
    .bind(id)
    .bind(family_id)
    .execute(pool)
    .await
    .db_context("TRANSACTION.UPDATE.EXECUTE")?;

    if result.rows_affected() == 0 {
        return Err(TransactionError::TransactionNotFound {
            action: "TRANSACTION.UPDATE.NOT_FOUND",
            id: id.to_string(),
        }
        .into());
    }

    get_transaction(pool, family_id, id)
        .await?
        .ok_or_else(|| AppError::ShouldNotBeHappening {
            action: "TRANSACTION.UPDATE.FETCH_AFTER_UPDATE",
            reason: format!("transaction with id {id} not found immediately after update"),
        })
}

/// Deletes a transaction directly from its corresponding table based on `source`.
pub(crate) async fn delete_transaction(
    pool: &DbPool,
    family_id: i64,
    id: &str,
    source: &str,
) -> Result<(), AppError> {
    let source_type =
        TransactionSourceType::try_new(source, "TRANSACTION.DELETE.PARSE_SOURCE_TYPE")?;

    let result = match source_type {
        TransactionSourceType::Manual => {
            sqlx::query("DELETE FROM manual_transactions WHERE id = ? AND family_id = ?;")
                .bind(id)
                .bind(family_id)
                .execute(pool)
                .await
                .db_context("TRANSACTION.DELETE.MANUAL")?
        }
        TransactionSourceType::Import => {
            sqlx::query("DELETE FROM imported_transactions WHERE id = ? AND family_id = ?;")
                .bind(id)
                .bind(family_id)
                .execute(pool)
                .await
                .db_context("TRANSACTION.DELETE.IMPORT")?
        }
    };

    if result.rows_affected() == 0 {
        return Err(TransactionError::TransactionNotFound {
            action: "TRANSACTION.DELETE.NOT_FOUND",
            id: id.to_string(),
        }
        .into());
    }

    Ok(())
}
