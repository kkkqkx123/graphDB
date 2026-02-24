//! 表达式上下文模块
//!
//! 提供表达式求值过程中的上下文管理，包括缓存、函数、错误处理等功能

pub mod basic_context;
pub mod cache_manager;
pub mod default_context;
pub mod evaluation;
pub mod query_expression_context;
pub mod row_context;
pub mod traits;
pub mod version_manager;

// 重新导出默认上下文类型
pub use default_context::{DefaultExpressionContext, StorageExpressionContext};

// 重新导出 ExpressionContext trait（来自 evaluator::traits）
pub use crate::expression::evaluator::traits::ExpressionContext;

// 重新导出基础上下文相关类型
pub use basic_context::{BasicExpressionContext, ExpressionContextType};

// 重新导出行上下文类型
pub use row_context::{RowContextRef, RowExpressionContext, RowExpressionContextBuilder};

// 重新导出查询表达式上下文
pub use query_expression_context::QueryExpressionContext;

// 重新导出组件
pub use cache_manager::CacheManager;
pub use version_manager::VersionManager;

// 重新导出分解 trait
pub use traits::{CacheContext, FunctionContext, GraphContext, ScopedContext, VariableContext};
