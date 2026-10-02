//! Database persistence and schema management for families, members, and accounts.
//!
//! Encapsulates all relational operations for the household domain slice:
//!
//! - **Household Overview Assembly**: Queries family metadata, member rosters, and accounts
//!   (depository bank accounts and revolving credit cards) to construct [`FamilyDetailsDto`].
//! - **Single-Shot Atomic Mutations**: Executes atomic `INSERT`, `UPDATE`, and `DELETE` SQL
//!   statements, mapping SQLite engine-level constraint violations (e.g. `UNIQUE(family_id, member_name)`,
//!   `UNIQUE(owner_member_id, account_name)`) to strongly typed [`FamilyError`] variants
//!   without multi-round-trip open transactions.
//! - **Referential Integrity & Cascades**: Enforces SQLite foreign key constraints where deleting
//!   a member cascades to all their owned financial accounts (`ON DELETE CASCADE`), while preserving
//!   household integrity.
//! - **Schema Management & Seeding**: Idempotently initializes domain tables and indexes via
//!   [`init_family_schema`] and seeds the default initial household on first boot via [`seed_default_family`].

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
