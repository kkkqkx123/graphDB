//! Session Error Type
//!
//! Covering session management related errors

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// Session operation result type alias
pub type SessionResult<T> = Result<T, SessionError>;

/// Session-related errors
#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(i64),

    #[error("Session expired")]
    SessionExpired,

    #[error("Maximum connections exceeded")]
    MaxConnectionsExceeded,

    #[error("Query not found: {0}")]
    QueryNotFound(u32),

    #[error("Failed to kill session: {0}")]
    KillSessionFailed(String),

    #[error("Session manager error: {0}")]
    ManagerError(String),

    #[error("Insufficient permission to perform this operation")]
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
