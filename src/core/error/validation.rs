//! 验证错误类型
//!
//! 涵盖查询验证和Schema验证相关的错误

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::core::error::query::QueryError;

/// 验证错误类型枚举（统一版本）
///
/// 提供完整的验证错误分类，与QueryError::InvalidQuery对应
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorType {
    SyntaxError,
    SemanticError,
    TypeError,
    TypeMismatch,
    AliasError,
    AggregateError,
    PaginationError,
    ExpressionDepthError,
    VariableNotFound,
    CyclicReference,
    DivisionByZero,
    TooManyArguments,
    TooManyElements,
    DuplicateKey,
    ConstraintViolation,
}

impl fmt::Display for ValidationErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationErrorType::SyntaxError => write!(f, "Syntax error"),
            ValidationErrorType::SemanticError => write!(f, "Semantic error"),
            ValidationErrorType::TypeError => write!(f, "Type error"),
            ValidationErrorType::TypeMismatch => write!(f, "Type mismatch"),
            ValidationErrorType::AliasError => write!(f, "Alias error"),
            ValidationErrorType::AggregateError => write!(f, "Aggregate error"),
            ValidationErrorType::PaginationError => write!(f, "Pagination error"),
            ValidationErrorType::ExpressionDepthError => write!(f, "Expression depth error"),
            ValidationErrorType::VariableNotFound => write!(f, "Variable not found"),
            ValidationErrorType::CyclicReference => write!(f, "Cyclic reference"),
            ValidationErrorType::DivisionByZero => write!(f, "Division by zero"),
            ValidationErrorType::TooManyArguments => write!(f, "Too many arguments"),
            ValidationErrorType::TooManyElements => write!(f, "Too many elements"),
            ValidationErrorType::DuplicateKey => write!(f, "Duplicate key"),
            ValidationErrorType::ConstraintViolation => write!(f, "Constraint violation"),
        }
    }
}

impl From<ValidationErrorType> for QueryError {
    fn from(e: ValidationErrorType) -> Self {
        match e {
            ValidationErrorType::SyntaxError => QueryError::ParseError(e.to_string()),
            _ => QueryError::InvalidQuery(e.to_string()),
        }
    }
}

/// 统一验证错误结构
///
/// 包含错误类型、错误消息和位置信息
/// 支持序列化/反序列化，用于跨模块传递
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub error_type: ValidationErrorType,
    pub context: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>, error_type: ValidationErrorType) -> Self {
        Self {
            message: message.into(),
            error_type,
            context: None,
            line: None,
            column: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn at_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Schema验证错误类型
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Invalid schema definition: {0}")]
    InvalidSchema(String),

    #[error("Property type error: {0}")]
    PropertyTypeError(String),

    #[error("Required property missing: {0}")]
    RequiredPropertyMissing(String),

    #[error("Property validation failed: {0}")]
    PropertyValidationFailed(String),

    #[error("Schema conflict: {0}")]
    SchemaConflict(String),

    #[error("Unsupported schema operation: {0}")]
    UnsupportedOperation(String),
}

/// Schema验证结果
#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub errors: Vec<SchemaValidationError>,
}

impl SchemaValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<SchemaValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: SchemaValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
}
