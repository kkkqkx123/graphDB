//! 用户管理类型定义

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct PasswordInfo {
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

/// 用户信息 - 参考nebula-graph UserItem实现
/// 包含密码哈希和资源限制
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
    pub username: String,
    /// 密码哈希（bcrypt加密）
    pub password_hash: String,
    /// 是否锁定
    pub is_locked: bool,
    /// 每小时最大查询数（0表示无限制）
    pub max_queries_per_hour: i32,
    /// 每小时最大更新数（0表示无限制）
    pub max_updates_per_hour: i32,
    /// 每小时最大连接数（0表示无限制）
    pub max_connections_per_hour: i32,
    /// 最大并发连接数（0表示无限制）
    pub max_user_connections: i32,
    /// 创建时间
    pub created_at: i64,
    /// 最后登录时间
    pub last_login_at: Option<i64>,
    /// 密码最后修改时间
    pub password_changed_at: i64,
}

impl UserInfo {
    /// 创建新用户（使用明文密码，内部自动哈希）
    pub fn new(username: String, password: String) -> Result<Self, crate::core::StorageError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::core::StorageError::DbError(format!("Password encryption failed: {}", e)))?;

        let now = chrono::Utc::now().timestamp_millis();

        Ok(Self {
            username,
            password_hash,
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: now,
            last_login_at: None,
            password_changed_at: now,
        })
    }

    /// 验证密码
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    /// 修改密码
    pub fn change_password(
        &mut self,
        new_password: String,
    ) -> Result<(), crate::core::StorageError> {
        self.password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::core::StorageError::DbError(format!("Password encryption failed: {}", e)))?;
        self.password_changed_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = is_locked;
        self
    }

    pub fn with_max_queries_per_hour(mut self, limit: i32) -> Self {
        self.max_queries_per_hour = limit;
        self
    }

    pub fn with_max_updates_per_hour(mut self, limit: i32) -> Self {
        self.max_updates_per_hour = limit;
        self
    }

    pub fn with_max_connections_per_hour(mut self, limit: i32) -> Self {
        self.max_connections_per_hour = limit;
        self
    }

    pub fn with_max_user_connections(mut self, limit: i32) -> Self {
        self.max_user_connections = limit;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserAlterInfo {
    pub username: String,
    /// 新的锁定状态
    pub is_locked: Option<bool>,
    /// 新的每小时最大查询数
    pub max_queries_per_hour: Option<i32>,
    /// 新的每小时最大更新数
    pub max_updates_per_hour: Option<i32>,
    /// 新的每小时最大连接数
    pub max_connections_per_hour: Option<i32>,
    /// 新的最大并发连接数
    pub max_user_connections: Option<i32>,
}

impl UserAlterInfo {
    pub fn new(username: String) -> Self {
        Self {
            username,
            is_locked: None,
            max_queries_per_hour: None,
            max_updates_per_hour: None,
            max_connections_per_hour: None,
            max_user_connections: None,
        }
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }
}
