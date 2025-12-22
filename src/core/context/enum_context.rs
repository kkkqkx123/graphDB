//! 统一上下文系统
//!
//! 提供零成本抽象的统一上下文实现，避免动态分发

use super::base::{ContextBase, ContextType, MutableContext};
use super::execution::ExecutionContext;
use super::expression::{BasicExpressionContext, ExpressionContext};
use super::query::QueryContext;
use super::request::RequestContext;
use super::runtime::RuntimeContext;
use super::session::{SessionContext, SessionInfo};
use super::storage::StorageContext;
use super::validation::ValidationContext;
use crate::core::types::query::QueryType;
use crate::Value;

/// 统一上下文枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum UnifiedContext {
    /// 会话上下文
    Session(SessionContext),
    /// 查询上下文
    Query(QueryContext),
    /// 执行上下文
    Execution(ExecutionContext),
    /// 表达式上下文
    Expression(BasicExpressionContext),
    /// 请求上下文
    Request(RequestContext),
    /// 运行时上下文
    Runtime(RuntimeContext),
    /// 验证上下文
    Validation(ValidationContext),
    /// 存储上下文
    Storage(StorageContext),
}

impl UnifiedContext {
    /// 获取上下文ID
    pub fn id(&self) -> &str {
        match self {
            UnifiedContext::Session(ctx) => ctx.session_id.as_str(),
            UnifiedContext::Query(ctx) => ctx.query_id.as_str(),
            UnifiedContext::Execution(ctx) => ctx.query_context.query_id.as_str(),
            UnifiedContext::Expression(_) => "expression_context",
            UnifiedContext::Request(ctx) => ctx.id.as_str(),
            UnifiedContext::Runtime(ctx) => ctx.id.as_str(),
            UnifiedContext::Validation(ctx) => ctx.id.as_str(),
            UnifiedContext::Storage(ctx) => ctx.id.as_str(),
        }
    }

    /// 获取上下文类型
    pub fn context_type(&self) -> ContextType {
        match self {
            UnifiedContext::Session(_) => ContextType::Session,
            UnifiedContext::Query(_) => ContextType::Query,
            UnifiedContext::Execution(_) => ContextType::Execution,
            UnifiedContext::Expression(_) => ContextType::Expression,
            UnifiedContext::Request(_) => ContextType::Request,
            UnifiedContext::Runtime(_) => ContextType::Runtime,
            UnifiedContext::Validation(_) => ContextType::Validation,
            UnifiedContext::Storage(_) => ContextType::Storage,
        }
    }

    /// 获取父上下文ID
    pub fn parent_id(&self) -> Option<&str> {
        match self {
            UnifiedContext::Session(_) => None,
            UnifiedContext::Query(_) => None,
            UnifiedContext::Execution(ctx) => Some(&ctx.query_context.query_id),
            UnifiedContext::Expression(ctx) => ctx.parent.as_ref().map(|_| "parent_expression"),
            UnifiedContext::Request(_) => None,
            UnifiedContext::Runtime(_) => None,
            UnifiedContext::Validation(_) => None,
            UnifiedContext::Storage(_) => None,
        }
    }

    /// 获取上下文深度
    pub fn depth(&self) -> usize {
        match self {
            UnifiedContext::Session(_) => 0,
            UnifiedContext::Query(_) => 1,
            UnifiedContext::Execution(_) => 2,
            UnifiedContext::Expression(ctx) => ctx.get_depth(),
            UnifiedContext::Request(_) => 1,
            UnifiedContext::Runtime(_) => 2,
            UnifiedContext::Validation(_) => 2,
            UnifiedContext::Storage(_) => 2,
        }
    }

    /// 获取创建时间
    pub fn created_at(&self) -> std::time::SystemTime {
        match self {
            UnifiedContext::Session(ctx) => ctx.created_at(),
            UnifiedContext::Query(_) => std::time::SystemTime::now(),
            UnifiedContext::Execution(_) => std::time::SystemTime::now(),
            UnifiedContext::Expression(_) => std::time::SystemTime::now(),
            UnifiedContext::Request(ctx) => ctx.created_at(),
            UnifiedContext::Runtime(_) => std::time::SystemTime::now(),
            UnifiedContext::Validation(_) => std::time::SystemTime::now(),
            UnifiedContext::Storage(ctx) => ctx.created_at(),
        }
    }

