//! 网络会话管理模块
//!
//! 提供网络连接会话的生命周期管理

pub mod request_context;
pub mod session_manager;

pub use crate::api::server::client::{ClientSession, Session, SpaceInfo};
pub use request_context::{build_query_request_context, RequestContext};
pub use session_manager::{GraphSessionManager, SessionInfo, DEFAULT_SESSION_IDLE_TIMEOUT};
