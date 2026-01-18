//! 会话上下文定义
//!
//! 提供会话级别的上下文管理

use super::base::ContextType;
use super::traits::BaseContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// 会话上下文
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// 会话ID
    pub session_id: String,
    /// 用户信息
    pub user_info: UserInfo,
    /// 会话状态
    pub session_state: SessionState,
    /// 会话变量
    pub session_variables: HashMap<String, SessionVariable>,
    /// 配置设置
    pub config: SessionConfig,
    /// 创建时间
    pub created_at: SystemTime,
    /// 最后活动时间
    pub last_activity: SystemTime,
    /// 活跃查询
    pub active_queries: Vec<String>,
}

/// 用户信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户名
    pub username: String,
    /// 用户ID
    pub user_id: String,
    /// 用户角色
    pub roles: Vec<String>,
    /// 权限
    pub permissions: Vec<String>,
    /// 认证令牌
    pub auth_token: Option<String>,
    /// 认证时间
    pub auth_time: Option<SystemTime>,
}

/// 会话状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    /// 活跃
    Active,
    /// 空闲
    Idle,
    /// 已过期
    Expired,
    /// 已终止
    Terminated,
}

/// 会话变量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionVariable {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<SessionVariable>),
    Map(HashMap<String, SessionVariable>),
    Null,
}

/// 会话配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 会话超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大空闲时间（秒）
    pub max_idle_seconds: u64,
    /// 最大并发查询数
    pub max_concurrent_queries: usize,
    /// 是否启用查询缓存
    pub enable_query_cache: bool,
    /// 内存限制（字节）
    pub memory_limit_bytes: Option<usize>,
    /// 是否启用自动提交
    pub auto_commit: bool,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
}

/// 隔离级别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// 读未提交
    ReadUncommitted,
    /// 读已提交
    ReadCommitted,
    /// 可重复读
    RepeatableRead,
    /// 串行化
    Serializable,
}

impl SessionContext {
    /// 创建新的会话上下文
    pub fn new(session_id: impl Into<String>, user_info: UserInfo, config: SessionConfig) -> Self {
        let now = SystemTime::now();
        Self {
            session_id: session_id.into(),
            user_info,
            session_state: SessionState::Active,
            session_variables: HashMap::new(),
            config,
            created_at: now,
            last_activity: now,
            active_queries: Vec::new(),
        }
    }

    /// 更新最后活动时间
    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now();
        if self.session_state == SessionState::Idle {
            self.session_state = SessionState::Active;
        }
    }

    /// 设置会话状态
    pub fn set_state(&mut self, state: SessionState) {
        self.session_state = state;
    }

    /// 添加会话变量
    pub fn set_variable(&mut self, name: impl Into<String>, value: SessionVariable) {
        self.session_variables.insert(name.into(), value);
    }

    /// 获取会话变量
    pub fn get_variable(&self, name: &str) -> Option<&SessionVariable> {
        self.session_variables.get(name)
    }

    /// 删除会话变量
    pub fn remove_variable(&mut self, name: &str) -> Option<SessionVariable> {
        self.session_variables.remove(name)
    }

    /// 添加活跃查询
    pub fn add_active_query(&mut self, query_id: impl Into<String>) {
        self.active_queries.push(query_id.into());
    }

    /// 移除活跃查询
    pub fn remove_active_query(&mut self, query_id: &str) {
        self.active_queries.retain(|id| id != query_id);
    }

    /// 获取活跃查询数量
    pub fn active_query_count(&self) -> usize {
        self.active_queries.len()
    }

    /// 检查是否可以添加新查询
    pub fn can_add_query(&self) -> bool {
        self.active_query_count() < self.config.max_concurrent_queries
    }

    /// 检查会话是否过期
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.last_activity.duration_since(self.created_at) {
            elapsed > Duration::from_secs(self.config.timeout_seconds)
        } else {
            true
        }
    }

    /// 检查会话是否空闲
    pub fn is_idle(&self) -> bool {
        if let Ok(idle_time) = SystemTime::now().duration_since(self.last_activity) {
            idle_time > Duration::from_secs(60) && self.active_queries.is_empty()
        } else {
            false
        }
    }

    /// 检查用户是否有指定权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.user_info.permissions.contains(&permission.to_string())
    }

    /// 检查用户是否有指定角色
    pub fn has_role(&self, role: &str) -> bool {
        self.user_info.roles.contains(&role.to_string())
    }

    /// 检查是否是管理员
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("administrator")
    }

    /// 获取会话持续时间
    pub fn session_duration(&self) -> Option<Duration> {
        self.created_at.elapsed().ok()
    }

    /// 获取空闲时间
    pub fn idle_duration(&self) -> Option<Duration> {
        self.last_activity.elapsed().ok()
    }
}

