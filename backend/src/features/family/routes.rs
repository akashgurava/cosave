//! Family household roster and financial accounts REST route handlers.
//!
//! Exposes HTTP endpoints mounted under `/config` for managing household settings,
//! member rosters, depository bank accounts, and credit cards. Read operations
//! allow overview assembly and dynamic currency resolution, while mutations
//! enforce authenticated operator access and parse-don't-validate domain invariants.
//!
//! # Architecture & CQS Design
//! - **Unified Namespace & Route Ergonomics**: Mounted under `/config`, providing canonical
//!   entry points (`GET /api/v1/config/family`) alongside singular/plural aliases (`/member`,
//!   `/members`, `/account/*`, `/accounts/*`) for client flexibility.
//! - **Command-Query Separation (CQS)**: Deletions return lean acknowledgement envelopes
//!   (`ApiResponse<()>`) rather than duplicating expensive multi-table hierarchy aggregations,
//!   eliminating read amplification across high-frequency write operations.
//! - **Strict Error Envelopes**: Handlers delegate persistence to atomic database routines,
//!   mapping domain validation failures and constraint collisions to structured error envelopes.

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
    CreditCardDto, CurrencyDto, DefaultCurrencyDto, DefaultCurrencyQuery, FamilyDetailsDto,
    FamilyDto, FamilyName, MemberDto, UpdateBankAccountRequest, UpdateCreditCardRequest,
    UpdateFamilyRequest, UpdateMemberRequest,
};

/// Retrieves the complete family details including members and all financial accounts.
///
/// Canonical route: `GET /api/v1/config/family`
///
/// Publicly accessible without authentication to allow frontend dashboards, settings views,
/// and initial configuration wizards to render household composition and linked instruments.
///
/// # Security & Access Control
/// - **Auth Requirement**: None (public read).
/// - **Role Authorization**: Public.
/// - **Resource Scoping**: Global primary household entity.
///
/// # Ingress
/// - `State(state)`: Application state containing the shared database connection pool [`DbPool`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<FamilyDetailsDto>))`: 200 OK with family metadata, member roster, and accounts.
///
/// # Errors
/// - 404 Not Found: [`FamilyError::FamilyNotFound`] if no household record exists.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
async fn get_family_details(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<FamilyDetailsDto>>, AppError> {
    let details = db::get_family_details(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), details)))
}

