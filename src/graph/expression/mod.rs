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
pub mod expression;
#[cfg(test)]
pub mod tests;

// Re-export common types at the root level
pub use context::*;
pub use error::*;
// 统一使用V2版本Expression
pub use expression::Expression;
pub use expression::LiteralValue;
pub use expression::BinaryOperator;
pub use expression::UnaryOperator;
pub use expression::AggregateFunction;
pub use expression::DataType;

// 统一使用V2版本ExpressionEvaluator
pub use evaluator::ExpressionEvaluator;

// 类型别名，为了兼容性
pub type ExpressionContext<'a> = EvalContext<'a>;