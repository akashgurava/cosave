//! Database persistence and schema management for category taxonomies.
//!
//! This module encapsulates all database interactions for budget classification:
//!
//! - **Hierarchy Assembly**: Queries the denormalized `v_category_hierarchy` view to assemble
//!   the complete 3-tier classification tree (Transaction Types → Categories → Subcategories)
//!   paired with available palette colors.
//! - **Single-Shot Taxonomy Mutations**: Executes atomic SQL statements for adding types,
//!   categories, and subcategories, computing sequential sort positions within inline subqueries
//!   and mapping SQLite engine-level constraint violations (`UNIQUE`, foreign keys) to domain errors
//!   without multi-round-trip open transactions.
//! - **Schema Setup & Seeding**: Initializes required tables and views via [`init_category_schema`],
//!   seeds default household categories on first boot via [`seed_default_categories`], and provides
//!   an atomic transactional reset back to system defaults following Command-Query Separation.

mod categories;
mod colors;
mod hierarchy;
mod schema;
mod subcategories;
mod transaction_types;

pub(crate) use hierarchy::seed_default_categories;
pub(crate) use schema::init_category_schema;

pub(super) use categories::{create_category, delete_category, update_category_name};
pub(super) use colors::fetch_colors;
pub(super) use hierarchy::{fetch_hierarchy, reset_defaults};
pub(super) use subcategories::{create_subcategory, delete_subcategory, update_subcategory_name};
pub(super) use transaction_types::{create_type, delete_type, update_type_color};

#[cfg(test)]
mod tests;
