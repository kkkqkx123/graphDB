pub mod aggregate;
pub mod aggregate_functions;
pub mod arithmetic;
pub mod binary;
pub mod comparison;
pub mod container;
pub mod context;
pub mod cypher;
pub mod evaluator;
pub mod evaluator_trait;
pub mod expression;
pub mod function;
pub mod operator_conversion;
pub mod property;
pub mod storage;
pub mod type_conversion;
pub mod unary;
pub mod visitor;

pub use visitor::{ExpressionVisitor, ExpressionAcceptor, DefaultExpressionVisitor};

// 从Core模块重新导出表达式类型
pub use crate::core::types::expression::{
    AggregateFunction, BinaryOperator, DataType, Expression, LiteralValue, UnaryOperator,
};

pub use context::{
    DefaultExpressionContext, ExpressionContext, ExpressionContextCore, StorageExpressionContext,
};
pub use evaluator::ExpressionEvaluator;
pub use evaluator_trait::{
    default_evaluator, evaluate_expression, evaluate_expressions, DefaultExpressionEvaluator,
    ExpressionEvaluator as ExpressionEvaluatorTrait,
};

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
