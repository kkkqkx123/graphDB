use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// 从 core 层重新导出权限类型
pub use crate::core::{Permission, RoleType};

/// God角色的Space ID标记（全局角色，不绑定特定Space）
pub const GOD_SPACE_ID: i64 = -1;

/// 权限管理器 - 数据层
/// 
/// 职责：
/// 1. 管理用户角色映射（username -> {space_id -> role}）
/// 2. 管理空间权限映射（space_id -> {username -> [permissions]}）
/// 3. 提供基础的角色查询和权限检查
/// 
/// 注意：本层不涉及业务逻辑判断（如God角色优先等），只提供基础数据操作
pub struct PermissionManager {
    /// 用户角色映射：username -> {space_id -> role}
    /// 注意：God角色使用特殊的space_id: -1表示全局角色，不绑定特定Space
    user_roles: Arc<RwLock<HashMap<String, HashMap<i64, RoleType>>>>,
    /// 空间权限映射：space_id -> {username -> [permissions]}
    /// 用于细粒度的权限控制
    space_permissions: Arc<RwLock<HashMap<i64, HashMap<String, Vec<Permission>>>>>,
}

impl PermissionManager {
    /// 创建新的权限管理器
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

    // ==================== 角色管理（基础CRUD） ====================

    /// 授予角色
    pub fn grant_role(&self, username: &str, space_id: i64, role: RoleType) -> Result<()> {
        let mut user_roles = self.user_roles.write();

        let roles = user_roles.entry(username.to_string()).or_insert_with(HashMap::new);
        roles.insert(space_id, role);
        Ok(())
    }

    /// 撤销角色
    pub fn revoke_role(&self, username: &str, space_id: i64) -> Result<()> {
        let mut user_roles = self.user_roles.write();

        if let Some(roles) = user_roles.get_mut(username) {
            roles.remove(&space_id);
        }
        Ok(())
    }

    /// 获取用户在指定空间的角色
    pub fn get_role(&self, username: &str, space_id: i64) -> Option<RoleType> {
        let user_roles = self.user_roles.read();
        user_roles.get(username)
            .and_then(|roles| roles.get(&space_id).copied())
    }

    /// 获取用户的所有角色
    pub fn get_user_roles(&self, username: &str) -> HashMap<i64, RoleType> {
        let user_roles = self.user_roles.read();
        user_roles.get(username).cloned().unwrap_or_default()
    }

