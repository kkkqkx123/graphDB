//! 会话上下文定义
//!
//! 提供会话级别的上下文管理

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use crate::core::Value;
use super::base::{ContextBase, ContextType, MutableContext};

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
    pub fn new(
        session_id: impl Into<String>,
        user_info: UserInfo,
        config: SessionConfig,
    ) -> Self {
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

impl ContextBase for SessionContext {
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
}

impl MutableContext for SessionContext {
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
            timeout_seconds: 3600,      // 1小时
            max_idle_seconds: 1800,     // 30分钟
            max_concurrent_queries: 10,
            enable_query_cache: true,
            memory_limit_bytes: Some(1024 * 1024 * 1024), // 1GB
            auto_commit: true,
            isolation_level: IsolationLevel::ReadCommitted,
        }
    }
}

// 重新导出类型以供其他模块使用

/// 会话信息（简化版本，用于兼容性）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// 会话ID
    pub session_id: String,
    /// 用户名
    pub username: String,
    /// 用户角色
    pub roles: Vec<String>,
}

impl SessionInfo {
    /// 创建新的会话信息
    pub fn new(
        session_id: impl Into<String>,
        username: impl Into<String>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            username: username.into(),
            roles,
        }
    }
}

/// 会话统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// 会话ID
    pub session_id: String,
    /// 用户名
    pub username: String,
    /// 创建时间
    pub created_at: SystemTime,
    /// 最后活动时间
    pub last_activity: SystemTime,
    /// 总查询数
    pub total_queries: usize,
    /// 成功查询数
    pub successful_queries: usize,
    /// 失败查询数
    pub failed_queries: usize,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    /// 内存使用峰值（字节）
    pub peak_memory_usage_bytes: usize,
    /// 网络IO总量（字节）
    pub total_network_io_bytes: usize,
}

impl SessionStatistics {
    /// 创建新的会话统计信息
    pub fn new(session_id: impl Into<String>, username: impl Into<String>) -> Self {
        let now = SystemTime::now();
        Self {
            session_id: session_id.into(),
            username: username.into(),
            created_at: now,
            last_activity: now,
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            peak_memory_usage_bytes: 0,
            total_network_io_bytes: 0,
        }
    }
    
    /// 记录查询完成
    pub fn record_query_completion(&mut self, execution_time_ms: u64, success: bool) {
        self.total_queries += 1;
        self.total_execution_time_ms += execution_time_ms;
        
        if success {
            self.successful_queries += 1;
        } else {
            self.failed_queries += 1;
        }
        
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.total_queries as f64;
    }
    
    /// 更新内存使用峰值
    pub fn update_memory_peak(&mut self, memory_bytes: usize) {
        if memory_bytes > self.peak_memory_usage_bytes {
            self.peak_memory_usage_bytes = memory_bytes;
        }
    }
    
    /// 增加网络IO量
    pub fn add_network_io(&mut self, bytes: usize) {
        self.total_network_io_bytes += bytes;
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64
        }
    }
    
    /// 获取失败率
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}