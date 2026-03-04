pub mod evaluation_context;
pub mod evaluator;
pub mod functions;

// 从 core 重新导出操作符类型
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// 从 core 重新导出类型工具
pub use crate::core::TypeUtils;

// 从 core 重新导出错误类型
pub use crate::core::error::{ExpressionError, ExpressionErrorType, ExpressionPosition};

// 从 evaluator 模块重新导出 ExpressionContext trait 和求值器
pub use evaluator::{ExpressionContext, ExpressionEvaluator};

// 从 evaluation_context 模块重新导出上下文类型
pub use evaluation_context::{DefaultExpressionContext, RowExpressionContext};
