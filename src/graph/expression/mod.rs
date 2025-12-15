pub mod aggregate;
pub mod aggregate_functions;
pub mod binary;
pub mod container;
pub mod error;
pub mod evaluator;
pub mod expression;
pub mod function;
pub mod property;
#[cfg(test)]
pub mod tests;
pub mod unary;
pub mod visitor;

// Re-export common types at the root level
pub use error::*;
// 统一使用V2版本Expression
pub use expression::AggregateFunction;
pub use expression::BinaryOperator;
pub use expression::DataType;
pub use expression::Expression;
pub use expression::LiteralValue;
pub use expression::UnaryOperator;

// 统一使用V2版本ExpressionEvaluator
pub use evaluator::ExpressionEvaluator;

// 类型别名，为了兼容性
pub type ExpressionContext<'a> = crate::query::context::EvalContext<'a>;
