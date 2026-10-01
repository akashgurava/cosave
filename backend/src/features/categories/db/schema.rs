//! Database schema migrations and DDL definitions for the category subsystem.
//!
//! Initializes relational tables for palette colors, root transaction types, intermediate
//! categories, and leaf subcategories within an active transaction. Foreign key constraints
//! enforce cascading deletions down the hierarchy while restricting the deletion of active
//! palette colors. The schema also provisions the denormalized `v_category_hierarchy` view
//! to optimize multi-tier tree queries.

use sqlx::{Sqlite, Transaction};

use crate::core::{create_db_object, AppError};

/// Creates category hierarchy domain tables, indices, and views within an active database transaction.
///
/// # Database Objects Created
/// - **Tables**:
///   - `colors`: Color palette entity table (`id INTEGER PRIMARY KEY AUTOINCREMENT`, `name TEXT UNIQUE`, `hex TEXT`, `sort_order INTEGER`, `created_at INTEGER`, `updated_at INTEGER`).
///   - `transaction_types`: Root classification types (e.g. Income, Expense, Transfer) (`id INTEGER PRIMARY KEY AUTOINCREMENT`, `name TEXT UNIQUE`, `color_id INTEGER REFERENCES colors(id) ON DELETE RESTRICT`, `sort_order INTEGER`, `created_at INTEGER`, `updated_at INTEGER`).
///   - `categories`: Level-1 categories scoped under transaction types (`id INTEGER PRIMARY KEY AUTOINCREMENT`, `type_id INTEGER REFERENCES transaction_types(id) ON DELETE CASCADE`, `name TEXT`, `sort_order INTEGER`, `created_at INTEGER`, `updated_at INTEGER`, `UNIQUE(type_id, name)`).
///   - `subcategories`: Level-2 leaf categories scoped under categories (`id INTEGER PRIMARY KEY AUTOINCREMENT`, `category_id INTEGER REFERENCES categories(id) ON DELETE CASCADE`, `name TEXT`, `sort_order INTEGER`, `created_at INTEGER`, `updated_at INTEGER`, `UNIQUE(category_id, name)`).
/// - **Indexes**:
///   - `idx_categories_type_id`: Fast lookup for category listing and cascade deletions by `type_id`.
///   - `idx_subcategories_category_id`: Fast lookup for subcategory listing and cascade deletions by `category_id`.
/// - **Views / Triggers**:
///   - `v_category_hierarchy`: Denormalized 3-tier joined view connecting transaction types, categories, and subcategories with color hex codes, pre-sorted by hierarchy sort orders.
///
/// # Invariants
/// - Executes within the caller's active database transaction.
/// - Uses idempotent `CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`, and `CREATE VIEW IF NOT EXISTS` DDL.
/// - Executed strictly via [`create_db_object`] with dedicated compile-time action tokens for each database object.
/// - Enforces foreign key referential cascade (`ON DELETE CASCADE`) on child categories/subcategories, and restrict (`ON DELETE RESTRICT`) on colors.
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any table, index, or view creation statement fails.
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
            name TEXT UNIQUE NOT NULL,
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
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, name)
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
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(category_id, name)
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
            t.name AS type_name,
            t.sort_order AS type_sort_order,
            c.id AS category_id,
            c.name AS category_name,
            c.sort_order AS category_sort_order,
            s.id AS subcategory_id,
            s.name AS subcategory_name,
            s.sort_order AS subcategory_sort_order
        FROM transaction_types t
        JOIN colors col ON col.id = t.color_id
        LEFT JOIN categories c ON c.type_id = t.id
        LEFT JOIN subcategories s ON s.category_id = c.id
        ORDER BY t.sort_order, t.name, c.sort_order, c.name, s.sort_order, s.name;
        "#,
    )
    .await?;

    Ok(())
}
