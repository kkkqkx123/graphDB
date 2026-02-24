//! 表达式模块特征定义
//!
//! 提供表达式求值所需的所有特征定义，包括：
//! - ExpressionContext: 主上下文特征（来自 evaluator::traits）

// 从 evaluator 模块重新导出 ExpressionContext
pub use crate::expression::evaluator::traits::ExpressionContext;
