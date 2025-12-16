//! 结果核心模块 - 定义Result的核心结构和功能

use super::memory_manager::MemoryManager;
use super::result_iterator::ResultIterator;
use crate::core::{NullType, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// 查询执行结果状态
#[derive(Debug, Clone, PartialEq)]
pub enum ResultState {
    UnExecuted,
    PartialSuccess,
    Success,
    Failed,
    Cancelled,
}

/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub value_bytes: u64,
    pub iterator_bytes: u64,
    pub overhead_bytes: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            total_bytes: 0,
            value_bytes: 0,
            iterator_bytes: 0,
            overhead_bytes: 0,
        }
    }

    pub fn total(&self) -> u64 {
        self.total_bytes
    }

    pub fn update_value_bytes(&mut self, bytes: u64) {
        self.value_bytes = bytes;
        self.recalculate_total();
    }

    pub fn update_iterator_bytes(&mut self, bytes: u64) {
        self.iterator_bytes = bytes;
        self.recalculate_total();
    }

    pub fn update_overhead_bytes(&mut self, bytes: u64) {
        self.overhead_bytes = bytes;
        self.recalculate_total();
    }

    fn recalculate_total(&mut self) {
        self.total_bytes = self.value_bytes + self.iterator_bytes + self.overhead_bytes;
    }
}

/// 执行结果核心
pub struct ResultCore {
    pub check_memory: bool,
    pub state: ResultState,
    pub msg: String,
    pub value: Arc<Value>,
    pub iterator: Option<Arc<dyn ResultIterator>>,
    pub memory_stats: MemoryStats,
    pub creation_time: std::time::SystemTime,
    pub access_count: AtomicU64,
    pub is_shared: bool,
    pub memory_manager: Option<Arc<dyn MemoryManager>>,
}

impl std::fmt::Debug for ResultCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCore")
            .field("check_memory", &self.check_memory)
            .field("state", &self.state)
            .field("msg", &self.msg)
            .field("value", &self.value)
            .field("has_iterator", &self.iterator.is_some())
            .field("memory_stats", &self.memory_stats)
            .field("creation_time", &self.creation_time)
            .field("access_count", &self.access_count.load(Ordering::Relaxed))
            .field("is_shared", &self.is_shared)
            .field("has_memory_manager", &self.memory_manager.is_some())
            .finish()
    }
}

impl Clone for ResultCore {
    fn clone(&self) -> Self {
        Self {
            check_memory: self.check_memory,
            state: self.state.clone(),
            msg: self.msg.clone(),
            value: self.value.clone(),
            iterator: self.iterator.clone(),
            memory_stats: self.memory_stats.clone(),
            creation_time: self.creation_time,
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
            is_shared: true, // 克隆后标记为共享
            memory_manager: self.memory_manager.clone(),
        }
    }
}

impl PartialEq for ResultCore {
    fn eq(&self, other: &Self) -> bool {
        // 比较主要属性：状态、消息和值
        self.state == other.state && self.msg == other.msg && self.value == other.value
    }
}

/// 执行结果
#[derive(Debug, PartialEq)]
pub struct Result {
    core: Arc<ResultCore>,
}

impl Clone for Result {
    fn clone(&self) -> Self {
        // 克隆ResultCore并标记为共享
        let mut cloned_core = (*self.core).clone();
        cloned_core.is_shared = true;

        Self {
            core: Arc::new(cloned_core),
        }
    }
}

impl Result {
    /// 创建新的结果
    pub fn new(value: Value, state: ResultState) -> Self {
        let mut memory_stats = MemoryStats::new();
        let value_bytes = std::mem::size_of_val(&value) as u64;
        memory_stats.update_value_bytes(value_bytes);

        let core = ResultCore {
            check_memory: false,
            state,
            msg: String::new(),
            value: Arc::new(value),
            iterator: None,
            memory_stats,
            creation_time: std::time::SystemTime::now(),
            access_count: AtomicU64::new(0),
            is_shared: false,
            memory_manager: None,
        };

        Self {
            core: Arc::new(core),
        }
    }

