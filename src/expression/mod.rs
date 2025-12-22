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
pub mod visitor;

pub use visitor::{DefaultExpressionVisitor, ExpressionAcceptor, ExpressionVisitor};

// Re-export operators_ext for backward compatibility
pub use operators_ext::{
    ExtendedBinaryOperator, ExtendedUnaryOperator, ExtendedAggregateFunction,
    BinaryOperator, UnaryOperator, AggregateFunction
};

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
