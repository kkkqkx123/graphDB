//! API会话层用户管理类型

use crate::core::StorageError;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
    pub username: String,
    pub password_hash: String,
    pub is_locked: bool,
    pub max_queries_per_hour: i32,
    pub max_updates_per_hour: i32,
    pub max_connections_per_hour: i32,
    pub max_user_connections: i32,
    pub created_at: i64,
    pub last_login_at: Option<i64>,
    pub password_changed_at: i64,
}

impl UserInfo {
    pub fn new(username: String, password: String) -> Result<Self, StorageError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| StorageError::DbError(format!("密码加密失败: {}", e)))?;

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

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct PasswordInfo {
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserAlterInfo {
    pub username: String,
    pub is_locked: Option<bool>,
    pub max_queries_per_hour: Option<i32>,
    pub max_updates_per_hour: Option<i32>,
    pub max_connections_per_hour: Option<i32>,
    pub max_user_connections: Option<i32>,
}