    /// 创建空结果
    pub fn empty() -> Self {
        Self::new(Value::Null(NullType::Null), ResultState::UnExecuted)
    }

    /// 创建带消息的结果
    pub fn with_message(value: Value, state: ResultState, msg: String) -> Self {
        let mut result = Self::new(value, state);
        Arc::get_mut(&mut result.core).unwrap().msg = msg;
        result
    }

    /// 完整构造方法 - 用于 ResultBuilder
    pub(crate) fn with_components(
        value: Value,
        state: ResultState,
        msg: String,
        iterator: Option<Arc<dyn ResultIterator>>,
        memory_stats: MemoryStats,
        check_memory: bool,
        memory_manager: Option<Arc<dyn MemoryManager>>,
    ) -> Self {
        let core = ResultCore {
            check_memory,
            state,
            msg,
            value: Arc::new(value),
            iterator,
            memory_stats,
            creation_time: std::time::SystemTime::now(),
            access_count: AtomicU64::new(0),
            is_shared: false,
            memory_manager,
        };

        Self {
            core: Arc::new(core),
        }
    }

    /// 更新迭代器并调整值 - 用于 ResultBuilder
    #[allow(dead_code)]
    pub(crate) fn update_iterator_and_value(&mut self, iterator: Option<Arc<dyn ResultIterator>>) {
        if let Some(core) = Arc::get_mut(&mut self.core) {
            core.iterator = iterator;

            // 如果迭代器存在，更新值为迭代器的值
            if let Some(iter) = &core.iterator {
                if !iter.is_empty() {
                    if let Some(row) = iter.current_row() {
                        if let Some(first_value) = row.first() {
                            core.value = Arc::new(first_value.clone());
                        }
                    }
                }
            }
        }
    }

    /// 获取值的引用
    pub fn value(&self) -> &Value {
        self.increment_access_count();
        &self.core.value
    }

    /// 获取值的Arc引用
    pub fn value_arc(&self) -> Arc<Value> {
        self.increment_access_count();
        self.core.value.clone()
    }

    /// 获取结果状态
    pub fn state(&self) -> &ResultState {
        &self.core.state
    }

    /// 获取结果消息
    pub fn msg(&self) -> &str {
        &self.core.msg
    }

    /// 获取迭代器的引用
    pub fn iterator(&self) -> Option<&Arc<dyn ResultIterator>> {
        self.core.iterator.as_ref()
    }

    /// 获取内存统计
    pub fn memory_stats(&self) -> &MemoryStats {
        &self.core.memory_stats
    }

    /// 获取创建时间
    pub fn creation_time(&self) -> std::time::SystemTime {
        self.core.creation_time
    }

    /// 获取访问计数
    pub fn access_count(&self) -> u64 {
        self.core.access_count.load(Ordering::Relaxed)
    }

    /// 检查是否为共享状态
    pub fn is_shared(&self) -> bool {
        self.core.is_shared
    }

    /// 检查内存
    pub fn check_memory(&self) -> bool {
        self.core.check_memory
    }

    /// 获取结果大小
    pub fn size(&self) -> usize {
        if let Some(iter) = &self.core.iterator {
            iter.size()
        } else {
            0
        }
    }

    /// 获取列名（如果值是数据集）
    pub fn get_col_names(&self) -> Vec<String> {
        match &*self.core.value {
            Value::DataSet(dataset) => dataset.col_names.clone(),
            _ => vec![],
        }
    }

    /// 设置内存管理器
    pub fn set_memory_manager(&mut self, manager: Arc<dyn MemoryManager>) {
        if let Some(core) = Arc::get_mut(&mut self.core) {
            core.memory_manager = Some(manager);
        }
    }

