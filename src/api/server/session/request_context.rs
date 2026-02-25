//! 请求上下文模块 - 管理查询请求的上下文信息
//! 对应原C++中的RequestContext.h

use crate::core::Value;
use crate::core::ErrorCode;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime};

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

impl SessionInfo {
    /// 创建新的会话信息
    pub fn new(
        session_id: i64,
        user_name: String,
        space_name: Option<String>,
        graph_addr: Option<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            session_id,
            user_name,
            space_name,
            graph_addr,
            create_time: now,
            last_access_time: now,
            active_queries: 0,
            timezone: None,
        }
    }

    /// 从字符串参数创建会话信息
    pub fn from_params(
        session_id_str: &str,
        user_name: &str,
        space_name: Option<String>,
        client_ip: &str,
        client_port: u16,
    ) -> Result<Self, String> {
        let session_id = session_id_str
            .parse::<i64>()
            .map_err(|_| format!("无效的会话ID: {}", session_id_str))?;

        let graph_addr = if client_ip.is_empty() {
            None
        } else {
            Some(format!("{}:{}", client_ip, client_port))
        };

        Ok(Self::new(session_id, user_name.to_string(), space_name, graph_addr))
    }

    /// 更新最后访问时间
    pub fn touch(&mut self) {
        self.last_access_time = SystemTime::now();
    }
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
    pub error_code: ErrorCode,
    pub data: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub affected_rows: u64,
    pub warnings: Vec<String>,
}

impl Response {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            error_code: if success { ErrorCode::Success } else { ErrorCode::Unknown },
            data: None,
            error_message: None,
            execution_time_ms: 0,
            affected_rows: 0,
            warnings: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.success = false;
        self.error_code = ErrorCode::ExecutionError;
        self
    }

    pub fn with_error_code(mut self, code: ErrorCode) -> Self {
        self.error_code = code;
        if code != ErrorCode::Success {
            self.success = false;
        }
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

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
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
#[derive(Debug)]
pub struct RequestContext {
    // 会话信息
    session_info: Option<SessionInfo>,

    // 请求参数
    request_params: Arc<RequestParams>,

    // 响应对象 - 使用 RwLock 支持内部可变性
    response: Arc<RwLock<Response>>,

    // 查询开始时间
    query_start_time: Instant,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(session_info: Option<SessionInfo>, request_params: RequestParams) -> Self {
        Self {
            session_info,
            request_params: Arc::new(request_params),
            response: Arc::new(RwLock::new(Response::new(true))),
            query_start_time: Instant::now(),
        }
    }

    /// 创建带会话信息的请求上下文
    pub fn with_session(
        query: String,
        session_id: &str,
        user_name: &str,
        client_ip: &str,
        client_port: u16,
    ) -> Result<Self, String> {
        let session_info = SessionInfo::from_params(
            session_id,
            user_name,
            None,
            client_ip,
            client_port,
        )?;
        let request_params = RequestParams::new(query);
        Ok(Self::new(Some(session_info), request_params))
    }

    /// 创建带参数的请求上下文
    pub fn with_parameters(
        query: String,
        parameters: HashMap<String, Value>,
        session_id: &str,
        user_name: &str,
        client_ip: &str,
        client_port: u16,
    ) -> Result<Self, String> {
        let session_info = SessionInfo::from_params(
            session_id,
            user_name,
            None,
            client_ip,
            client_port,
        )?;
        let request_params = RequestParams::new(query).with_parameters(parameters);
        Ok(Self::new(Some(session_info), request_params))
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
    pub fn response(&self) -> Result<Response, String> {
        self.response
            .read()
            .map(|guard| guard.clone())
            .map_err(|_| "获取响应锁失败".to_string())
    }

    /// 设置响应
    pub fn set_response(&self, response: Response) -> Result<(), String> {
        let mut guard = self
            .response
            .write()
            .map_err(|_| "获取响应写锁失败".to_string())?;
        *guard = response;
        Ok(())
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

    /// 设置图空间名称
    pub fn set_space_name(&mut self, space_name: String) {
        if let Some(ref mut session) = self.session_info {
            session.space_name = Some(space_name);
        }
    }

    /// 获取图空间ID
    /// 
    /// # 注意
    /// 此方法已废弃，请使用 `QueryContext::space_id()` 获取空间ID。
    /// RequestContext 只保存 space_name，不直接访问元数据服务。
    /// 
    /// # 替代方案
    /// 通过 QueryContext 获取 space_id：
    /// ```rust,ignore
    /// let space_id = query_context.space_id();
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "请使用 QueryContext::space_id() 替代"
    )]
    pub fn space_id(&self) -> Option<i64> {
        None
    }

    /// 设置响应错误
    pub fn set_response_error(&self, error: String) -> Result<(), String> {
        let mut guard = self
            .response
            .write()
            .map_err(|_| "获取响应写锁失败".to_string())?;
        guard.success = false;
        guard.error_code = ErrorCode::ExecutionError;
        guard.error_message = Some(error);
        Ok(())
    }

    /// 设置响应错误带错误码
    pub fn set_response_error_with_code(
        &self,
        error: String,
        code: ErrorCode,
    ) -> Result<(), String> {
        let mut guard = self
            .response
            .write()
            .map_err(|_| "获取响应写锁失败".to_string())?;
        guard.success = false;
        guard.error_code = code;
        guard.error_message = Some(error);
        Ok(())
    }

    /// 添加警告信息
    pub fn add_warning(&self, warning: String) -> Result<(), String> {
        let mut guard = self
            .response
            .write()
            .map_err(|_| "获取响应写锁失败".to_string())?;
        guard.warnings.push(warning);
        Ok(())
    }

    /// 设置执行时间
    pub fn set_execution_time(&self) -> Result<(), String> {
        let elapsed = self.query_start_time.elapsed().as_millis() as u64;
        let mut guard = self
            .response
            .write()
            .map_err(|_| "获取响应写锁失败".to_string())?;
        guard.execution_time_ms = elapsed;
        Ok(())
    }

    /// 获取执行时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.query_start_time.elapsed().as_millis() as u64
    }

    /// 获取参数
    pub fn get_parameter(&self, param: &str) -> Option<Value> {
        self.request_params.parameters.get(param).cloned()
    }

    /// 更新会话最后访问时间
    pub fn touch_session(&mut self) {
        if let Some(ref mut session) = self.session_info {
            session.touch();
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            session_info: None,
            request_params: Arc::new(RequestParams::new(String::new())),
            response: Arc::new(RwLock::new(Response::new(true))),
            query_start_time: Instant::now(),
        }
    }
}

impl Clone for RequestContext {
    fn clone(&self) -> Self {
        Self {
            session_info: self.session_info.clone(),
            request_params: self.request_params.clone(),
            response: self.response.clone(),
            query_start_time: self.query_start_time,
        }
    }
}

/// 从 ClientSession 创建 QueryRequestContext
/// 
/// 这个转换函数确保 api 层的会话信息能正确传递到 query 层
pub fn build_query_request_context(
    session: &super::ClientSession,
    query: String,
    parameters: std::collections::HashMap<String, crate::core::Value>,
) -> crate::query::query_request_context::QueryRequestContext {
    use crate::query::query_request_context::QueryRequestContext;

    QueryRequestContext {
        session_id: Some(session.id()),
        user_name: Some(session.user()),
        space_name: session.space_name(),
        query,
        parameters,
    }
}
