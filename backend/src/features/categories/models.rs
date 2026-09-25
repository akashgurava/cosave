use serde::{Deserialize, Serialize};

/// Database record and presentation DTO for an available palette color.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct ColorItem {
    id: i64,
    name: String,
    hex: String,
}

/// Row structure representing the flattened SQLite join view `v_category_hierarchy`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct CategoryHierarchyRow {
    type_id: String,
    type_name: String,
    type_color: String,
    type_color_id: i64,
    #[sqlx(rename = "type_sort_order")]
    _type_sort_order: i64,
    category_id: Option<String>,
    category_name: Option<String>,
    #[sqlx(rename = "category_sort_order")]
    _category_sort_order: Option<i64>,
    subcategory_id: Option<String>,
    subcategory_name: Option<String>,
    #[sqlx(rename = "subcategory_sort_order")]
    _subcategory_sort_order: Option<i64>,
}

impl CategoryHierarchyRow {
    pub(crate) fn type_id(&self) -> &str {
        &self.type_id
    }

    pub(crate) fn type_name(&self) -> &str {
        &self.type_name
    }

    pub(crate) fn type_color(&self) -> &str {
        &self.type_color
    }

    pub(crate) fn type_color_id(&self) -> i64 {
        self.type_color_id
    }

    pub(crate) fn category_id(&self) -> Option<&str> {
        self.category_id.as_deref()
    }

    pub(crate) fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    pub(crate) fn subcategory_id(&self) -> Option<&str> {
        self.subcategory_id.as_deref()
    }

    pub(crate) fn subcategory_name(&self) -> Option<&str> {
        self.subcategory_name.as_deref()
    }
}

/// Presentation DTO for a leaf subcategory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SubcategoryItem {
    id: String,
    name: String,
}

impl SubcategoryItem {
    pub(crate) fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Presentation DTO for a category containing its associated subcategories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CategoryItem {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    subcategories: Vec<SubcategoryItem>,
}

impl CategoryItem {
    pub(crate) fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            type_name: type_name.into(),
            subcategories: Vec::new(),
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn subcategories(&self) -> &[SubcategoryItem] {
        &self.subcategories
    }

    pub(crate) fn subcategories_mut(&mut self) -> &mut Vec<SubcategoryItem> {
        &mut self.subcategories
    }
}

/// Presentation DTO for a transaction type and its display color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionTypeItem {
    id: String,
    name: String,
    color: String,
    #[serde(default)]
    color_id: i64,
}

impl TransactionTypeItem {
    pub(crate) fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        color: impl Into<String>,
        color_id: i64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color: color.into(),
            color_id,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Complete hierarchical category response returned by `GET /api/v1/categories`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CategoryHierarchyResponse {
    types: Vec<TransactionTypeItem>,
    categories: Vec<CategoryItem>,
    #[serde(default)]
    colors: Vec<ColorItem>,
}

impl CategoryHierarchyResponse {
    pub(crate) fn new(
        types: Vec<TransactionTypeItem>,
        categories: Vec<CategoryItem>,
        colors: Vec<ColorItem>,
    ) -> Self {
        Self {
            types,
            categories,
            colors,
        }
    }
}

/// Request payload to create a new transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateTypeRequest {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    color_id: Option<i64>,
}

impl CreateTypeRequest {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub(crate) fn color_id(&self) -> Option<i64> {
        self.color_id
    }
}

/// Request payload to update the hex color of an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateTypeColorRequest {
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    color_id: Option<i64>,
}

impl UpdateTypeColorRequest {
    pub(crate) fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub(crate) fn color_id(&self) -> Option<i64> {
        self.color_id
    }
}

/// Request payload to create a new category under an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateCategoryRequest {
    type_name: String,
    name: String,
}

impl CreateCategoryRequest {
    pub(crate) fn type_name(&self) -> &str {
        &self.type_name
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Generic request payload to rename an entity (category or subcategory).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateNameRequest {
    name: String,
}

impl UpdateNameRequest {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Request payload to create a new subcategory under an existing category.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateSubcategoryRequest {
    category_id: String,
    name: String,
}

impl CreateSubcategoryRequest {
    pub(crate) fn category_id(&self) -> &str {
        &self.category_id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}
