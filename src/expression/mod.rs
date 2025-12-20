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

pub use expression::AggregateFunction;
pub use expression::BinaryOperator;
pub use expression::DataType;
pub use expression::Expression;
pub use expression::LiteralValue;
pub use expression::UnaryOperator;

pub use context::{ExpressionContext, SimpleExpressionContext, ExpressionContextCore, StorageExpressionContext};
pub use evaluator::ExpressionEvaluator;
pub use evaluator_trait::{ExpressionEvaluator as ExpressionEvaluatorTrait, DefaultExpressionEvaluator, default_evaluator, evaluate_expression, evaluate_expressions};

// Re-export cypher module types for convenience
pub use cypher::{
    CypherEvaluator, CypherExpressionOptimizer, CypherProcessor, ExpressionConverter,
};

// Re-export storage module types for convenience
pub use storage::{Schema, RowReaderWrapper, FieldType, FieldDef, ColumnDef};
