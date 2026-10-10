//! Transactions REST route handlers and Axum routing configuration.
//!
//! Exposes HTTP endpoints mounted under `/transactions` for querying the master household ledger,
//! recording manual transactions, updating transaction attributes with automatic lineage tracking,
//! and deleting transactions.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::core::{ApiResponse, AppError, AppState, Status};
use crate::features::auth::AuthUser;

use super::db;
use super::error::TransactionError;
use super::models::{
    CreateTransactionRequest, TransactionDto, TransactionFilterQuery, UpdateTransactionRequest,
};

/// Default household family identifier for single-tenant / local family deployments.
const DEFAULT_FAMILY_ID: i64 = 1;

/// Retrieves the list of transactions for the household matching optional filter criteria.
///
/// Canonical route: `GET /api/v1/transactions`
///
/// Supports query parameter filtering across free text (`query`/`q`), date ranges (`fromDate`, `toDate`),
/// categorization (`typeId`, `categoryId`, `subcategoryId`), account ownership (`accountId`, `memberId`),
/// and settlement status (`status`).
///
/// # Security & Access Control
/// - **Auth Requirement**: Public read for household overview / local single-tenant access.
/// - **Resource Scoping**: Scoped to the household family (`family_id = 1`).
///
/// # Returns
/// - `Ok(Json(ApiResponse<Vec<TransactionDto>>))`: 200 OK with list of transactions matching criteria.
async fn list_transactions(
    State(state): State<AppState>,
    Query(filters): Query<TransactionFilterQuery>,
) -> Result<Json<ApiResponse<Vec<TransactionDto>>>, AppError> {
    let transactions = db::list_transactions(state.db(), DEFAULT_FAMILY_ID, &filters).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), transactions)))
}

/// Retrieves a single transaction by ID.
///
/// Canonical route: `GET /api/v1/transactions/:id`
///
/// # Errors
/// - 404 Not Found: [`TransactionError::TransactionNotFound`] if the transaction does not exist.
async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<TransactionDto>>, AppError> {
    let tx = db::get_transaction(state.db(), DEFAULT_FAMILY_ID, id).await?;

    match tx {
        Some(item) => Ok(Json(ApiResponse::ok(Status::ok(), item))),
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
    _user: AuthUser,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TransactionDto>>), AppError> {
    let created = db::create_manual_transaction(state.db(), DEFAULT_FAMILY_ID, &payload).await?;
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
    _user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionDto>>, AppError> {
    let updated = db::update_transaction(state.db(), DEFAULT_FAMILY_ID, id, &payload).await?;
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
    _user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_transaction(state.db(), DEFAULT_FAMILY_ID, id).await?;
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
