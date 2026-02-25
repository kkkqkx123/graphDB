//! 请求上下文模块 - 管理查询请求的上下文信息
//! 对应原C++中的RequestContext.h
//!
//! 注意：实际实现已移动到 query::request_context，此模块仅用于向后兼容

pub use crate::query::request_context::{RequestContext, RequestParams, Response, SessionInfo};
