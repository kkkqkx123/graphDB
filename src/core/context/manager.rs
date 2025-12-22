//! 上下文管理器模块
//!
//! 提供统一的上下文生命周期管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::base::{
    ContextBase, ContextConfig, ContextEvent, ContextEventListener, ContextManager,
    ContextStatistics, ContextType, SimpleEventListener,
};
use super::enum_context::UnifiedContext;
use crate::core::Value;

/// 事件监听器类型别名
pub type EventListenerType = SimpleEventListener;

/// 默认上下文管理器实现
#[derive(Debug)]
pub struct DefaultContextManager {
    /// 上下文存储
    contexts: Arc<RwLock<HashMap<String, UnifiedContext>>>,

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
            contexts: Arc::new(RwLock::new(HashMap::new())),
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

    /// 检查上下文是否过期
    fn is_context_expired(&self, context: &UnifiedContext) -> bool {
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

        let mut expired_ids = Vec::new();

        for (id, context) in contexts.iter() {
            if self.is_context_expired(context.as_ref()) {
                expired_ids.push(id.clone());
            }
        }

        for id in expired_ids {
            if let Some(context) = contexts.remove(&id) {
                // 更新统计信息
                if let Ok(mut stats) = self.statistics.write() {
                    let lifetime_ms = context
                        .created_at()
                        .elapsed()
                        .unwrap_or_default()
                        .as_millis() as u64;
                    stats.record_destroyed(context.context_type(), lifetime_ms);
                }

                // 触发销毁事件
                self.emit_event(ContextEvent::Destroyed {
                    id: id.clone(),
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
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

impl ContextManager for DefaultContextManager {
    fn create_context(&mut self, context_type: ContextType) -> UnifiedContext {
        // 检查是否超过最大活跃上下文数量
        if self.is_max_contexts_exceeded() {
            // 如果启用自动清理，先尝试清理过期上下文
            if self.config.enable_auto_cleanup {
                self.cleanup_expired_contexts_internal();
            }

            // 如果仍然超过限制，返回一个无效的上下文
            if self.is_max_contexts_exceeded() {
                // 创建一个无效的上下文
                let id = self.generate_context_id(context_type);
                return Box::new(super::query::QueryContext::new(
                    id,
                    crate::core::types::query::QueryType::DataQuery,
                    "".to_string(),
                    super::session::SessionInfo::new(
                        "overflow".to_string(),
                        "system".to_string(),
                        vec!["system".to_string()],
                    ),
                ));
            }
        }

        let id = self.generate_context_id(context_type.clone());
        let context = match context_type {
            ContextType::Session => UnifiedContext::Session(super::session::SessionContext::new(
                id.clone(),
                super::session::UserInfo::new(
                    "default_user".to_string(),
                    "default_user_id".to_string(),
                    vec!["user".to_string()],
                    vec!["read".to_string()],
                ),
                super::session::SessionConfig::default(),
            )),
            ContextType::Query => UnifiedContext::Query(super::query::QueryContext::new(
                id.clone(),
                crate::core::types::query::QueryType::DataQuery,
                "SELECT 1".to_string(),
                super::session::SessionInfo::new(
                    "default_session".to_string(),
                    "default_user".to_string(),
                    vec!["user".to_string()],
                ),
            )),
            ContextType::Execution => UnifiedContext::Execution(super::execution::ExecutionContext::new(
                super::query::QueryContext::new(
                    "default_query".to_string(),
                    crate::core::types::query::QueryType::DataQuery,
                    "SELECT 1".to_string(),
                    super::session::SessionInfo::new(
                        "default_session".to_string(),
                        "default_user".to_string(),
                        vec!["user".to_string()],
                    ),
                ),
            )),
            ContextType::Expression => {
                UnifiedContext::Expression(super::expression::BasicExpressionContext::new())
            }
            ContextType::Request => UnifiedContext::Request(super::request::RequestContext::with_session(
                id.clone(),
                "SELECT 1".to_string(),
                "default_session",
                "default_user",
                "localhost",
                0,
            )),
            ContextType::Runtime => {
                // 运行时上下文需要计划上下文，这里创建一个默认的
                let storage_env = Arc::new(super::runtime::StorageEnv {
                    storage_engine: Arc::new(MockStorageEngine),
                    schema_manager: Arc::new(MockSchemaManager),
                    index_manager: Arc::new(MockIndexManager),
                });
                let plan_context = Arc::new(super::runtime::PlanContext {
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
                UnifiedContext::Runtime(super::runtime::RuntimeContext::new(
                    id.clone(),
                    plan_context,
                ))
            }
            ContextType::Validation => {
                UnifiedContext::Validation(super::validation::ValidationContext::new(id.clone()))
            }
            ContextType::Storage => UnifiedContext::Storage(super::storage::StorageContext::new(id.clone(), 0, 0)),
        };

        // 更新统计信息
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_created(context_type.clone());
            stats.update_max_depth(context.depth());
        }

        // 存储上下文
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.insert(id.clone(), context.clone_context());
        }

        // 触发创建事件
        self.emit_event(ContextEvent::Created {
            id,
            context_type,
            timestamp: std::time::SystemTime::now(),
        });

        context
    }

    fn get_context(&self, id: &str) -> Option<&UnifiedContext> {
        // 注意：这个实现有生命周期限制，实际返回需要从缓存的引用获取
        // 由于RwLock的限制，这个trait需要调整或使用内部缓存
        let contexts = self.contexts.read().ok()?;
        if contexts.contains_key(id) {
            // 无法安全地返回引用，因为guard会被drop
            // 此处是设计问题，应该在trait定义中修改
            None // 暂时返回None作为解决方案
        } else {
            None
        }
    }

    fn get_context_mut(&mut self, id: &str) -> Option<&mut UnifiedContext> {
        let mut contexts = self.contexts.write().ok()?;
        // 由于RwLock的限制，无法返回引用，返回克隆的Box
        contexts.get_mut(id)
    }

    fn remove_context(&mut self, id: &str) -> Option<UnifiedContext> {
        let mut contexts = self.contexts.write().ok()?;
        if let Some(context) = contexts.remove(id) {
            // 更新统计信息
            if let Ok(mut stats) = self.statistics.write() {
                let lifetime_ms = context
                    .created_at()
                    .elapsed()
                    .unwrap_or_default()
                    .as_millis() as u64;
                stats.record_destroyed(context.context_type(), lifetime_ms);
            }

            // 触发销毁事件
            self.emit_event(ContextEvent::Destroyed {
                id: id.to_string(),
                timestamp: std::time::SystemTime::now(),
            });

            Some(context)
        } else {
            None
        }
    }

    fn cleanup_expired_contexts(&mut self) {
        if self.config.enable_auto_cleanup {
            self.cleanup_expired_contexts_internal();
        }
    }

    fn context_ids(&self) -> Vec<String> {
        if let Ok(contexts) = self.contexts.read() {
            contexts.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn context_count(&self) -> usize {
        if let Ok(contexts) = self.contexts.read() {
            contexts.len()
        } else {
            0
        }
    }
}


// Mock实现，用于RuntimeContext的创建
#[derive(Debug)]
pub(crate) struct MockStorageEngine;

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

#[derive(Debug)]
pub(crate) struct MockSchemaManager;

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

#[derive(Debug)]
pub(crate) struct MockIndexManager;

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

