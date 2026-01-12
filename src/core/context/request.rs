//! 请求上下文模块
//!
//! 管理查询请求的上下文信息，整合自query/context/request_context.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::base::ContextType;
use super::traits::BaseContext;
use crate::core::Value;

// SessionInfo 现在统一使用 src/core/context/session.rs 中的定义
use super::session::{SessionInfo, SessionStatus};

/// 请求参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestParams {
    pub query: String,
    pub parameters: HashMap<String, Value>,
    pub timeout_ms: u64,
    pub max_retry_times: u32,
}

impl RequestParams {
    pub fn new(query: String) -> Self {
        Self {
            query,
            parameters: HashMap::new(),
            timeout_ms: 30000, // 默认30秒
            max_retry_times: 3,
        }
    }

    pub fn with_parameters(mut self, params: HashMap<String, Value>) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_retry(mut self, max_retry: u32) -> Self {
        self.max_retry_times = max_retry;
        self
    }
}

/// 响应对象
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub affected_rows: u64,
}

impl Response {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            data: None,
            error_message: None,
            execution_time_ms: 0,
            affected_rows: 0,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.success = false;
        self
    }

    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }

    pub fn with_affected_rows(mut self, rows: u64) -> Self {
        self.affected_rows = rows;
        self
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn get_data(&self) -> Option<&Value> {
        self.data.as_ref()
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error_message.as_ref()
    }
}

/// 请求状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// 请求上下文
///
/// 管理查询请求的完整生命周期，包括：
/// 1. 请求参数管理
/// 2. 会话信息管理
/// 3. 响应对象管理
/// 4. 请求生命周期管理
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 上下文ID
    pub id: String,

    // 会话信息
    pub session_info: Option<SessionInfo>,

    // 请求参数
    pub request_params: RequestParams,

    // 响应对象
    pub response: Arc<RwLock<Response>>,

    // 请求开始时间
    pub start_time: std::time::SystemTime,

    // 请求状态
    pub status: Arc<RwLock<RequestStatus>>,

    // 自定义属性
    pub attributes: Arc<RwLock<HashMap<String, Value>>>,

    // 最后更新时间
    pub updated_at: std::time::SystemTime,

    // 是否有效
    pub valid: bool,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(id: String, session_info: SessionInfo, request_params: RequestParams) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            session_info: Some(session_info),
            request_params,
            response: Arc::new(RwLock::new(Response::new(true))),
            start_time: now,
            status: Arc::new(RwLock::new(RequestStatus::Pending)),
            attributes: Arc::new(RwLock::new(HashMap::new())),
            updated_at: now,
            valid: true,
        }
    }

    /// 创建带会话信息的请求上下文
    pub fn with_session(
        id: String,
        query: String,
        session_id: &str,
        user_name: &str,
        client_ip: &str,
        client_port: u16,
    ) -> Self {
        let session_info = SessionInfo::new(
            session_id.to_string(),
            user_name.to_string(),
            vec![], // 默认无角色
            client_ip.to_string(),
            client_port,
            "", // 默认客户端信息
            "", // 默认连接信息
        );
        let request_params = RequestParams::new(query);
        Self::new(id, session_info, request_params)
    }

    /// 创建带参数的请求上下文
    pub fn with_parameters(
        id: String,
        query: String,
        parameters: HashMap<String, Value>,
        session_id: &str,
        user_name: &str,
        client_ip: &str,
        client_port: u16,
    ) -> Self {
        let session_info = SessionInfo::new(
            session_id.to_string(),
            user_name.to_string(),
            vec![], // 默认无角色
            client_ip.to_string(),
            client_port,
            "", // 默认客户端信息
            "", // 默认连接信息
        );
        let request_params = RequestParams::new(query).with_parameters(parameters);
        Self::new(id, session_info, request_params)
    }

    /// 创建带超时设置的请求上下文
    pub fn with_timeout(
        id: String,
        query: String,
        timeout_ms: u64,
        session_id: &str,
        user_name: &str,
        client_ip: &str,
        client_port: u16,
    ) -> Self {
        let session_info = SessionInfo::new(
            session_id.to_string(),
            user_name.to_string(),
            vec![], // 默认无角色
            client_ip.to_string(),
            client_port,
            "", // 默认客户端信息
            "", // 默认连接信息
        );
        let request_params = RequestParams::new(query).with_timeout(timeout_ms);
        Self::new(id, session_info, request_params)
    }

    /// 获取会话信息
    pub fn session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    /// 获取会话ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_info.as_ref().map(|s| s.session_id.as_str())
    }

    /// 获取用户名
    pub fn user_name(&self) -> Option<&str> {
        self.session_info.as_ref().map(|s| s.username.as_str())
    }

    /// 获取客户端IP
    pub fn client_ip(&self) -> Option<&str> {
        self.session_info.as_ref().map(|s| s.client_ip.as_str())
    }

    /// 获取查询字符串
    pub fn query(&self) -> &str {
        &self.request_params.query
    }

    /// 获取请求参数
    pub fn request_params(&self) -> &RequestParams {
        &self.request_params
    }

    /// 获取参数值
    pub fn get_parameter(&self, name: &str) -> Option<Value> {
        self.request_params.parameters.get(name).cloned()
    }

    /// 设置参数值
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.request_params.parameters.insert(name, value);
        self.touch();
    }

    /// 获取超时时间（毫秒）
    pub fn timeout_ms(&self) -> u64 {
        self.request_params.timeout_ms
    }

    /// 获取最大重试次数
    pub fn max_retry_times(&self) -> u32 {
        self.request_params.max_retry_times
    }

    /// 设置响应数据
    pub fn set_response_data(&self, data: Value) -> Result<(), String> {
        let mut response = self
            .response
            .write()
            .map_err(|e| format!("Failed to acquire write lock on response: {}", e))?;

        response.data = Some(data);
        response.success = true;
        Ok(())
    }

    /// 设置响应错误
    pub fn set_response_error(&self, error: String) -> Result<(), String> {
        let mut response = self
            .response
            .write()
            .map_err(|e| format!("Failed to acquire write lock on response: {}", e))?;

        response.error_message = Some(error);
        response.success = false;
        Ok(())
    }

    /// 获取响应
    pub fn get_response(&self) -> Result<Response, String> {
        let response = self
            .response
            .read()
            .map_err(|e| format!("Failed to acquire read lock on response: {}", e))?;

        Ok(response.clone())
    }

    /// 获取响应数据
    pub fn get_response_data(&self) -> Result<Option<Value>, String> {
        let response = self
            .response
            .read()
            .map_err(|e| format!("Failed to acquire read lock on response: {}", e))?;

        Ok(response.data.clone())
    }

    /// 获取响应错误
    pub fn get_response_error(&self) -> Result<Option<String>, String> {
        let response = self
            .response
            .read()
            .map_err(|e| format!("Failed to acquire read lock on response: {}", e))?;

        Ok(response.error_message.clone())
    }

    /// 设置执行时间
    pub fn set_execution_time(&self, time_ms: u64) -> Result<(), String> {
        let mut response = self
            .response
            .write()
            .map_err(|e| format!("Failed to acquire write lock on response: {}", e))?;

        response.execution_time_ms = time_ms;
        Ok(())
    }

    /// 设置影响行数
    pub fn set_affected_rows(&self, rows: u64) -> Result<(), String> {
        let mut response = self
            .response
            .write()
            .map_err(|e| format!("Failed to acquire write lock on response: {}", e))?;

        response.affected_rows = rows;
        Ok(())
    }

    /// 获取请求持续时间
    pub fn duration(&self) -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
    }

    /// 获取请求状态
    pub fn status(&self) -> Result<RequestStatus, String> {
        let status = self
            .status
            .read()
            .map_err(|e| format!("Failed to acquire read lock on status: {}", e))?;

        Ok(status.clone())
    }

    /// 设置请求状态
    pub fn set_status(&self, status: RequestStatus) -> Result<(), String> {
        let mut current_status = self
            .status
            .write()
            .map_err(|e| format!("Failed to acquire write lock on status: {}", e))?;

        *current_status = status;
        Ok(())
    }

    /// 标记请求为处理中
    pub fn mark_processing(&self) -> Result<(), String> {
        self.set_status(RequestStatus::Processing)
    }

    /// 标记请求为完成
    pub fn mark_completed(&self) -> Result<(), String> {
        self.set_status(RequestStatus::Completed)
    }

    /// 标记请求为失败
    pub fn mark_failed(&self) -> Result<(), String> {
        self.set_status(RequestStatus::Failed)
    }

    /// 标记请求为取消
    pub fn mark_cancelled(&self) -> Result<(), String> {
        self.set_status(RequestStatus::Cancelled)
    }

    /// 检查请求是否完成
    pub fn is_completed(&self) -> Result<bool, String> {
        let status = self.status()?;
        Ok(matches!(
            status,
            RequestStatus::Completed | RequestStatus::Failed | RequestStatus::Cancelled
        ))
    }

    /// 检查请求是否失败
    pub fn is_failed(&self) -> Result<bool, String> {
        let status = self.status()?;
        Ok(matches!(status, RequestStatus::Failed))
    }

    /// 检查请求是否取消
    pub fn is_cancelled(&self) -> Result<bool, String> {
        let status = self.status()?;
        Ok(matches!(status, RequestStatus::Cancelled))
    }
}

