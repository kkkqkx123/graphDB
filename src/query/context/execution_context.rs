//! 执行上下文 - 执行状态管理
//!
//! 执行上下文，管理执行期间的状态
//! 对应原C++中的ExecutionContext.h/cpp

use crate::core::Value;
use crate::core::result::Result as ResultType;
use crate::core::result::ResultState;
use crate::query::context::QueryContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// 初始化
    Initialized,
    /// 执行中
    Running,
    /// 暂停
    Paused,
    /// 完成
    Completed,
    /// 错误
    Error(String),
    /// 取消
    Cancelled,
}

/// 资源管理器
#[derive(Debug)]
pub struct ResourceManager {
    /// 内存使用量（字节）
    memory_usage: Arc<RwLock<u64>>,
    /// 打开的文件数
    open_files: Arc<RwLock<u32>>,
    /// 网络连接数
    network_connections: Arc<RwLock<u32>>,
}

impl ResourceManager {
    /// 创建新的资源管理器
    pub fn new() -> Self {
        Self {
            memory_usage: Arc::new(RwLock::new(0)),
            open_files: Arc::new(RwLock::new(0)),
            network_connections: Arc::new(RwLock::new(0)),
        }
    }

    /// 获取内存使用量
    pub fn memory_usage(&self) -> u64 {
        *self.memory_usage.read().unwrap()
    }

    /// 增加内存使用量
    pub fn add_memory_usage(&self, bytes: u64) {
        *self.memory_usage.write().unwrap() += bytes;
    }

    /// 减少内存使用量
    pub fn subtract_memory_usage(&self, bytes: u64) {
        let mut usage = self.memory_usage.write().unwrap();
        *usage = usage.saturating_sub(bytes);
    }

    /// 获取打开的文件数
    pub fn open_files(&self) -> u32 {
        *self.open_files.read().unwrap()
    }

    /// 增加打开的文件数
    pub fn add_open_file(&self) {
        *self.open_files.write().unwrap() += 1;
    }

    /// 减少打开的文件数
    pub fn remove_open_file(&self) {
        let mut count = self.open_files.write().unwrap();
        *count = count.saturating_sub(1);
    }

    /// 获取网络连接数
    pub fn network_connections(&self) -> u32 {
        *self.network_connections.read().unwrap()
    }

    /// 增加网络连接数
    pub fn add_network_connection(&self) {
        *self.network_connections.write().unwrap() += 1;
    }

    /// 减少网络连接数
    pub fn remove_network_connection(&self) {
        let mut count = self.network_connections.write().unwrap();
        *count = count.saturating_sub(1);
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行指标
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// 开始时间
    pub start_time: Option<std::time::Instant>,
    /// 结束时间
    pub end_time: Option<std::time::Instant>,
    /// 执行的步骤数
    pub steps_executed: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
}

impl ExecutionMetrics {
    /// 创建新的执行指标
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始计时
    pub fn start_timing(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }

    /// 结束计时
    pub fn end_timing(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }

    /// 获取执行持续时间（毫秒）
    pub fn duration_ms(&self) -> Option<u64> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start).as_millis() as u64),
            _ => None,
        }
    }

    /// 增加执行步骤数
    pub fn add_step(&mut self) {
        self.steps_executed += 1;
    }

    /// 增加缓存命中次数
    pub fn add_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// 增加缓存未命中次数
    pub fn add_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// 执行上下文，管理执行期间的状态