impl BaseContext for SessionContext {
    fn id(&self) -> &str {
        &self.session_id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Session
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.last_activity
    }

    fn is_valid(&self) -> bool {
        !self.is_expired() && self.session_state != SessionState::Terminated
    }

    fn touch(&mut self) {
        self.update_activity();
    }

    fn invalidate(&mut self) {
        self.session_state = SessionState::Terminated;
    }

    fn revalidate(&mut self) -> bool {
        if self.is_expired() {
            self.session_state = SessionState::Expired;
            false
        } else {
            self.session_state = SessionState::Active;
            true
        }
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn depth(&self) -> usize {
        1
    }
}

impl UserInfo {
    /// 创建新的用户信息
    pub fn new(
        username: impl Into<String>,
        user_id: impl Into<String>,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            username: username.into(),
            user_id: user_id.into(),
            roles,
            permissions,
            auth_token: None,
            auth_time: None,
        }
    }

    /// 设置认证信息
    pub fn set_auth(&mut self, token: impl Into<String>) {
        self.auth_token = Some(token.into());
        self.auth_time = Some(SystemTime::now());
    }

    /// 检查认证是否有效
    pub fn is_auth_valid(&self, max_age_seconds: u64) -> bool {
        if let (Some(_token), Some(auth_time)) = (&self.auth_token, &self.auth_time) {
            if let Ok(elapsed) = auth_time.elapsed() {
                elapsed < Duration::from_secs(max_age_seconds)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 添加角色
    pub fn add_role(&mut self, role: impl Into<String>) {
        let role = role.into();
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }

    /// 移除角色
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
    }

    /// 添加权限
    pub fn add_permission(&mut self, permission: impl Into<String>) {
        let permission = permission.into();
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// 移除权限
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 3600,  // 1小时
            max_idle_seconds: 1800, // 30分钟
            max_concurrent_queries: 10,
            enable_query_cache: true,
            memory_limit_bytes: Some(1024 * 1024 * 1024), // 1GB
            auto_commit: true,
            isolation_level: IsolationLevel::ReadCommitted,
        }
    }
}

// 重新导出类型以供其他模块使用

/// 会话信息（统一版本，包含完整的会话生命周期管理）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// 会话ID
    pub session_id: String,
    /// 用户名
    pub username: String,
    /// 用户角色
    pub roles: Vec<String>,
    /// 客户端IP
    pub client_ip: String,
    /// 客户端端口
    pub client_port: u16,
    /// 客户端信息
    pub client_info: String,
    /// 连接信息
    pub connection_info: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 最后访问时间
    pub last_accessed: std::time::SystemTime,
    /// 会话状态
    pub status: SessionStatus,
}

/// 会话状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionStatus {
    /// 活跃
    Active,
    /// 空闲
    Idle,
    /// 已过期
    Expired,
    /// 已关闭
    Closed,
}

impl SessionInfo {
    /// 创建新的会话信息（完整版本）
    pub fn new(
        session_id: impl Into<String>,
        username: impl Into<String>,
        roles: Vec<String>,
        client_ip: impl Into<String>,
        client_port: u16,
        client_info: impl Into<String>,
        connection_info: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            session_id: session_id.into(),
            username: username.into(),
            roles,
            client_ip: client_ip.into(),
            client_port,
            client_info: client_info.into(),
            connection_info: connection_info.into(),
            created_at: now,
            last_accessed: now,
            status: SessionStatus::Active,
        }
    }

    /// 更新最后访问时间
    pub fn touch(&mut self) {
        self.last_accessed = std::time::SystemTime::now();
    }

    /// 检查会话是否有效
    pub fn is_valid(&self, timeout: std::time::Duration) -> bool {
        if let Ok(elapsed) = self.last_accessed.elapsed() {
            elapsed < timeout && matches!(self.status, SessionStatus::Active | SessionStatus::Idle)
        } else {
            false
        }
    }

    /// 关闭会话
    pub fn close(&mut self) {
        self.status = SessionStatus::Closed;
        self.touch();
    }
}
