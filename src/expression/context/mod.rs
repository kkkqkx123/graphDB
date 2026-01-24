//! 表达式上下文模块
//!
//! 提供表达式求值过程中的上下文管理，包括缓存、函数、错误处理等功能

pub mod basic_context;
pub mod default_context;
pub mod evaluation;
pub mod row_context;

// 重新导出默认上下文类型
pub use default_context::{DefaultExpressionContext, StorageExpressionContext};

// 重新导出统一的ExpressionContext trait
pub use crate::expression::evaluator::traits::ExpressionContext;

// 重新导出基础上下文相关类型
pub use basic_context::{BasicExpressionContext, ExpressionContextType};

// 重新导出求值相关类型
pub use evaluation::{EvaluationOptions, EvaluationStatistics};

// 重新导出错误相关类型（从 core::error）
pub use crate::core::error::{ExpressionError, ExpressionErrorType, ExpressionPosition};

// 重新导出扩展trait
pub use basic_context::ExpressionContextCoreExtended;

// 重新导出行上下文类型
pub use row_context::{RowExpressionContext, RowContextRef, RowExpressionContextBuilder};
