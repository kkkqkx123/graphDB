//! 上下文管理器模块
//!
//! 提供类型安全的上下文生命周期管理，避免过度抽象

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::base::{
    ContextConfig, ContextEvent, ContextEventListener, ContextManager, ContextStatistics,
    ContextType, SimpleEventListener,
};
use super::execution::ExecutionContext;
use super::query::QueryContext;
use super::request::RequestContext;
use super::runtime::{PlanContext, TestRuntimeContext};
use super::session::SessionContext;
use super::storage::StorageContext;
use super::traits::BaseContext;
use super::validation::ValidationContext;
use crate::core::Value;
use crate::expression::BasicExpressionContext;

/// 事件监听器类型别名
pub type EventListenerType = SimpleEventListener;

/// 上下文存储 - 使用类型安全的HashMap而非枚举
#[derive(Debug)]
pub struct ContextStorage {
    session_contexts: HashMap<String, SessionContext>,
    query_contexts: HashMap<String, QueryContext>,
    execution_contexts: HashMap<String, ExecutionContext>,
    expression_contexts: HashMap<String, BasicExpressionContext>,
    request_contexts: HashMap<String, RequestContext>,
    runtime_contexts: HashMap<String, TestRuntimeContext>,
    validation_contexts: HashMap<String, ValidationContext>,
    storage_contexts: HashMap<String, StorageContext>,
}

impl ContextStorage {
    pub fn new() -> Self {
        Self {
            session_contexts: HashMap::new(),
            query_contexts: HashMap::new(),
            execution_contexts: HashMap::new(),
            expression_contexts: HashMap::new(),
            request_contexts: HashMap::new(),
            runtime_contexts: HashMap::new(),
            validation_contexts: HashMap::new(),
            storage_contexts: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.session_contexts.clear();
        self.query_contexts.clear();
        self.execution_contexts.clear();
        self.expression_contexts.clear();
        self.request_contexts.clear();
        self.runtime_contexts.clear();
        self.validation_contexts.clear();
        self.storage_contexts.clear();
    }

    pub fn len(&self) -> usize {
        self.session_contexts.len()
            + self.query_contexts.len()
            + self.execution_contexts.len()
            + self.expression_contexts.len()
            + self.request_contexts.len()
            + self.runtime_contexts.len()
            + self.validation_contexts.len()
            + self.storage_contexts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 默认上下文管理器实现
#[derive(Debug)]
pub struct DefaultContextManager {
    /// 上下文存储
    contexts: Arc<RwLock<ContextStorage>>,

    /// 配置
    config: ContextConfig,

    /// 统计信息
    statistics: Arc<RwLock<ContextStatistics>>,

    /// 事件监听器列表
    event_listeners: Arc<RwLock<Vec<EventListenerType>>>,

    /// 创建时间
    created_at: std::time::SystemTime,
}

impl DefaultContextManager {
    /// 创建新的上下文管理器
    pub fn new() -> Self {
        Self::with_config(ContextConfig::default())
    }

    /// 使用配置创建上下文管理器
    pub fn with_config(config: ContextConfig) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(ContextStorage::new())),
            config,
            statistics: Arc::new(RwLock::new(ContextStatistics::new())),
            event_listeners: Arc::new(RwLock::new(Vec::new())),
            created_at: std::time::SystemTime::now(),
        }
    }

    /// 添加事件监听器
    pub fn add_event_listener(&self, listener: EventListenerType) {
        if let Ok(mut listeners) = self.event_listeners.write() {
            listeners.push(listener);
        }
    }

    /// 移除事件监听器
    pub fn remove_event_listeners(&self) {
        if let Ok(mut listeners) = self.event_listeners.write() {
            listeners.clear();
        }
    }

    /// 触发事件
    fn emit_event(&self, event: ContextEvent) {
        if !self.config.enable_event_listening {
            return;
        }

        if let Ok(listeners) = self.event_listeners.read() {
            for listener in listeners.iter() {
                listener.on_event(&event);
            }
        }
    }

