//! 会话错误类型
//!
//! 涵盖会话管理相关的错误

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// 会话操作结果类型别名
pub type SessionResult<T> = Result<T, SessionError>;

/// 会话相关错误
#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("会话不存在: {0}")]
    SessionNotFound(i64),

    #[error("会话已过期")]
    SessionExpired,

    #[error("超过最大连接数限制")]
    MaxConnectionsExceeded,

    #[error("查询不存在: {0}")]
    QueryNotFound(u32),

    #[error("无法终止会话: {0}")]
    KillSessionFailed(String),

    #[error("会话管理器错误: {0}")]
    ManagerError(String),

    #[error("权限不足，无法执行此操作")]
    InsufficientPermission,
}

impl ToPublicError for SessionError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            SessionError::SessionNotFound(_) => ErrorCode::ResourceNotFound,
            SessionError::SessionExpired => ErrorCode::Unauthorized,
            SessionError::MaxConnectionsExceeded => ErrorCode::ResourceExhausted,
            SessionError::QueryNotFound(_) => ErrorCode::ResourceNotFound,
            SessionError::KillSessionFailed(_) => ErrorCode::InternalError,
            SessionError::ManagerError(_) => ErrorCode::InternalError,
            SessionError::InsufficientPermission => ErrorCode::PermissionDenied,
        }
    }

    fn to_public_message(&self) -> String {
        self.to_string()
    }
}
