//! 请求上下文模块 - 管理查询请求的上下文信息
//! 对应原C++中的RequestContext.h

use crate::core::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};

// SessionInfo 现在统一使用 src/core/context/session.rs 中的定义
use crate::core::context::session::SessionInfo;

/// 请求参数
#[derive(Debug, Clone)]
pub struct RequestParams {
    pub query: String,
    pub parameters: HashMap<String, Value>,
    pub timeout_ms: u64,
    pub max_retry_times: u32,
    pub retry_count: u32,
}

impl RequestParams {
    pub fn new(query: String) -> Self {
        Self {
            query,
            parameters: HashMap::new(),
            timeout_ms: 30000, // 默认30秒
            max_retry_times: 3,
            retry_count: 0,
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

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retry_times
    }
}

/// 响应对象
#[derive(Debug, Clone)]
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

/// 请求上下文
///
/// 管理查询请求的完整生命周期，包括：
/// 1. 请求参数管理
/// 2. 会话信息管理
/// 3. 响应对象管理
/// 4. 请求生命周期管理
/// 5. 请求取消和超时控制
/// 6. 重试逻辑
/// 7. 日志记录
/// 8. 统计信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    // 会话信息
    session_info: Option<SessionInfo>,

    // 请求参数
    request_params: Arc<RwLock<RequestParams>>,

    // 响应对象
    response: Arc<RwLock<Response>>,

    // 请求开始时间
    start_time: std::time::SystemTime,

    // 请求状态
    status: Arc<RwLock<RequestStatus>>,

    // 自定义属性
    attributes: Arc<RwLock<HashMap<String, Value>>>,

    // 请求取消标志
    cancelled: Arc<AtomicBool>,

    // 请求超时标志
    timed_out: Arc<AtomicBool>,

    // 查询执行次数
    execution_count: Arc<AtomicU64>,

    // 日志记录
    logs: Arc<RwLock<Vec<RequestLog>>>,

    // 统计信息
    statistics: Arc<RwLock<RequestStatistics>>,
}

/// 请求日志
#[derive(Debug, Clone)]
pub struct RequestLog {
    pub timestamp: i64,
    pub level: LogLevel,
    pub message: String,
    pub context: Option<String>,
}

/// 日志级别
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 请求统计信息
#[derive(Debug, Clone)]
pub struct RequestStatistics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub cancelled_queries: u64,
    pub timed_out_queries: u64,
    pub total_execution_time_ms: u64,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: u64,
    pub min_execution_time_ms: u64,
}

