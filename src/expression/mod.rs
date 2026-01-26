pub mod aggregate_functions;
pub mod context;
pub mod evaluator;
pub mod functions;
pub mod storage;

// 重新导出expression模块的访问器
pub use crate::core::types::expression::visitor::{
    ExpressionAcceptor, ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionVisitor,
};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// Re-export Core type utils
pub use crate::core::TypeUtils;

// Re-export evaluator module
pub use evaluator::{Evaluator, ExpressionContext, ExpressionEvaluator};

// Re-export context module types
pub use context::{
    BasicExpressionContext, DefaultExpressionContext, ExpressionContextType, StorageExpressionContext,
};

// Re-export error types
pub use crate::core::error::{ExpressionError, ExpressionErrorType, ExpressionPosition};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
