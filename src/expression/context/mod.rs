//! 表达式上下文模块
//!
//! 提供统一的表达式求值上下文实现

pub mod core;
pub mod simple;
pub mod storage;

// 重新导出主要类型
pub use core::ExpressionContextCore;
pub use simple::{SimpleExpressionContext, ExpressionContext, QueryContextAdapter};
pub use storage::StorageExpressionContext;