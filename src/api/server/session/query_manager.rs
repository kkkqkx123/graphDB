//! 查询管理器
//!
//! 负责跟踪和管理正在运行的查询。
//! 注意：实际实现已移动到 query::query_manager，此模块仅用于向后兼容

pub use crate::query::{QueryManager, QueryInfo, QueryStatus, QueryStats, GLOBAL_QUERY_MANAGER, init_global_query_manager, get_global_query_manager};
