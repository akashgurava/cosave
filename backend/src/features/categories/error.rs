use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::core::{ApiResponse, Code, ErrorPayload, Status};

#[derive(Debug)]
pub enum CategoryError {
    EmptyTypeName {
        action: &'static str,
    },
    EmptyCategoryName {
        action: &'static str,
    },
    EmptySubcategoryName {
        action: &'static str,
    },
    MissingColor {
        action: &'static str,
    },
    EmptyColor {
        action: &'static str,
    },
    ColorNotFound {
        action: &'static str,
        id: i64,
    },
    UnrecognizedColor {
        action: &'static str,
        color: String,
    },
    TypeNotFound {
        action: &'static str,
        id: String,
    },
    CategoryNotFound {
        action: &'static str,
        id: String,
    },
    SubcategoryNotFound {
        action: &'static str,
        id: String,
    },
    TypeAlreadyExists {
        action: &'static str,
        name: String,
    },
    CategoryAlreadyExists {
        action: &'static str,
        name: String,
        type_name: String,
    },
    SubcategoryAlreadyExists {
        action: &'static str,
        name: String,
    },
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

impl fmt::Display for CategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::EmptyTypeName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptyCategoryName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptySubcategoryName { action } => write!(f, "{code}. ACTION: {action}"),
            Self::MissingColor { action } => write!(f, "{code}. ACTION: {action}"),
            Self::EmptyColor { action } => write!(f, "{code}. ACTION: {action}"),
            Self::ColorNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Color ID: {id}")
            }
            Self::UnrecognizedColor { action, color } => {
                write!(f, "{code}. ACTION: {action}. Color: '{color}'")
            }
            Self::TypeNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Type: '{id}'")
            }
            Self::CategoryNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Category: '{id}'")
            }
            Self::SubcategoryNotFound { action, id } => {
                write!(f, "{code}. ACTION: {action}. Subcategory: '{id}'")
            }
            Self::TypeAlreadyExists { action, name } => {
                write!(f, "{code}. ACTION: {action}. Type: '{name}'")
            }
            Self::CategoryAlreadyExists {
                action,
                name,
                type_name,
            } => {
                write!(
                    f,
                    "{code}. ACTION: {action}. Category: '{name}' under '{type_name}'"
                )
            }
            Self::SubcategoryAlreadyExists { action, name } => {
                write!(f, "{code}. ACTION: {action}. Subcategory: '{name}'")
            }
        }
    }
}

impl Error for CategoryError {}

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
