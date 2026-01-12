//! 验证策略接口定义
//! 定义验证策略的统一接口，使用core模块中的统一错误类型

use crate::core::error::{DBError, QueryError};
use crate::query::validator::structs::*;
use std::collections::HashMap;

/// 验证错误类型（为了向后兼容保留，但建议使用DBError）
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    SyntaxError,
    SemanticError,
    TypeError,
    AliasError,
    AggregateError,
    PaginationError,
}

/// 验证错误结构（为了向后兼容保留，但建议使用DBError）
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub error_type: ValidationErrorType,
    pub context: Option<String>, // 错误上下文信息
}

impl ValidationError {
    pub fn new(message: String, error_type: ValidationErrorType) -> Self {
        Self {
            message,
            error_type,
            context: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// 转换为统一的DBError
    pub fn to_db_error(&self) -> DBError {
        let error_msg = if let Some(ref ctx) = self.context {
            format!("{} (上下文: {})", self.message, ctx)
        } else {
            self.message.clone()
        };

        match self.error_type {
            ValidationErrorType::SyntaxError => DBError::Query(QueryError::ParseError(error_msg)),
            ValidationErrorType::SemanticError | ValidationErrorType::TypeError => {
                DBError::Query(QueryError::InvalidQuery(error_msg))
            }
            _ => DBError::Query(QueryError::ExecutionError(error_msg)),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for DBError {
    fn from(err: ValidationError) -> Self {
        err.to_db_error()
    }
}

/// 验证策略类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStrategyType {
    Alias,
    Expression,
    Clause,
    Aggregate,
    Pagination,
}

/// 验证上下文接口
pub trait ValidationContext {
    fn get_query_parts(&self) -> &[QueryPart];
    fn get_aliases(&self) -> &HashMap<String, AliasType>;
    fn add_error(&mut self, error: ValidationError);
    fn has_errors(&self) -> bool;
    fn get_errors(&self) -> &[ValidationError];
}

/// 验证策略统一接口
pub trait ValidationStrategy {
    /// 执行验证
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError>;

    /// 获取策略类型
    fn strategy_type(&self) -> ValidationStrategyType;

    /// 策略名称（用于调试和日志）
    fn strategy_name(&self) -> &'static str;
}
