//! Database schema migrations and DDL definitions for the category subsystem.
//!
//! Initializes relational tables for palette colors, root transaction types, intermediate
//! categories, and leaf subcategories within an active transaction. Foreign key constraints
//! enforce cascading deletions down the hierarchy while restricting the deletion of active
//! palette colors. The schema also provisions the denormalized `v_category_hierarchy` view
//! to optimize multi-tier tree queries.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Initializes the category taxonomy schema, indexes, and denormalized hierarchy view.
///
/// Provisions the relational structures for the 3-tier financial category taxonomy:
/// palette colors (`colors`), root transaction types (`transaction_types`), intermediate
/// categories (`categories`), leaf subcategories (`subcategories`), and the flattened
/// read projection `v_category_hierarchy`.
///
/// # Domain Rules & Referential Integrity
/// - **Hierarchical Cascade**: Deleting a transaction type cascades through its child categories
///   and subcategories (`ON DELETE CASCADE`). Deleting a category cascades to its subcategories.
/// - **Color Protection**: Deleting a palette color is restricted (`ON DELETE RESTRICT`) if any
///   transaction type currently references it.
/// - **Scoped Uniqueness**: Root transaction type names are globally unique (`UNIQUE(type_name)`),
///   while category and subcategory names are scoped to their immediate parent (`UNIQUE(type_id, category_name)`
///   and `UNIQUE(category_id, subcategory_name)`).
/// - **Fast Hierarchy Traversal**: Dedicated foreign-key indexes (`idx_categories_type_id`,
///   `idx_subcategories_category_id`) optimize parent-child joins and cascading deletes.
/// - **Pre-Sorted Denormalized View**: The `v_category_hierarchy` view pre-joins types, categories,
///   subcategories, and color hexes ordered by hierarchy sort orders for read queries.
///
/// # Execution & Idempotency
/// - Executes atomically within the caller-provided [`Transaction`].
/// - Idempotent across restarts using `CREATE ... IF NOT EXISTS` DDL.
/// - Each object creation is tracked via [`create_db_object`] under granular action tokens
///   (`CONFIG.CATEGORIES.INIT_SCHEMA.*`) for precise error pinpointing.
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL statement fails to execute.
pub(crate) async fn init_category_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), AppError> {
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.COLORS_TABLE",
        "colors",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS colors (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT UNIQUE NOT NULL,
            hex TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.TRANSACTION_TYPES_TABLE",
        "transaction_types",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS transaction_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            type_name TEXT UNIQUE NOT NULL,
            color_id INTEGER NOT NULL REFERENCES colors(id) ON DELETE RESTRICT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE",
        "categories",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            type_id INTEGER NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
            category_name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, category_name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX_TYPE_ID",
        "categories",
        tx,
        "CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);",
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.SUBCATEGORIES_TABLE",
        "subcategories",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS subcategories (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
            subcategory_name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(category_id, subcategory_name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.SUBCATEGORIES_INDEX_CATEGORY_ID",
        "subcategories",
        tx,
        "CREATE INDEX IF NOT EXISTS idx_subcategories_category_id ON subcategories(category_id);",
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.V_CATEGORY_HIERARCHY_VIEW",
        "v_category_hierarchy",
        tx,
        r#"
        CREATE VIEW IF NOT EXISTS v_category_hierarchy AS
        SELECT
            t.color_id AS type_color_id,
            col.hex AS type_color,
            t.id AS type_id,
            t.type_name AS type_name,
            t.sort_order AS type_sort_order,
            c.id AS category_id,
            c.category_name AS category_name,
            c.sort_order AS category_sort_order,
            s.id AS subcategory_id,
            s.subcategory_name AS subcategory_name,
            s.sort_order AS subcategory_sort_order
        FROM transaction_types t
        JOIN colors col ON col.id = t.color_id
        LEFT JOIN categories c ON c.type_id = t.id
        LEFT JOIN subcategories s ON s.category_id = c.id
        ORDER BY t.sort_order, t.type_name, c.sort_order, c.category_name, s.sort_order, s.subcategory_name;
        "#,
    )
    .await?;

    Ok(())
}
