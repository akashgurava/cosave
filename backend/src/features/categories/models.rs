use serde::{Deserialize, Serialize};

use super::error::CategoryError;

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
    type_id: i64,
    type_name: String,
    type_color: String,
    type_color_id: i64,
    #[sqlx(rename = "type_sort_order")]
    _type_sort_order: i64,
    category_id: Option<i64>,
    category_name: Option<String>,
    #[sqlx(rename = "category_sort_order")]
    _category_sort_order: Option<i64>,
    subcategory_id: Option<i64>,
    subcategory_name: Option<String>,
    #[sqlx(rename = "subcategory_sort_order")]
    _subcategory_sort_order: Option<i64>,
}

impl CategoryHierarchyRow {
    pub(crate) fn type_id(&self) -> i64 {
        self.type_id
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

    pub(crate) fn category_id(&self) -> Option<i64> {
        self.category_id
    }

    pub(crate) fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    pub(crate) fn subcategory_id(&self) -> Option<i64> {
        self.subcategory_id
    }

    pub(crate) fn subcategory_name(&self) -> Option<&str> {
        self.subcategory_name.as_deref()
    }
}

/// Presentation DTO for a leaf subcategory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SubcategoryItem {
    id: i64,
    name: String,
}

impl SubcategoryItem {
    pub(crate) fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Presentation DTO for a category containing its associated subcategories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CategoryItem {
    id: i64,
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    subcategories: Vec<SubcategoryItem>,
}

impl CategoryItem {
    pub(crate) fn new(id: i64, name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            type_name: type_name.into(),
            subcategories: Vec::new(),
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
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
    id: i64,
    name: String,
    color: String,
    #[serde(default)]
    color_id: i64,
}

impl TransactionTypeItem {
    pub(crate) fn new(
        id: i64,
        name: impl Into<String>,
        color: impl Into<String>,
        color_id: i64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            color: color.into(),
            color_id,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
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

#[cfg(test)]
impl CategoryHierarchyResponse {
    pub(crate) fn types(&self) -> &[TransactionTypeItem] {
        &self.types
    }

    pub(crate) fn categories(&self) -> &[CategoryItem] {
        &self.categories
    }

    pub(crate) fn colors(&self) -> &[ColorItem] {
        &self.colors
    }
}

/// Request payload to create a new transaction type.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSubcategoryRequest {
    category_id: i64,
    name: String,
}

impl CreateSubcategoryRequest {
    pub(crate) fn category_id(&self) -> i64 {
        self.category_id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Validated transaction type name value object ("Parse, Don't Validate").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypeName(String);

impl TypeName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyTypeName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated category name value object ("Parse, Don't Validate").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CategoryName(String);

impl CategoryName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyCategoryName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

/// Validated subcategory name value object ("Parse, Don't Validate").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SubcategoryName(String);

impl SubcategoryName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptySubcategoryName { action });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name_validation() {
        let name = TypeName::try_new("  Income  ", "TEST.TYPE_NAME").expect("valid type name");
        assert_eq!(name.as_str(), "Income");
        assert_eq!(name.into_inner(), "Income");

        let err = TypeName::try_new("", "TEST.EMPTY").unwrap_err();
        assert_eq!(err.action(), "TEST.EMPTY");
        assert_eq!(err.code(), "EMPTY_TYPE_NAME");

        let err_ws = TypeName::try_new("   \t\n  ", "TEST.WS").unwrap_err();
        assert_eq!(err_ws.action(), "TEST.WS");
        assert_eq!(err_ws.code(), "EMPTY_TYPE_NAME");
    }

    #[test]
    fn test_category_name_validation() {
        let name =
            CategoryName::try_new("  Housing  ", "TEST.CAT_NAME").expect("valid category name");
        assert_eq!(name.as_str(), "Housing");
        assert_eq!(name.into_inner(), "Housing");

        let err = CategoryName::try_new("", "TEST.EMPTY").unwrap_err();
        assert_eq!(err.action(), "TEST.EMPTY");
        assert_eq!(err.code(), "EMPTY_CATEGORY_NAME");

        let err_ws = CategoryName::try_new("   ", "TEST.WS").unwrap_err();
        assert_eq!(err_ws.action(), "TEST.WS");
        assert_eq!(err_ws.code(), "EMPTY_CATEGORY_NAME");
    }

    #[test]
    fn test_subcategory_name_validation() {
        let name =
            SubcategoryName::try_new("  Rent  ", "TEST.SUB_NAME").expect("valid subcategory name");
        assert_eq!(name.as_str(), "Rent");
        assert_eq!(name.into_inner(), "Rent");

        let err = SubcategoryName::try_new("", "TEST.EMPTY").unwrap_err();
        assert_eq!(err.action(), "TEST.EMPTY");
        assert_eq!(err.code(), "EMPTY_SUBCATEGORY_NAME");

        let err_ws = SubcategoryName::try_new("  \t ", "TEST.WS").unwrap_err();
        assert_eq!(err_ws.action(), "TEST.WS");
        assert_eq!(err_ws.code(), "EMPTY_SUBCATEGORY_NAME");
    }
}
