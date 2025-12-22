pub mod aggregate;
pub mod aggregate_functions;
pub mod arithmetic;
pub mod binary;
pub mod comparison;
pub mod container;
pub mod cypher;
pub mod function;
pub mod operators_ext;
pub mod property;
pub mod storage;
pub mod type_conversion;
pub mod unary;

// 重新导出Core访问器
pub use crate::core::visitor::{ExpressionAcceptor, ExpressionVisitor};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// Re-export Core expression types
pub use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};

// Re-export Core evaluator
pub use crate::core::evaluator::ExpressionEvaluator;

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