    /// 生成唯一上下文ID
    fn generate_context_id(&self, context_type: ContextType) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        match context_type {
            ContextType::Session => format!("session_{}", count),
            ContextType::Query => format!("query_{}", count),
            ContextType::Execution => format!("execution_{}", count),
            ContextType::Expression => format!("expression_{}", count),
            ContextType::Request => format!("request_{}", count),
            ContextType::Runtime => format!("runtime_{}", count),
            ContextType::Validation => format!("validation_{}", count),
            ContextType::Storage => format!("storage_{}", count),
        }
    }

    /// 检查上下文是否过期 - 泛型实现
    fn is_context_expired<T: BaseContext>(&self, context: &T) -> bool {
        if let Some(timeout_ms) = self.config.timeout_ms {
            if let Ok(elapsed) = context.created_at().elapsed() {
                elapsed.as_millis() as u64 > timeout_ms
            } else {
                true // 如果无法计算时间，认为已过期
            }
        } else {
            false
        }
    }

    /// 清理过期上下文
    fn cleanup_expired_contexts_internal(&self) {
        let mut contexts = match self.contexts.write() {
            Ok(ctx) => ctx,
            Err(_) => return,
        };

        // 收集过期的会话上下文
        let mut expired_session_ids = Vec::new();
        for (id, context) in contexts.session_contexts.iter() {
            if self.is_context_expired(context) {
                expired_session_ids.push(id.clone());
            }
        }

        // 收集过期的查询上下文
        let mut expired_query_ids = Vec::new();
        for (id, context) in contexts.query_contexts.iter() {
            if self.is_context_expired(context) {
                expired_query_ids.push(id.clone());
            }
        }

        // 收集过期的执行上下文
        let mut expired_execution_ids = Vec::new();
        for (id, context) in contexts.execution_contexts.iter() {
            if self.is_context_expired(context) {
                expired_execution_ids.push(id.clone());
            }
        }

        // 收集过期的请求上下文
        let mut expired_request_ids = Vec::new();
        for (id, context) in contexts.request_contexts.iter() {
            if self.is_context_expired(context) {
                expired_request_ids.push(id.clone());
            }
        }

        // 收集过期的运行时上下文
        let mut expired_runtime_ids = Vec::new();
        for (id, context) in contexts.runtime_contexts.iter() {
            if self.is_context_expired(context) {
                expired_runtime_ids.push(id.clone());
            }
        }

        // 收集过期的验证上下文
        let mut expired_validation_ids = Vec::new();
        for (id, context) in contexts.validation_contexts.iter() {
            if self.is_context_expired(context) {
                expired_validation_ids.push(id.clone());
            }
        }

        // 收集过期的存储上下文
        let mut expired_storage_ids = Vec::new();
        for (id, context) in contexts.storage_contexts.iter() {
            if self.is_context_expired(context) {
                expired_storage_ids.push(id.clone());
            }
        }

        // 移除过期的上下文并触发事件
        for id in expired_session_ids {
            if let Some(context) = contexts.session_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Session, &context);
            }
        }

        for id in expired_query_ids {
            if let Some(context) = contexts.query_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Query, &context);
            }
        }

        for id in expired_execution_ids {
            if let Some(context) = contexts.execution_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Execution, &context);
            }
        }

        for id in expired_request_ids {
            if let Some(context) = contexts.request_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Request, &context);
            }
        }

        for id in expired_runtime_ids {
            if let Some(context) = contexts.runtime_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Runtime, &context);
            }
        }

        for id in expired_validation_ids {
            if let Some(context) = contexts.validation_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Validation, &context);
            }
        }

        for id in expired_storage_ids {
            if let Some(context) = contexts.storage_contexts.remove(&id) {
                self.emit_context_destroyed_event(&id, ContextType::Storage, &context);
            }
        }
    }

    /// 触发上下文销毁事件
    fn emit_context_destroyed_event<T: BaseContext>(
        &self,
        id: &str,
        context_type: ContextType,
        context: &T,
    ) {
        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            let lifetime_ms = context
                .created_at()
                .elapsed()
                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                .as_millis() as u64;
            stats.record_destroyed(context_type, lifetime_ms);
        }

        // 触发销毁事件
        self.emit_event(ContextEvent::Destroyed {
            id: id.to_string(),
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// 检查是否超过最大活跃上下文数量
    fn is_max_contexts_exceeded(&self) -> bool {
        if let Some(max_contexts) = self.config.max_active_contexts {
            if let Ok(contexts) = self.contexts.read() {
                contexts.len() >= max_contexts
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl DefaultContextManager {
    /// 创建会话上下文
    pub fn create_session_context(
        &mut self,
        user_info: super::session::UserInfo,
        config: super::session::SessionConfig,
    ) -> String {
        let id = self.generate_context_id(ContextType::Session);
        let context = SessionContext::new(id.clone(), user_info, config);

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Session);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.session_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Session,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建查询上下文
    pub fn create_query_context(
        &mut self,
        query: String,
        session_info: super::session::SessionInfo,
    ) -> String {
        let id = self.generate_context_id(ContextType::Query);
        let context = QueryContext::new(
            id.clone(),
            super::query::QueryType::DataQuery,
            query,
            session_info,
        );

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Query);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.query_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Query,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建执行上下文
    pub fn create_execution_context(&mut self, query_context: QueryContext) -> String {
        let id = self.generate_context_id(ContextType::Execution);
        let context = ExecutionContext::new(query_context);

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Execution);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.execution_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Execution,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建表达式上下文
    pub fn create_expression_context(&mut self) -> String {
        let id = self.generate_context_id(ContextType::Expression);
        let context = BasicExpressionContext::new();

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Expression);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.expression_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Expression,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建请求上下文
    pub fn create_request_context(
        &mut self,
        query: String,
        session_id: &str,
        user: &str,
        host: &str,
        port: u16,
    ) -> String {
        let id = self.generate_context_id(ContextType::Request);
        let context = RequestContext::with_session(id.clone(), query, session_id, user, host, port);

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Request);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.request_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Request,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建运行时上下文
    pub fn create_runtime_context(
        &mut self,
        plan_context: Arc<PlanContext<MockStorageEngine, MockSchemaManager, MockIndexManager>>,
    ) -> String {
        let id = self.generate_context_id(ContextType::Runtime);
        let context = TestRuntimeContext::new(id.clone(), plan_context);

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Runtime);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.runtime_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Runtime,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建验证上下文
    pub fn create_validation_context(&mut self) -> String {
        let id = self.generate_context_id(ContextType::Validation);
        let context = ValidationContext::new(id.clone());

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Validation);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.validation_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Validation,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 创建存储上下文
    pub fn create_storage_context(&mut self, space_id: u32, part_id: u32) -> String {
        let id = self.generate_context_id(ContextType::Storage);
        let context = StorageContext::new(id.clone(), space_id as i32, part_id as i64);

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(ContextType::Storage);
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.storage_contexts.insert(id.clone(), context);
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id: id.clone(),
            context_type: ContextType::Storage,
            timestamp: std::time::SystemTime::now(),
        });

        id
    }

    /// 获取会话上下文
    pub fn get_session_context(&self, id: &str) -> Option<SessionContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.session_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取查询上下文
    pub fn get_query_context(&self, id: &str) -> Option<QueryContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.query_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取执行上下文
    pub fn get_execution_context(&self, id: &str) -> Option<ExecutionContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.execution_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取表达式上下文
    pub fn get_expression_context(&self, id: &str) -> Option<BasicExpressionContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.expression_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取请求上下文
    pub fn get_request_context(&self, id: &str) -> Option<RequestContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.request_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取运行时上下文
    pub fn get_runtime_context(&self, id: &str) -> Option<TestRuntimeContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.runtime_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取验证上下文
    pub fn get_validation_context(&self, id: &str) -> Option<ValidationContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.validation_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 获取存储上下文
    pub fn get_storage_context(&self, id: &str) -> Option<StorageContext> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.storage_contexts.get(id).cloned()
        } else {
            None
        }
    }

    /// 移除会话上下文
    pub fn remove_session_context(&mut self, id: &str) -> Option<SessionContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.session_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Session, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除查询上下文
    pub fn remove_query_context(&mut self, id: &str) -> Option<QueryContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.query_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Query, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除执行上下文
    pub fn remove_execution_context(&mut self, id: &str) -> Option<ExecutionContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.execution_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Execution, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除表达式上下文
    pub fn remove_expression_context(&mut self, id: &str) -> Option<BasicExpressionContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.expression_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Expression, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除请求上下文
    pub fn remove_request_context(&mut self, id: &str) -> Option<RequestContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.request_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Request, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除运行时上下文
    pub fn remove_runtime_context(&mut self, id: &str) -> Option<TestRuntimeContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.runtime_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Runtime, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除验证上下文
    pub fn remove_validation_context(&mut self, id: &str) -> Option<ValidationContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.validation_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Validation, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 移除存储上下文
    pub fn remove_storage_context(&mut self, id: &str) -> Option<StorageContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.storage_contexts.remove(id) {
            self.emit_context_destroyed_event(id, ContextType::Storage, &context);
            Some(context)
        } else {
            None
        }
    }

    /// 清理过期上下文
    pub fn cleanup_expired_contexts(&mut self) {
        if self.config.enable_auto_cleanup {
            self.cleanup_expired_contexts_internal();
        }
    }

    /// 获取所有上下文ID
    pub fn context_ids(&self) -> Vec<String> {
        if let Ok(contexts) = self.contexts.read() {
            let mut ids = Vec::new();
            ids.extend(contexts.session_contexts.keys().cloned());
            ids.extend(contexts.query_contexts.keys().cloned());
            ids.extend(contexts.execution_contexts.keys().cloned());
            ids.extend(contexts.expression_contexts.keys().cloned());
            ids.extend(contexts.request_contexts.keys().cloned());
            ids.extend(contexts.runtime_contexts.keys().cloned());
            ids.extend(contexts.validation_contexts.keys().cloned());
            ids.extend(contexts.storage_contexts.keys().cloned());
            ids
        } else {
            Vec::new()
        }
    }

    /// 获取上下文总数
    pub fn context_count(&self) -> usize {
        if let Ok(contexts) = self.contexts.read() {
            contexts.len()
        } else {
            0
        }
    }
}

