//! 上下文系统模块
//!
//! 提供类型安全的上下文管理系统，包括查询上下文、执行上下文、会话上下文等

use crate::core::Value;

/// 上下文类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextType {
    /// 会话上下文
    Session,
    /// 查询上下文
    Query,
    /// 执行上下文
    Execution,
    /// 表达式上下文
    Expression,
    /// 请求上下文
    Request,
    /// 运行时上下文
    Runtime,
    /// 验证上下文
    Validation,
    /// 存储上下文
    Storage,
}

pub mod base;
pub mod execution;
pub mod manager;
pub mod query;
pub mod query_execution;
pub mod request;
pub mod runtime;
pub mod session;
pub mod storage;
pub mod traits;
pub mod validation;

// 重新导出常用类型
pub use base::*;
pub use execution::*;
pub use manager::*;
pub use query::*;
pub use query_execution::*;
pub use request::*;
pub use runtime::*;
pub use session::*;
pub use storage::*;
pub use traits::*;
pub use validation::*;
