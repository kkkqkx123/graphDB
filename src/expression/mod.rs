pub mod aggregate_functions;
pub mod storage;
pub mod visitor;
pub mod types;

// 重新导出expression模块的访问器
pub use visitor::{ExpressionAcceptor, ExpressionVisitor, ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionTypeFilter};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// Re-export Core expression types
pub use types::{DataType, Expression, ExpressionType, LiteralValue};

// Re-export Core evaluator
pub use crate::core::evaluator::ExpressionEvaluator;

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
