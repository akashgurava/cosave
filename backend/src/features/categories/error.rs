use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

#[derive(Error, Debug)]
pub enum CategoryError {
    #[error("EMPTY_TYPE_NAME. ACTION: {action}")]
    EmptyTypeName { action: &'static str },

    #[error("EMPTY_CATEGORY_NAME. ACTION: {action}")]
    EmptyCategoryName { action: &'static str },

    #[error("EMPTY_SUBCATEGORY_NAME. ACTION: {action}")]
    EmptySubcategoryName { action: &'static str },

    #[error("MISSING_COLOR. ACTION: {action}")]
    MissingColor { action: &'static str },

    #[error("EMPTY_COLOR. ACTION: {action}")]
    EmptyColor { action: &'static str },

    #[error("COLOR_NOT_FOUND. ACTION: {action}. Color ID: {id}")]
    ColorNotFound { action: &'static str, id: i64 },

    #[error("UNRECOGNIZED_COLOR. ACTION: {action}. Color: '{color}'")]
    UnrecognizedColor { action: &'static str, color: String },

    #[error("TYPE_NOT_FOUND. ACTION: {action}. Type: '{id}'")]
    TypeNotFound { action: &'static str, id: String },

    #[error("CATEGORY_NOT_FOUND. ACTION: {action}. Category: '{id}'")]
    CategoryNotFound { action: &'static str, id: String },

    #[error("SUBCATEGORY_NOT_FOUND. ACTION: {action}. Subcategory: '{id}'")]
    SubcategoryNotFound { action: &'static str, id: String },

    #[error("TYPE_ALREADY_EXISTS. ACTION: {action}. Type: '{name}'")]
    TypeAlreadyExists { action: &'static str, name: String },

    #[error("CATEGORY_ALREADY_EXISTS. ACTION: {action}. Category: '{name}' under '{type_name}'")]
    CategoryAlreadyExists {
        action: &'static str,
        name: String,
        type_name: String,
    },

    #[error("SUBCATEGORY_ALREADY_EXISTS. ACTION: {action}. Subcategory: '{name}'")]
    SubcategoryAlreadyExists { action: &'static str, name: String },
}

impl CategoryError {
    pub fn action(&self) -> &'static str {
        match self {
            Self::EmptyTypeName { action } => action,
            Self::EmptyCategoryName { action } => action,
            Self::EmptySubcategoryName { action } => action,
            Self::MissingColor { action } => action,
            Self::EmptyColor { action } => action,
            Self::ColorNotFound { action, .. } => action,
            Self::UnrecognizedColor { action, .. } => action,
            Self::TypeNotFound { action, .. } => action,
            Self::CategoryNotFound { action, .. } => action,
            Self::SubcategoryNotFound { action, .. } => action,
            Self::TypeAlreadyExists { action, .. } => action,
            Self::CategoryAlreadyExists { action, .. } => action,
            Self::SubcategoryAlreadyExists { action, .. } => action,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyTypeName { .. } => "EMPTY_TYPE_NAME",
            Self::EmptyCategoryName { .. } => "EMPTY_CATEGORY_NAME",
            Self::EmptySubcategoryName { .. } => "EMPTY_SUBCATEGORY_NAME",
            Self::MissingColor { .. } => "MISSING_COLOR",
            Self::EmptyColor { .. } => "EMPTY_COLOR",
            Self::ColorNotFound { .. } => "COLOR_NOT_FOUND",
            Self::UnrecognizedColor { .. } => "UNRECOGNIZED_COLOR",
            Self::TypeNotFound { .. } => "TYPE_NOT_FOUND",
            Self::CategoryNotFound { .. } => "CATEGORY_NOT_FOUND",
            Self::SubcategoryNotFound { .. } => "SUBCATEGORY_NOT_FOUND",
            Self::TypeAlreadyExists { .. } => "TYPE_ALREADY_EXISTS",
            Self::CategoryAlreadyExists { .. } => "CATEGORY_ALREADY_EXISTS",
            Self::SubcategoryAlreadyExists { .. } => "SUBCATEGORY_ALREADY_EXISTS",
        }
    }
}

impl IntoResponse for CategoryError {
    fn into_response(self) -> Response {
        let action = self.action();
        let code_str = self.code();

        let (status_code, code, message) = match &self {
            Self::EmptyTypeName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Transaction type name cannot be empty.".to_string(),
            ),
            Self::EmptyCategoryName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Category and parent type name cannot be empty.".to_string(),
            ),
            Self::EmptySubcategoryName { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Subcategory name and parent category ID cannot be empty.".to_string(),
            ),
            Self::MissingColor { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Color must be specified.".to_string(),
            ),
            Self::EmptyColor { .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                "Color cannot be empty.".to_string(),
            ),
            Self::ColorNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Color with id {id} was not found in the palette."),
            ),
            Self::UnrecognizedColor { color, .. } => (
                StatusCode::BAD_REQUEST,
                Code::bad_request(),
                format!("Color '{color}' is not recognized in the palette."),
            ),
            Self::TypeNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Transaction type '{id}' was not found."),
            ),
            Self::CategoryNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Category '{id}' was not found."),
            ),
            Self::SubcategoryNotFound { id, .. } => (
                StatusCode::NOT_FOUND,
                Code::not_found(),
                format!("Subcategory '{id}' was not found."),
            ),
            Self::TypeAlreadyExists { name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Transaction type '{name}' already exists."),
            ),
            Self::CategoryAlreadyExists {
                name, type_name, ..
            } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                if type_name.is_empty() {
                    format!("Category name '{name}' already exists under this type.")
                } else {
                    format!("Category '{name}' already exists under type '{type_name}'.")
                },
            ),
            Self::SubcategoryAlreadyExists { name, .. } => (
                StatusCode::CONFLICT,
                Code::conflict(),
                format!("Subcategory '{name}' already exists under this category."),
            ),
        };

        if status_code.is_server_error() {
            tracing::error!(action = action, code = code_str, error = %self, "request failed");
        } else {
            tracing::warn!(action = action, code = code_str, error = %self, "client error");
        }

        let body = Json(ApiResponse::err(
            code,
            Status::custom(code_str),
            ErrorPayload::new(action, message),
        ));
        (status_code, body).into_response()
    }
}
