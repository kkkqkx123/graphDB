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

    #[error("用户 {0} 在空间 {1} 中没有角色")]
    NoRoleInSpace(String, i64),

    #[error("权限被拒绝: {permission:?} for user {user}")]
    PermissionDenied { permission: String, user: String },

    #[error("权限被拒绝: 只有 GOD 角色可以创建/删除空间")]
    OnlyGodCanManageSpaces,

    #[error("权限被拒绝: 只有 GOD 角色可以管理用户")]
    OnlyGodCanManageUsers,

    #[error("读取Space操作需要提供Space ID")]
    SpaceIdRequired,

    #[error("读取Schema操作需要提供Space ID")]
    SchemaSpaceIdRequired,

    #[error("写入Schema操作需要提供Space ID")]
    SchemaWriteSpaceIdRequired,

    #[error("写入Schema失败: 在空间 {space_id} 中用户 {user} 权限不足")]
    SchemaWritePermissionDenied { space_id: i64, user: String },

    #[error("读取数据操作需要提供Space ID")]
    DataReadSpaceIdRequired,

    #[error("写入数据操作需要提供Space ID")]
    DataWriteSpaceIdRequired,

    #[error("Guest角色没有写入数据的权限")]
    GuestCannotWriteData,

    #[error("没有权限读取用户信息")]
    CannotReadUserInfo,

    #[error("角色操作需要提供Space ID")]
    RoleOperationSpaceIdRequired,

    #[error("角色操作需要提供目标角色")]
    RoleOperationTargetRoleRequired,

    #[error("权限被拒绝: 只有 Admin 或 God 可以管理角色")]
    OnlyAdminOrGodCanManageRoles,

    #[error("权限被拒绝: 无法授予角色 {role:?}")]
    CannotGrantRole { role: String },

    #[error("修改密码操作需要提供目标用户")]
    ChangePasswordTargetUserRequired,

    #[error("只能修改自己的密码")]
    CanOnlyChangeOwnPassword,
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
