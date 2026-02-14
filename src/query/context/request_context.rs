//! 请求上下文模块 - 管理查询请求的上下文信息
//! 对应原C++中的RequestContext.h

use crate::core::Value;
use std::collections::HashMap;
use std::sync::Arc;

// SessionInfo 现在统一使用 api/session/session_manager.rs 中的定义
use crate::api::session::session_manager::SessionInfo;

/// 请求参数
#[derive(Debug, Clone)]
pub struct RequestParams {
    pub query: String,
    pub parameters: HashMap<String, Value>,
}

impl RequestParams {
    pub fn new(query: String) -> Self {
        Self {
            query,
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameters(mut self, params: HashMap<String, Value>) -> Self {
        self.parameters = params;
        self
    }
}

/// 响应对象
#[derive(Debug, Clone)]
pub struct Response {
    pub success: bool,
    pub data: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl Response {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            data: None,
            error_message: None,
            execution_time_ms: 0,
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
/// 1. 会话信息管理
/// 2. 请求参数管理
/// 3. 响应对象管理
#[derive(Debug, Clone)]
pub struct RequestContext {
    // 会话信息
    session_info: Option<SessionInfo>,

    // 请求参数
    request_params: Arc<RequestParams>,

    // 响应对象
    response: Arc<Response>,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(session_info: Option<SessionInfo>, request_params: RequestParams) -> Self {
        Self {
            session_info,
            request_params: Arc::new(request_params),
            response: Arc::new(Response::new(true)),
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
        let session_info = SessionInfo {
            session_id: session_id.parse().unwrap_or(0),
            user_name: user_name.to_string(),
            space_name: None,
            graph_addr: Some(format!("{}:{}", client_ip, client_port)),
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let request_params = RequestParams::new(query);
        Self::new(Some(session_info), request_params)
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
        let session_info = SessionInfo {
            session_id: session_id.parse().unwrap_or(0),
            user_name: user_name.to_string(),
            space_name: None,
            graph_addr: Some(format!("{}:{}", client_ip, client_port)),
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let request_params = RequestParams::new(query).with_parameters(parameters);
        Self::new(Some(session_info), request_params)
    }

    /// 基于现有请求上下文创建带参数的请求上下文
    pub fn with_parameters_from_context(&self, parameters: HashMap<String, Value>) -> Self {
        let session_info = self.session_info.clone().unwrap_or_else(|| {
            SessionInfo {
                session_id: 0,
                user_name: "unknown_user".to_string(),
                space_name: None,
                graph_addr: None,
                create_time: std::time::SystemTime::now(),
                last_access_time: std::time::SystemTime::now(),
                active_queries: 0,
                timezone: None,
            }
        });
        let query = self.request_params.query.clone();
        let request_params = RequestParams::new(query).with_parameters(parameters);
        Self::new(Some(session_info), request_params)
    }

    // ==================== 会话信息管理 ====================

    /// 获取会话信息
    pub fn session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    /// 获取会话ID
    pub fn session_id(&self) -> Option<i64> {
        self.session_info.as_ref().map(|s| s.session_id)
    }

    /// 获取用户名
    pub fn user_name(&self) -> Option<&str> {
        self.session_info.as_ref().map(|s| s.user_name.as_str())
    }

    /// 获取客户端IP
    pub fn client_ip(&self) -> Option<&str> {
        self.session_info.as_ref().and_then(|s| s.graph_addr.as_deref())
    }

    // ==================== 请求参数管理 ====================

    /// 获取查询字符串
    pub fn query(&self) -> String {
        self.request_params.query.clone()
    }

    /// 获取请求参数
    pub fn request_params(&self) -> RequestParams {
        self.request_params.as_ref().clone()
    }

    /// 获取参数值
    pub fn get_parameter(&self, name: &str) -> Option<Value> {
        self.request_params.parameters.get(name).cloned()
    }

    /// 设置参数值
    pub fn set_parameter(&self, _name: String, _value: Value) -> Result<(), String> {
        Err("RequestParams 是不可变的".to_string())
    }

    // ==================== 响应对象管理 ====================

    /// 设置响应数据
    pub fn set_response_data(&self, _data: Value) -> Result<(), String> {
        Err("Response 是不可变的".to_string())
    }

    /// 设置响应错误
    pub fn set_response_error(&self, _error: String) -> Result<(), String> {
        Err("Response 是不可变的".to_string())
    }

    /// 获取响应
    pub fn get_response(&self) -> Response {
        self.response.as_ref().clone()
    }

    /// 获取响应数据
    pub fn get_response_data(&self) -> Option<Value> {
        self.response.data.clone()
    }

    /// 获取响应错误
    pub fn get_response_error(&self) -> Option<String> {
        self.response.error_message.clone()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        let session_info = SessionInfo {
            session_id: 0,
            user_name: "default_user".to_string(),
            space_name: None,
            graph_addr: Some("localhost:0".to_string()),
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let request_params = RequestParams::new("SELECT 1".to_string());
        Self::new(Some(session_info), request_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::dataset::List;

    #[test]
    fn test_request_context_creation() {
        let session_info = SessionInfo {
            session_id: 12345,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: Some("192.168.1.1:8080".to_string()),
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let request_params = RequestParams::new("MATCH (n) RETURN n".to_string());
        let ctx = RequestContext::new(Some(session_info), request_params);

        assert_eq!(ctx.session_id(), Some(12345));
        assert_eq!(ctx.user_name(), Some("test_user"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.1:8080"));
        assert_eq!(ctx.query(), "MATCH (n) RETURN n");
    }

    #[test]
    fn test_request_context_simple() {
        let ctx = RequestContext::with_session(
            "SELECT * FROM users".to_string(),
            "12345",
            "test_user",
            "127.0.0.1",
            0,
        );
        assert_eq!(ctx.query(), "SELECT * FROM users");
        assert_eq!(ctx.session_id(), Some(12345));
        assert_eq!(ctx.user_name(), Some("test_user"));
    }

    #[test]
    fn test_request_context_with_session() {
        let ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "99999",
            "admin",
            "192.168.1.100",
            8080,
        );
        assert_eq!(ctx.query(), "MATCH (n) RETURN n");
        assert_eq!(ctx.session_id(), Some(99999));
        assert_eq!(ctx.user_name(), Some("admin"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.100:8080"));
    }

    #[test]
    fn test_request_context_with_parameters() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), Value::String("Alice".to_string()));
        params.insert("age".to_string(), Value::Int(25));

        let ctx = RequestContext::with_parameters(
            "MATCH (n) WHERE n.name = $name AND n.age = $age RETURN n".to_string(),
            params,
            "88888",
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
        assert_eq!(ctx.session_id(), Some(88888));
        assert_eq!(ctx.user_name(), Some("admin"));
        assert_eq!(ctx.client_ip(), Some("192.168.1.100:8080"));
    }

    #[test]
    fn test_request_context_with_parameters_from_context() {
        let base_ctx = RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "55555",
            "base_user",
            "192.168.1.50",
            6060,
        );

        let mut params = HashMap::new();
        params.insert("limit".to_string(), Value::Int(10));

        let new_ctx = base_ctx.with_parameters_from_context(params);

        assert_eq!(new_ctx.query(), "MATCH (n) RETURN n");
        assert_eq!(new_ctx.get_parameter("limit"), Some(Value::Int(10)));
        assert_eq!(new_ctx.session_id(), Some(55555));
        assert_eq!(new_ctx.user_name(), Some("base_user"));
    }

    #[test]
    fn test_request_parameters() {
        let ctx = RequestContext::with_session(
            "MATCH (n) WHERE n.name = $name RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        );

        // 设置参数
        let result = ctx.set_parameter("name".to_string(), Value::String("Alice".to_string()));
        assert!(result.is_err());

        // 获取参数
        let param = ctx.get_parameter("name");
        assert!(param.is_none());
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
        let result = ctx.set_response_data(Value::List(List::from(vec![])));
        assert!(result.is_err());

        // 获取响应
        let response = ctx.get_response();
        assert!(response.is_success());
        assert!(response.get_data().is_none());

        // 设置响应错误
        let result = ctx.set_response_error("Query failed".to_string());
        assert!(result.is_err());
    }
}