/// 请求状态
#[derive(Debug, Clone, PartialEq)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(session_info: SessionInfo, request_params: RequestParams) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        Self {
            session_info: Some(session_info),
            request_params: Arc::new(RwLock::new(request_params)),
            response: Arc::new(RwLock::new(Response::new(true))),
            start_time: std::time::SystemTime::now(),
            status: Arc::new(RwLock::new(RequestStatus::Pending)),
            attributes: Arc::new(RwLock::new(HashMap::new())),
            cancelled: Arc::new(AtomicBool::new(false)),
            timed_out: Arc::new(AtomicBool::new(false)),
            execution_count: Arc::new(AtomicU64::new(0)),
            logs: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(RequestStatistics {
                total_queries: 0,
                successful_queries: 0,
                failed_queries: 0,
                cancelled_queries: 0,
                timed_out_queries: 0,
                total_execution_time_ms: 0,
                avg_execution_time_ms: 0.0,
                max_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
            })),
        }
    }

    /// 创建带会话信息的请求上下文
    pub fn with_session(
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
        Self::new(session_info, request_params)
    }

    /// 创建带参数的请求上下文
    pub fn with_parameters(
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
        Self::new(session_info, request_params)
    }

    /// 创建带超时设置的请求上下文
    pub fn with_timeout(
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
        Self::new(session_info, request_params)
    }

    /// 创建带重试设置的请求上下文
    pub fn with_retry(
        query: String,
        max_retry_times: u32,
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
        let request_params = RequestParams::new(query).with_max_retry(max_retry_times);
        Self::new(session_info, request_params)
    }

    /// 基于现有请求上下文创建带参数的请求上下文
    pub fn with_parameters_from_context(&self, parameters: HashMap<String, Value>) -> Self {
        let session_info = self.session_info.clone().unwrap_or_else(|| {
            SessionInfo::new(
                "unknown_session".to_string(),
                "unknown_user".to_string(),
                vec![], // 默认无角色
                "unknown_ip".to_string(),
                0,
                "", // 默认客户端信息
                "", // 默认连接信息
            )
        });
        let request_params =
            RequestParams::new(self.request_params.query.clone()).with_parameters(parameters);
        Self::new(session_info, request_params)
    }

    /// 基于现有请求上下文创建带超时设置的请求上下文
    pub fn with_timeout_from_context(&self, timeout_ms: u64) -> Self {
        let session_info = self.session_info.clone().unwrap_or_else(|| {
            SessionInfo::new(
                "unknown_session".to_string(),
                "unknown_user".to_string(),
                vec![], // 默认无角色
                "unknown_ip".to_string(),
                0,
                "", // 默认客户端信息
                "", // 默认连接信息
            )
        });
        let request_params =
            RequestParams::new(self.request_params.query.clone()).with_timeout(timeout_ms);
        Self::new(session_info, request_params)
    }

    /// 基于现有请求上下文创建带重试设置的请求上下文
    pub fn with_retry_from_context(&self, max_retry_times: u32) -> Self {
        let session_info = self.session_info.clone().unwrap_or_else(|| {
            SessionInfo::new(
                "unknown_session".to_string(),
                "unknown_user".to_string(),
                vec![], // 默认无角色
                "unknown_ip".to_string(),
                0,
                "", // 默认客户端信息
                "", // 默认连接信息
            )
        });
        let request_params =
            RequestParams::new(self.request_params.query.clone()).with_max_retry(max_retry_times);
        Self::new(session_info, request_params)
    }

    // ==================== 会话信息管理 ====================

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

    // ==================== 请求参数管理 ====================

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
        let request_params = self.request_params.read().ok()?;
        request_params.parameters.get(name).cloned()
    }

    /// 设置参数值
    pub fn set_parameter(&self, name: String, value: Value) -> Result<(), String> {
        let mut request_params = self
            .request_params
            .write()
            .map_err(|e| format!("Failed to acquire write lock on request_params: {}", e))?;
        request_params.parameters.insert(name, value);
        Ok(())
    }

    /// 获取超时时间（毫秒）
    pub fn timeout_ms(&self) -> u64 {
        let request_params = self.request_params.read().ok();
        request_params.map(|p| p.timeout_ms).unwrap_or(30000)
    }

    /// 获取最大重试次数
    pub fn max_retry_times(&self) -> u32 {
        let request_params = self.request_params.read().ok();
        request_params.map(|p| p.max_retry_times).unwrap_or(3)
    }

    /// 获取当前重试次数
    pub fn retry_count(&self) -> u32 {
        let request_params = self.request_params.read().ok();
        request_params.map(|p| p.retry_count).unwrap_or(0)
    }

    /// 增加重试计数
    pub fn increment_retry(&self) -> Result<(), String> {
        let mut request_params = self
            .request_params
            .write()
            .map_err(|e| format!("Failed to acquire write lock on request_params: {}", e))?;
        request_params.increment_retry();
        Ok(())
    }

    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        let request_params = self.request_params.read().ok();
        request_params.map(|p| p.can_retry()).unwrap_or(false)
    }

    // ==================== 响应对象管理 ====================

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

    // ==================== 请求生命周期管理 ====================

    /// 获取请求开始时间
    pub fn start_time(&self) -> std::time::SystemTime {
        self.start_time
    }

    /// 获取请求持续时间
    pub fn duration(&self) -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(std::time::Duration::from_secs(0))
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

    // ==================== 自定义属性管理 ====================

    /// 设置自定义属性
    pub fn set_attribute(&self, key: String, value: Value) -> Result<(), String> {
        let mut attributes = self
            .attributes
            .write()
            .map_err(|e| format!("Failed to acquire write lock on attributes: {}", e))?;

        attributes.insert(key, value);
        Ok(())
    }

    /// 获取自定义属性
    pub fn get_attribute(&self, key: &str) -> Result<Option<Value>, String> {
        let attributes = self
            .attributes
            .read()
            .map_err(|e| format!("Failed to acquire read lock on attributes: {}", e))?;

        Ok(attributes.get(key).cloned())
    }

    /// 检查属性是否存在
    pub fn has_attribute(&self, key: &str) -> Result<bool, String> {
        let attributes = self
            .attributes
            .read()
            .map_err(|e| format!("Failed to acquire read lock on attributes: {}", e))?;

        Ok(attributes.contains_key(key))
    }

    /// 获取所有属性键
    pub fn get_attribute_keys(&self) -> Result<Vec<String>, String> {
        let attributes = self
            .attributes
            .read()
            .map_err(|e| format!("Failed to acquire read lock on attributes: {}", e))?;

        Ok(attributes.keys().cloned().collect())
    }

    /// 删除属性
    pub fn remove_attribute(&self, key: &str) -> Result<Option<Value>, String> {
        let mut attributes = self
            .attributes
            .write()
            .map_err(|e| format!("Failed to acquire write lock on attributes: {}", e))?;

        Ok(attributes.remove(key))
    }

    // ==================== 请求取消和超时控制 ====================

    /// 取消请求
    pub fn cancel(&self) -> Result<(), String> {
        self.cancelled.store(true, Ordering::SeqCst);
        self.mark_cancelled()?;
        self.log(LogLevel::Warning, "请求已取消".to_string(), None)?;
        
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        stats.cancelled_queries += 1;
        
        Ok(())
    }

    /// 检查请求是否已取消
    pub fn is_cancelled_flag(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 标记请求超时
    pub fn mark_timed_out(&self) -> Result<(), String> {
        self.timed_out.store(true, Ordering::SeqCst);
        self.mark_failed()?;
        self.log(LogLevel::Error, "请求超时".to_string(), None)?;
        
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        stats.timed_out_queries += 1;
        
        Ok(())
    }

    /// 检查请求是否超时
    pub fn is_timed_out(&self) -> bool {
        self.timed_out.load(Ordering::SeqCst)
    }

    /// 检查是否超时
    pub fn check_timeout(&self) -> Result<bool, String> {
        let timeout_ms = self.timeout_ms();
        let elapsed_ms = self.duration().as_millis() as u64;
        
        if elapsed_ms > timeout_ms {
            self.mark_timed_out()?;
            return Ok(true);
        }
        
        Ok(false)
    }

    // ==================== 日志记录 ====================

    /// 记录日志
    pub fn log(&self, level: LogLevel, message: String, context: Option<String>) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let log_entry = RequestLog {
            timestamp: now,
            level,
            message,
            context,
        };
        
        let mut logs = self
            .logs
            .write()
            .map_err(|e| format!("Failed to acquire write lock on logs: {}", e))?;
        logs.push(log_entry);
        
        Ok(())
    }

    /// 获取所有日志
    pub fn get_logs(&self) -> Result<Vec<RequestLog>, String> {
        let logs = self
            .logs
            .read()
            .map_err(|e| format!("Failed to acquire read lock on logs: {}", e))?;
        Ok(logs.clone())
    }

    /// 获取指定级别的日志
    pub fn get_logs_by_level(&self, level: LogLevel) -> Result<Vec<RequestLog>, String> {
        let logs = self
            .logs
            .read()
            .map_err(|e| format!("Failed to acquire read lock on logs: {}", e))?;
        Ok(logs.iter().filter(|log| log.level == level).cloned().collect())
    }

    /// 清除日志
    pub fn clear_logs(&self) -> Result<(), String> {
        let mut logs = self
            .logs
            .write()
            .map_err(|e| format!("Failed to acquire write lock on logs: {}", e))?;
        logs.clear();
        Ok(())
    }

    // ==================== 统计信息 ====================

    /// 获取统计信息
    pub fn get_statistics(&self) -> Result<RequestStatistics, String> {
        let stats = self
            .statistics
            .read()
            .map_err(|e| format!("Failed to acquire read lock on statistics: {}", e))?;
        Ok(stats.clone())
    }

    /// 更新统计信息
    pub fn update_statistics(&self, success: bool, execution_time_ms: u64) -> Result<(), String> {
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        
        stats.total_queries += 1;
        stats.total_execution_time_ms += execution_time_ms;
        
        if success {
            stats.successful_queries += 1;
        } else {
            stats.failed_queries += 1;
        }
        
        if execution_time_ms > stats.max_execution_time_ms {
            stats.max_execution_time_ms = execution_time_ms;
        }
        
        if execution_time_ms < stats.min_execution_time_ms {
            stats.min_execution_time_ms = execution_time_ms;
        }
        
        if stats.total_queries > 0 {
            stats.avg_execution_time_ms = stats.total_execution_time_ms as f64 / stats.total_queries as f64;
        }
        
        Ok(())
    }

    /// 重置统计信息
    pub fn reset_statistics(&self) -> Result<(), String> {
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        
        stats.total_queries = 0;
        stats.successful_queries = 0;
        stats.failed_queries = 0;
        stats.cancelled_queries = 0;
        stats.timed_out_queries = 0;
        stats.total_execution_time_ms = 0;
        stats.avg_execution_time_ms = 0.0;
        stats.max_execution_time_ms = 0;
        stats.min_execution_time_ms = u64::MAX;
        
        Ok(())
    }

    /// 获取执行计数
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// 增加执行计数
    pub fn increment_execution_count(&self) {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
    }

    /// 生成请求上下文的字符串表示
    pub fn to_string(&self) -> Result<String, String> {
        let status = self.status()?;
        let response = self.get_response()?;
        let attributes = self
            .attributes
            .read()
            .map_err(|e| format!("Failed to acquire read lock on attributes: {}", e))?;

        let mut result = String::new();
        result.push_str("RequestContext {\n");

        if let Some(session) = &self.session_info {
            result.push_str(&format!("  session_id: {},\n", session.session_id));
            result.push_str(&format!("  user_name: {},\n", session.username));
            result.push_str(&format!("  client_ip: {},\n", session.client_ip));
        }

        result.push_str(&format!("  query: {},\n", self.request_params.query));
        result.push_str(&format!("  status: {:?},\n", status));
        result.push_str(&format!("  response_success: {},\n", response.success));
        result.push_str(&format!(
            "  execution_time_ms: {},\n",
            response.execution_time_ms
        ));
        result.push_str(&format!("  affected_rows: {},\n", response.affected_rows));
        result.push_str(&format!("  attributes_count: {},\n", attributes.len()));
        result.push_str(&format!("  duration: {:?},\n", self.duration()));
        result.push_str("}");

        Ok(result)
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
        Self::new(session_info, request_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let session_info = SessionInfo::new(
            "test_session".to_string(),
            "test_user".to_string(),
            vec![],
            "192.168.1.1".to_string(),
            12345,
            "test_client".to_string(),
            "test_connection".to_string(),
        );
        let request_params = RequestParams::new("MATCH (n) RETURN n".to_string());
        let ctx = RequestContext::new(session_info, request_params);

        assert_eq!(ctx.session_id(), Some("test_session"));
        assert_eq!(ctx.user_name(), Some("test_user"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.1"));
        assert_eq!(ctx.query(), "MATCH (n) RETURN n");
        assert_eq!(ctx.timeout_ms(), 30000);
        assert_eq!(ctx.max_retry_times(), 3);
    }

    #[test]
    fn test_request_context_simple() {
        let ctx = RequestContext::with_session(
            "SELECT * FROM users".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );
        assert_eq!(ctx.query(), "SELECT * FROM users");
        assert_eq!(ctx.session_id(), Some("test_session"));
        assert_eq!(ctx.user_name(), Some("test_user"));
    }

    #[test]
    fn test_request_context_with_session() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "custom_session",
            "admin",
            "192.168.1.100",
            8080,
        );
        assert_eq!(ctx.query(), "MATCH (n) RETURN n");
        assert_eq!(ctx.session_id(), Some("custom_session"));
        assert_eq!(ctx.user_name(), Some("admin"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.100"));
    }

    #[test]
    fn test_request_context_with_parameters() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), Value::String("Alice".to_string()));
        params.insert("age".to_string(), Value::Int(25));

        let ctx = RequestContext::with_parameters(
            "MATCH (n) WHERE n.name = $name AND n.age = $age RETURN n".to_string(),
            params,
            "custom_session",
            "admin",
            "192.168.1.100",
            8080,
        );

        assert_eq!(
            ctx.query(),
            "MATCH (n) WHERE n.name = $name AND n.age = $age RETURN n"
        );
        assert_eq!(
            ctx.get_parameter("name"),
            Some(Value::String("Alice".to_string()))
        );
        assert_eq!(ctx.get_parameter("age"), Some(Value::Int(25)));
        assert_eq!(ctx.session_id(), Some("custom_session"));
        assert_eq!(ctx.user_name(), Some("admin"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.100"));
    }

    #[test]
    fn test_request_context_with_timeout() {
        let ctx = RequestContext::with_timeout(
            "LONG RUNNING QUERY".to_string(),
            60000,
            "timeout_session",
            "user",
            "192.168.1.200",
            9090,
        );
        assert_eq!(ctx.query(), "LONG RUNNING QUERY");
        assert_eq!(ctx.timeout_ms(), 60000);
        assert_eq!(ctx.session_id(), Some("timeout_session"));
    }

    #[test]
    fn test_request_context_with_retry() {
        let ctx = RequestContext::with_retry(
            "QUERY WITH RETRY".to_string(),
            5,
            "retry_session",
            "retry_user",
            "192.168.1.150",
            7070,
        );
        assert_eq!(ctx.query(), "QUERY WITH RETRY");
        assert_eq!(ctx.max_retry_times(), 5);
        assert_eq!(ctx.session_id(), Some("retry_session"));
    }

    #[test]
    fn test_request_context_with_parameters_from_context() {
        let base_ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "base_session",
            "base_user",
            "192.168.1.50",
            6060,
        );

        let mut params = HashMap::new();
        params.insert("limit".to_string(), Value::Int(10));

        let new_ctx = base_ctx.with_parameters_from_context(params);

        assert_eq!(new_ctx.query(), "MATCH (n) RETURN n");
        assert_eq!(new_ctx.get_parameter("limit"), Some(Value::Int(10)));
        assert_eq!(new_ctx.session_id(), Some("base_session"));
        assert_eq!(new_ctx.user_name(), Some("base_user"));
    }

    #[test]
    fn test_request_context_with_timeout_from_context() {
        let base_ctx = RequestContext::with_session(
            "LONG QUERY".to_string(),
            "base_session",
            "base_user",
            "192.168.1.60",
            5050,
        );

        let new_ctx = base_ctx.with_timeout_from_context(120000);

        assert_eq!(new_ctx.query(), "LONG QUERY");
        assert_eq!(new_ctx.timeout_ms(), 120000);
        assert_eq!(new_ctx.session_id(), Some("base_session"));
    }

    #[test]
    fn test_request_context_with_retry_from_context() {
        let base_ctx = RequestContext::with_session(
            "RETRY QUERY".to_string(),
            "base_session",
            "base_user",
            "192.168.1.70",
            4040,
        );

        let new_ctx = base_ctx.with_retry_from_context(10);

        assert_eq!(new_ctx.query(), "RETRY QUERY");
        assert_eq!(new_ctx.max_retry_times(), 10);
        assert_eq!(new_ctx.session_id(), Some("base_session"));
    }

    #[test]
    fn test_request_parameters() {
        let mut ctx = RequestContext::with_session(
            "MATCH (n) WHERE n.name = $name RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 设置参数
        ctx.set_parameter("name".to_string(), Value::String("Alice".to_string()));

        // 获取参数
        let param = ctx.get_parameter("name");
        assert!(param.is_some());
        assert_eq!(
            param.expect("Expected parameter 'name' to exist"),
            Value::String("Alice".to_string())
        );

        // 获取不存在的参数
        let missing_param = ctx.get_parameter("missing");
        assert!(missing_param.is_none());
    }

    #[test]
    fn test_response_management() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 设置响应数据
        let data = Value::List(vec![
            Value::Map(std::collections::HashMap::new()),
            Value::Map(std::collections::HashMap::new()),
        ]);
        ctx.set_response_data(data.clone())
            .expect("Expected successful setting of response data");

        // 获取响应
        let response = ctx
            .get_response()
            .expect("Expected successful retrieval of response");
        assert!(response.is_success());
        assert!(response.get_data().is_some());

        // 设置响应错误
        ctx.set_response_error("Query failed".to_string())
            .expect("Expected successful setting of response error");
        let response = ctx
            .get_response()
            .expect("Expected successful retrieval of response after error");
        assert!(!response.is_success());
        assert!(response.get_error().is_some());
        assert_eq!(
            response
                .get_error()
                .expect("Expected error message to exist"),
            "Query failed"
        );
    }

    #[test]
    fn test_request_lifecycle() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 初始状态
        assert_eq!(
            ctx.status()
                .expect("Expected successful retrieval of status"),
            RequestStatus::Pending
        );
        assert!(!ctx
            .is_completed()
            .expect("Expected successful check for completion"));

        // 标记为处理中
        ctx.mark_processing()
            .expect("Expected successful marking as processing");
        assert_eq!(
            ctx.status()
                .expect("Expected successful retrieval of processing status"),
            RequestStatus::Processing
        );

        // 标记为完成
        ctx.mark_completed()
            .expect("Expected successful marking as completed");
        assert_eq!(
            ctx.status()
                .expect("Expected successful retrieval of completed status"),
            RequestStatus::Completed
        );
        assert!(ctx
            .is_completed()
            .expect("Expected successful check for completion"));

        // 测试失败状态
        let ctx2 = RequestContext::with_session(
            "INVALID QUERY".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );
        ctx2.mark_failed()
            .expect("Expected successful marking as failed");
        assert!(ctx2
            .is_failed()
            .expect("Expected successful check for failure"));

        // 测试取消状态
        let ctx3 = RequestContext::with_session(
            "LONG RUNNING QUERY".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );
        ctx3.mark_cancelled()
            .expect("Expected successful marking as cancelled");
        assert!(ctx3
            .is_cancelled()
            .expect("Expected successful check for cancellation"));
    }

    #[test]
    fn test_attributes() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 设置属性
        ctx.set_attribute("query_type".to_string(), Value::String("read".to_string()))
            .expect("Expected successful setting of 'query_type' attribute");
        ctx.set_attribute("priority".to_string(), Value::Int(1))
            .expect("Expected successful setting of 'priority' attribute");

        // 获取属性
        let query_type = ctx
            .get_attribute("query_type")
            .expect("Expected successful retrieval of 'query_type' attribute");
        assert!(query_type.is_some());
        assert_eq!(
            query_type.expect("Expected 'query_type' attribute to exist"),
            Value::String("read".to_string())
        );

        // 检查属性存在
        assert!(ctx
            .has_attribute("priority")
            .expect("Expected successful check for 'priority' attribute"));
        assert!(!ctx
            .has_attribute("missing")
            .expect("Expected successful check for 'missing' attribute"));

        // 获取所有属性键
        let keys = ctx
            .get_attribute_keys()
            .expect("Expected successful retrieval of attribute keys");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"query_type".to_string()));
        assert!(keys.contains(&"priority".to_string()));

        // 删除属性
        let removed = ctx
            .remove_attribute("priority")
            .expect("Expected successful removal of 'priority' attribute");
        assert!(removed.is_some());
        assert!(!ctx
            .has_attribute("priority")
            .expect("Expected successful check after removing 'priority' attribute"));
    }

    #[test]
    fn test_duration() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        let duration = ctx.duration();
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_to_string() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );
        ctx.set_attribute("test".to_string(), Value::String("value".to_string()))
            .expect("Expected successful setting of 'test' attribute");

        let ctx_str = ctx
            .to_string()
            .expect("Expected successful retrieval of context string");
        assert!(ctx_str.contains("RequestContext"));
        assert!(ctx_str.contains("MATCH (n) RETURN n"));
        assert!(ctx_str.contains("test_session"));
        assert!(ctx_str.contains("test_user"));
        assert!(ctx_str.contains("attributes_count: 1"));
    }
}
