//! Category domain entities, wire Data Transfer Objects, and Value Objects.
//!
//! Enforces three-tier model separation across database rows, API transport payloads,
//! and validated domain newtypes. Value Objects validate and trim names for transaction
//! types, categories, and subcategories to guarantee consistent taxonomy data.
//! Wire models protect ingress by rejecting unknown fields and structure egress payloads
//! into a clean hierarchical tree for the frontend.

use serde::{Deserialize, Serialize};

use super::error::CategoryError;

/// Database record and presentation DTO for an available palette color.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub(super) struct ColorItem {
    id: i64,
    name: String,
    hex: String,
}

/// Row structure representing the flattened SQLite join view `v_category_hierarchy`.
///
/// Maps the canonical 11-column projection order from the view:
/// `type_color_id`, `type_color`, `type_id`, `type_name`, `type_sort_order`,
/// `category_id`, `category_name`, `category_sort_order`,
/// `subcategory_id`, `subcategory_name`, `subcategory_sort_order`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct CategoryHierarchyRow {
    type_color_id: i64,
    type_color: String,
    type_id: i64,
    type_name: String,
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
    /// Returns the transaction type color ID reference.
    pub(super) fn type_color_id(&self) -> i64 {
        self.type_color_id
    }

    /// Returns the transaction type hex color.
    pub(super) fn type_color(&self) -> &str {
        &self.type_color
    }

    /// Returns the transaction type ID.
    pub(super) fn type_id(&self) -> i64 {
        self.type_id
    }

    /// Returns the transaction type name.
    pub(super) fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Returns the optional category ID.
    pub(super) fn category_id(&self) -> Option<i64> {
        self.category_id
    }

    /// Returns the optional category name.
    pub(super) fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    /// Returns the optional subcategory ID.
    pub(super) fn subcategory_id(&self) -> Option<i64> {
        self.subcategory_id
    }

    /// Returns the optional subcategory name.
    pub(super) fn subcategory_name(&self) -> Option<&str> {
        self.subcategory_name.as_deref()
    }
}

/// Presentation DTO for a leaf subcategory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SubcategoryItem {
    id: i64,
    name: String,
}

impl SubcategoryItem {
    /// Constructs a new [`SubcategoryItem`].
    pub(super) fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Returns the subcategory ID.
    pub(super) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the subcategory name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

/// Presentation DTO for a category containing its associated subcategories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CategoryItem {
    id: i64,
    name: String,
    subcategories: Vec<SubcategoryItem>,
}

impl CategoryItem {
    /// Constructs a new [`CategoryItem`] with an empty subcategories list.
    pub(super) fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            subcategories: Vec::new(),
        }
    }

    /// Returns the category ID.
    pub(super) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the category name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    /// Returns a slice of nested subcategories.
    pub(super) fn subcategories(&self) -> &[SubcategoryItem] {
        &self.subcategories
    }

    /// Returns a mutable reference to the subcategories vector.
    pub(super) fn subcategories_mut(&mut self) -> &mut Vec<SubcategoryItem> {
        &mut self.subcategories
    }
}

/// Presentation DTO for a transaction type and its display color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TransactionTypeItem {
    id: i64,
    name: String,
    color: String,
    #[serde(default)]
    color_id: i64,
    #[serde(default)]
    categories: Vec<CategoryItem>,
}

impl TransactionTypeItem {
    /// Constructs a new [`TransactionTypeItem`].
    pub(super) fn new(
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
            categories: Vec::new(),
        }
    }

    /// Returns the transaction type ID.
    pub(super) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the transaction type name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    /// Returns a slice of categories belonging to this transaction type.
    pub(super) fn categories(&self) -> &[CategoryItem] {
        &self.categories
    }

    /// Returns a mutable reference to the categories vector.
    pub(super) fn categories_mut(&mut self) -> &mut Vec<CategoryItem> {
        &mut self.categories
    }
}

/// Complete hierarchical category response returned by `GET /api/v1/config/hierarchy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CategoryHierarchyResponse {
    types: Vec<TransactionTypeItem>,
    #[serde(default)]
    colors: Vec<ColorItem>,
}

impl CategoryHierarchyResponse {
    /// Constructs a new [`CategoryHierarchyResponse`].
    pub(super) fn new(types: Vec<TransactionTypeItem>, colors: Vec<ColorItem>) -> Self {
        Self { types, colors }
    }
}

#[cfg(test)]
impl CategoryHierarchyResponse {
    /// Returns a slice of transaction types for test assertions.
    pub(super) fn types(&self) -> &[TransactionTypeItem] {
        &self.types
    }

