//! 请求上下文模块 - 管理查询请求的上下文信息

use crate::core::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

/// 会话信息简化版 - 用于查询层
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: i64,
    pub user_name: String,
    pub space_name: Option<String>,
    pub graph_addr: Option<String>,
    pub create_time: SystemTime,
    pub last_access_time: SystemTime,
    pub active_queries: i32,
    pub timezone: Option<i32>,
}

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
            create_time: SystemTime::now(),
            last_access_time: SystemTime::now(),
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
            create_time: SystemTime::now(),
            last_access_time: SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let request_params = RequestParams::new(query).with_parameters(parameters);
        Self::new(Some(session_info), request_params)
    }

    /// 获取会话信息
    pub fn session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    /// 获取请求参数
    pub fn request_params(&self) -> &RequestParams {
        &self.request_params
    }

    /// 获取查询字符串
    pub fn query(&self) -> &str {
        &self.request_params.query
    }

    /// 获取参数
    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.request_params.parameters
    }

    /// 获取响应
    pub fn response(&self) -> &Response {
        &self.response
    }

    /// 设置响应
    pub fn set_response(&mut self, response: Response) {
        self.response = Arc::new(response);
    }

    /// 获取会话ID
    pub fn session_id(&self) -> Option<i64> {
        self.session_info.as_ref().map(|s| s.session_id)
    }

    /// 获取用户名
    pub fn user_name(&self) -> Option<&str> {
        self.session_info.as_ref().map(|s| s.user_name.as_str())
    }

    /// 获取图空间名称
    pub fn space_name(&self) -> Option<&str> {
        self.session_info.as_ref().and_then(|s| s.space_name.as_deref())
    }
}
