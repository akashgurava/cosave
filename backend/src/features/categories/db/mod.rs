//! Database persistence and schema management for category taxonomies.
//!
//! This module encapsulates all database interactions for budget classification:
//!
//! - **Hierarchy Assembly**: Queries the database view to assemble the full classification tree
//!   (Transaction Types → Categories → Subcategories) paired with available palette colors.
//! - **Taxonomy Mutations**: Executes scoped database operations for adding, updating, and removing
//!   types, categories, and subcategories, enforcing cascading deletions across children and preserving
//!   palette color integrity via foreign key restrictions.
//! - **Schema Setup & Seeding**: Initializes required tables and views via [`init_category_schema`],
//!   seeds default household categories on first boot via [`seed_default_categories`], and provides
//!   atomic transactional resets back to system defaults.

mod categories;
mod colors;
mod hierarchy;
mod schema;
mod subcategories;
mod transaction_types;
mod util;

pub(crate) use hierarchy::seed_default_categories;
pub(crate) use schema::init_category_schema;

pub(super) use categories::{create_category, delete_category, update_category_name};
pub(super) use colors::fetch_colors;
pub(super) use hierarchy::{fetch_hierarchy, reset_defaults};
pub(super) use subcategories::{create_subcategory, delete_subcategory, update_subcategory_name};
pub(super) use transaction_types::{create_type, delete_type, update_type_color};

#[cfg(test)]
mod tests;
