//! 验证器上下文模块
//!
//! 提供验证阶段所需的上下文信息。

pub mod expression_context;

pub use expression_context::{ExpressionAnalysisContext, OptimizationFlags};
