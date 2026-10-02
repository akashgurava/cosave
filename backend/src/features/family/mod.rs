//! Family unit, member roster, and financial accounts management.
//!
//! Exposes the household management feature slice:
//! - **Family Overview**: Complete tree of family details, members, and owned bank and credit card accounts.
//! - **Relational Cascades**: Member deletion cascades to owned financial accounts.
//! - **Multi-Currency SSOT**: Explicit base currency and account currencies without assumptions.
//! - **Domain Error Handling**: Exposes [`FamilyError`] for validation and constraint violations.

use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;

pub(crate) use db::{init_family_schema, seed_default_family};
pub use error::FamilyError;

/// Returns the family and accounts feature router mounted under `/config`.
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