    /// Returns a slice of all available palette colors for test assertions.
    pub(super) fn colors(&self) -> &[ColorItem] {
        &self.colors
    }
}

/// Request payload to create a new transaction type.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateTypeRequest {
    #[serde(alias = "type_name")]
    name: String,
    color_id: i64,
}

impl CreateTypeRequest {
    /// Returns the desired transaction type name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    /// Returns the palette color ID reference.
    pub(super) fn color_id(&self) -> i64 {
        self.color_id
    }
}

/// Request payload to update the hex color of an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UpdateTypeColorRequest {
    color_id: i64,
}

impl UpdateTypeColorRequest {
    /// Returns the palette color ID reference.
    pub(super) fn color_id(&self) -> i64 {
        self.color_id
    }
}

/// Request payload to create a new category under an existing transaction type.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateCategoryRequest {
    type_id: i64,
    #[serde(alias = "category_name")]
    name: String,
}

impl CreateCategoryRequest {
    /// Returns the target parent transaction type ID.
    pub(super) fn type_id(&self) -> i64 {
        self.type_id
    }

    /// Returns the proposed category name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

/// Generic request payload to rename an entity (category or subcategory).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UpdateNameRequest {
    #[serde(
        alias = "category_name",
        alias = "subcategory_name",
        alias = "type_name"
    )]
    name: String,
}

impl UpdateNameRequest {
    /// Returns the updated name string.
    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

/// Request payload to create a new subcategory under an existing category.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateSubcategoryRequest {
    category_id: i64,
    #[serde(alias = "subcategory_name")]
    name: String,
}

impl CreateSubcategoryRequest {
    /// Returns the parent category ID.
    pub(super) fn category_id(&self) -> i64 {
        self.category_id
    }

    /// Returns the proposed subcategory name.
    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

/// Validated transaction type name Value Object ("Parse, Don't Validate").
///
/// Guarantees that empty or whitespace-only type names cannot be represented.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TypeName(String);

impl TypeName {
    /// Trims the input and validates non-emptiness.
    ///
    /// # Errors
    /// Returns [`CategoryError::EmptyTypeName`] if the trimmed string is empty.
    pub(super) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyTypeName { action });
        }
        Ok(Self(trimmed))
    }

    /// Borrows the validated inner name string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    /// Unwraps and consumes into the owned name [`String`].
    pub(super) fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated category name Value Object ("Parse, Don't Validate").
///
/// Guarantees that empty or whitespace-only category names cannot be represented.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct CategoryName(String);

impl CategoryName {
    /// Trims the input and validates non-emptiness.
    ///
    /// # Errors
    /// Returns [`CategoryError::EmptyCategoryName`] if the trimmed string is empty.
    pub(super) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyCategoryName { action });
        }
        Ok(Self(trimmed))
    }

    /// Borrows the validated inner name string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    /// Unwraps and consumes into the owned name [`String`].
    pub(super) fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for CategoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated subcategory name Value Object ("Parse, Don't Validate").
///
/// Guarantees that empty or whitespace-only subcategory names cannot be represented.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SubcategoryName(String);

impl SubcategoryName {
    /// Trims the input and validates non-emptiness.
    ///
    /// # Errors
    /// Returns [`CategoryError::EmptySubcategoryName`] if the trimmed string is empty.
    pub(super) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptySubcategoryName { action });
        }
        Ok(Self(trimmed))
    }

    /// Borrows the validated inner name string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    /// Unwraps and consumes into the owned name [`String`].
    pub(super) fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for SubcategoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name_validation() {
        let name = TypeName::try_new("  Income  ", "TEST.TYPE_NAME").expect("valid type name");
        assert_eq!(name.as_str(), "Income");
        assert_eq!(format!("{name}"), "Income");
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
        assert_eq!(format!("{name}"), "Housing");
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
        assert_eq!(format!("{name}"), "Rent");
        assert_eq!(name.into_inner(), "Rent");

        let err = SubcategoryName::try_new("", "TEST.EMPTY").unwrap_err();
        assert_eq!(err.action(), "TEST.EMPTY");
        assert_eq!(err.code(), "EMPTY_SUBCATEGORY_NAME");

        let err_ws = SubcategoryName::try_new("  \t ", "TEST.WS").unwrap_err();
        assert_eq!(err_ws.action(), "TEST.WS");
        assert_eq!(err_ws.code(), "EMPTY_SUBCATEGORY_NAME");
    }
}
