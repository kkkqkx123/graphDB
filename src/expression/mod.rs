pub mod aggregate;
pub mod aggregate_functions;
pub mod arithmetic;
pub mod binary;
pub mod comparison;
pub mod container;
pub mod cypher;
pub mod expression;
pub mod function;
pub mod operator_conversion;
pub mod property;
pub mod storage;
pub mod type_conversion;
pub mod unary;
pub mod visitor;

pub use visitor::{DefaultExpressionVisitor, ExpressionAcceptor, ExpressionVisitor};

// 从Core模块重新导出表达式类型
pub use crate::core::types::expression::{
    AggregateFunction, BinaryOperator, DataType, Expression, LiteralValue, UnaryOperator,
};

// ExpressionContext相关功能已迁移到Core模块
// ExpressionEvaluator已迁移到Core模块
// evaluator_trait已迁移到Core模块

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