impl BaseContext for RequestContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Request
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.start_time
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now();
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.updated_at = std::time::SystemTime::now();
    }

    fn revalidate(&mut self) -> bool {
        self.valid = true;
        self.updated_at = std::time::SystemTime::now();
        true
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn depth(&self) -> usize {
        1
    }

    fn get_attribute(&self, key: &str) -> Option<Value> {
        if let Ok(attributes) = self.attributes.read() {
            attributes.get(key).cloned()
        } else {
            None
        }
    }

    fn set_attribute(&mut self, key: String, value: Value) {
        let attributes_ref = &self.attributes;
        if let Ok(mut attributes) = attributes_ref.write() {
            attributes.insert(key, value);
        }
        self.updated_at = std::time::SystemTime::now();
    }

    fn attribute_keys(&self) -> Vec<String> {
        if let Ok(attributes) = self.attributes.read() {
            attributes.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn remove_attribute(&mut self, key: &str) -> Option<Value> {
        let removed = if let Ok(mut attributes) = self.attributes.write() {
            attributes.remove(key)
        } else {
            None
        };
        self.updated_at = std::time::SystemTime::now();
        removed
    }

    fn clear_attributes(&mut self) {
        if let Ok(mut attributes) = self.attributes.write() {
            attributes.clear();
        }
        self.updated_at = std::time::SystemTime::now();
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        let session_info = SessionInfo::new(
            "default_session".to_string(),
            "default_user".to_string(),
            vec![],
            "localhost".to_string(),
            0,
            "default_client".to_string(),
            "default_connection".to_string(),
        );
        let request_params = RequestParams::new("SELECT 1".to_string());
        Self::new("default_request".to_string(), session_info, request_params)
    }
}
