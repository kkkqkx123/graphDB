pub mod expr_type;
pub mod property;
pub mod binary;
pub mod unary;
pub mod function;
pub mod container;
pub mod aggregate;
pub mod aggregate_functions;
pub mod context;
pub mod evaluator;
pub mod visitor;
pub mod error;
#[cfg(test)]
pub mod tests;

// Re-export common types at the root level
pub use context::*;
pub use error::*;
pub use binary::BinaryOperator;
pub use unary::UnaryOperator;

// 明确导出两个版本的Expression，避免歧义
pub use expr_type::Expression as ExpressionV1;  // 旧版本Expression
pub use expression::Expression as ExpressionV2;  // 新版本Expression

// 为了向后兼容，默认使用V1版本
pub use expr_type::Expression;
pub use expr_type::ExpressionKind;
pub use expr_type::InputPropertyExpression;

// 明确导出ExpressionEvaluator，避免歧义
pub use evaluator::ExpressionEvaluator as ExpressionEvaluatorV1;  // 旧版本ExpressionEvaluator
pub use evaluator::ExpressionEvaluator as ExpressionEvaluatorV2;  // 新版本ExpressionEvaluator

// 为了向后兼容，默认使用V1版本
pub use evaluator::ExpressionEvaluator;

// 类型别名，为了兼容性
pub type ExpressionContext<'a> = EvalContext<'a>;

// 重新导出新的操作符类型，避免冲突
pub use expression::{BinaryOperator as BinaryOperatorV2, UnaryOperator as UnaryOperatorV2, AggregateFunction};