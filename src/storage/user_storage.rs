use crate::core::types::{PasswordInfo, UserAlterInfo, UserInfo};
use crate::core::{RoleType, StorageError};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// 用户存储管理器
/// 
/// 负责管理用户账户的创建、修改、删除以及角色授权
#[derive(Clone)]
pub struct UserStorage {
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
}

impl std::fmt::Debug for UserStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserStorage")
            .field("user_count", &self.users.lock().len())
            .finish()
    }
}

impl Default for UserStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStorage {
    /// 创建新的用户存储实例
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 修改用户密码
    pub fn change_password(&self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock();
        let username = info
            .username
            .clone()
            .ok_or_else(|| StorageError::DbError("用户名不能为空".to_string()))?;
        if let Some(user) = users.get_mut(&username) {
            user.change_password(info.new_password.clone())?;
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("用户 {} 不存在", username)))
        }
    }

    /// 创建新用户
    pub fn create_user(&self, info: &UserInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock();
        if users.contains_key(&info.username) {
            return Err(StorageError::DbError(format!(
                "用户 {} 已存在",
                info.username
            )));
        }
        users.insert(info.username.clone(), info.clone());
        Ok(true)
    }

    /// 修改用户信息
    pub fn alter_user(&self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock();
        if let Some(user) = users.get_mut(&info.username) {
            // 修改锁定状态
            if let Some(is_locked) = info.is_locked {
                user.is_locked = is_locked;
            }
            // 修改资源限制
            if let Some(limit) = info.max_queries_per_hour {
                user.max_queries_per_hour = limit;
            }
            if let Some(limit) = info.max_updates_per_hour {
                user.max_updates_per_hour = limit;
            }
            if let Some(limit) = info.max_connections_per_hour {
                user.max_connections_per_hour = limit;
            }
            if let Some(limit) = info.max_user_connections {
                user.max_user_connections = limit;
            }
            Ok(true)
        } else {
            Err(StorageError::DbError(format!(
                "用户 {} 不存在",
                info.username
            )))
        }
    }

    /// 删除用户
    pub fn drop_user(&self, username: &str) -> Result<bool, StorageError> {
        let mut users = self.users.lock();
        users.remove(username);
        Ok(true)
    }

    /// 获取用户信息
    pub fn get_user(&self, username: &str) -> Option<UserInfo> {
        self.users.lock().get(username).cloned()
    }

    /// 检查用户是否存在
    pub fn user_exists(&self, username: &str) -> bool {
        self.users.lock().contains_key(username)
    }

    /// 授予角色（仅做用户存在性检查，实际授权由 PermissionManager 处理）
    pub fn grant_role(
        &self,
        username: &str,
        _space_id: u64,
        _role: RoleType,
    ) -> Result<bool, StorageError> {
        let users = self.users.lock();
        if users.contains_key(username) {
            Ok(true)
        } else {
            Err(StorageError::DbError(format!(
                "User {} not found",
                username
            )))
        }
    }

    /// 撤销角色（仅做用户存在性检查，实际撤销由 PermissionManager 处理）
    pub fn revoke_role(&self, username: &str, _space_id: u64) -> Result<bool, StorageError> {
        let users = self.users.lock();
        if users.contains_key(username) {
            Ok(true)
        } else {
            Err(StorageError::DbError(format!(
                "User {} not found",
                username
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::RoleType;

    #[test]
    fn test_create_user() {
        let storage = UserStorage::new();
        let user = UserInfo {
            username: "test_user".to_string(),
            password_hash: "hash".to_string(),
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: 0,
            last_login_at: None,
            password_changed_at: 0,
        };

        assert!(storage.create_user(&user).unwrap());
        assert!(storage.user_exists("test_user"));
    }

    #[test]
    fn test_create_duplicate_user() {
        let storage = UserStorage::new();
        let user = UserInfo {
            username: "test_user".to_string(),
            password_hash: "hash".to_string(),
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: 0,
            last_login_at: None,
            password_changed_at: 0,
        };

        storage.create_user(&user).unwrap();
        let result = storage.create_user(&user);
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_user() {
        let storage = UserStorage::new();
        let user = UserInfo {
            username: "test_user".to_string(),
            password_hash: "hash".to_string(),
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: 0,
            last_login_at: None,
            password_changed_at: 0,
        };

        storage.create_user(&user).unwrap();
        assert!(storage.drop_user("test_user").unwrap());
        assert!(!storage.user_exists("test_user"));
    }

    #[test]
    fn test_alter_user() {
        let storage = UserStorage::new();
        let user = UserInfo {
            username: "test_user".to_string(),
            password_hash: "hash".to_string(),
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: 0,
            last_login_at: None,
            password_changed_at: 0,
        };

        storage.create_user(&user).unwrap();

        let alter_info = UserAlterInfo {
            username: "test_user".to_string(),
            is_locked: Some(true),
            max_queries_per_hour: Some(100),
            max_updates_per_hour: None,
            max_connections_per_hour: None,
            max_user_connections: None,
        };

        assert!(storage.alter_user(&alter_info).unwrap());

        let updated_user = storage.get_user("test_user").unwrap();
        assert!(updated_user.is_locked);
        assert_eq!(updated_user.max_queries_per_hour, 100);
    }

    #[test]
    fn test_grant_role_user_not_found() {
        let storage = UserStorage::new();
        let result = storage.grant_role("nonexistent", 1, RoleType::Admin);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_role_user_not_found() {
        let storage = UserStorage::new();
        let result = storage.revoke_role("nonexistent", 1);
        assert!(result.is_err());
    }
}
