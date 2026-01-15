//! 存储层运行时上下文
//!
//! RuntimeContext用于存储层执行节点，包含计划上下文引用和运行时可变信息
//! 对应C++版本中的RuntimeContext结构

use crate::common::base::id::{EdgeType, TagId};
use crate::core::error::ManagerResult;
use crate::core::Value;
use crate::core::Direction;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::query::context::managers::SchemaManager;

/// 结果状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    /// 正常结果
    Normal = 0,
    /// 非法数据
    IllegalData = -1,
    /// 被过滤掉的结果
    FilterOut = -2,
    /// 标签被过滤掉
    TagFilterOut = -3,
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// 初始化
    Initialized,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
    /// 已暂停
    Paused,
}

/// 执行统计信息
#[derive(Debug, Clone)]
pub struct ExecutionStatistics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub skipped_operations: u64,
    pub total_execution_time_ms: u64,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: u64,
    pub min_execution_time_ms: u64,
    pub memory_usage_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// 执行日志
#[derive(Debug, Clone)]
pub struct ExecutionLog {
    pub timestamp: i64,
    pub operation: String,
    pub state: ExecutionState,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used_bytes: u64,
    pub memory_peak_bytes: u64,
    pub cpu_time_ms: u64,
    pub io_operations: u64,
    pub network_bytes: u64,
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: Value,
    pub hit_count: u64,
    pub last_accessed: i64,
    pub size_bytes: usize,
}

/// 属性上下文（简化版本）
#[derive(Debug, Clone)]
pub struct PropContext {
    pub name: String,
    pub prop_type: String,
    pub nullable: bool,
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv>,
    /// 空间ID
    pub space_id: i32,
    /// 会话ID
    pub session_id: i64,
    /// 计划ID
    pub plan_id: i64,
    /// 顶点ID长度
    pub v_id_len: usize,
    /// 是否为整数ID
    pub is_int_id: bool,
    /// 是否为边查询
    pub is_edge: bool,
    /// 默认边版本
    pub default_edge_ver: i64,
    /// 是否被终止
    pub is_killed: bool,
}

/// 存储环境（简化版本）
#[derive(Debug, Clone)]
pub struct StorageEnv {
    /// 存储引擎
    pub storage_engine: Arc<dyn StorageEngine>,
    /// Schema管理器
    pub schema_manager: Arc<dyn SchemaManager>,
    /// 索引管理器
    pub index_manager: Arc<dyn IndexManager>,
}

/// 存储引擎trait
pub trait StorageEngine: Send + Sync + std::fmt::Debug {
    // 基本存储操作
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    /// 全表扫描所有顶点
    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError>;
    /// 按标签扫描顶点
    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError>;

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: Direction,
    ) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
}

/// 存储Schema管理器trait
pub trait StorageSchemaManager: Send + Sync + std::fmt::Debug {
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn get_all_schemas(&self) -> Vec<Schema>;
    fn add_schema(&mut self, name: String, schema: Schema);
    fn remove_schema(&mut self, name: &str) -> bool;
}

/// 索引管理器trait
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn create_index(&mut self, name: String, schema: Schema) -> ManagerResult<()>;
    fn drop_index(&mut self, name: &str) -> ManagerResult<()>;
    fn get_index(&self, name: &str) -> Option<Index>;
}

/// 运行时上下文
/// 存储处理过程中可能变化的信息
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 计划上下文引用
    pub plan_context: Arc<PlanContext>,

    /// 标签ID
    pub tag_id: TagId,
    /// 标签名称
    pub tag_name: String,
    /// 标签Schema（可选）
    pub tag_schema: Option<Arc<dyn SchemaManager>>,

    /// 边类型
    pub edge_type: EdgeType,
    /// 边名称
    pub edge_name: String,
    /// 边Schema（可选）
    pub edge_schema: Option<Arc<dyn SchemaManager>>,

    /// 列索引（用于GetNeighbors）
    pub column_idx: usize,
    /// 属性上下文列表（可选）
    pub props: Option<Vec<PropContext>>,

    /// 是否为插入操作
    pub insert: bool,
    /// 是否过滤无效结果
    pub filter_invalid_result_out: bool,
    /// 结果状态
    pub result_stat: ResultStatus,

    /// 执行状态
    pub execution_state: Arc<RwLock<ExecutionState>>,
    /// 执行开始时间
    pub execution_start_time: Arc<RwLock<SystemTime>>,
    /// 执行结束时间
    pub execution_end_time: Arc<RwLock<Option<SystemTime>>>,

    /// 执行统计信息
    pub statistics: Arc<RwLock<ExecutionStatistics>>,
    /// 执行日志
    pub logs: Arc<RwLock<Vec<ExecutionLog>>>,

    /// 资源使用情况
    pub resource_usage: Arc<RwLock<ResourceUsage>>,
    /// 缓存
    pub cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// 缓存大小限制（字节）
    pub cache_size_limit: Arc<RwLock<usize>>,

    /// 是否暂停
    pub paused: Arc<AtomicBool>,
    /// 是否终止
    pub terminated: Arc<AtomicBool>,
    /// 错误信息
    pub error: Arc<RwLock<Option<String>>>,
}

