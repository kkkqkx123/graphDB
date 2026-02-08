//! 查询上下文模块 - 重构版本
//!
//! 新的模块结构：
//! - execution/: 执行相关上下文
//! - validate/: 验证上下文（保持现有结构）
//! - symbol/: 符号表管理
//! - core_query_context.rs: 核心查询上下文
//! - components.rs: 组件访问器
//! - request_context.rs: 请求上下文
//! - runtime_context.rs: 运行时上下文

pub mod ast;
pub mod request_context;
pub mod runtime_context;
pub mod validate;

// 新的模块结构
pub mod execution;
pub mod symbol;

// 新的重构模块
pub mod core_query_context;
pub mod components;

// 重新导出主要类型
pub use ast::*;
pub use request_context::RequestContext;
pub use validate::*;

// 导出新的模块结构
pub use execution::*;
pub use symbol::{Symbol, SymbolTable};

// 导出重构的模块
pub use core_query_context::CoreQueryContext;
pub use components::{ComponentAccessor, QueryComponents};

// 导出核心执行状态类型（推荐）
pub use crate::query::core::{ExecutorState, RowStatus};
