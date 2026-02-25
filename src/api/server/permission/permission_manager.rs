use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// 从 core 层重新导出权限类型
pub use crate::core::{Permission, RoleType};

/// God角色的Space ID标记（全局角色，不绑定特定Space）
pub const GOD_SPACE_ID: i64 = -1;

pub struct PermissionManager {
    /// 用户角色映射：username -> {space_id -> role}
    /// 注意：God角色使用特殊的space_id: -1表示全局角色，不绑定特定Space
    user_roles: Arc<RwLock<HashMap<String, HashMap<i64, RoleType>>>>,
    space_permissions: Arc<RwLock<HashMap<i64, HashMap<String, Vec<Permission>>>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut user_roles = HashMap::new();
        let mut root_roles = HashMap::new();
        // root用户作为God角色（全局超级管理员）
        // 使用GOD_SPACE_ID(-1)表示全局角色，不绑定特定Space
        root_roles.insert(GOD_SPACE_ID, RoleType::God);
        user_roles.insert("root".to_string(), root_roles);

        Self {
            user_roles: Arc::new(RwLock::new(user_roles)),
            space_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn grant_role(&self, username: &str, space_id: i64, role: RoleType) -> Result<()> {
        let mut user_roles = self.user_roles.write();

        let roles = user_roles.entry(username.to_string()).or_insert_with(HashMap::new);
        roles.insert(space_id, role);
        Ok(())
    }

    pub fn revoke_role(&self, username: &str, space_id: i64) -> Result<()> {
        let mut user_roles = self.user_roles.write();

        if let Some(roles) = user_roles.get_mut(username) {
            roles.remove(&space_id);
        }
        Ok(())
    }

    pub fn get_role(&self, username: &str, space_id: i64) -> Option<RoleType> {
        let user_roles = self.user_roles.read();
        user_roles.get(username)
            .and_then(|roles| roles.get(&space_id).copied())
    }

    pub fn check_permission(&self, username: &str, space_id: i64, permission: Permission) -> Result<()> {
        let role = self.get_role(username, space_id)
            .ok_or_else(|| anyhow!("User {} has no role in space {}", username, space_id))?;

        if role.has_permission(permission) {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: {:?} for user {}", permission, username))
        }
    }

    pub fn can_grant_role(&self, granter: &str, space_id: i64, target_role: RoleType) -> bool {
        let user_roles = self.user_roles.read();
        if let Some(roles) = user_roles.get(granter) {
            if let Some(&role) = roles.get(&space_id) {
                return role.can_grant(target_role);
            }
        }
        false
    }

    pub fn can_revoke_role(&self, revoker: &str, space_id: i64, target_role: RoleType) -> bool {
        self.can_grant_role(revoker, space_id, target_role)
    }

    pub fn get_user_roles(&self, username: &str) -> HashMap<i64, RoleType> {
        let user_roles = self.user_roles.read();
        user_roles.get(username).cloned().unwrap_or_default()
    }

    pub fn is_god(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read();
        if let Some(roles) = user_roles.get(username) {
            return roles.values().any(|&role| role == RoleType::God);
        }
        false
    }

    /// 检查用户是否是管理员（God 或 Admin 角色）
    pub fn is_admin(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read();
        if let Some(roles) = user_roles.get(username) {
            return roles.values().any(|&role| matches!(role, RoleType::God | RoleType::Admin));
        }
        false
    }

    /// 检查用户是否可以读取指定空间
    pub fn can_read_space(&self, username: &str, space_id: i64) -> Result<()> {
        // 首先检查是否是 God 角色（全局权限）
        if self.is_god(username) {
            return Ok(());
        }
        
        // 检查特定空间的权限
        let role = self.get_role(username, space_id)
            .or_else(|| self.get_role(username, GOD_SPACE_ID))
            .ok_or_else(|| anyhow!("User {} has no role in space {}", username, space_id))?;

        if role.has_permission(Permission::Read) {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: read space {} for user {}", space_id, username))
        }
    }

    /// 检查用户是否可以写入空间（只有 God 角色可以创建/删除空间）
    pub fn can_write_space(&self, username: &str) -> Result<()> {
        if self.is_god(username) {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: only GOD role can create/drop spaces"))
        }
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