impl ContextManager for DefaultContextManager {
    /// 清理过期上下文
    fn cleanup_expired_contexts(&mut self) {
        if self.config.enable_auto_cleanup {
            self.cleanup_expired_contexts_internal();
        }
    }

    /// 获取所有上下文ID
    fn context_ids(&self) -> Vec<String> {
        if let Ok(contexts) = self.contexts.read() {
            let mut ids = Vec::new();
            ids.extend(contexts.session_contexts.keys().cloned());
            ids.extend(contexts.query_contexts.keys().cloned());
            ids.extend(contexts.execution_contexts.keys().cloned());
            ids.extend(contexts.expression_contexts.keys().cloned());
            ids.extend(contexts.request_contexts.keys().cloned());
            ids.extend(contexts.runtime_contexts.keys().cloned());
            ids.extend(contexts.validation_contexts.keys().cloned());
            ids.extend(contexts.storage_contexts.keys().cloned());
            ids
        } else {
            Vec::new()
        }
    }

    /// 获取上下文总数
    fn context_count(&self) -> usize {
        if let Ok(contexts) = self.contexts.read() {
            contexts.len()
        } else {
            0
        }
    }
}

// Mock实现，用于RuntimeContext的创建
#[derive(Debug, Clone)]
pub struct MockStorageEngine;