    /// 获取更新时间
    pub fn updated_at(&self) -> std::time::SystemTime {
        match self {
            UnifiedContext::Session(ctx) => ctx.last_activity,
            UnifiedContext::Query(_) => std::time::SystemTime::now(),
            UnifiedContext::Execution(_) => std::time::SystemTime::now(),
            UnifiedContext::Expression(_) => std::time::SystemTime::now(),
            UnifiedContext::Request(ctx) => ctx.updated_at,
            UnifiedContext::Runtime(_) => std::time::SystemTime::now(),
            UnifiedContext::Validation(_) => std::time::SystemTime::now(),
            UnifiedContext::Storage(ctx) => ctx.updated_at(),
        }
    }

    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        match self {
            UnifiedContext::Session(ctx) => ctx.is_valid(),
            UnifiedContext::Query(ctx) => ctx.is_valid(),
            UnifiedContext::Execution(ctx) => ctx.is_valid(),
            UnifiedContext::Expression(_) => true,
            UnifiedContext::Request(ctx) => ctx.valid,
            UnifiedContext::Runtime(ctx) => ctx.is_valid(),
            UnifiedContext::Validation(ctx) => ctx.is_valid(),
            UnifiedContext::Storage(ctx) => ctx.is_valid(),
        }
    }

    /// 获取属性
    pub fn get_attribute(&self, key: &str) -> Option<Value> {
        match self {
            UnifiedContext::Session(_) => None,
            UnifiedContext::Query(_) => None,
            UnifiedContext::Execution(_) => None,
            UnifiedContext::Expression(_) => None,
            UnifiedContext::Request(ctx) => {
                if let Ok(attributes) = ctx.attributes.read() {
                    attributes.get(key).cloned()
                } else {
                    None
                }
            }
            UnifiedContext::Runtime(_) => None,
            UnifiedContext::Validation(_) => None,
            UnifiedContext::Storage(ctx) => ctx.attributes.get(key).cloned(),
        }
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: String, value: Value) {
        match self {
            UnifiedContext::Session(_) => {}
            UnifiedContext::Query(_) => {}
            UnifiedContext::Execution(_) => {}
            UnifiedContext::Expression(_) => {}
            UnifiedContext::Request(ctx) => {
                if let Ok(mut attributes) = ctx.attributes.write() {
                    attributes.insert(key, value);
                }
            }
            UnifiedContext::Runtime(_) => {}
            UnifiedContext::Validation(_) => {}
            UnifiedContext::Storage(ctx) => {
                ctx.attributes.insert(key, value);
            }
        }
    }

    /// 获取属性键列表
    pub fn attribute_keys(&self) -> Vec<String> {
        match self {
            UnifiedContext::Session(_) => Vec::new(),
            UnifiedContext::Query(_) => Vec::new(),
            UnifiedContext::Execution(_) => Vec::new(),
            UnifiedContext::Expression(_) => Vec::new(),
            UnifiedContext::Request(ctx) => {
                if let Ok(attributes) = ctx.attributes.read() {
                    attributes.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            UnifiedContext::Runtime(_) => Vec::new(),
            UnifiedContext::Validation(_) => Vec::new(),
            UnifiedContext::Storage(ctx) => ctx.attributes.keys().cloned().collect(),
        }
    }

    /// 移除属性
    pub fn get_remove_attribute(&mut self, key: &str) -> Option<Value> {
        match self {
            UnifiedContext::Session(_) => None,
            UnifiedContext::Query(_) => None,
            UnifiedContext::Execution(_) => None,
            UnifiedContext::Expression(_) => None,
            UnifiedContext::Request(ctx) => {
                if let Ok(mut attributes) = ctx.attributes.write() {
                    attributes.remove(key)
                } else {
                    None
                }
            }
            UnifiedContext::Runtime(_) => None,
            UnifiedContext::Validation(_) => None,
            UnifiedContext::Storage(ctx) => ctx.attributes.remove(key),
        }
    }

    /// 清空属性
    pub fn get_clear_attributes(&mut self) {
        match self {
            UnifiedContext::Session(_) => {}
            UnifiedContext::Query(_) => {}
            UnifiedContext::Execution(_) => {}
            UnifiedContext::Expression(_) => {}
            UnifiedContext::Request(ctx) => {
                if let Ok(mut attributes) = ctx.attributes.write() {
                    attributes.clear();
                }
            }
            UnifiedContext::Runtime(_) => {}
            UnifiedContext::Validation(_) => {}
            UnifiedContext::Storage(ctx) => ctx.attributes.clear(),
        }
    }

    /// 更新时间戳
    pub fn touch(&mut self) {
        match self {
            UnifiedContext::Session(ctx) => ctx.touch(),
            UnifiedContext::Query(ctx) => ctx.touch(),
            UnifiedContext::Execution(ctx) => ctx.touch(),
            UnifiedContext::Expression(_) => {}
            UnifiedContext::Request(ctx) => ctx.touch(),
            UnifiedContext::Runtime(ctx) => ctx.touch(),
            UnifiedContext::Validation(ctx) => ctx.touch(),
            UnifiedContext::Storage(ctx) => ctx.touch(),
        }
    }

    /// 标记为无效
    pub fn invalidate(&mut self) {
        match self {
            UnifiedContext::Session(ctx) => ctx.invalidate(),
            UnifiedContext::Query(ctx) => ctx.invalidate(),
            UnifiedContext::Execution(ctx) => ctx.invalidate(),
            UnifiedContext::Expression(_) => {}
            UnifiedContext::Request(ctx) => ctx.invalidate(),
            UnifiedContext::Runtime(ctx) => ctx.invalidate(),
            UnifiedContext::Validation(ctx) => ctx.invalidate(),
            UnifiedContext::Storage(ctx) => ctx.invalidate(),
        }
    }

    /// 重新验证
    pub fn revalidate(&mut self) -> bool {
        match self {
            UnifiedContext::Session(ctx) => ctx.revalidate(),
            UnifiedContext::Query(ctx) => ctx.revalidate(),
            UnifiedContext::Execution(ctx) => ctx.revalidate(),
            UnifiedContext::Expression(_) => true,
            UnifiedContext::Request(ctx) => ctx.revalidate(),
            UnifiedContext::Runtime(ctx) => ctx.revalidate(),
            UnifiedContext::Validation(ctx) => ctx.revalidate(),
            UnifiedContext::Storage(ctx) => ctx.revalidate(),
        }
    }


    /// 创建测试上下文
    pub fn create_test_context(context_type: ContextType) -> Self {
        match context_type {
            ContextType::Session => UnifiedContext::Session(SessionContext::new(
                "test_session".to_string(),
                super::session::UserInfo::new(
                    "test_user".to_string(),
                    "test_user_id".to_string(),
                    vec!["user".to_string()],
                    vec!["read".to_string()],
                ),
                super::session::SessionConfig::default(),
            )),
            ContextType::Query => UnifiedContext::Query(QueryContext::new(
                "test_query".to_string(),
                QueryType::DataQuery,
                "MATCH (n) RETURN n".to_string(),
                SessionInfo::new(
                    "test_session".to_string(),
                    "test_user".to_string(),
                    vec!["user".to_string()],
                ),
            )),
            ContextType::Execution => {
                UnifiedContext::Execution(ExecutionContext::new(QueryContext::new(
                    "test_query".to_string(),
                    QueryType::DataQuery,
                    "MATCH (n) RETURN n".to_string(),
                    SessionInfo::new(
                        "test_session".to_string(),
                        "test_user".to_string(),
                        vec!["user".to_string()],
                    ),
                )))
            }
            ContextType::Expression => UnifiedContext::Expression(BasicExpressionContext::new()),
            ContextType::Request => UnifiedContext::Request(RequestContext::with_session(
                "test_request".to_string(),
                "MATCH (n) RETURN n".to_string(),
                "test_session",
                "test_user",
                "localhost",
                0,
            )),
            ContextType::Runtime => {
                let storage_env = std::sync::Arc::new(super::runtime::StorageEnv {
                    storage_engine: std::sync::Arc::new(super::manager::MockStorageEngine),
                    schema_manager: std::sync::Arc::new(super::manager::MockSchemaManager),
                    index_manager: std::sync::Arc::new(super::manager::MockIndexManager),
                });
                let plan_context = std::sync::Arc::new(super::runtime::PlanContext {
                    storage_env,
                    space_id: 0,
                    session_id: 0,
                    plan_id: 0,
                    v_id_len: 0,
                    is_int_id: false,
                    is_edge: false,
                    default_edge_ver: 0,
                    is_killed: false,
                });
                UnifiedContext::Runtime(RuntimeContext::new(
                    "test_runtime".to_string(),
                    plan_context,
                ))
            }
            ContextType::Validation => {
                UnifiedContext::Validation(ValidationContext::new("test_validation".to_string()))
            }
            ContextType::Storage => {
                UnifiedContext::Storage(StorageContext::new("test_storage".to_string(), 0, 0))
            }
        }
    }
}

impl ContextBase for UnifiedContext {
    fn id(&self) -> &str {
        self.id()
    }

    fn context_type(&self) -> ContextType {
        self.context_type()
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.created_at()
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at()
    }

    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl MutableContext for UnifiedContext {
    fn touch(&mut self) {
        self.touch();
    }

    fn invalidate(&mut self) {
        self.invalidate();
    }

    fn revalidate(&mut self) -> bool {
        self.revalidate()
    }
}

impl super::base::AttributeSupport for UnifiedContext {
    fn get_attribute(&self, key: &str) -> Option<Value> {
        self.get_attribute(key)
    }

    fn set_attribute(&mut self, key: String, value: Value) {
        self.set_attribute(key, value);
    }

    fn attribute_keys(&self) -> Vec<String> {
        self.attribute_keys()
    }

    fn remove_attribute(&mut self, key: &str) -> Option<crate::core::Value> {
        self.get_remove_attribute(key)
    }

    fn clear_attributes(&mut self) {
        self.get_clear_attributes();
    }
}

impl super::base::HierarchicalContext for UnifiedContext {
    fn parent_id(&self) -> Option<&str> {
        self.parent_id()
    }

    fn depth(&self) -> usize {
        self.depth()
    }
}
