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
    #[error("Insufficient permission")]
    InsufficientPermission,

    #[error("User {user} has no role in space {space_id}")]
    NoRoleInSpace { user: String, space_id: i64 },

    #[error("Permission denied: {permission} for user {user}")]
    PermissionDenied { permission: String, user: String },

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Failed to grant role: {0}")]
    GrantRoleFailed(String),

    #[error("Failed to revoke role: {0}")]
    RevokeRoleFailed(String),

    #[error("Permission denied: only GOD role can create/delete spaces")]
    OnlyGodCanManageSpaces,

    #[error("Permission denied: only GOD role can manage users")]
    OnlyGodCanManageUsers,

    #[error("Space ID required for read Space operation")]
    SpaceIdRequired,

    #[error("Space ID required for read Schema operation")]
    SchemaSpaceIdRequired,

    #[error("Space ID required for write Schema operation")]
    SchemaWriteSpaceIdRequired,

    #[error("Schema write permission denied: user {user} has insufficient privileges in space {space_id}")]
    SchemaWritePermissionDenied { space_id: i64, user: String },

    #[error("Space ID required for read data operation")]
    DataReadSpaceIdRequired,

    #[error("Space ID required for write data operation")]
    DataWriteSpaceIdRequired,

    #[error("Guest role has no permission to write data")]
    GuestCannotWriteData,

    #[error("No permission to read user information")]
    CannotReadUserInfo,

    #[error("Space ID required for role operation")]
    RoleOperationSpaceIdRequired,

    #[error("Target role required for role operation")]
    RoleOperationTargetRoleRequired,

    #[error("Permission denied: only Admin or God can manage roles")]
    OnlyAdminOrGodCanManageRoles,

    #[error("Permission denied: cannot grant role {role}")]
    CannotGrantRole { role: String },

    #[error("Cannot modify own role")]
    CannotModifyOwnRole,

    #[error("Target user required for change password operation")]
    ChangePasswordTargetUserRequired,

    #[error("Can only change own password")]
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
