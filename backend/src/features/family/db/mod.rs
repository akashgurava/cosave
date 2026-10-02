//! Database persistence and schema management for families, members, and accounts.
//!
//! Encapsulates all relational operations for the household domain slice:
//! - **Family & Member Lifecycle**: Persists household units, member rosters, and display metadata.
//! - **Account Management**: Supports depository bank accounts and revolving credit cards
//!   with relational cascade integrity tied to member ownership.
//! - **Schema Management & Seeding**: Idempotent DDL migrations and default household seeding.

mod accounts;
mod family;
mod members;
mod schema;

pub(crate) use family::seed_default_family;
pub(crate) use schema::init_family_schema;

pub(super) use accounts::{
    create_bank_account, create_credit_card, delete_account, update_bank_account,
    update_credit_card,
};
pub(super) use family::{get_default_currency, get_family_details, update_family};
pub(super) use members::{create_member, delete_member, update_member};

#[cfg(test)]
mod tests;