impl super::runtime::StorageEngine for MockStorageEngine {
    fn insert_node(&mut self, _vertex: super::runtime::Vertex) -> Result<Value, String> {
        Ok(Value::Int(1))
    }

    fn get_node(&self, _id: &Value) -> Result<Option<super::runtime::Vertex>, String> {
        Ok(None)
    }

    fn update_node(&mut self, _vertex: super::runtime::Vertex) -> Result<(), String> {
        Ok(())
    }

    fn delete_node(&mut self, _id: &Value) -> Result<(), String> {
        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<super::runtime::Vertex>, String> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(&self, _tag: &str) -> Result<Vec<super::runtime::Vertex>, String> {
        Ok(Vec::new())
    }

    fn insert_edge(&mut self, _edge: super::runtime::Edge) -> Result<(), String> {
        Ok(())
    }

    fn get_edge(
        &self,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<Option<super::runtime::Edge>, String> {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _node_id: &Value,
        _direction: super::runtime::Direction,
    ) -> Result<Vec<super::runtime::Edge>, String> {
        Ok(Vec::new())
    }

    fn delete_edge(&mut self, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MockSchemaManager;

impl super::runtime::SchemaManager for MockSchemaManager {
    fn get_schema(&self, _name: &str) -> Option<super::runtime::Schema> {
        None
    }

    fn get_all_schemas(&self) -> Vec<super::runtime::Schema> {
        Vec::new()
    }

    fn add_schema(&mut self, _name: String, _schema: super::runtime::Schema) {
        // Mock实现
    }

    fn remove_schema(&mut self, _name: &str) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct MockIndexManager;

impl super::runtime::IndexManager for MockIndexManager {
    fn create_index(
        &mut self,
        _name: String,
        _schema: super::runtime::Schema,
    ) -> Result<(), String> {
        Ok(())
    }

    fn drop_index(&mut self, _name: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_index(&self, _name: &str) -> Option<super::runtime::Index> {
        None
    }
}

impl Default for DefaultContextManager {
    fn default() -> Self {
        Self::new()
    }
}