    /// 列出用户的所有角色
    /// 返回 Vec<(space_id, role)>，包含用户在所有Space的角色
    pub fn list_user_roles(&self, username: &str) -> Vec<(i64, RoleType)> {
        let user_roles = self.user_roles.read();
        user_roles
            .get(username)
            .map(|roles| {
                roles
                    .iter()
                    .map(|(&space_id, &role)| (space_id, role))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 列出Space中的所有用户及其角色
    /// 返回 Vec<(username, role)>，包含指定Space中的所有用户
    pub fn list_space_users(&self, space_id: i64) -> Vec<(String, RoleType)> {
        let user_roles = self.user_roles.read();
        user_roles
            .iter()
            .filter_map(|(username, roles)| {
                roles.get(&space_id).map(|&role| (username.clone(), role))
            })
            .collect()
    }

    // ==================== 角色查询（基础查询） ====================

    /// 检查用户是否是God角色
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

    /// 检查用户在指定空间是否有指定角色
    pub fn has_role(&self, username: &str, space_id: i64, role: RoleType) -> bool {
        self.get_role(username, space_id)
            .map(|r| r == role)
            .unwrap_or(false)
    }

    // ==================== 权限检查（基础检查） ====================

    /// 基础权限检查
    /// 检查用户在指定空间是否有指定权限
    pub fn check_permission(&self, username: &str, space_id: i64, permission: Permission) -> Result<()> {
        let role = self.get_role(username, space_id)
            .or_else(|| self.get_role(username, GOD_SPACE_ID))
            .ok_or_else(|| anyhow!("User {} has no role in space {}", username, space_id))?;

        if role.has_permission(permission) {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: {:?} for user {}", permission, username))
        }
    }

    /// 检查用户是否可以授予角色
    pub fn can_grant_role(&self, granter: &str, space_id: i64, target_role: RoleType) -> bool {
        let user_roles = self.user_roles.read();
        if let Some(roles) = user_roles.get(granter) {
            if let Some(&role) = roles.get(&space_id) {
                return role.can_grant(target_role);
            }
        }
        false
    }

    /// 检查用户是否可以撤销角色
    pub fn can_revoke_role(&self, revoker: &str, space_id: i64, target_role: RoleType) -> bool {
        self.can_grant_role(revoker, space_id, target_role)
    }

    // ==================== 空间权限管理（细粒度权限） ====================

    /// 为用户在空间添加特定权限
    pub fn grant_permission(&self, username: &str, space_id: i64, permission: Permission) -> Result<()> {
        let mut space_permissions = self.space_permissions.write();
        let space_map = space_permissions.entry(space_id).or_insert_with(HashMap::new);
        let user_permissions = space_map.entry(username.to_string()).or_insert_with(Vec::new);
        if !user_permissions.contains(&permission) {
            user_permissions.push(permission);
        }
        Ok(())
    }

    /// 撤销用户在空间的特定权限
    pub fn revoke_permission(&self, username: &str, space_id: i64, permission: Permission) -> Result<()> {
        let mut space_permissions = self.space_permissions.write();
        if let Some(space_map) = space_permissions.get_mut(&space_id) {
            if let Some(user_permissions) = space_map.get_mut(username) {
                user_permissions.retain(|&p| p != permission);
            }
        }
        Ok(())
    }

    /// 获取用户在空间的特定权限列表
    pub fn get_permissions(&self, username: &str, space_id: i64) -> Vec<Permission> {
        let space_permissions = self.space_permissions.read();
        space_permissions
            .get(&space_id)
            .and_then(|space_map| space_map.get(username).cloned())
            .unwrap_or_default()
    }

    /// 检查用户在空间是否有特定权限（细粒度检查）
    pub fn has_permission(&self, username: &str, space_id: i64, permission: Permission) -> bool {
        self.get_permissions(username, space_id).contains(&permission)
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
    fn test_grant_and_get_role() {
        let pm = PermissionManager::new();
        
        pm.grant_role("user1", 1, RoleType::Admin).unwrap();
        
        assert_eq!(pm.get_role("user1", 1), Some(RoleType::Admin));
        assert_eq!(pm.get_role("user1", 2), None);
        assert_eq!(pm.get_role("nonexistent", 1), None);
    }

    #[test]
    fn test_is_god() {
        let pm = PermissionManager::new();
        
        // root 默认是 God
        assert!(pm.is_god("root"));
        
        let _ = pm.grant_role("user1", 1, RoleType::Admin);
        assert!(!pm.is_god("user1"));

        let _ = pm.grant_role("user2", GOD_SPACE_ID, RoleType::God);
        assert!(pm.is_god("user2"));
    }

    #[test]
    fn test_check_permission() {
        let pm = PermissionManager::new();
        
        pm.grant_role("user1", 1, RoleType::User).unwrap();
        pm.grant_role("guest1", 1, RoleType::Guest).unwrap();
        
        // User 角色有 Read 和 Write 权限
        assert!(pm.check_permission("user1", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("user1", 1, Permission::Write).is_ok());
        // Guest 角色只有 Read 权限
        assert!(pm.check_permission("guest1", 1, Permission::Read).is_ok());
        assert!(pm.check_permission("guest1", 1, Permission::Write).is_err());
        // 未授权用户
        assert!(pm.check_permission("nonexistent", 1, Permission::Read).is_err());
    }

    #[test]
    fn test_can_grant_role() {
        let pm = PermissionManager::new();
        
        pm.grant_role("admin", 1, RoleType::Admin).unwrap();
        pm.grant_role("user", 1, RoleType::User).unwrap();
        
        // Admin 可以授予 User 和 Guest 角色
        assert!(pm.can_grant_role("admin", 1, RoleType::User));
        assert!(pm.can_grant_role("admin", 1, RoleType::Guest));
        // Admin 不能授予 Admin 或 God 角色
        assert!(!pm.can_grant_role("admin", 1, RoleType::Admin));
        assert!(!pm.can_grant_role("admin", 1, RoleType::God));
        
        // User 不能授予任何角色
        assert!(!pm.can_grant_role("user", 1, RoleType::Guest));
    }

    #[test]
    fn test_god_role_global_permission() {
        let pm = PermissionManager::new();
        
        // God 角色在任何空间都有权限
        assert!(pm.check_permission("root", 999, Permission::Write).is_ok());
        assert!(pm.check_permission("root", 999, Permission::Read).is_ok());
    }

    #[test]
    fn test_list_user_roles() {
        let pm = PermissionManager::new();

        // 给用户在不同Space授予不同角色
        pm.grant_role("multi_user", 1, RoleType::Admin).unwrap();
        pm.grant_role("multi_user", 2, RoleType::User).unwrap();
        pm.grant_role("multi_user", 3, RoleType::Guest).unwrap();

        // 列出用户所有角色
        let roles = pm.list_user_roles("multi_user");
        assert_eq!(roles.len(), 3);

        // 验证包含正确的角色
        let role_map: HashMap<i64, RoleType> = roles.into_iter().collect();
        assert_eq!(role_map.get(&1), Some(&RoleType::Admin));
        assert_eq!(role_map.get(&2), Some(&RoleType::User));
        assert_eq!(role_map.get(&3), Some(&RoleType::Guest));

        // 不存在的用户返回空列表
        let empty_roles = pm.list_user_roles("nonexistent");
        assert!(empty_roles.is_empty());
    }

    #[test]
    fn test_list_space_users() {
        let pm = PermissionManager::new();
        let space_id = 1i64;

        // 给多个用户授予角色
        pm.grant_role("user1", space_id, RoleType::User).unwrap();
        pm.grant_role("user2", space_id, RoleType::Admin).unwrap();
        pm.grant_role("user3", space_id, RoleType::Guest).unwrap();

        // 列出Space中的所有用户
        let users = pm.list_space_users(space_id);
        assert_eq!(users.len(), 3);

        // 验证包含正确的用户和角色
        let user_map: HashMap<String, RoleType> = users.into_iter().collect();
        assert_eq!(user_map.get("user1"), Some(&RoleType::User));
        assert_eq!(user_map.get("user2"), Some(&RoleType::Admin));
        assert_eq!(user_map.get("user3"), Some(&RoleType::Guest));

        // 空的Space返回空列表
        let empty_users = pm.list_space_users(999);
        assert!(empty_users.is_empty());
    }
}
