//! 查询上下文模块 - 重构版本
//!
//! 新的模块结构：
//! - query_context.rs: 核心查询上下文
//! - execution_context.rs: 执行上下文
//! - expression_context.rs: 表达式上下文
//! - ast_context.rs: AST上下文
//! - managers/: 管理器接口
//! - validate/: 验证上下文（保持现有结构）

// 核心上下文模块
pub mod query_context;
pub mod execution_context;
pub mod expression_context;
pub mod ast_context;

// 保留的模块
pub mod managers;
pub mod validate;

// 重新导出主要类型
pub use query_context::{QueryContext, QueryStatistics, Function};
pub use execution_context::{ExecutionContext, ExecutionState, ResourceManager, ExecutionMetrics};
pub use expression_context::ExpressionContext;
pub use ast_context::{AstContext, ColumnDefinition, VariableInfo, VariableType, Statement};

// 导出管理器接口
pub use managers::*;

// 导出验证相关
pub use validate::*;

// 测试模块
#[cfg(test)]
mod tests;