    /// 检查内存使用情况
    pub fn check_memory_usage(&self) -> std::result::Result<bool, String> {
        if !self.core.check_memory {
            return Ok(true);
        }

        if let Some(manager) = &self.core.memory_manager {
            let total_bytes = self.core.memory_stats.total();
            manager.check_memory(total_bytes)
        } else {
            // 简化的内存检查逻辑
            let total_bytes = self.core.memory_stats.total();
            const MEMORY_LIMIT: u64 = 100 * 1024 * 1024; // 100MB

            if total_bytes > MEMORY_LIMIT {
                Err(format!(
                    "Memory usage exceeded limit: {} bytes > {} bytes",
                    total_bytes, MEMORY_LIMIT
                ))
            } else {
                Ok(true)
            }
        }
    }

    /// 更新内存统计
    pub fn update_memory_stats(&mut self) {
        if let Some(core) = Arc::get_mut(&mut self.core) {
            if let Some(iter) = &core.iterator {
                let iter_size = iter.size();
                let iter_bytes = (iter_size * std::mem::size_of::<Value>()) as u64;
                core.memory_stats.update_iterator_bytes(iter_bytes);
            }

            let value_bytes = std::mem::size_of_val(&*core.value) as u64;
            core.memory_stats.update_value_bytes(value_bytes);
        }
    }

    /// 增加访问计数
    fn increment_access_count(&self) {
        self.core.access_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 转换为字符串表示
    pub fn to_string(&self) -> String {
        format!(
            "Result {{ state: {:?}, size: {}, memory: {} bytes, access_count: {}, shared: {} }}",
            self.core.state,
            self.size(),
            self.core.memory_stats.total(),
            self.access_count(),
            self.is_shared()
        )
    }
}

impl Default for Result {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_creation() {
        let value = Value::String("test_value".to_string());
        let result = Result::new(value.clone(), ResultState::Success);

        assert_eq!(result.state(), &ResultState::Success);
        assert_eq!(result.value(), &value);
        assert_eq!(result.access_count(), 1); // 调用value()会增加访问计数
        assert!(!result.is_shared());
    }

    #[test]
    fn test_result_with_message() {
        let value = Value::Int(42);
        let result = Result::with_message(
            value.clone(),
            ResultState::Success,
            "Test message".to_string(),
        );

        assert_eq!(result.value(), &value);
        assert_eq!(result.state(), &ResultState::Success);
        assert_eq!(result.msg(), "Test message");
    }

    #[test]
    fn test_memory_stats() {
        let mut stats = MemoryStats::new();
        assert_eq!(stats.total(), 0);

        stats.update_value_bytes(100);
        assert_eq!(stats.total(), 100);

        stats.update_iterator_bytes(200);
        assert_eq!(stats.total(), 300);

        stats.update_overhead_bytes(50);
        assert_eq!(stats.total(), 350);
    }

    #[test]
    fn test_access_count() {
        let result = Result::new(Value::Int(42), ResultState::Success);

        assert_eq!(result.access_count(), 0);

        // 访问值会增加访问计数
        let _ = result.value();
        assert_eq!(result.access_count(), 1);

        let _ = result.value_arc();
        assert_eq!(result.access_count(), 2);
    }

    #[test]
    fn test_to_string() {
        let result = Result::new(Value::Int(42), ResultState::Success);
        let result_str = result.to_string();

        assert!(result_str.contains("Result"));
        assert!(result_str.contains("Success"));
        assert!(result_str.contains("size: 0"));
        assert!(result_str.contains("access_count: 0"));
        assert!(result_str.contains("shared: false"));
    }

    #[test]
    fn test_clone_behavior() {
        let result1 = Result::new(Value::Int(42), ResultState::Success);
        let _ = result1.value(); // 增加访问计数

        let result2 = result1.clone();
        assert!(result2.is_shared()); // 克隆后应该标记为共享
        assert_eq!(result2.value(), result1.value()); // 值应该相同
    }
}
