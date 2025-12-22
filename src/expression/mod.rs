pub mod aggregate;
pub mod aggregate_functions;
pub mod arithmetic;
pub mod binary;
pub mod comparison;
pub mod container;
pub mod cypher;
pub mod function;
pub mod operator_conversion;
pub mod operators_ext;
pub mod property;
pub mod storage;
pub mod type_conversion;
pub mod unary;

// 重新导出Core访问器
pub use crate::core::visitor::{ExpressionVisitor, ExpressionAcceptor};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{
    BinaryOperator, UnaryOperator, AggregateFunction
};

// Re-export Core expression types
pub use crate::core::types::expression::{
    Expression, LiteralValue, DataType, ExpressionType
};

// Re-export Core evaluator
pub use crate::core::evaluator::ExpressionEvaluator;

// Legacy aliases for backward compatibility (deprecated)
#[deprecated(note = "使用 BinaryOperator 替代")]
pub type ExtendedBinaryOperator = BinaryOperator;

#[deprecated(note = "使用 UnaryOperator 替代")]
pub type ExtendedUnaryOperator = UnaryOperator;

#[deprecated(note = "使用 AggregateFunction 替代")]
pub type ExtendedAggregateFunction = AggregateFunction;

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
