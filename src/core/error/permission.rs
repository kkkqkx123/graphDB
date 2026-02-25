//! 权限错误类型
//!
//! 涵盖权限管理相关的错误

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// 权限操作结果类型别名
pub type PermissionResult<T> = Result<T, PermissionError>;

/// 权限相关错误
#[derive(Error, Debug, Clone)]
pub enum PermissionError {
    #[error("权限不足")]
    InsufficientPermission,
    
    #[error("角色不存在: {0}")]
    RoleNotFound(String),
    
    #[error("无法授予角色: {0}")]
    GrantRoleFailed(String),
    
    #[error("无法撤销角色: {0}")]
    RevokeRoleFailed(String),
    
    #[error("用户不存在: {0}")]
    UserNotFound(String),
}

impl ToPublicError for PermissionError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        ErrorCode::PermissionDenied
    }

    fn to_public_message(&self) -> String {
        self.to_string()
    }
}