/// Updates the family display name and/or base currency.
///
/// Canonical route: `PATCH /api/v1/config/family`
///
/// Requires an authenticated session. Updates the primary household entity's display
/// name and base reporting currency without touching member rosters or accounts.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Member or Admin.
/// - **Resource Scoping**: Primary household entity.
///
/// # Ingress
/// - `State(state)`: Application state containing [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Json(payload)`: Inbound [`UpdateFamilyRequest`] with optional `family_name` and `currency_id`.
///
/// # Returns
/// - `Ok(Json(ApiResponse<FamilyDto>))`: 200 OK with updated family representation.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyFamilyName`] if family name is empty/whitespace.
/// - 401 Unauthorized: unauthenticated session token missing or expired.
/// - 404 Not Found: [`FamilyError::CurrencyNotFound`] if the specified currency ID does not exist.
/// - 409 Conflict: [`FamilyError::FamilyAlreadyExists`] if new family name collides with another household.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
async fn update_family(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<UpdateFamilyRequest>,
) -> Result<Json<ApiResponse<FamilyDto>>, AppError> {
    let name = payload
        .family_name()
        .map(|n| FamilyName::try_new(n, "FAMILY.ROUTE.UPDATE_FAMILY.NAME"))
        .transpose()?;

    let currency_id = payload.currency_id();

    let updated = db::update_family(state.db(), name, currency_id).await?;

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
/// Consults existing household configuration, falling back to regional inference based on
/// country code without assuming USD as an arbitrary default.
///
/// # Security & Access Control
/// - **Auth Requirement**: None (public read).
/// - **Role Authorization**: Public.
///
/// # Ingress
/// - `State(state)`: Application state containing [`DbPool`].
/// - `Query(query)`: Inbound [`DefaultCurrencyQuery`] containing optional ISO region code.
///
/// # Returns
/// - `Ok(Json(ApiResponse<DefaultCurrencyDto>))`: 200 OK with resolved 3-letter currency code.
///
/// # Errors
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Retrieves all supported currencies loaded from authoritative backend configuration.
///
/// Canonical route: `GET /api/v1/config/currencies`
/// Aliases: `GET /api/v1/config/family/currencies`
///
/// Publicly accessible without authentication. Returns the list of standard supported currencies
/// with symbol, name, and scale loaded from `default_currency.json`.
///
/// # Security & Access Control
/// - **Auth Requirement**: None (public read).
/// - **Role Authorization**: Public.
///
/// # Ingress
/// - `State(state)`: Application state containing [`DbPool`].
///
/// # Returns
/// - `Ok(Json(ApiResponse<Vec<CurrencyDto>>))`: 200 OK with list of supported currencies.
async fn get_supported_currencies(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CurrencyDto>>>, AppError> {
    let currencies = db::get_supported_currencies(state.db()).await?;
    Ok(Json(ApiResponse::ok(Status::ok(), currencies)))
}

/// Creates a new family member within the household roster.
///
/// Canonical route: `POST /api/v1/config/members`
/// Aliases: `POST /api/v1/config/member`
///
/// Requires authentication. Inserts a new member scoped to the family, enforcing
/// unique member names within the household.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
/// - **Resource Scoping**: Scoped to the target household family.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Json(payload)`: Validated [`CreateMemberRequest`] containing member name and optional family ID.
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<MemberDto>)))`: 201 Created with new member entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyMemberName`] if member name is empty or whitespace-only.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::FamilyNotFound`] if the specified family does not exist.
/// - 409 Conflict: [`FamilyError::MemberAlreadyExists`] if a member with this name already exists in the family.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Updates an existing member's display name.
///
/// Canonical route: `PATCH /api/v1/config/members/{id}`
/// Aliases: `PATCH /api/v1/config/member/{id}`
///
/// Requires authentication. Modifies the member's name while verifying it does not
/// collide with another member in the same household.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target member 64-bit integer identifier.
/// - `Json(payload)`: Validated [`UpdateMemberRequest`] with trimmed member name.
///
/// # Returns
/// - `Ok(Json(ApiResponse<MemberDto>))`: 200 OK with updated member entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyMemberName`] if new member name is empty or whitespace-only.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::MemberNotFound`] if the member does not exist.
/// - 409 Conflict: [`FamilyError::MemberAlreadyExists`] if the new name collides with another member in the same family.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Deletes a member and cascades removal to all owned financial accounts.
///
/// Canonical route: `DELETE /api/v1/config/members/{id}`
/// Aliases: `DELETE /api/v1/config/member/{id}`
///
/// Requires authentication. Deletes the target member entity. Database foreign key cascades
/// automatically purge all depository and revolving accounts owned by this member. In accordance
/// with Command-Query Separation (CQS), returns `ApiResponse<()>` without read amplification.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target member 64-bit integer identifier to remove.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with confirmation (`data: null`).
///
/// # Errors
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::MemberNotFound`] if the member does not exist.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Creates a new manually tracked depository checking or savings bank account.
///
/// Canonical route: `POST /api/v1/config/accounts/bank`
/// Aliases: `POST /api/v1/config/account/bank`
///
/// Requires authentication. Inserts a new bank account scoped under an owning family member,
/// enforcing uniqueness of account names per member.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Json(payload)`: Inbound [`CreateBankAccountRequest`] containing owner member ID, bank name, account name, currency ID, and initial balance cents.
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<BankAccountDto>)))`: 201 Created with created bank account entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyBankName`], [`FamilyError::EmptyAccountName`], [`FamilyError::InvalidLast4`], or [`FamilyError::NegativeAmount`] if validation fails.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::MemberNotFound`] if the owner member does not exist, or [`FamilyError::CurrencyNotFound`] if the currency does not exist.
/// - 409 Conflict: [`FamilyError::AccountAlreadyExists`] if an account with this name already exists for this member.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Updates an existing depository bank account's details and balances.
///
/// Canonical route: `PATCH /api/v1/config/accounts/bank/{id}`
/// Aliases: `PATCH /api/v1/config/account/bank/{id}`
///
/// Requires authentication. Updates bank name, account name, currency, and balance cents,
/// verifying uniqueness against sibling accounts owned by the same member.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target bank account 64-bit integer identifier.
/// - `Json(payload)`: Inbound [`UpdateBankAccountRequest`] containing updated fields.
///
/// # Returns
/// - `Ok(Json(ApiResponse<BankAccountDto>))`: 200 OK with updated bank account entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyBankName`], [`FamilyError::EmptyAccountName`], [`FamilyError::InvalidLast4`], or [`FamilyError::NegativeAmount`] if validation fails.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::AccountNotFound`] if the account does not exist or is not a bank account, or [`FamilyError::CurrencyNotFound`] if the currency does not exist.
/// - 409 Conflict: [`FamilyError::AccountAlreadyExists`] if the new account name collides with another account for this owner.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Creates a new manually tracked revolving credit card account.
///
/// Canonical route: `POST /api/v1/config/accounts/credit`
/// Aliases: `POST /api/v1/config/account/credit`
///
/// Requires authentication. Inserts a new credit card account scoped under an owning family
/// member, enforcing 4-digit last4 requirements and name uniqueness per member.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Json(payload)`: Inbound [`CreateCreditCardRequest`] containing owner member ID, card name, last 4 digits, currency ID, and optional credit limit.
///
/// # Returns
/// - `Ok((StatusCode::CREATED, Json(ApiResponse<CreditCardDto>)))`: 201 Created with created credit card entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyCardName`], [`FamilyError::InvalidLast4`], or [`FamilyError::NegativeAmount`] if validation fails.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::MemberNotFound`] if the owner member does not exist, or [`FamilyError::CurrencyNotFound`] if the currency does not exist.
/// - 409 Conflict: [`FamilyError::AccountAlreadyExists`] if a card/account with this name already exists for this member.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Updates an existing credit card account's details and limits.
///
/// Canonical route: `PATCH /api/v1/config/accounts/credit/{id}`
/// Aliases: `PATCH /api/v1/config/account/credit/{id}`
///
/// Requires authentication. Updates card name, last 4 digits, currency, and credit limit cents,
/// verifying uniqueness against sibling accounts owned by the same member.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target credit card account 64-bit integer identifier.
/// - `Json(payload)`: Inbound [`UpdateCreditCardRequest`] containing updated fields.
///
/// # Returns
/// - `Ok(Json(ApiResponse<CreditCardDto>))`: 200 OK with updated credit card entity.
///
/// # Errors
/// - 400 Bad Request: [`FamilyError::EmptyCardName`], [`FamilyError::InvalidLast4`], or [`FamilyError::NegativeAmount`] if validation fails.
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::AccountNotFound`] if the account does not exist or is not a credit card, or [`FamilyError::CurrencyNotFound`] if the currency does not exist.
/// - 409 Conflict: [`FamilyError::AccountAlreadyExists`] if the new card name collides with another account for this owner.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Deletes a depository or revolving financial account by ID.
///
/// Canonical route: `DELETE /api/v1/config/accounts/{id}`
/// Aliases: `DELETE /api/v1/config/account/{id}`
///
/// Requires authentication. Removes the target financial account entity by its 64-bit primary key.
/// In accordance with Command-Query Separation (CQS), returns `ApiResponse<()>` without read
/// amplification.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context [`AuthUser`].
/// - **Role Authorization**: Public / Member / Admin.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target account 64-bit integer identifier to remove.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with deletion confirmation (`data: null`).
///
/// # Errors
/// - 401 Unauthorized: [`AppError::Unauthorized`] if session token is missing or expired.
/// - 404 Not Found: [`FamilyError::AccountNotFound`] if the account does not exist.
/// - 500 Internal Server Error: [`AppError::ShouldNotBeHappening`] on database failure.
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

/// Configures and returns the Axum router for family, member, and account endpoints.
///
/// Mounts route definitions under canonical paths with plural and singular resource aliases:
/// - Family configuration: `/family` (`GET`, `PATCH`)
/// - Default currency inference: `/currency/default`, `/family/currency/default` (`GET`)
/// - Member roster: `/members`, `/member` (`POST`), `/members/{id}`, `/member/{id}` (`PATCH`, `DELETE`)
/// - Bank accounts: `/accounts/bank`, `/account/bank` (`POST`), `/accounts/bank/{id}`, `/account/bank/{id}` (`PATCH`)
/// - Credit cards: `/accounts/credit`, `/account/credit` (`POST`), `/accounts/credit/{id}`, `/account/credit/{id}` (`PATCH`)
/// - Account removal: `/accounts/{id}`, `/account/{id}` (`DELETE`)
pub(super) fn router() -> Router<AppState> {
    Router::new()
        // Family details and settings
        .route(
            "/family",
            get(get_family_details)
                .patch(update_family)
                .post(update_family),
        )
        // Default currency resolution and supported currencies
        .route("/currency/default", get(get_default_currency))
        .route("/family/currency/default", get(get_default_currency))
        .route("/currencies", get(get_supported_currencies))
        .route("/family/currencies", get(get_supported_currencies))
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