#[derive(Debug)]
pub struct ExecutionContext {
    /// 查询上下文
    pub query_context: Arc<QueryContext>,
    /// 执行状态
    execution_state: Arc<RwLock<ExecutionState>>,
    /// 资源管理器
    pub resource_manager: ResourceManager,
    /// 执行指标
    pub metrics: ExecutionMetrics,
    /// 执行变量（运行时变量）
    variables: Arc<RwLock<HashMap<String, Value>>>,
    /// 执行结果
    results: Arc<RwLock<HashMap<String, Vec<ResultType>>>>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(query_context: Arc<QueryContext>) -> Self {
        Self {
            query_context,
            execution_state: Arc::new(RwLock::new(ExecutionState::Initialized)),
            resource_manager: ResourceManager::new(),
            metrics: ExecutionMetrics::new(),
            variables: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取执行状态
    pub fn get_execution_state(&self) -> ExecutionState {
        self.execution_state.read().unwrap().clone()
    }

    /// 设置执行状态
    pub fn set_execution_state(&self, state: ExecutionState) {
        *self.execution_state.write().unwrap() = state;
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self.get_execution_state(), ExecutionState::Running)
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(self.get_execution_state(), ExecutionState::Completed)
    }

    /// 检查是否有错误
    pub fn has_error(&self) -> bool {
        matches!(self.get_execution_state(), ExecutionState::Error(_))
    }

    /// 获取错误信息
    pub fn get_error(&self) -> Option<String> {
        match self.get_execution_state() {
            ExecutionState::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// 开始执行
    pub fn start(&self) {
        self.set_execution_state(ExecutionState::Running);
    }

    /// 暂停执行
    pub fn pause(&self) {
        if self.is_running() {
            self.set_execution_state(ExecutionState::Paused);
        }
    }

    /// 恢复执行
    pub fn resume(&self) {
        if matches!(self.get_execution_state(), ExecutionState::Paused) {
            self.set_execution_state(ExecutionState::Running);
        }
    }

    /// 完成执行
    pub fn complete(&self) {
        self.set_execution_state(ExecutionState::Completed);
    }

    /// 设置错误状态
    pub fn set_error(&self, error: String) {
        self.set_execution_state(ExecutionState::Error(error));
    }

    /// 取消执行
    pub fn cancel(&self) {
        self.set_execution_state(ExecutionState::Cancelled);
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // 首先检查执行变量
        if let Some(value) = self.variables.read().unwrap().get(name) {
            return Some(value.clone());
        }
        // 然后检查查询上下文中的变量
        self.query_context.get_variable(name).cloned()
    }

    /// 设置变量值
    pub fn set_variable(&self, name: String, value: Value) {
        self.variables.write().unwrap().insert(name, value);
    }

    /// 删除变量
    pub fn remove_variable(&self, name: &str) -> Option<Value> {
        self.variables.write().unwrap().remove(name)
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.read().unwrap().contains_key(name) || self.query_context.has_variable(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        let mut names = self.variables.read().unwrap().keys().cloned().collect::<Vec<_>>();
        names.extend(self.query_context.variable_names());
        names.sort();
        names.dedup();
        names
    }

    /// 获取变量的最新值
    pub fn get_value(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.get_variable(name) {
            Ok(value)
        } else {
            Err(format!("变量 '{}' 不存在", name))
        }
    }

    /// 设置变量的最新值
    pub fn set_value(&self, name: &str, value: Value) -> Result<(), String> {
        self.set_variable(name.to_string(), value);
        Ok(())
    }

    /// 获取变量的最新结果
    pub fn get_result(&self, name: &str) -> std::result::Result<ResultType, String> {
        let results = self.results.read().unwrap();
        if let Some(result_list) = results.get(name) {
            if let Some(result) = result_list.first() {
                Ok(result.clone())
            } else {
                Err(format!("变量 '{}' 没有结果", name))
            }
        } else {
            Err(format!("变量 '{}' 不存在", name))
        }
    }

    /// 设置变量的最新结果
    pub fn set_result(&self, name: &str, result: ResultType) -> std::result::Result<(), String> {
        let mut results = self.results.write().unwrap();
        let result_list = results.entry(name.to_string()).or_insert_with(Vec::new);
        result_list.insert(0, result);
        Ok(())
    }

    /// 获取变量的所有历史结果
    pub fn get_history(&self, name: &str) -> std::result::Result<Vec<ResultType>, String> {
        let results = self.results.read().unwrap();
        if let Some(result_list) = results.get(name) {
            Ok(result_list.clone())
        } else {
            Err(format!("变量 '{}' 不存在", name))
        }
    }

    /// 删除变量的结果
    pub fn drop_result(&self, name: &str) -> std::result::Result<(), String> {
        let mut results = self.results.write().unwrap();
        results.remove(name);
        Ok(())
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.has_variable(name) || self.results.read().unwrap().contains_key(name)
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.variable_names().len()
    }

    /// 清除所有执行变量
    pub fn clear_variables(&self) {
        self.variables.write().unwrap().clear();
    }

    /// 清除所有结果
    pub fn clear_results(&self) {
        self.results.write().unwrap().clear();
    }

    /// 重置执行上下文
    pub fn reset(&mut self) {
        self.set_execution_state(ExecutionState::Initialized);
        self.clear_variables();
        self.clear_results();
        self.metrics = ExecutionMetrics::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::managers::r#impl::{
        MemoryIndexManager, MemoryMetaClient, MemorySchemaManager, MemoryStorageClient,
    };

    fn create_test_context() -> Arc<QueryContext> {
        let schema_manager = Arc::new(MemorySchemaManager::new());
        let index_manager = Arc::new(MemoryIndexManager::new());
        let meta_client = Arc::new(MemoryMetaClient::new());
        let storage_client = Arc::new(MemoryStorageClient::new());

        Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ))
    }

    #[test]
    fn test_execution_context_creation() {
        let query_context = create_test_context();
        let exec_ctx = ExecutionContext::new(query_context);

        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Initialized);
        assert!(!exec_ctx.is_running());
        assert!(!exec_ctx.is_completed());
        assert!(!exec_ctx.has_error());
    }

    #[test]
    fn test_execution_state_management() {
        let query_context = create_test_context();
        let exec_ctx = ExecutionContext::new(query_context);

        // 开始执行
        exec_ctx.start();
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Running);
        assert!(exec_ctx.is_running());

        // 暂停执行
        exec_ctx.pause();
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Paused);
        assert!(!exec_ctx.is_running());

