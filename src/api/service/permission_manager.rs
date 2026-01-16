use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Schema,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleType {
    God,
    Admin,
    Dba,
    User,
    Guest,
}

impl RoleType {
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete | Permission::Schema
            ),
            RoleType::Dba => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete
            ),
            RoleType::User => matches!(permission, Permission::Read | Permission::Write),
            RoleType::Guest => matches!(permission, Permission::Read),
        }
    }

    pub fn can_grant(&self, target_role: RoleType) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(
                target_role,
                RoleType::Dba | RoleType::User | RoleType::Guest
            ),
            RoleType::Dba => matches!(target_role, RoleType::User | RoleType::Guest),
            RoleType::User | RoleType::Guest => false,
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
        user_roles.insert("root".to_string(), HashMap::new());

        Self {
            user_roles: Arc::new(RwLock::new(user_roles)),
            space_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn grant_role(&self, username: &str, space_id: i64, role: RoleType) -> Result<()> {
        let mut user_roles = self
            .user_roles
            .write()
            .map_err(|e| anyhow!("获取写锁失败: {}", e))?;

        let roles = user_roles.entry(username.to_string()).or_insert_with(HashMap::new);
        roles.insert(space_id, role);
        Ok(())
    }

    pub fn revoke_role(&self, username: &str, space_id: i64) -> Result<()> {
        let mut user_roles = self
            .user_roles
            .write()
            .map_err(|e| anyhow!("获取写锁失败: {}", e))?;

        if let Some(roles) = user_roles.get_mut(username) {
            roles.remove(&space_id);
        }
        Ok(())
    }

    pub fn get_role(&self, username: &str, space_id: i64) -> Option<RoleType> {
        let user_roles = self.user_roles.read().expect("获取读锁失败");
        user_roles.get(username)?.get(&space_id).copied()
    }

    pub fn check_permission(
        &self,
        username: &str,
        space_id: i64,
        permission: Permission,
    ) -> Result<()> {
        let user_roles = self.user_roles.read().expect("获取读锁失败");

        if let Some(roles) = user_roles.get(username) {
            if let Some(&role) = roles.get(&space_id) {
                if role.has_permission(permission) {
                    return Ok(());
                }
            }
        }

        Err(anyhow!("用户 {} 在空间 {} 没有权限 {:?}", username, space_id, permission))
    }

    pub fn is_god(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read().expect("获取读锁失败");
        user_roles
            .get(username)
            .map_or(false, |roles| roles.values().any(|&role| role == RoleType::God))
    }

    pub fn grant_permission(
        &self,
        space_id: i64,
        username: &str,
        permission: Permission,
    ) -> Result<()> {
        let mut space_permissions = self
            .space_permissions
            .write()
            .map_err(|e| anyhow!("获取写锁失败: {}", e))?;

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
        let mut space_permissions = self
            .space_permissions
            .write()
            .map_err(|e| anyhow!("获取写锁失败: {}", e))?;

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
        let space_permissions = self.space_permissions.read().expect("获取读锁失败");

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
        assert!(RoleType::God.has_permission(Permission::Admin));
        assert!(RoleType::Admin.has_permission(Permission::Schema));
        assert!(!RoleType::Admin.has_permission(Permission::Admin));
        assert!(RoleType::User.has_permission(Permission::Read));
        assert!(!RoleType::User.has_permission(Permission::Schema));
        assert!(RoleType::Guest.has_permission(Permission::Read));
        assert!(!RoleType::Guest.has_permission(Permission::Write));
    }

    #[test]
    fn test_role_grant() {
        assert!(RoleType::God.can_grant(RoleType::Admin));
        assert!(RoleType::Admin.can_grant(RoleType::User));
        assert!(!RoleType::User.can_grant(RoleType::Admin));
    }

    #[test]
    fn test_permission_manager_creation() {
        let pm = PermissionManager::new();
        assert!(pm.is_god("root"));
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
        pm.grant_role("testuser", 1, RoleType::User).unwrap();

        let result = pm.revoke_role("testuser", 1);
        assert!(result.is_ok());

        let role = pm.get_role("testuser", 1);
        assert_eq!(role, None);
    }

    #[test]
    fn test_check_permission() {
        let pm = PermissionManager::new();
        pm.grant_role("testuser", 1, RoleType::User).unwrap();

        assert!(pm.check_permission("testuser", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("testuser", 1, Permission::Write).is_ok());
        assert!(pm.check_permission("testuser", 1, Permission::Delete).is_err());
    }

    #[test]
    fn test_is_god() {
        let pm = PermissionManager::new();
        pm.grant_role("admin", 1, RoleType::God).unwrap();

        assert!(pm.is_god("admin"));
        assert!(!pm.is_god("testuser"));
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
