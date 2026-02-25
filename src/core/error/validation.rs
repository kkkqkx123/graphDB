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
            ValidationErrorType::SyntaxError => write!(f, "语法错误"),
            ValidationErrorType::SemanticError => write!(f, "语义错误"),
            ValidationErrorType::TypeError => write!(f, "类型错误"),
            ValidationErrorType::TypeMismatch => write!(f, "类型不匹配"),
            ValidationErrorType::AliasError => write!(f, "别名错误"),
            ValidationErrorType::AggregateError => write!(f, "聚合函数错误"),
            ValidationErrorType::PaginationError => write!(f, "分页错误"),
            ValidationErrorType::ExpressionDepthError => write!(f, "表达式深度错误"),
            ValidationErrorType::VariableNotFound => write!(f, "变量未找到"),
            ValidationErrorType::CyclicReference => write!(f, "循环引用"),
            ValidationErrorType::DivisionByZero => write!(f, "除零错误"),
            ValidationErrorType::TooManyArguments => write!(f, "参数过多"),
            ValidationErrorType::TooManyElements => write!(f, "元素过多"),
            ValidationErrorType::DuplicateKey => write!(f, "重复键"),
            ValidationErrorType::ConstraintViolation => write!(f, "约束违反"),
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
    #[error("Schema未找到: {0}")]
    SchemaNotFound(String),

    #[error("无效的Schema定义: {0}")]
    InvalidSchema(String),

    #[error("属性类型错误: {0}")]
    PropertyTypeError(String),

    #[error("必需的属性缺失: {0}")]
    RequiredPropertyMissing(String),

    #[error("属性值验证失败: {0}")]
    PropertyValidationFailed(String),

    #[error("Schema冲突: {0}")]
    SchemaConflict(String),

    #[error("不支持的Schema操作: {0}")]
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
