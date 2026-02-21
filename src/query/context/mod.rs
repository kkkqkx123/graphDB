//! 查询上下文模块 - 重构版本
//!
//! 新的模块结构：
//! - execution/: 执行相关上下文
//! - symbol/: 符号表管理
//! - components.rs: 组件访问器
//! - request_context.rs: 请求上下文
//! - runtime_context.rs: 运行时上下文

pub mod ast;
pub mod request_context;
pub mod runtime_context;

// 新的模块结构
pub mod execution;
pub mod symbol;

// 新的重构模块
pub mod components;

// 重新导出主要类型
pub use ast::*;
pub use request_context::RequestContext;

// 导出新的模块结构
pub use execution::*;
pub use symbol::{Symbol, SymbolTable};

// 导出重构的模块
pub use components::{ComponentAccessor, QueryComponents};

// 导出核心执行状态类型（推荐）
pub use crate::query::core::{ExecutorState, RowStatus};
