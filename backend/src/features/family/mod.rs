//! Family unit, membership rosters, and financial accounts subsystem.
//!
//! This module coordinates multi-member family management and account tracking:
//!
//! - **Family Entity & Base Currency**: Establishes the authoritative family unit with
//!   customizable display naming and base currency. Provides dynamic currency resolution
//!   based on regional headers with fallback to family configuration without default assumptions.
//! - **Member Rosters & Lifecycle**: Supports onboarding individual family members with uniqueness
//!   enforced per family. Member removals cascade relational cleanup across owned accounts.
//! - **Account Ownership & Instrument Discrimination**: Persists depository bank accounts and
//!   revolving credit cards under a unified relational table. Enforces member-level ownership,
//!   independent account-level currencies, and credit calculation invariants (limit, available, outstanding).
//! - **Single-Shot Operations & Integrity**: Workflows execute as atomic single-shot operations with
//!   immediate constraint classification ([`is_unique_violation`], [`is_foreign_key_violation`]).
//! - **Default Seeding & Idempotent Bootstrap**: Provisions canonical starter family entities
//!   idempotently on initial startup via [`seed_default_family`].
//! - **Domain Error Handling**: Exposes [`FamilyError`] with screaming machine-readable codes and
//!   compile-time action tokens pinpointing failure locations.

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
