//! 验证策略接口定义
//! 定义验证策略的统一接口，使用core模块中的统一错误类型

use crate::core::error::{DBError, QueryError, ValidationError as CoreValidationError, ValidationErrorType as CoreValidationErrorType};
use crate::query::validator::structs::*;
use std::collections::HashMap;

pub use crate::core::error::{ValidationError, ValidationErrorType};

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
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError>;
    fn strategy_type(&self) -> ValidationStrategyType;
    fn strategy_name(&self) -> &'static str;
}