impl RuntimeContext {
    /// 创建新的运行时上下文
    pub fn new(plan_context: Arc<PlanContext>) -> Self {
        let now = SystemTime::now();
        
        Self {
            plan_context,
            tag_id: TagId::new(0),
            tag_name: String::new(),
            tag_schema: None,
            edge_type: EdgeType::new(0),
            edge_name: String::new(),
            edge_schema: None,
            column_idx: 0,
            props: None,
            insert: false,
            filter_invalid_result_out: false,
            result_stat: ResultStatus::Normal,
            execution_state: Arc::new(RwLock::new(ExecutionState::Initialized)),
            execution_start_time: Arc::new(RwLock::new(now)),
            execution_end_time: Arc::new(RwLock::new(None)),
            statistics: Arc::new(RwLock::new(ExecutionStatistics {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                skipped_operations: 0,
                total_execution_time_ms: 0,
                avg_execution_time_ms: 0.0,
                max_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
                memory_usage_bytes: 0,
                cache_hits: 0,
                cache_misses: 0,
            })),
            logs: Arc::new(RwLock::new(Vec::new())),
            resource_usage: Arc::new(RwLock::new(ResourceUsage {
                memory_used_bytes: 0,
                memory_peak_bytes: 0,
                cpu_time_ms: 0,
                io_operations: 0,
                network_bytes: 0,
            })),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_size_limit: Arc::new(RwLock::new(100 * 1024 * 1024)), // 默认100MB
            paused: Arc::new(AtomicBool::new(false)),
            terminated: Arc::new(AtomicBool::new(false)),
            error: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv> {
        &self.plan_context.storage_env
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.plan_context.space_id
    }

    /// 获取顶点ID长度
    pub fn v_id_len(&self) -> usize {
        self.plan_context.v_id_len
    }

    /// 检查是否为整数ID
    pub fn is_int_id(&self) -> bool {
        self.plan_context.is_int_id
    }

    /// 检查是否为边查询
    pub fn is_edge(&self) -> bool {
        self.plan_context.is_edge
    }

    /// 检查计划是否被终止
    pub fn is_plan_killed(&self) -> bool {
        self.plan_context.is_killed
    }

    /// 设置标签信息
    pub fn set_tag_info(
        &mut self,
        tag_id: TagId,
        tag_name: String,
        tag_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.tag_id = tag_id;
        self.tag_name = tag_name;
        self.tag_schema = tag_schema;
    }

    /// 设置边信息
    pub fn set_edge_info(
        &mut self,
        edge_type: EdgeType,
        edge_name: String,
        edge_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.edge_type = edge_type;
        self.edge_name = edge_name;
        self.edge_schema = edge_schema;
    }

    /// 设置属性上下文
    pub fn set_props(&mut self, props: Vec<PropContext>) {
        self.props = Some(props);
    }

    /// 设置属性上下文（引用版本）
    pub fn set_props_ref(&mut self, props: &[PropContext]) {
        self.props = Some(props.to_vec());
    }

    /// 设置插入标志
    pub fn set_insert(&mut self, insert: bool) {
        self.insert = insert;
    }

    /// 设置过滤标志
    pub fn set_filter_invalid_result_out(&mut self, filter: bool) {
        self.filter_invalid_result_out = filter;
    }

    /// 设置结果状态
    pub fn set_result_stat(&mut self, stat: ResultStatus) {
        self.result_stat = stat;
    }

    /// 重置运行时状态（保留计划上下文）
    pub fn reset(&mut self) {
        self.tag_id = TagId::new(0);
        self.tag_name.clear();
        self.tag_schema = None;
        self.edge_type = EdgeType::new(0);
        self.edge_name.clear();
        self.edge_schema = None;
        self.column_idx = 0;
        self.props = None;
        self.insert = false;
        self.filter_invalid_result_out = false;
        self.result_stat = ResultStatus::Normal;
        
        if let Ok(mut state) = self.execution_state.write() {
            *state = ExecutionState::Initialized;
        }
        
        if let Ok(mut start_time) = self.execution_start_time.write() {
            *start_time = SystemTime::now();
        }
        
        if let Ok(mut end_time) = self.execution_end_time.write() {
            *end_time = None;
        }
        
        self.paused.store(false, Ordering::SeqCst);
        self.terminated.store(false, Ordering::SeqCst);
        
        if let Ok(mut error) = self.error.write() {
            *error = None;
        }
    }

    // ==================== 执行状态管理 ====================

    /// 获取执行状态
    pub fn get_execution_state(&self) -> Result<ExecutionState, String> {
        let state = self
            .execution_state
            .read()
            .map_err(|e| format!("Failed to acquire read lock on execution_state: {}", e))?;
        Ok(*state)
    }

    /// 设置执行状态
    pub fn set_execution_state(&self, state: ExecutionState) -> Result<(), String> {
        let mut current_state = self
            .execution_state
            .write()
            .map_err(|e| format!("Failed to acquire write lock on execution_state: {}", e))?;
        *current_state = state;
        Ok(())
    }

    /// 开始执行
    pub fn start_execution(&self) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Executing)?;
        self.log_operation("开始执行".to_string(), None, None)?;
        Ok(())
    }

