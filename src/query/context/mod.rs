//! 查询上下文模块 - 重构版本
//!
//! 新的模块结构：
//! - managers/: 管理器接口
//! - execution/: 执行相关上下文
//! - validate/: 验证上下文（保持现有结构）
//! - 其他模块保持不变

pub mod ast_context;
pub mod validate;
pub mod request_context;
pub mod execution_context;
pub mod expression_context;
pub mod expression_eval_context;
pub mod runtime_context;
pub mod expression;

// 新的模块结构
pub mod managers;
pub mod execution;

// 重新导出主要类型
pub use ast_context::*;
pub use validate::*;
pub use request_context::RequestContext;
pub use execution_context::{QueryExecutionContext};
pub use expression_context::*;
pub use expression_eval_context::*;
pub use runtime_context::*;
pub use expression::*;

// 导出新的模块结构
pub use managers::*;
pub use execution::*;

// 为了向后兼容，重新导出QueryContext
pub use execution::QueryContext;
