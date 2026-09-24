use crate::core::{create_db_object, db::DbPool, error::AppError};

/// Creates category hierarchy domain tables, indices, and views.
pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), AppError> {
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.COLORS_TABLE",
        "colors",
        pool,
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
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS transaction_types (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT UNIQUE NOT NULL,
            color_id INTEGER NOT NULL REFERENCES colors(id),
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
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY NOT NULL,
            type_id TEXT NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
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
        pool,
        "CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);",
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.SUBCATEGORIES_TABLE",
        "subcategories",
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS subcategories (
            id TEXT PRIMARY KEY NOT NULL,
            category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
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
        pool,
        "CREATE INDEX IF NOT EXISTS idx_subcategories_category_id ON subcategories(category_id);",
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.V_CATEGORY_HIERARCHY_VIEW",
        "v_category_hierarchy",
        pool,
        r#"
        CREATE VIEW IF NOT EXISTS v_category_hierarchy AS
        SELECT
            t.id AS type_id,
            t.name AS type_name,
            t.color_id AS type_color_id,
            col.hex AS type_color,
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
