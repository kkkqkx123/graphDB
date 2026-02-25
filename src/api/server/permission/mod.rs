//! 权限管理模块
//!
//! 提供用户权限检查和验证功能

pub mod permission_manager;
pub mod permission_checker;

// 从 core 层重新导出权限类型
pub use crate::core::{Permission, RoleType};

pub use permission_manager::{PermissionManager, GOD_SPACE_ID};
pub use permission_checker::PermissionChecker;
