//! 权限类型定义
//!
//! 提供核心的权限模型和角色类型定义

/// 权限类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Schema,
    Admin,
}

/// 5级权限模型 - 参考nebula-graph实现
/// - God: 全局超级管理员，拥有所有权限（类似Linux root）
/// - Admin: Space管理员，可以管理Space内的Schema和用户
/// - Dba: 数据库管理员，可以修改Schema
/// - User: 普通用户，可以读写数据
/// - Guest: 只读用户，只能读取数据
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleType {
    God = 0x01,
    Admin = 0x02,
    Dba = 0x03,
    User = 0x04,
    Guest = 0x05,
}

impl RoleType {
    /// 检查角色是否拥有指定权限
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete | Permission::Schema | Permission::Admin
            ),
            RoleType::Dba => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete | Permission::Schema
            ),
            RoleType::User => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete
            ),
            RoleType::Guest => matches!(permission, Permission::Read),
        }
    }

    /// 检查是否可以授予指定角色
    pub fn can_grant(&self, target_role: RoleType) -> bool {
        match self {
            RoleType::God => target_role != RoleType::God,
            RoleType::Admin => matches!(target_role, RoleType::Dba | RoleType::User | RoleType::Guest),
            RoleType::Dba => matches!(target_role, RoleType::User | RoleType::Guest),
            _ => false,
        }
    }

    /// 检查是否可以撤销指定角色
    pub fn can_revoke(&self, target_role: RoleType) -> bool {
        self.can_grant(target_role)
    }

    /// 从字节解析角色类型
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(RoleType::God),
            0x02 => Some(RoleType::Admin),
            0x03 => Some(RoleType::Dba),
            0x04 => Some(RoleType::User),
            0x05 => Some(RoleType::Guest),
            _ => None,
        }
    }

    /// 转换为字节
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

impl std::fmt::Display for RoleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleType::God => write!(f, "GOD"),
            RoleType::Admin => write!(f, "ADMIN"),
            RoleType::Dba => write!(f, "DBA"),
            RoleType::User => write!(f, "USER"),
            RoleType::Guest => write!(f, "GUEST"),
        }
    }
}

impl std::str::FromStr for RoleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GOD" => Ok(RoleType::God),
            "ADMIN" => Ok(RoleType::Admin),
            "DBA" => Ok(RoleType::Dba),
            "USER" => Ok(RoleType::User),
            "GUEST" => Ok(RoleType::Guest),
            _ => Err(format!("Unknown role type: {}", s)),
        }
    }
}
