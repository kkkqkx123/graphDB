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

    pub fn can_grant(&self, target_role: RoleType) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(target_role, RoleType::Dba | RoleType::User | RoleType::Guest),
            RoleType::Dba => matches!(target_role, RoleType::User | RoleType::Guest),
            _ => false,
        }
    }
}

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
        user_roles.get(username)?.get(&space_id).copied()
    }

    pub fn check_permission(
        &self,
        username: &str,
        space_id: i64,
        permission: Permission,
    ) -> Result<()> {
        // God角色拥有所有权限
        if self.is_god(username) {
            return Ok(());
        }

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

    /// 检查用户是否是God角色（全局超级管理员）
    pub fn is_god(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read();
        user_roles
            .get(username)
            .map_or(false, |roles| roles.values().any(|&role| role == RoleType::God))
    }

    /// 检查用户是否是Admin角色（Space管理员）
    pub fn is_admin(&self, username: &str) -> bool {
        let user_roles = self.user_roles.read();
        user_roles
            .get(username)
            .map_or(false, |roles| roles.values().any(|&role| role == RoleType::Admin || role == RoleType::God))
    }

    /// 检查是否可以读取Space（只要有任何角色就可以读取）
    pub fn can_read_space(&self, username: &str, space_id: i64) -> Result<()> {
        // God角色可以读取任何Space
        if self.is_god(username) {
            return Ok(());
        }

        let user_roles = self.user_roles.read();

        if let Some(roles) = user_roles.get(username) {
            if roles.contains_key(&space_id) {
                return Ok(());
            }
        }

        Err(anyhow!("用户 {} 没有访问空间 {} 的权限", username, space_id))
    }

    /// 检查是否可以写入Space（创建Space）- 只有God可以
    pub fn can_write_space(&self, username: &str) -> Result<()> {
        if self.is_god(username) {
            Ok(())
        } else {
            Err(anyhow!("用户 {} 没有创建空间的权限，需要God角色", username))
        }
    }

    /// 检查是否可以写入Schema
    pub fn can_write_schema(&self, username: &str, space_id: i64) -> Result<()> {
        self.check_permission(username, space_id, Permission::Schema)
    }

    /// 检查是否可以写入角色
    pub fn can_write_role(
        &self,
        username: &str,
        target_role: RoleType,
        space_id: i64,
        target_user: &str,
    ) -> Result<()> {
        // 不能修改自己的角色
        if username == target_user {
            return Err(anyhow!("不能修改自己的角色"));
        }

        // God可以授予任何角色
        if self.is_god(username) {
            return Ok(());
        }

        let user_roles = self.user_roles.read();
        
        // 获取当前用户在目标Space的角色
        match user_roles.get(username).and_then(|roles| roles.get(&space_id)) {
            Some(&role) => {
                // 检查是否可以授予目标角色
                if role.can_grant(target_role) {
                    Ok(())
                } else {
                    Err(anyhow!("角色 {:?} 不能授予角色 {:?}", role, target_role))
                }
            }
            None => Err(anyhow!("用户 {} 在空间 {} 没有管理角色的权限", username, space_id)),
        }
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

    /// 列出用户的所有角色
    pub fn list_user_roles(&self, username: &str) -> Vec<(i64, RoleType)> {
        let user_roles = self.user_roles.read();
        user_roles
            .get(username)
            .map(|roles| roles.iter().map(|(&space_id, &role)| (space_id, role)).collect())
            .unwrap_or_default()
    }

    /// 列出Space中的所有用户及其角色
    pub fn list_space_users(&self, space_id: i64) -> Vec<(String, RoleType)> {
        let user_roles = self.user_roles.read();
        user_roles
            .iter()
            .filter_map(|(username, roles)| {
                roles.get(&space_id).map(|&role| (username.clone(), role))
            })
            .collect()
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
    fn test_god_role_permissions() {
        // God 拥有所有权限
        assert!(RoleType::God.has_permission(Permission::Admin));
        assert!(RoleType::God.has_permission(Permission::Schema));
        assert!(RoleType::God.has_permission(Permission::Read));
        assert!(RoleType::God.has_permission(Permission::Write));
        assert!(RoleType::God.has_permission(Permission::Delete));
        
        // God 可以授予任何角色
        assert!(RoleType::God.can_grant(RoleType::God));
        assert!(RoleType::God.can_grant(RoleType::Admin));
        assert!(RoleType::God.can_grant(RoleType::Dba));
        assert!(RoleType::God.can_grant(RoleType::User));
        assert!(RoleType::God.can_grant(RoleType::Guest));
    }

    #[test]
    fn test_admin_role_permissions() {
        // Admin 拥有所有权限
        assert!(RoleType::Admin.has_permission(Permission::Admin));
        assert!(RoleType::Admin.has_permission(Permission::Schema));
        assert!(RoleType::Admin.has_permission(Permission::Read));
        assert!(RoleType::Admin.has_permission(Permission::Write));
        assert!(RoleType::Admin.has_permission(Permission::Delete));
        
        // Admin 可以授予 Dba, User, Guest
        assert!(RoleType::Admin.can_grant(RoleType::Dba));
        assert!(RoleType::Admin.can_grant(RoleType::User));
        assert!(RoleType::Admin.can_grant(RoleType::Guest));
        // Admin 不能授予 God 或 Admin
        assert!(!RoleType::Admin.can_grant(RoleType::God));
        assert!(!RoleType::Admin.can_grant(RoleType::Admin));
    }

    #[test]
    fn test_dba_role_permissions() {
        // Dba 拥有读写删和Schema权限，没有Admin权限
        assert!(RoleType::Dba.has_permission(Permission::Schema));
        assert!(RoleType::Dba.has_permission(Permission::Read));
        assert!(RoleType::Dba.has_permission(Permission::Write));
        assert!(RoleType::Dba.has_permission(Permission::Delete));
        assert!(!RoleType::Dba.has_permission(Permission::Admin));
        
        // Dba 可以授予 User, Guest
        assert!(RoleType::Dba.can_grant(RoleType::User));
        assert!(RoleType::Dba.can_grant(RoleType::Guest));
        // Dba 不能授予 God, Admin, Dba
        assert!(!RoleType::Dba.can_grant(RoleType::God));
        assert!(!RoleType::Dba.can_grant(RoleType::Admin));
        assert!(!RoleType::Dba.can_grant(RoleType::Dba));
    }

    #[test]
    fn test_user_role_permissions() {
        // User 拥有读写删权限，没有Schema和Admin权限
        assert!(RoleType::User.has_permission(Permission::Read));
        assert!(RoleType::User.has_permission(Permission::Write));
        assert!(RoleType::User.has_permission(Permission::Delete));
        assert!(!RoleType::User.has_permission(Permission::Schema));
        assert!(!RoleType::User.has_permission(Permission::Admin));
        
        // User 不能授予任何角色
        assert!(!RoleType::User.can_grant(RoleType::God));
        assert!(!RoleType::User.can_grant(RoleType::Admin));
        assert!(!RoleType::User.can_grant(RoleType::Dba));
        assert!(!RoleType::User.can_grant(RoleType::User));
        assert!(!RoleType::User.can_grant(RoleType::Guest));
    }

    #[test]
    fn test_guest_role_permissions() {
        // Guest 只有读权限
        assert!(RoleType::Guest.has_permission(Permission::Read));
        assert!(!RoleType::Guest.has_permission(Permission::Write));
        assert!(!RoleType::Guest.has_permission(Permission::Delete));
        assert!(!RoleType::Guest.has_permission(Permission::Schema));
        assert!(!RoleType::Guest.has_permission(Permission::Admin));
        
        // Guest 不能授予任何角色
        assert!(!RoleType::Guest.can_grant(RoleType::User));
        assert!(!RoleType::Guest.can_grant(RoleType::Guest));
    }

    #[test]
    fn test_permission_manager_creation() {
        let pm = PermissionManager::new();
        // root用户应该是God角色
        assert!(pm.is_god("root"));
        assert!(pm.is_admin("root"));
    }

    #[test]
    fn test_god_space_id() {
        // 验证God角色使用特殊的Space ID
        assert_eq!(GOD_SPACE_ID, -1);
    }

    #[test]
    fn test_grant_role() {
        let pm = PermissionManager::new();
        
        // 授予不同角色
        assert!(pm.grant_role("user1", 1, RoleType::User).is_ok());
        assert!(pm.grant_role("dba1", 1, RoleType::Dba).is_ok());
        assert!(pm.grant_role("admin1", 1, RoleType::Admin).is_ok());
        assert!(pm.grant_role("guest1", 1, RoleType::Guest).is_ok());

        assert_eq!(pm.get_role("user1", 1), Some(RoleType::User));
        assert_eq!(pm.get_role("dba1", 1), Some(RoleType::Dba));
        assert_eq!(pm.get_role("admin1", 1), Some(RoleType::Admin));
        assert_eq!(pm.get_role("guest1", 1), Some(RoleType::Guest));
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
    fn test_check_permission_by_role() {
        let pm = PermissionManager::new();
        
        // 测试不同角色的权限
        pm.grant_role("guest", 1, RoleType::Guest).expect("Failed to grant guest role");
        pm.grant_role("user", 1, RoleType::User).expect("Failed to grant user role");
        pm.grant_role("dba", 1, RoleType::Dba).expect("Failed to grant dba role");
        pm.grant_role("admin", 1, RoleType::Admin).expect("Failed to grant admin role");

        // Guest 只有读权限
        assert!(pm.check_permission("guest", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("guest", 1, Permission::Write).is_err());

        // User 有读写删权限
        assert!(pm.check_permission("user", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("user", 1, Permission::Write).is_ok());
        assert!(pm.check_permission("user", 1, Permission::Delete).is_ok());
        assert!(pm.check_permission("user", 1, Permission::Schema).is_err());

        // Dba 有Schema权限
        assert!(pm.check_permission("dba", 1, Permission::Schema).is_ok());
        assert!(pm.check_permission("dba", 1, Permission::Admin).is_err());

        // Admin 有所有权限
        assert!(pm.check_permission("admin", 1, Permission::Admin).is_ok());
    }

    #[test]
    fn test_god_can_access_any_space() {
        let pm = PermissionManager::new();
        
        // God角色可以读取任何Space
        assert!(pm.can_read_space("root", 1).is_ok());
        assert!(pm.can_read_space("root", 999).is_ok());
        
        // God可以写入Space
        assert!(pm.can_write_space("root").is_ok());
        
        // God可以写入Schema
        assert!(pm.can_write_schema("root", 1).is_ok());
    }

    #[test]
    fn test_can_write_role_hierarchy() {
        let pm = PermissionManager::new();
        
        // 设置角色
        pm.grant_role("admin", 1, RoleType::Admin).expect("Failed to grant admin role");
        pm.grant_role("dba", 1, RoleType::Dba).expect("Failed to grant dba role");
        pm.grant_role("user", 1, RoleType::User).expect("Failed to grant user role");
        
        // Admin 可以授予 Dba, User, Guest
        assert!(pm.can_write_role("admin", RoleType::Dba, 1, "target").is_ok());
        assert!(pm.can_write_role("admin", RoleType::User, 1, "target").is_ok());
        assert!(pm.can_write_role("admin", RoleType::Guest, 1, "target").is_ok());
        // Admin 不能授予 God, Admin
        assert!(pm.can_write_role("admin", RoleType::God, 1, "target").is_err());
        assert!(pm.can_write_role("admin", RoleType::Admin, 1, "target").is_err());
        
        // Dba 可以授予 User, Guest
        assert!(pm.can_write_role("dba", RoleType::User, 1, "target").is_ok());
        assert!(pm.can_write_role("dba", RoleType::Guest, 1, "target").is_ok());
        // Dba 不能授予 God, Admin, Dba
        assert!(pm.can_write_role("dba", RoleType::God, 1, "target").is_err());
        assert!(pm.can_write_role("dba", RoleType::Admin, 1, "target").is_err());
        assert!(pm.can_write_role("dba", RoleType::Dba, 1, "target").is_err());
        
        // User 不能授予任何角色
        assert!(pm.can_write_role("user", RoleType::Guest, 1, "target").is_err());
    }

    #[test]
    fn test_cannot_modify_own_role() {
        let pm = PermissionManager::new();
        
        // 即使是God也不能修改自己的角色
        assert!(pm.can_write_role("root", RoleType::User, 1, "root").is_err());
    }

    #[test]
    fn test_list_user_roles() {
        let pm = PermissionManager::new();
        
        pm.grant_role("user1", 1, RoleType::Admin).expect("Failed to grant admin role to user1");
        pm.grant_role("user1", 2, RoleType::User).expect("Failed to grant user role to user1");
        
        let roles = pm.list_user_roles("user1");
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&(1, RoleType::Admin)));
        assert!(roles.contains(&(2, RoleType::User)));
    }

    #[test]
    fn test_list_space_users() {
        let pm = PermissionManager::new();
        
        pm.grant_role("user1", 1, RoleType::Admin).expect("Failed to grant admin role to user1");
        pm.grant_role("user2", 1, RoleType::User).expect("Failed to grant user role to user2");
        pm.grant_role("user3", 2, RoleType::Guest).expect("Failed to grant guest role to user3");
        
        let space1_users = pm.list_space_users(1);
        assert_eq!(space1_users.len(), 2);
        assert!(space1_users.contains(&("user1".to_string(), RoleType::Admin)));
        assert!(space1_users.contains(&("user2".to_string(), RoleType::User)));
        
        let space2_users = pm.list_space_users(2);
        assert_eq!(space2_users.len(), 1);
        assert!(space2_users.contains(&("user3".to_string(), RoleType::Guest)));
    }

    #[test]
    fn test_custom_permission() {
        let pm = PermissionManager::new();
        let result = pm.grant_permission(1, "testuser", Permission::Delete);
        assert!(result.is_ok());

        assert!(pm
            .check_custom_permission(1, "testuser", Permission::Delete)
            .is_ok());
        assert!(pm
            .check_custom_permission(1, "testuser", Permission::Write)
            .is_err());
    }
}
