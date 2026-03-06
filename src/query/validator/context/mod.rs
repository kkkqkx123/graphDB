//! 验证器上下文模块
//!
//! 提供验证阶段所需的上下文信息，包括表达式分析上下文。

pub mod expression_cache;
pub mod expression_context;

pub use expression_cache::{ExpressionCacheConfig, ExpressionCacheStats, GlobalExpressionCache};
pub use expression_context::{ExpressionAnalysisContext, OptimizationFlags};
