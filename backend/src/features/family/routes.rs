//! Family and account REST route handlers.
//!
//! Exposes HTTP endpoints under `/config` for managing household settings,
//! member rosters, depository bank accounts, and credit cards. Read operations
//! allow overview assembly and dynamic currency resolution, while mutations
//! enforce authenticated operator access and parse-don't-validate domain invariants.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};

use crate::core::{ApiResponse, AppError, AppState, Status};
use crate::features::auth::AuthUser;

use super::db;
use super::models::{
    BankAccountDto, CreateBankAccountRequest, CreateCreditCardRequest, CreateMemberRequest,
    CreditCardDto, CurrencyCode, DefaultCurrencyDto, DefaultCurrencyQuery, FamilyDetailsDto,
    FamilyDto, FamilyName, MemberDto, UpdateBankAccountRequest, UpdateCreditCardRequest,
    UpdateFamilyRequest, UpdateMemberRequest,
};

/// Retrieves the complete family details including members and all financial accounts.
///
/// Canonical route: `GET /api/v1/config/family`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
///
/// # Returns
/// - `Ok(Json(ApiResponse<FamilyDetailsDto>))`: 200 OK with family details.
/// - `Err(AppError)`: Database error if query fails.
async fn get_family_details(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<FamilyDetailsDto>>, AppError> {
    let details = db::get_family_details(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), details)))
}

/// Updates the family name and/or base currency.
///
/// Canonical route: `PATCH /api/v1/config/family`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`UpdateFamilyRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<FamilyDto>))`: 200 OK with updated family representation.
/// - `Err(AppError)`: 400 Bad Request if fields invalid, 401 if unauthenticated.
async fn update_family(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<UpdateFamilyRequest>,
) -> Result<Json<ApiResponse<FamilyDto>>, AppError> {
    let name = payload
        .name()
        .map(|n| FamilyName::try_new(n, "FAMILY.ROUTE.UPDATE_FAMILY.NAME"))
        .transpose()?;

    let currency = payload
        .currency()
        .map(|c| CurrencyCode::try_new(c, "FAMILY.ROUTE.UPDATE_FAMILY.CURRENCY"))
        .transpose()?;

    let updated = db::update_family(state.db(), name, currency).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        family_id = %updated.id(),
        "FAMILY.ROUTE.UPDATE_FAMILY.SUCCESS. Family updated"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Resolves the default currency based on browser region and household configuration.
///
/// Canonical route: `GET /api/v1/config/currency/default`
/// Aliases: `GET /api/v1/config/family/currency/default`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `Query(query)`: Optional region parameter.
///
/// # Returns
/// - `Ok(Json(ApiResponse<DefaultCurrencyDto>))`: 200 OK with resolved currency code.
/// - `Err(AppError)`: Database error if query fails.
async fn get_default_currency(
    State(state): State<AppState>,
    Query(query): Query<DefaultCurrencyQuery>,
) -> Result<Json<ApiResponse<DefaultCurrencyDto>>, AppError> {
    let currency = db::get_default_currency(state.db(), query.region()).await?;
    Ok(Json(ApiResponse::ok(
        Status::ok(),
        DefaultCurrencyDto::new(currency),
    )))
}

/// Creates a new family member.
///
/// Canonical route: `POST /api/v1/config/members`
/// Aliases: `POST /api/v1/config/member`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`CreateMemberRequest`].
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<MemberDto>)))`: 201 Created with new member.
/// - `Err(AppError)`: 400 Bad Request if name empty, 401 if unauthenticated.
async fn create_member(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateMemberRequest>,
) -> Result<(StatusCode, Json<ApiResponse<MemberDto>>), AppError> {
    let created = db::create_member(state.db(), payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        member_id = %created.id(),
        "FAMILY.ROUTE.CREATE_MEMBER.SUCCESS. Member created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates an existing member's name.
///
/// Canonical route: `PATCH /api/v1/config/members/{id}`
/// Aliases: `PATCH /api/v1/config/member/{id}`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Member identifier.
/// - `Json(payload)`: Validated [`UpdateMemberRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<MemberDto>))`: 200 OK with updated member.
/// - `Err(AppError)`: 400 Bad Request if name empty, 404 if not found.
async fn update_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMemberRequest>,
) -> Result<Json<ApiResponse<MemberDto>>, AppError> {
    let updated = db::update_member(state.db(), id, payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        member_id = %id,
        "FAMILY.ROUTE.UPDATE_MEMBER.SUCCESS. Member updated"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Deletes a member and cascades removal to all owned accounts.
///
/// Canonical route: `DELETE /api/v1/config/members/{id}`
/// Aliases: `DELETE /api/v1/config/member/{id}`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Member identifier.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with confirmation.
/// - `Err(AppError)`: 404 Not Found if member does not exist.
async fn delete_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_member(state.db(), id).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        member_id = %id,
        "FAMILY.ROUTE.DELETE_MEMBER.SUCCESS. Member deleted"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Creates a new bank account.
///
/// Canonical route: `POST /api/v1/config/accounts/bank`
/// Aliases: `POST /api/v1/config/account/bank`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Validated [`CreateBankAccountRequest`].
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<BankAccountDto>)))`: 201 Created with bank account.
/// - `Err(AppError)`: 400 Bad Request if validation fails, 404 if owner not found.
async fn create_bank_account(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateBankAccountRequest>,
) -> Result<(StatusCode, Json<ApiResponse<BankAccountDto>>), AppError> {
    let created = db::create_bank_account(state.db(), payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        account_id = %created.id(),
        "FAMILY.ROUTE.CREATE_BANK.SUCCESS. Bank account created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates an existing bank account.
///
/// Canonical route: `PATCH /api/v1/config/accounts/bank/{id}`
/// Aliases: `PATCH /api/v1/config/account/bank/{id}`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Account identifier.
/// - `Json(payload)`: Inbound [`UpdateBankAccountRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<BankAccountDto>))`: 200 OK with updated bank account.
/// - `Err(AppError)`: 400 Bad Request if validation fails, 404 if account not found.
async fn update_bank_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateBankAccountRequest>,
) -> Result<Json<ApiResponse<BankAccountDto>>, AppError> {
    let updated = db::update_bank_account(state.db(), id, payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        account_id = %id,
        "FAMILY.ROUTE.UPDATE_BANK.SUCCESS. Bank account updated"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Creates a new credit card account.
///
/// Canonical route: `POST /api/v1/config/accounts/credit`
/// Aliases: `POST /api/v1/config/account/credit`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Json(payload)`: Inbound [`CreateCreditCardRequest`].
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<CreditCardDto>)))`: 201 Created with credit card.
/// - `Err(AppError)`: 400 Bad Request if validation fails, 404 if owner not found.
async fn create_credit_card(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCreditCardRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreditCardDto>>), AppError> {
    let created = db::create_credit_card(state.db(), payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        account_id = %created.id(),
        "FAMILY.ROUTE.CREATE_CREDIT.SUCCESS. Credit card created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(Status::ok(), created)),
    ))
}

/// Updates an existing credit card account.
///
/// Canonical route: `PATCH /api/v1/config/accounts/credit/{id}`
/// Aliases: `PATCH /api/v1/config/account/credit/{id}`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Account identifier.
/// - `Json(payload)`: Inbound [`UpdateCreditCardRequest`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<CreditCardDto>))`: 200 OK with updated credit card.
/// - `Err(AppError)`: 400 Bad Request if validation fails, 404 if account not found.
async fn update_credit_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCreditCardRequest>,
) -> Result<Json<ApiResponse<CreditCardDto>>, AppError> {
    let updated = db::update_credit_card(state.db(), id, payload).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        account_id = %id,
        "FAMILY.ROUTE.UPDATE_CREDIT.SUCCESS. Credit card updated"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), updated)))
}

