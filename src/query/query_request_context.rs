//! 查询请求上下文 - query层专用简化版
//!
//! 此模块提供查询执行所需的最小上下文信息，避免query层依赖api层。

use crate::core::Value;
use std::collections::HashMap;

/// 查询请求上下文 - 简化版
///
/// 仅包含查询执行所需的最小信息：
/// - 会话ID
/// - 图空间名称
/// - 查询字符串
/// - 查询参数
#[derive(Debug, Clone)]
pub struct QueryRequestContext {
    /// 会话ID
    pub session_id: Option<i64>,
    /// 用户名
    pub user_name: Option<String>,
    /// 图空间名称
    pub space_name: Option<String>,
    /// 查询字符串
    pub query: String,
    /// 查询参数
    pub parameters: HashMap<String, Value>,
}

impl QueryRequestContext {
    /// 创建新的查询请求上下文
    pub fn new(query: String) -> Self {
        Self {
            session_id: None,
            user_name: None,
            space_name: None,
            query,
            parameters: HashMap::new(),
        }
    }

    /// 创建带参数的查询请求上下文
    pub fn with_parameters(mut self, parameters: HashMap<String, Value>) -> Self {
        self.parameters = parameters;
        self
    }

    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: i64) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 设置用户名
    pub fn with_user_name(mut self, user_name: String) -> Self {
        self.user_name = Some(user_name);
        self
    }

    /// 设置图空间名称
    pub fn with_space_name(mut self, space_name: String) -> Self {
        self.space_name = Some(space_name);
        self
    }

    /// 获取参数
    pub fn get_parameter(&self, param: &str) -> Option<Value> {
        self.parameters.get(param).cloned()
    }

    /// 检查参数是否存在
    pub fn has_parameter(&self, param: &str) -> bool {
        self.parameters.contains_key(param)
    }
}

impl Default for QueryRequestContext {
    fn default() -> Self {
        Self {
            session_id: None,
            user_name: None,
            space_name: None,
            query: String::new(),
            parameters: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_request_context_new() {
        let ctx = QueryRequestContext::new("MATCH (n) RETURN n".to_string());
        assert_eq!(ctx.query, "MATCH (n) RETURN n");
        assert!(ctx.session_id.is_none());
        assert!(ctx.space_name.is_none());
    }

    #[test]
    fn test_query_request_context_with_params() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), Value::from("test"));

        let ctx = QueryRequestContext::new("QUERY".to_string())
            .with_parameters(params)
            .with_session_id(123)
            .with_space_name("test_space".to_string());

        assert_eq!(ctx.session_id, Some(123));
        assert_eq!(ctx.space_name, Some("test_space".to_string()));
        assert!(ctx.has_parameter("name"));
    }
}
