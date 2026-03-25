//! 客户端会话模块
//!
//! 将 ClientSession 的多重职责拆分为独立的上下文模块：
//! - `session`: 基础会话信息
//! - `space_context`: 空间上下文
//! - `role_context`: 角色上下文
//! - `query_context`: 查询上下文
//! - `transaction_context`: 事务上下文
//! - `statistics`: 统计信息

pub mod client_session;
pub mod query_context;
pub mod role_context;
pub mod session;
pub mod space_context;
pub mod statistics;
pub mod transaction_context;

pub use client_session::ClientSession;
pub use query_context::QueryContext;
pub use role_context::RoleContext;
pub use session::{Session, SpaceInfo};
pub use space_context::SpaceContext;
pub use transaction_context::TransactionContext;
