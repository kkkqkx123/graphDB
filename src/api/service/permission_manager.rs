use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Schema,
    Admin,
}

/// 简化的2级权限模型
/// - Admin: 管理员，拥有所有权限
/// - User: 普通用户，拥有读写权限（不能修改Schema）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleType {
    Admin,
    User,
}

impl RoleType {
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            RoleType::Admin => true,
            RoleType::User => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete
            ),
        }
    }

    pub fn can_grant(&self, target_role: RoleType) -> bool {
        match self {
            RoleType::Admin => matches!(target_role, RoleType::User),
            RoleType::User => false,
        }
    }
}

pub struct PermissionManager {
    user_roles: Arc<RwLock<HashMap<String, HashMap<i64, RoleType>>>>,
    space_permissions: Arc<RwLock<HashMap<i64, HashMap<String, Vec<Permission>>>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut user_roles = HashMap::new();
        let mut root_roles = HashMap::new();
        root_roles.insert(0, RoleType::Admin);
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
        user_roles.get(username)?.get(&space_id).copied()
    }

    pub fn check_permission(
        &self,
        username: &str,
        space_id: i64,
        permission: Permission,
    ) -> Result<()> {
        let user_roles = self.user_roles.read();

        if let Some(roles) = user_roles.get(username) {
            if let Some(&role) = roles.get(&space_id) {
                if role.has_permission(permission) {
                    return Ok(());
                }
            }
        }

        Err(anyhow!("用户 {} 在空间 {} 没有权限 {:?}", username, space_id, permission))
    }

    pub fn is_admin(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read();
        user_roles
            .get(username)
            .map_or(false, |roles| roles.values().any(|&role| role == RoleType::Admin))
    }

    pub fn grant_permission(
        &self,
        space_id: i64,
        username: &str,
        permission: Permission,
    ) -> Result<()> {
        let mut space_permissions = self.space_permissions.write();

        let permissions = space_permissions
            .entry(space_id)
            .or_insert_with(HashMap::new);

        let user_permissions = permissions.entry(username.to_string()).or_insert_with(Vec::new);
        if !user_permissions.contains(&permission) {
            user_permissions.push(permission);
        }

        Ok(())
    }

    pub fn revoke_permission(
        &self,
        space_id: i64,
        username: &str,
        permission: Permission,
    ) -> Result<()> {
        let mut space_permissions = self.space_permissions.write();

        if let Some(permissions) = space_permissions.get_mut(&space_id) {
            if let Some(user_permissions) = permissions.get_mut(username) {
                user_permissions.retain(|p| p != &permission);
            }
        }

        Ok(())
    }

    pub fn check_custom_permission(
        &self,
        space_id: i64,
        username: &str,
        permission: Permission,
    ) -> Result<()> {
        let space_permissions = self.space_permissions.read();

        if let Some(permissions) = space_permissions.get(&space_id) {
            if let Some(user_permissions) = permissions.get(username) {
                if user_permissions.contains(&permission) {
                    return Ok(());
                }
            }
        }

        Err(anyhow!(
            "用户 {} 在空间 {} 没有自定义权限 {:?}",
            username,
            space_id,
            permission
        ))
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_type_permissions() {
        // Admin 拥有所有权限
        assert!(RoleType::Admin.has_permission(Permission::Admin));
        assert!(RoleType::Admin.has_permission(Permission::Schema));
        assert!(RoleType::Admin.has_permission(Permission::Read));
        assert!(RoleType::Admin.has_permission(Permission::Write));
        assert!(RoleType::Admin.has_permission(Permission::Delete));

        // User 拥有读写删权限，但没有Schema和Admin权限
        assert!(RoleType::User.has_permission(Permission::Read));
        assert!(RoleType::User.has_permission(Permission::Write));
        assert!(RoleType::User.has_permission(Permission::Delete));
        assert!(!RoleType::User.has_permission(Permission::Schema));
        assert!(!RoleType::User.has_permission(Permission::Admin));
    }

    #[test]
    fn test_role_grant() {
        // Admin 可以授权 User
        assert!(RoleType::Admin.can_grant(RoleType::User));
        // Admin 不能授权 Admin
        assert!(!RoleType::Admin.can_grant(RoleType::Admin));
        // User 不能授权
        assert!(!RoleType::User.can_grant(RoleType::Admin));
        assert!(!RoleType::User.can_grant(RoleType::User));
    }

    #[test]
    fn test_permission_manager_creation() {
        let pm = PermissionManager::new();
        assert!(pm.is_admin("root"));
    }

    #[test]
    fn test_grant_role() {
        let pm = PermissionManager::new();
        let result = pm.grant_role("testuser", 1, RoleType::User);
        assert!(result.is_ok());

        let role = pm.get_role("testuser", 1);
        assert_eq!(role, Some(RoleType::User));
    }

    #[test]
    fn test_revoke_role() {
        let pm = PermissionManager::new();
        pm.grant_role("testuser", 1, RoleType::User).expect("grant_role should succeed");

        let result = pm.revoke_role("testuser", 1);
        assert!(result.is_ok());

        let role = pm.get_role("testuser", 1);
        assert_eq!(role, None);
    }

    #[test]
    fn test_check_permission() {
        let pm = PermissionManager::new();
        pm.grant_role("testuser", 1, RoleType::User).expect("grant_role should succeed");

        // User 可以读写删
        assert!(pm.check_permission("testuser", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("testuser", 1, Permission::Write).is_ok());
        assert!(pm.check_permission("testuser", 1, Permission::Delete).is_ok());
        // User 不能修改Schema
        assert!(pm.check_permission("testuser", 1, Permission::Schema).is_err());
    }

    #[test]
    fn test_is_admin() {
        let pm = PermissionManager::new();
        pm.grant_role("admin", 1, RoleType::Admin).expect("grant_role should succeed");

        assert!(pm.is_admin("admin"));
        assert!(!pm.is_admin("testuser"));
    }

    #[test]
    fn test_custom_permission() {
        let pm = PermissionManager::new();
        let result = pm.grant_permission(1, "testuser", Permission::Delete);
        assert!(result.is_ok());

        assert!(pm
            .check_custom_permission(1, "testuser", Permission::Delete)
            .is_ok());
    }
}
