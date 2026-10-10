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

/// Inserts a new manual transaction within an atomic multi-statement transaction block.
///
/// Workflow:
/// 1. Begins transaction (`pool.begin()`).
/// 2. If `member_id` is omitted, resolves `owner_member_id` from the target account.
/// 3. Inserts into `manual_transactions`.
/// 4. Inserts into `transaction_sources` (`source_type = 'manual'`).
/// 5. Inserts into master `transactions` using the generated source ID.
/// 6. Commits transaction and returns the populated [`TransactionDto`].
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
    let description = TransactionDescription::try_new(
        req.description().unwrap_or("Transaction"),
        "TRANSACTION.CREATE_MANUAL.VALIDATE_DESC",
    )?;
    let payee = TransactionPayee::new(req.payee()).into_inner();
    let status = req
        .status()
        .map(|s| TransactionStatus::try_new(s, "TRANSACTION.CREATE_MANUAL.PARSE_STATUS"))
        .transpose()?
        .unwrap_or(TransactionStatus::Cleared);

    let mut tx = pool
        .begin()
        .await
        .db_context("TRANSACTION.CREATE_MANUAL.TX_BEGIN")?;

    // Resolve member_id: if not provided in payload, inherit from the account's owner_member_id
    let member_id = match req.member_id() {
        Some(m) => m,
        None => {
            let row = sqlx::query(
                "SELECT owner_member_id FROM accounts WHERE id = ? AND family_id = ? LIMIT 1;",
            )
            .bind(req.account_id())
            .bind(family_id)
            .fetch_optional(&mut *tx)
            .await
            .db_context("TRANSACTION.CREATE_MANUAL.RESOLVE_ACCOUNT_MEMBER")?;

            match row {
                Some(r) => r.get::<i64, _>("owner_member_id"),
                None => {
                    return Err(TransactionError::AccountNotFound {
                        action: "TRANSACTION.CREATE_MANUAL.ACCOUNT_NOT_FOUND",
                        id: req.account_id(),
                    }
                    .into());
                }
            }
        }
    };

    let now = now_epoch_secs();

    // 1. Insert into manual_transactions
    let insert_manual_result = sqlx::query(
        r#"
        INSERT INTO manual_transactions (
            family_id, account_id, member_id, type_id, category_id, subcategory_id,
            amount, date, description, payee, notes, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(req.account_id())
    .bind(member_id)
    .bind(req.type_id())
    .bind(req.category_id())
    .bind(req.subcategory_id())
    .bind(amount.get())
    .bind(date.epoch_secs())
    .bind(description.into_inner())
    .bind(&payee)
    .bind(req.notes())
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await;

    let manual_id: i64 = match insert_manual_result {
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

    // 2. Insert into transaction_sources (generates master transaction ID)
    let source_row = sqlx::query(
        r#"
        INSERT INTO transaction_sources (
            family_id, source_type, staging_id, manual_id, created_at, updated_at
        )
        VALUES (?, ?, NULL, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(TransactionSourceType::Manual.as_str())
    .bind(manual_id)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .db_context("TRANSACTION.CREATE_MANUAL.INSERT_SOURCE")?;

    let transaction_id: i64 = source_row.get("id");

    // 3. Insert into transactions (inherits transaction_id from transaction_sources)
    let desc_for_ledger = req.description().unwrap_or("Transaction").trim();
    sqlx::query(
        r#"
        INSERT INTO transactions (
            id, family_id, account_id, member_id, type_id, category_id, subcategory_id,
            amount, date, description, payee, notes, status, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(transaction_id)
    .bind(family_id)
    .bind(req.account_id())
    .bind(member_id)
    .bind(req.type_id())
    .bind(req.category_id())
    .bind(req.subcategory_id())
    .bind(amount.get())
    .bind(date.epoch_secs())
    .bind(desc_for_ledger)
    .bind(&payee)
    .bind(req.notes())
    .bind(status.as_str())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .db_context("TRANSACTION.CREATE_MANUAL.INSERT_LEDGER")?;

    tx.commit()
        .await
        .db_context("TRANSACTION.CREATE_MANUAL.TX_COMMIT")?;

    get_transaction(pool, family_id, transaction_id)
        .await?
        .ok_or_else(|| AppError::ShouldNotBeHappening {
            action: "TRANSACTION.CREATE_MANUAL.FETCH_AFTER_COMMIT",
            reason: format!(
                "transaction with id {transaction_id} not found immediately after creation"
            ),
        })
}

/// Retrieves a single transaction by ID scoped strictly to the family.
pub(crate) async fn get_transaction(
    pool: &DbPool,
    family_id: i64,
    id: i64,
) -> Result<Option<TransactionDto>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            t.id,
            t.date,
            t.description,
            t.payee,
            t.amount,
            t.type_id,
            tt.type_name,
            c.hex AS type_color,
            t.member_id,
            t.account_id,
            t.category_id,
            t.subcategory_id,
            t.notes,
            t.status
        FROM transactions t
        LEFT JOIN transaction_types tt ON t.type_id = tt.id
        LEFT JOIN colors c ON tt.color_id = c.id
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
            date_iso,
            r.get("description"),
            r.get("payee"),
            r.get("amount"),
            r.get("type_id"),
            r.get("type_name"),
            r.get("type_color"),
            r.get("member_id"),
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
            t.date,
            t.description,
            t.payee,
            t.amount,
            t.type_id,
            tt.type_name,
            c.hex AS type_color,
            t.member_id,
            t.account_id,
            t.category_id,
            t.subcategory_id,
            t.notes,
            t.status
        FROM transactions t
        LEFT JOIN transaction_types tt ON t.type_id = tt.id
        LEFT JOIN colors c ON tt.color_id = c.id
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
                date_iso,
                r.get("description"),
                r.get("payee"),
                r.get("amount"),
                r.get("type_id"),
                r.get("type_name"),
                r.get("type_color"),
                r.get("member_id"),
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

/// Updates an existing transaction, redirecting provenance to manual entry if imported.
pub(crate) async fn update_transaction(
    pool: &DbPool,
    family_id: i64,
    id: i64,
    req: &UpdateTransactionRequest,
) -> Result<TransactionDto, AppError> {
    let mut tx = pool
        .begin()
        .await
        .db_context("TRANSACTION.UPDATE.TX_BEGIN")?;

    // Fetch existing transaction and source record
    let row = sqlx::query(
        r#"
        SELECT 
            t.id, t.family_id, t.account_id, t.member_id, t.type_id, t.category_id,
            t.subcategory_id, t.amount, t.date, t.description, t.payee, t.notes, t.status,
            ts.source_type, ts.manual_id
        FROM transactions t
        JOIN transaction_sources ts ON t.id = ts.id
        WHERE t.id = ? AND t.family_id = ?
        LIMIT 1;
        "#,
    )
    .bind(id)
    .bind(family_id)
    .fetch_optional(&mut *tx)
    .await
    .db_context("TRANSACTION.UPDATE.FETCH_EXISTING")?;

    let Some(r) = row else {
        return Err(TransactionError::TransactionNotFound {
            action: "TRANSACTION.UPDATE.NOT_FOUND",
            id,
        }
        .into());
    };

    let existing_date_epoch: i64 = r.get("date");
    let existing_amount: i64 = r.get("amount");
    let existing_description: String = r.get("description");
    let existing_payee: Option<String> = r.get("payee");
    let existing_account_id: Option<i64> = r.get("account_id");
    let existing_member_id: Option<i64> = r.get("member_id");
    let existing_type_id: Option<i64> = r.get("type_id");
    let existing_category_id: Option<i64> = r.get("category_id");
    let existing_subcategory_id: Option<i64> = r.get("subcategory_id");
    let existing_notes: Option<String> = r.get("notes");
    let existing_status: String = r.get("status");
    let source_type_raw: String = r.get("source_type");
    let source_type =
        TransactionSourceType::try_new(&source_type_raw, "TRANSACTION.UPDATE.PARSE_SOURCE_TYPE")?;
    let manual_id: Option<i64> = r.get("manual_id");

    // Compute updated fields
    let updated_date_epoch = match req.date() {
        Some(d) => TransactionDate::try_from_iso(d, "TRANSACTION.UPDATE.PARSE_DATE")?.epoch_secs(),
        None => existing_date_epoch,
    };
    let updated_amount = match req.amount() {
        Some(a) => TransactionAmount::try_new(a, "TRANSACTION.UPDATE.VALIDATE_AMOUNT")?.get(),
        None => existing_amount,
    };
    let updated_description = match req.description() {
        Some(d) => {
            TransactionDescription::try_new(d, "TRANSACTION.UPDATE.VALIDATE_DESC")?.into_inner()
        }
        None => existing_description,
    };
    let updated_payee = match req.payee() {
        Some(p) => TransactionPayee::new(Some(p)).into_inner(),
        None => existing_payee,
    };
    let updated_account_id = req.account_id().or(existing_account_id);
    let updated_member_id = req.member_id().or(existing_member_id);
    let updated_type_id = req.type_id().or(existing_type_id);
    let updated_category_id = req.category_id().or(existing_category_id);
    let updated_subcategory_id = req.subcategory_id().or(existing_subcategory_id);
    let updated_notes = req.notes().map(|n| n.to_string()).or(existing_notes);
    let updated_status = match req.status() {
        Some(s) => TransactionStatus::try_new(s, "TRANSACTION.UPDATE.PARSE_STATUS")?
            .as_str()
            .to_string(),
        None => existing_status,
    };

    let now = now_epoch_secs();

    // Lineage handling: if previously from 'import', insert a new manual record and switch source
    if source_type == TransactionSourceType::Import || manual_id.is_none() {
        let new_manual_row = sqlx::query(
            r#"
            INSERT INTO manual_transactions (
                family_id, account_id, member_id, type_id, category_id, subcategory_id,
                amount, date, description, payee, notes, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id;
            "#,
        )
        .bind(family_id)
        .bind(updated_account_id)
        .bind(updated_member_id)
        .bind(updated_type_id)
        .bind(updated_category_id)
        .bind(updated_subcategory_id)
        .bind(updated_amount)
        .bind(updated_date_epoch)
        .bind(&updated_description)
        .bind(&updated_payee)
        .bind(&updated_notes)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .db_context("TRANSACTION.UPDATE.INSERT_MANUAL")?;

        let new_manual_id: i64 = new_manual_row.get("id");

        sqlx::query(
            r#"
            UPDATE transaction_sources SET
                source_type = ?,
                manual_id = ?,
                updated_at = ?
            WHERE id = ? AND family_id = ?;
            "#,
        )
        .bind(TransactionSourceType::Manual.as_str())
        .bind(new_manual_id)
        .bind(now)
        .bind(id)
        .bind(family_id)
        .execute(&mut *tx)
        .await
        .db_context("TRANSACTION.UPDATE.SWITCH_SOURCE")?;
    } else if let Some(mid) = manual_id {
        sqlx::query(
            r#"
            UPDATE manual_transactions SET
                account_id = ?, member_id = ?, type_id = ?, category_id = ?, subcategory_id = ?,
                amount = ?, date = ?, description = ?, payee = ?, notes = ?, updated_at = ?
            WHERE id = ? AND family_id = ?;
            "#,
        )
        .bind(updated_account_id)
        .bind(updated_member_id)
        .bind(updated_type_id)
        .bind(updated_category_id)
        .bind(updated_subcategory_id)
        .bind(updated_amount)
        .bind(updated_date_epoch)
        .bind(&updated_description)
        .bind(&updated_payee)
        .bind(&updated_notes)
        .bind(now)
        .bind(mid)
        .bind(family_id)
        .execute(&mut *tx)
        .await
        .db_context("TRANSACTION.UPDATE.UPDATE_MANUAL")?;
    }

    // Update master ledger row
    sqlx::query(
        r#"
        UPDATE transactions SET
            account_id = ?, member_id = ?, type_id = ?, category_id = ?, subcategory_id = ?,
            amount = ?, date = ?, description = ?, payee = ?, notes = ?, status = ?, updated_at = ?
        WHERE id = ? AND family_id = ?;
        "#,
    )
    .bind(updated_account_id)
    .bind(updated_member_id)
    .bind(updated_type_id)
    .bind(updated_category_id)
    .bind(updated_subcategory_id)
    .bind(updated_amount)
    .bind(updated_date_epoch)
    .bind(&updated_description)
    .bind(&updated_payee)
    .bind(&updated_notes)
    .bind(&updated_status)
    .bind(now)
    .bind(id)
    .bind(family_id)
    .execute(&mut *tx)
    .await
    .db_context("TRANSACTION.UPDATE.UPDATE_LEDGER")?;

    tx.commit()
        .await
        .db_context("TRANSACTION.UPDATE.TX_COMMIT")?;

    get_transaction(pool, family_id, id)
        .await?
        .ok_or_else(|| AppError::ShouldNotBeHappening {
            action: "TRANSACTION.UPDATE.FETCH_AFTER_COMMIT",
            reason: format!("transaction with id {id} not found immediately after update"),
        })
}

/// Deletes a transaction and its manual provenance record.
pub(crate) async fn delete_transaction(
    pool: &DbPool,
    family_id: i64,
    id: i64,
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .db_context("TRANSACTION.DELETE.TX_BEGIN")?;

    let row = sqlx::query(
        "SELECT id, manual_id FROM transaction_sources WHERE id = ? AND family_id = ? LIMIT 1;",
    )
    .bind(id)
    .bind(family_id)
    .fetch_optional(&mut *tx)
    .await
    .db_context("TRANSACTION.DELETE.FETCH_SOURCE")?;

    let Some(r) = row else {
        return Err(TransactionError::TransactionNotFound {
            action: "TRANSACTION.DELETE.NOT_FOUND",
            id,
        }
        .into());
    };

    let manual_id: Option<i64> = r.get("manual_id");

    // Deleting from transaction_sources cascades to transactions table via ON DELETE CASCADE
    sqlx::query("DELETE FROM transaction_sources WHERE id = ? AND family_id = ?;")
        .bind(id)
        .bind(family_id)
        .execute(&mut *tx)
        .await
        .db_context("TRANSACTION.DELETE.DELETE_SOURCE")?;

    // Cleanup manual_transactions record if one existed
    if let Some(mid) = manual_id {
        sqlx::query("DELETE FROM manual_transactions WHERE id = ? AND family_id = ?;")
            .bind(mid)
            .bind(family_id)
            .execute(&mut *tx)
            .await
            .db_context("TRANSACTION.DELETE.DELETE_MANUAL")?;
    }

    tx.commit()
        .await
        .db_context("TRANSACTION.DELETE.TX_COMMIT")?;

    Ok(())
}
