//! 表达式上下文模块
//!
//! 提供表达式求值过程中的上下文管理，包括缓存、函数、错误处理等功能

pub mod cache_manager;
pub mod default_context;
pub mod row_context;

// 重新导出默认上下文类型
pub use default_context::DefaultExpressionContext;

// 重新导出 ExpressionContext trait（来自 evaluator::traits）
pub use crate::expression::evaluator::traits::ExpressionContext;

// 重新导出行上下文类型
pub use row_context::RowExpressionContext;

// 重新导出组件
pub use cache_manager::CacheManager;
