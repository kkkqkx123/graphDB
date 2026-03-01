//! 验证策略模块
//! 包含所有验证策略的实现

pub mod agg_functions;
pub mod aggregate_strategy;
pub mod alias_strategy;
pub mod clause_strategy;
pub mod expression_strategy;
pub mod expression_operations;
pub mod pagination_strategy;

pub mod helpers;
pub mod metadata;

#[cfg(test)]
pub mod expression_strategy_test;

pub use agg_functions::*;
pub use aggregate_strategy::*;
pub use alias_strategy::*;
pub use clause_strategy::*;
pub use expression_strategy::*;
pub use expression_operations::*;
pub use pagination_strategy::*;

pub use helpers::{TypeDeduceValidator, TypeValidator, VariableChecker, ExpressionChecker, ExpressionValidationContext, deduce_expression_type};
pub use metadata::AggFunctionMeta;
