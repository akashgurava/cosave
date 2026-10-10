//! Transactions domain feature subsystem.
//!
//! Coordinates statement ingestion, staging normalization, and the master financial ledger:
//! - **Statement Imports**: Batch metadata tracking for uploaded statement files.
//! - **Raw Rows**: Immutable, unbiased ingestion of raw statement lines as JSON key-value mappings.
//! - **Staging**: Normalized statement rows sharing 1:1 primary key identity with raw rows for deduplication.
//! - **Transactions**: Canonical master family ledger preserving financial history with resilient classification links.

use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;

pub(crate) use db::init_transaction_schema;
pub use error::TransactionError;

/// Returns the transactions feature router mounted under `/transactions`.
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