        // 恢复执行
        exec_ctx.resume();
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Running);
        assert!(exec_ctx.is_running());

        // 完成执行
        exec_ctx.complete();
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Completed);
        assert!(exec_ctx.is_completed());

        // 设置错误
        exec_ctx.set_error("测试错误".to_string());
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Error("测试错误".to_string()));
        assert!(exec_ctx.has_error());
        assert_eq!(exec_ctx.get_error(), Some("测试错误".to_string()));
    }

    #[test]
    fn test_variable_management() {
        let query_context = create_test_context();
        let exec_ctx = ExecutionContext::new(query_context);

        // 设置执行变量
        exec_ctx.set_variable("exec_var".to_string(), Value::Int(100));
        assert_eq!(exec_ctx.get_variable("exec_var"), Some(Value::Int(100)));
        assert!(exec_ctx.has_variable("exec_var"));

        // 检查变量名列表
        let names = exec_ctx.variable_names();
        assert!(names.contains(&"exec_var".to_string()));
    }

    #[test]
    fn test_result_management() {
        let query_context = create_test_context();
        let exec_ctx = ExecutionContext::new(query_context);

        // 设置结果
        let result = ResultType::new(Value::Int(42), ResultState::Success);
        exec_ctx.set_result("test_var", result.clone()).unwrap();

        // 获取结果
        let retrieved = exec_ctx.get_result("test_var").unwrap();
        assert_eq!(retrieved, result);

        // 获取历史记录
        let history = exec_ctx.get_history("test_var").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], result);

        // 删除结果
        exec_ctx.drop_result("test_var").unwrap();
        assert!(exec_ctx.get_result("test_var").is_err());
    }

    #[test]
    fn test_resource_manager() {
        let resource_manager = ResourceManager::new();

        // 测试内存管理
        assert_eq!(resource_manager.memory_usage(), 0);
        resource_manager.add_memory_usage(1024);
        assert_eq!(resource_manager.memory_usage(), 1024);
        resource_manager.subtract_memory_usage(512);
        assert_eq!(resource_manager.memory_usage(), 512);

        // 测试文件管理
        assert_eq!(resource_manager.open_files(), 0);
        resource_manager.add_open_file();
        assert_eq!(resource_manager.open_files(), 1);
        resource_manager.remove_open_file();
        assert_eq!(resource_manager.open_files(), 0);

        // 测试网络连接管理
        assert_eq!(resource_manager.network_connections(), 0);
        resource_manager.add_network_connection();
        assert_eq!(resource_manager.network_connections(), 1);
        resource_manager.remove_network_connection();
        assert_eq!(resource_manager.network_connections(), 0);
    }

    #[test]
    fn test_execution_metrics() {
        let mut metrics = ExecutionMetrics::new();

        // 测试计时
        assert!(metrics.duration_ms().is_none());
        metrics.start_timing();
        std::thread::sleep(std::time::Duration::from_millis(10));
        metrics.end_timing();
        assert!(metrics.duration_ms().unwrap() >= 10);

        // 测试步骤计数
        assert_eq!(metrics.steps_executed, 0);
        metrics.add_step();
        metrics.add_step();
        assert_eq!(metrics.steps_executed, 2);

        // 测试缓存统计
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);

        metrics.add_cache_hit();
        metrics.add_cache_hit();
        metrics.add_cache_miss();
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_reset() {
        let query_context = create_test_context();
        let mut exec_ctx = ExecutionContext::new(query_context);

        // 设置一些状态
        exec_ctx.start();
        exec_ctx.set_variable("test".to_string(), Value::Int(42));
        exec_ctx.set_result("test", ResultType::new(Value::Int(42), ResultState::Success))
            .unwrap();

        // 重置
        exec_ctx.reset();

        // 检查状态
        assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Initialized);
        assert!(!exec_ctx.has_variable("test"));
        assert!(exec_ctx.get_result("test").is_err());
    }
}