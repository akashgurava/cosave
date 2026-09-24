use serde::{Deserialize, Serialize};

/// Database record and presentation DTO for an available palette color.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct ColorItem {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) hex: String,
}

/// Database record for a top-level transaction type (e.g. Income, Expense, Transfer, Invest).
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct TransactionTypeRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color_id: i64,
    pub(crate) sort_order: i64,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

/// Database record for a mid-level category linked to a transaction type.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct CategoryRecord {
    pub(crate) id: String,
    pub(crate) type_id: String,
    pub(crate) name: String,
    pub(crate) sort_order: i64,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

/// Database record for a leaf subcategory linked to a category.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct SubcategoryRecord {
    pub(crate) id: String,
    pub(crate) category_id: String,
    pub(crate) name: String,
    pub(crate) sort_order: i64,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

/// Row structure representing the flattened SQLite join view `v_category_hierarchy`.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct CategoryHierarchyRow {
    pub(crate) type_id: String,
    pub(crate) type_name: String,
    pub(crate) type_color: String,
    pub(crate) type_color_id: i64,
    pub(crate) type_sort_order: i64,
    pub(crate) category_id: Option<String>,
    pub(crate) category_name: Option<String>,
    pub(crate) category_sort_order: Option<i64>,
    pub(crate) subcategory_id: Option<String>,
    pub(crate) subcategory_name: Option<String>,
    pub(crate) subcategory_sort_order: Option<i64>,
}

/// Presentation DTO for a leaf subcategory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SubcategoryItem {
    pub(crate) id: String,
    pub(crate) name: String,
}

/// Presentation DTO for a category containing its associated subcategories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CategoryItem {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_name: String,
    pub(crate) subcategories: Vec<SubcategoryItem>,
}

/// Presentation DTO for a transaction type and its display color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionTypeItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
    #[serde(default)]
    pub(crate) color_id: i64,
}

/// Complete hierarchical category response returned by `GET /api/v1/categories`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CategoryHierarchyResponse {
    pub(crate) types: Vec<TransactionTypeItem>,
    pub(crate) categories: Vec<CategoryItem>,
    #[serde(default)]
    pub(crate) colors: Vec<ColorItem>,
}

/// Request payload to create a new transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateTypeRequest {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) color_id: Option<i64>,
}

/// Request payload to update the hex color of an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateTypeColorRequest {
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) color_id: Option<i64>,
}

/// Request payload to create a new category under an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateCategoryRequest {
    pub(crate) type_name: String,
    pub(crate) name: String,
}

/// Generic request payload to rename an entity (category or subcategory).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateNameRequest {
    pub(crate) name: String,
}

/// Request payload to create a new subcategory under an existing category.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateSubcategoryRequest {
    pub(crate) category_id: String,
    pub(crate) name: String,
}
