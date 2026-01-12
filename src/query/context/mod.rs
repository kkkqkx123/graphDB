//! 查询上下文模块 - 重构版本
//!
//! 新的模块结构：
//! - managers/: 管理器接口
//! - execution/: 执行相关上下文
//! - validate/: 验证上下文（保持现有结构）
//! - 其他模块保持不变

pub mod ast;
pub mod request_context;
pub mod runtime_context;
pub mod validate;

// 新的模块结构
pub mod execution;
pub mod managers;

// 重新导出主要类型
pub use ast::*;
pub use request_context::RequestContext;
pub use validate::*;

// 导出新的模块结构
pub use execution::*;

// 使用 core::context 中的 QueryExecutionContext
pub use crate::core::context::QueryExecutionContext;
