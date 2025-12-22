//! 表达式上下文模块
//!
//! 提供表达式求值过程中的上下文管理，包括缓存、函数、错误处理等功能

pub mod basic_context;
pub mod cache;
pub mod default_context;
pub mod error;
pub mod evaluation;
pub mod functions;

// 重新导出默认上下文类型
pub use default_context::{
    with_edge, with_variables, with_vertex, DefaultExpressionContext, ExpressionContext,
    ExpressionContextBuilder, ExpressionContextCore, QueryContextAdapter,
    StorageExpressionContextCore,
};

// 重新导出缓存相关类型
pub use cache::{ExpressionCacheManager, ExpressionCacheStats};

// 重新导出函数相关类型
pub use functions::{
    AggregateFunction, BuiltinFunction, ConversionFunction, CustomFunction, DateTimeFunction,
    ExpressionFunction, FunctionRef, MathFunction, StringFunction,
};

// 重新导出错误相关类型（现在从核心错误模块导出）
pub use crate::core::{ExpressionError, ExpressionErrorType, ExpressionPosition};

// 重新导出基础上下文相关类型
pub use basic_context::{BasicExpressionContext, ExpressionContextType};

// 重新导出求值相关类型
pub use evaluation::{EvaluationOptions, EvaluationStatistics};

// 重新导出扩展trait
pub use basic_context::ExpressionContextCoreExtended;
