//! 查询上下文模块
//! 
//! 包含与查询处理流程相关的各种上下文：
//! - AST上下文：表示解析的查询
//! - 验证上下文：验证阶段的上下文信息  
//! - 查询上下文：整个查询请求的上下文
//! - 查询执行上下文：查询执行期间的上下文（管理变量的多版本历史）
//! - 表达式求值上下文：为表达式求值提供运行时上下文
//!
//! 注意：这里的上下文都是查询级别的，不同于应用级别的 services::context 中的上下文

pub mod ast_context;
pub mod validate_context;
pub mod query_context;
pub mod execution_context;
pub mod expression_context;

pub use ast_context::*;
pub use validate_context::*;
pub use query_context::*;
pub use execution_context::{QueryExecutionContext};
pub use expression_context::*;