/// Deletes a financial account by ID.
///
/// Canonical route: `DELETE /api/v1/config/accounts/{id}`
/// Aliases: `DELETE /api/v1/config/account/{id}`
///
/// # Ingress
/// - `State(state)`: Application state with database pool.
/// - `user`: Authenticated operator session context.
/// - `Path(id)`: Account identifier.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with confirmation.
/// - `Err(AppError)`: 404 Not Found if account does not exist.
async fn delete_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    db::delete_account(state.db(), id).await?;

    tracing::debug!(
        user_id = %user.user_id(),
        account_id = %id,
        "FAMILY.ROUTE.DELETE_ACCOUNT.SUCCESS. Account deleted"
    );

    Ok(Json(ApiResponse::ok(Status::ok(), ())))
}

/// Configures and returns the Axum router for family and account endpoints.
pub(super) fn router() -> Router<AppState> {
    Router::new()
        // Family details and settings
        .route("/family", get(get_family_details).patch(update_family))
        // Default currency resolution
        .route("/currency/default", get(get_default_currency))
        .route("/family/currency/default", get(get_default_currency))
        // Member roster
        .route("/members", post(create_member))
        .route("/member", post(create_member))
        .route("/members/{id}", patch(update_member).delete(delete_member))
        .route("/member/{id}", patch(update_member).delete(delete_member))
        // Bank accounts
        .route("/accounts/bank", post(create_bank_account))
        .route("/account/bank", post(create_bank_account))
        .route("/accounts/bank/{id}", patch(update_bank_account))
        .route("/account/bank/{id}", patch(update_bank_account))
        // Credit cards
        .route("/accounts/credit", post(create_credit_card))
        .route("/account/credit", post(create_credit_card))
        .route("/accounts/credit/{id}", patch(update_credit_card))
        .route("/account/credit/{id}", patch(update_credit_card))
        // Generic account deletion
        .route("/accounts/{id}", delete(delete_account))
        .route("/account/{id}", delete(delete_account))
}
