//! Transaction category taxonomy and cashflow classification for the application.
//!
//! This module provides hierarchical organization and visual categorization for financial data:
//!
//! - **3-Tier Classification**: Structures cashflows into an intuitive hierarchy: top-level
//!   transaction types (Income, Expense, Transfer), grouping categories (Housing, Food & Dining),
//!   and granular subcategories (Rent, Groceries).
//! - **Palette Color Association**: Links transaction types to selectable palette colors to power
//!   spending breakdowns, cashflow charts, and Sankey diagrams across frontend views.
//! - **Flexible Taxonomy Authoring**: Supports creating, renaming, and deleting types, categories, and subcategories
//!   via single-shot atomic statements with relational cascade integrity and duplicate prevention.
//! - **Default Seeding & Restoration**: Populates sensible household budgeting defaults on first boot,
//!   with the ability to atomically reset back to defaults at any time following Command-Query Separation.
//! - **Domain Error Handling**: Exposes [`CategoryError`] for descriptive validation failures and conflict reporting.

use axum::Router;

use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;

pub(crate) use db::{init_category_schema, seed_default_categories};
pub use error::CategoryError;

/// Returns the category hierarchy and configuration feature router mounted under `/config`.
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
