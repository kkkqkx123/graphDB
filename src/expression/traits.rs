//! 表达式模块特征定义
//!
//! 提供表达式求值所需的所有特征定义，包括：
//! - ExpressionContext: 主上下文特征（来自 evaluator::traits）
//! - VariableContext: 变量管理特征（来自 context::traits）
//! - FunctionContext: 函数管理特征（来自 context::traits）
//! - CacheContext: 缓存管理特征（来自 context::traits）
//! - GraphContext: 图数据访问特征（来自 context::traits）
//! - ScopedContext: 作用域管理特征（来自 context::traits）

// 从 evaluator 模块重新导出 ExpressionContext
pub use crate::expression::evaluator::traits::ExpressionContext;

// 从 context 模块重新导出分解 trait
pub use crate::expression::context::traits::{
    CacheContext, FunctionContext, GraphContext, ScopedContext, VariableContext,
};
