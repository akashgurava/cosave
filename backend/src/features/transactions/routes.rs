//! Transactions REST route handlers and Axum routing configuration.
//!
//! Exposes HTTP endpoints mounted under `/transactions` for querying the master family ledger,
//! recording manual transactions, updating transaction attributes with automatic lineage tracking,
//! and deleting transactions.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::core::{ApiResponse, AppError, AppState, Status};
use crate::features::auth::AuthUser;

use super::db;
use super::error::TransactionError;
use super::models::{
    CreateTransactionRequest, PaginatedTransactionsDto, TransactionDto, TransactionFilterQuery,
    UpdateTransactionRequest,
};

/// Default family identifier for single-tenant / local family deployments.
const DEFAULT_FAMILY_ID: i64 = 1;

/// Retrieves a paginated list of transactions for the family matching optional filter criteria.
///
/// Canonical route: `GET /api/v1/transactions`
///
/// Supports query parameter filtering across free text (`query`/`q`), date ranges (`fromDate`, `toDate`),
/// amount bounds (`minAmount`, `maxAmount`), and multi-select filters (`accountIds`, `typeIds`, `categoryIds`,
/// `subcategoryIds`, `statuses`), along with server-side pagination (`page`, `pageSize`).
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated user session [`AuthUser`].
/// - **Resource Scoping**: Scoped to the family (`family_id = 1`).
///
/// # Returns
/// - `Ok(Json(ApiResponse<PaginatedTransactionsDto>))`: 200 OK with paginated envelope.
/// - 401 Unauthorized: unauthenticated session token missing or expired.
async fn list_transactions(
    State(state): State<AppState>,
    user: AuthUser,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ApiResponse<PaginatedTransactionsDto>>, AppError> {
    let query_str = raw_query.unwrap_or_default();
    let filters = TransactionFilterQuery::from_query_str(&query_str);
    let paginated = db::list_transactions(state.db(), DEFAULT_FAMILY_ID, &filters).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        count = paginated.items().len(),
        total = paginated.total_count(),
        page = paginated.page(),
        page_size = paginated.page_size(),
        total_pages = paginated.total_pages(),
        "TRANSACTIONS.ROUTE.LIST.SUCCESS. Listed transactions matching filter"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), paginated)))
}

/// Retrieves a single transaction by ID.
///
/// Canonical route: `GET /api/v1/transactions/:id`
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated user session [`AuthUser`].
/// - **Resource Scoping**: Scoped to the family (`family_id = 1`).
///
/// # Errors
/// - 401 Unauthorized: unauthenticated session token missing or expired.
/// - 404 Not Found: [`TransactionError::TransactionNotFound`] if the transaction does not exist.
async fn get_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<TransactionDto>>, AppError> {
    let tx = db::get_transaction(state.db(), DEFAULT_FAMILY_ID, id).await?;

    match tx {
        Some(item) => {
            tracing::debug!(
                user_id = %user.user_id(),
                transaction_id = %item.id(),
                "TRANSACTIONS.ROUTE.GET.SUCCESS. Transaction retrieved"
            );
            Ok(Json(ApiResponse::ok(Status::ok(), item)))
        }
        None => Err(TransactionError::TransactionNotFound {
            action: "TRANSACTION.ROUTE.GET.NOT_FOUND",
            id,
        }
        .into()),
    }
}

/// Records a new manual transaction within an atomic transaction boundary.
///
/// Canonical route: `POST /api/v1/transactions`
///
/// Requires an authenticated session. Enforces domain validation on date, non-zero amount,
/// and non-empty description. Writes to `manual_transactions`, creates the master `transaction_sources`
/// identifier, and populates the master `transactions` ledger.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated user session [`AuthUser`].
///
/// # Errors
/// - 400 Bad Request: validation failures on date, amount, description, or status.
/// - 401 Unauthorized: missing or invalid session credentials.
/// - 404 Not Found: referenced account, member, type, or category does not exist.
async fn create_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TransactionDto>>), AppError> {
    let created = db::create_manual_transaction(state.db(), DEFAULT_FAMILY_ID, &payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        transaction_id = %created.id(),
        amount = %created.amount(),
        "TRANSACTIONS.ROUTE.CREATE.SUCCESS. Manual transaction created"
    );
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Modifies an existing transaction.
///
/// Canonical route: `PATCH /api/v1/transactions/:id`
///
/// Requires an authenticated session. If the target transaction originated from a statement import,
/// this mutation automatically provisions a new manual record and redirects the source discriminator
/// to `'manual'`, preserving historical lineage.
///
/// # Errors
/// - 400 Bad Request: validation failures on updated attributes.
/// - 401 Unauthorized: missing or invalid session credentials.
/// - 404 Not Found: [`TransactionError::TransactionNotFound`] if target does not exist.
async fn update_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionDto>>, AppError> {
    let updated = db::update_transaction(state.db(), DEFAULT_FAMILY_ID, id, &payload).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        transaction_id = %updated.id(),
        "TRANSACTIONS.ROUTE.UPDATE.SUCCESS. Transaction updated"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Deletes a transaction and cleans up its manual record.
///
/// Canonical route: `DELETE /api/v1/transactions/:id`
///
/// Requires an authenticated session. Deleting from `transaction_sources` cascades through
/// SQLite foreign key constraints to the `transactions` ledger row.
///
/// # Errors
/// - 401 Unauthorized: missing or invalid session credentials.
/// - 404 Not Found: [`TransactionError::TransactionNotFound`] if target does not exist.
async fn delete_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_transaction(state.db(), DEFAULT_FAMILY_ID, id).await?;
    tracing::debug!(
        user_id = %user.user_id(),
        transaction_id = %id,
        "TRANSACTIONS.ROUTE.DELETE.SUCCESS. Transaction deleted"
    );
    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Returns the transactions feature router mounted under `/transactions`.
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_transactions).post(create_transaction))
        .route(
            "/{id}",
            get(get_transaction)
                .patch(update_transaction)
                .delete(delete_transaction),
        )
}
