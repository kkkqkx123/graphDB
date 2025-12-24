pub mod aggregate_functions;
pub mod storage;
pub mod visitor;
pub mod evaluator;
pub mod context;
pub mod functions;
pub mod cache;

// 重新导出expression模块的访问器
pub use visitor::{ExpressionAcceptor, ExpressionVisitor, ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionTypeFilter};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// Re-export Core expression types
pub use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};

// Re-export evaluator module
pub use evaluator::{ExpressionEvaluator, Evaluator, ExpressionContext};

// Re-export context module types
pub use context::{DefaultExpressionContext, StorageExpressionContext, BasicExpressionContext, ExpressionContextType, ExpressionContextCoreExtended};

// Re-export functions module types
pub use functions::{BuiltinFunction, ConversionFunction, CustomFunction, DateTimeFunction, ExpressionFunction, FunctionRef, MathFunction, StringFunction};

// Re-export cache module types
pub use cache::{ExpressionCacheManager, ExpressionCacheStats};

// Re-export evaluation types
pub use context::{EvaluationOptions, EvaluationStatistics};

// Re-export error types
pub use context::error::{ExpressionError, ExpressionErrorType, ExpressionPosition};

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