    /// 完成执行
    pub fn complete_execution(&self) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Completed)?;
        
        let mut end_time = self
            .execution_end_time
            .write()
            .map_err(|e| format!("Failed to acquire write lock on execution_end_time: {}", e))?;
        *end_time = Some(SystemTime::now());
        
        self.log_operation("执行完成".to_string(), None, None)?;
        Ok(())
    }

    /// 失败执行
    pub fn fail_execution(&self, error: String) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Failed)?;
        
        let mut error_ref = self
            .error
            .write()
            .map_err(|e| format!("Failed to acquire write lock on error: {}", e))?;
        *error_ref = Some(error.clone());
        
        self.log_operation("执行失败".to_string(), None, Some(error))?;
        Ok(())
    }

    /// 取消执行
    pub fn cancel_execution(&self) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Cancelled)?;
        self.terminated.store(true, Ordering::SeqCst);
        self.log_operation("执行取消".to_string(), None, None)?;
        Ok(())
    }

    /// 暂停执行
    pub fn pause_execution(&self) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Paused)?;
        self.paused.store(true, Ordering::SeqCst);
        self.log_operation("执行暂停".to_string(), None, None)?;
        Ok(())
    }

    /// 恢复执行
    pub fn resume_execution(&self) -> Result<(), String> {
        self.set_execution_state(ExecutionState::Executing)?;
        self.paused.store(false, Ordering::SeqCst);
        self.log_operation("执行恢复".to_string(), None, None)?;
        Ok(())
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// 检查是否终止
    pub fn is_terminated(&self) -> bool {
        self.terminated.load(Ordering::SeqCst)
    }

    /// 获取错误信息
    pub fn get_error(&self) -> Result<Option<String>, String> {
        let error = self
            .error
            .read()
            .map_err(|e| format!("Failed to acquire read lock on error: {}", e))?;
        Ok(error.clone())
    }

    // ==================== 执行统计管理 ====================

    /// 获取执行统计信息
    pub fn get_statistics(&self) -> Result<ExecutionStatistics, String> {
        let stats = self
            .statistics
            .read()
            .map_err(|e| format!("Failed to acquire read lock on statistics: {}", e))?;
        Ok(stats.clone())
    }

    /// 更新执行统计信息
    pub fn update_statistics(&self, success: bool, duration_ms: u64) -> Result<(), String> {
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        
        stats.total_operations += 1;
        stats.total_execution_time_ms += duration_ms;
        
        if success {
            stats.successful_operations += 1;
        } else {
            stats.failed_operations += 1;
        }
        
        if duration_ms > stats.max_execution_time_ms {
            stats.max_execution_time_ms = duration_ms;
        }
        
        if duration_ms < stats.min_execution_time_ms {
            stats.min_execution_time_ms = duration_ms;
        }
        
        if stats.total_operations > 0 {
            stats.avg_execution_time_ms = stats.total_execution_time_ms as f64 / stats.total_operations as f64;
        }
        
        Ok(())
    }

    /// 增加跳过的操作计数
    pub fn increment_skipped_operations(&self) -> Result<(), String> {
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        stats.skipped_operations += 1;
        Ok(())
    }

    /// 重置统计信息
    pub fn reset_statistics(&self) -> Result<(), String> {
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        
        stats.total_operations = 0;
        stats.successful_operations = 0;
        stats.failed_operations = 0;
        stats.skipped_operations = 0;
        stats.total_execution_time_ms = 0;
        stats.avg_execution_time_ms = 0.0;
        stats.max_execution_time_ms = 0;
        stats.min_execution_time_ms = u64::MAX;
        
        Ok(())
    }

    // ==================== 执行日志管理 ====================

    /// 记录操作日志
    pub fn log_operation(&self, operation: String, message: Option<String>, error: Option<String>) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let state = self.get_execution_state()?;
        let duration_ms = self.get_execution_duration_ms();
        
        let log_entry = ExecutionLog {
            timestamp: now,
            operation,
            state,
            duration_ms,
            message,
            error,
        };
        
        let mut logs = self
            .logs
            .write()
            .map_err(|e| format!("Failed to acquire write lock on logs: {}", e))?;
        logs.push(log_entry);
        
        Ok(())
    }

    /// 获取所有日志
    pub fn get_logs(&self) -> Result<Vec<ExecutionLog>, String> {
        let logs = self
            .logs
            .read()
            .map_err(|e| format!("Failed to acquire read lock on logs: {}", e))?;
        Ok(logs.clone())
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

    /// 获取执行持续时间（毫秒）
    pub fn get_execution_duration_ms(&self) -> u64 {
        let start_time = self.execution_start_time.read().ok();
        let end_time = self.execution_end_time.read().ok();
        
        let end = end_time.and_then(|t| *t).unwrap_or_else(SystemTime::now);
        let start = start_time.map(|t| *t).unwrap_or_else(SystemTime::now);
        
        end.duration_since(start)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    // ==================== 缓存管理 ====================

    /// 获取缓存值
    pub fn get_cache(&self, key: &str) -> Result<Option<Value>, String> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| format!("Failed to acquire write lock on cache: {}", e))?;
        
        let mut stats = self
            .statistics
            .write()
            .map_err(|e| format!("Failed to acquire write lock on statistics: {}", e))?;
        
        if let Some(entry) = cache.get_mut(key) {
            entry.hit_count += 1;
            entry.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            stats.cache_hits += 1;
            Ok(Some(entry.value.clone()))
        } else {
            stats.cache_misses += 1;
            Ok(None)
        }
    }

    /// 设置缓存值
    pub fn set_cache(&self, key: String, value: Value) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let size_bytes = std::mem::size_of_val(&value);
        
        let mut cache = self
            .cache
            .write()
            .map_err(|e| format!("Failed to acquire write lock on cache: {}", e))?;
        
        let cache_size_limit = self
            .cache_size_limit
            .read()
            .map_err(|e| format!("Failed to acquire read lock on cache_size_limit: {}", e))?;
        
        let current_size: usize = cache.values().map(|e| e.size_bytes).sum();
        
        if current_size + size_bytes > *cache_size_limit {
            self.evict_cache_entries(&mut *cache, *cache_size_limit, size_bytes)?;
        }
        
        cache.insert(key.clone(), CacheEntry {
            key,
            value,
            hit_count: 0,
            last_accessed: now,
            size_bytes,
        });
        
        Ok(())
    }

    /// 删除缓存值
    pub fn remove_cache(&self, key: &str) -> Result<Option<Value>, String> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| format!("Failed to acquire write lock on cache: {}", e))?;
        Ok(cache.remove(key).map(|e| e.value))
    }

    /// 清空缓存
    pub fn clear_cache(&self) -> Result<(), String> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| format!("Failed to acquire write lock on cache: {}", e))?;
        cache.clear();
        Ok(())
    }

    /// 设置缓存大小限制
    pub fn set_cache_size_limit(&self, limit: usize) -> Result<(), String> {
        let mut cache_size_limit = self
            .cache_size_limit
            .write()
            .map_err(|e| format!("Failed to acquire write lock on cache_size_limit: {}", e))?;
        *cache_size_limit = limit;
        Ok(())
    }

    /// 获取缓存大小
    pub fn get_cache_size(&self) -> Result<usize, String> {
        let cache = self
            .cache
            .read()
            .map_err(|e| format!("Failed to acquire read lock on cache: {}", e))?;
        Ok(cache.values().map(|e| e.size_bytes).sum())
    }

    /// 淘汰缓存条目
    fn evict_cache_entries(&self, cache: &mut std::collections::HashMap<String, CacheEntry>, limit: usize, new_size: usize) -> Result<(), String> {
        let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by(|a, b| {
            let a_score = a.1.hit_count as f64 / (a.1.last_accessed as f64 + 1.0);
            let b_score = b.1.hit_count as f64 / (b.1.last_accessed as f64 + 1.0);
            a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let mut current_size: usize = cache.values().map(|e| e.size_bytes).sum();
        
        for (key, _) in entries {
            if current_size + new_size <= limit {
                break;
            }
            
            if let Some(entry) = cache.remove(&key) {
                current_size -= entry.size_bytes;
            }
        }
        
        Ok(())
    }

    // ==================== 资源管理 ====================

    /// 获取资源使用情况
    pub fn get_resource_usage(&self) -> Result<ResourceUsage, String> {
        let usage = self
            .resource_usage
            .read()
            .map_err(|e| format!("Failed to acquire read lock on resource_usage: {}", e))?;
        Ok(usage.clone())
    }

    /// 更新内存使用量
    pub fn update_memory_usage(&self, bytes: u64) -> Result<(), String> {
        let mut usage = self
            .resource_usage
            .write()
            .map_err(|e| format!("Failed to acquire write lock on resource_usage: {}", e))?;
        
        usage.memory_used_bytes = bytes;
        
        if bytes > usage.memory_peak_bytes {
            usage.memory_peak_bytes = bytes;
        }
        
        Ok(())
    }

    /// 增加IO操作计数
    pub fn increment_io_operations(&self) -> Result<(), String> {
        let mut usage = self
            .resource_usage
            .write()
            .map_err(|e| format!("Failed to acquire write lock on resource_usage: {}", e))?;
        usage.io_operations += 1;
        Ok(())
    }

    /// 增加网络字节数
    pub fn add_network_bytes(&self, bytes: u64) -> Result<(), String> {
        let mut usage = self
            .resource_usage
            .write()
            .map_err(|e| format!("Failed to acquire write lock on resource_usage: {}", e))?;
        usage.network_bytes += bytes;
        Ok(())
    }
}

// 类型别名和简化定义
pub type StorageError = String;
pub type IndexError = String;
pub type Schema = crate::core::schema::Schema;
pub type Index = crate::core::schema::Schema;
pub type Vertex = crate::core::vertex_edge_path::Vertex;
pub type Edge = crate::core::vertex_edge_path::Edge;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct MockStorageEngine;

    impl StorageEngine for MockStorageEngine {
        fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, StorageError> {
            Ok(Value::Int(1))
        }

        fn get_node(&self, _id: &Value) -> Result<Option<Vertex>, StorageError> {
            Ok(None)
        }

        fn update_node(&mut self, _vertex: Vertex) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_node(&mut self, _id: &Value) -> Result<(), StorageError> {
            Ok(())
        }

        fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(&self, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_edge(&mut self, _edge: Edge) -> Result<(), StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<Option<Edge>, StorageError> {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &Value,
            _direction: Direction,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockSchemaManager {
        schemas: HashMap<String, crate::query::context::managers::Schema>,
    }

    impl MockSchemaManager {
        fn new() -> Self {
            Self {
                schemas: HashMap::new(),
            }
        }
    }

    impl SchemaManager for MockSchemaManager {
        fn get_schema(&self, name: &str) -> Option<crate::query::context::managers::Schema> {
            self.schemas.get(name).cloned()
        }

        fn list_schemas(&self) -> Vec<String> {
            self.schemas.keys().cloned().collect()
        }

        fn has_schema(&self, name: &str) -> bool {
            self.schemas.contains_key(name)
        }

        fn create_tag(
            &self,
            _space_id: i32,
            _tag_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_tag(&self, _space_id: i32, _tag_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_tag(
            &self,
            _space_id: i32,
            _tag_id: i32,
        ) -> Option<crate::query::context::managers::TagDef> {
            None
        }

        fn list_tags(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::TagDef>> {
            Ok(Vec::new())
        }

        fn has_tag(&self, _space_id: i32, _tag_id: i32) -> bool {
            false
        }

        fn create_edge_type(
            &self,
            _space_id: i32,
            _edge_type_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_edge_type(
            &self,
            _space_id: i32,
            _edge_type_id: i32,
        ) -> Option<crate::query::context::managers::EdgeTypeDef> {
            None
        }

        fn list_edge_types(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::EdgeTypeDef>> {
            Ok(Vec::new())
        }

        fn has_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> bool {
            false
        }

        fn load_from_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn save_to_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn create_schema_version(
            &self,
            _space_id: i32,
            _comment: Option<String>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn get_schema_version(
            &self,
            _space_id: i32,
            _version: i32,
        ) -> Option<crate::query::context::managers::SchemaVersion> {
            None
        }

        fn get_latest_schema_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }

        fn get_schema_history(
            &self,
            _space_id: i32,
        ) -> ManagerResult<crate::query::context::managers::SchemaHistory> {
            Ok(crate::query::context::managers::SchemaHistory {
                space_id: _space_id,
                versions: Vec::new(),
                current_version: 1,
            })
        }

        fn rollback_schema(&self, _space_id: i32, _version: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_current_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }
    }

    #[derive(Debug)]
    struct MockIndexManager;

    impl IndexManager for MockIndexManager {
        fn create_index(&mut self, _name: String, _schema: Schema) -> ManagerResult<()> {
            Ok(())
        }

        fn drop_index(&mut self, _name: &str) -> ManagerResult<()> {
            Ok(())
        }

        fn get_index(&self, _name: &str) -> Option<Index> {
            None
        }
    }

    #[test]
    fn test_runtime_context_creation() {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(MockStorageEngine),
            schema_manager: Arc::new(MockSchemaManager::new()),
            index_manager: Arc::new(MockIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            session_id: 100,
            plan_id: 200,
            v_id_len: 16,
            is_int_id: false,
            is_edge: false,
            default_edge_ver: 0,
            is_killed: false,
        });

        let runtime_ctx = RuntimeContext::new(plan_context);

        assert_eq!(runtime_ctx.space_id(), 1);
        assert_eq!(runtime_ctx.v_id_len(), 16);
        assert!(!runtime_ctx.is_int_id());
        assert!(!runtime_ctx.is_edge());
        assert!(!runtime_ctx.is_plan_killed());
        assert_eq!(runtime_ctx.result_stat, ResultStatus::Normal);
    }

    #[test]
    fn test_runtime_context_setters() {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(MockStorageEngine),
            schema_manager: Arc::new(MockSchemaManager::new()),
            index_manager: Arc::new(MockIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            session_id: 100,
            plan_id: 200,
            v_id_len: 16,
            is_int_id: false,
            is_edge: false,
            default_edge_ver: 0,
            is_killed: false,
        });

        let mut runtime_ctx = RuntimeContext::new(plan_context);

        // 设置标签信息
        runtime_ctx.set_tag_info(TagId::new(1), "player".to_string(), None);
        assert_eq!(runtime_ctx.tag_id.as_i32(), 1);
        assert_eq!(runtime_ctx.tag_name, "player");

        // 设置边信息
        runtime_ctx.set_edge_info(EdgeType::new(2), "follow".to_string(), None);
        assert_eq!(runtime_ctx.edge_type.as_i32(), 2);
        assert_eq!(runtime_ctx.edge_name, "follow");

        // 设置属性
        let props = vec![PropContext {
            name: "name".to_string(),
            prop_type: "string".to_string(),
            nullable: false,
        }];
        runtime_ctx.set_props_ref(&props);
        assert_eq!(
            runtime_ctx
                .props
                .as_ref()
                .expect("Props should exist")
                .len(),
            1
        );

        // 设置标志
        runtime_ctx.set_insert(true);
        assert!(runtime_ctx.insert);

        runtime_ctx.set_filter_invalid_result_out(true);
        assert!(runtime_ctx.filter_invalid_result_out);

        runtime_ctx.set_result_stat(ResultStatus::FilterOut);
        assert_eq!(runtime_ctx.result_stat, ResultStatus::FilterOut);

        // 重置
        runtime_ctx.reset();
        assert_eq!(runtime_ctx.tag_id.as_i32(), 0);
        assert!(runtime_ctx.tag_name.is_empty());
        assert_eq!(runtime_ctx.result_stat, ResultStatus::Normal);
    }
}
