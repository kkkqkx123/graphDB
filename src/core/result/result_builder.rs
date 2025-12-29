//! 结果构建器模块 - 用于构建Result对象

use super::memory_manager::MemoryManager;
use super::result_core::{MemoryStats, Result, ResultState};
use super::result_iterator::ResultIterator;
use crate::core::{NullType, Value};
use std::sync::Arc;

/// 结果构建器
///
/// 用于构建Result对象
/// 对应原C++中的ResultBuilder类
pub struct ResultBuilder {
    check_memory: bool,
    state: ResultState,
    msg: String,
    value: Option<Value>,
    iterator: Option<Arc<dyn ResultIterator>>,
    memory_stats: MemoryStats,
    memory_manager: Option<Arc<dyn MemoryManager>>,
    memory_limit: Option<u64>,
}

impl std::fmt::Debug for ResultBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultBuilder")
            .field("check_memory", &self.check_memory)
            .field("state", &self.state)
            .field("msg", &self.msg)
            .field("has_value", &self.value.is_some())
            .field("has_iterator", &self.iterator.is_some())
            .field("memory_stats", &self.memory_stats)
            .field("has_memory_manager", &self.memory_manager.is_some())
            .field("memory_limit", &self.memory_limit)
            .finish()
    }
}

impl ResultBuilder {
    pub fn new() -> Self {
        Self {
            check_memory: false,
            state: ResultState::Success,
            msg: String::new(),
            value: None,
            iterator: None,
            memory_stats: MemoryStats::new(),
            memory_manager: None,
            memory_limit: None,
        }
    }

    /// 设置值
    pub fn value(mut self, value: Value) -> Self {
        let value_bytes = std::mem::size_of_val(&value) as u64;
        self.value = Some(value);
        self.memory_stats.update_value_bytes(value_bytes);
        self
    }

    /// 设置状态
    pub fn state(mut self, state: ResultState) -> Self {
        self.state = state;
        self
    }

    /// 设置消息
    pub fn msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    /// 设置迭代器
    pub fn iterator(mut self, iterator: Arc<dyn ResultIterator>) -> Self {
        let iter_bytes = (iterator.size() * std::mem::size_of::<Value>()) as u64;
        self.iterator = Some(iterator);
        self.memory_stats.update_iterator_bytes(iter_bytes);
        self
    }

    /// 设置内存检查标志
    pub fn check_memory(mut self, check_memory: bool) -> Self {
        self.check_memory = check_memory;
        self
    }

    /// 设置内存统计
    pub fn memory_stats(mut self, stats: MemoryStats) -> Self {
        self.memory_stats = stats;
        self
    }

    /// 设置内存管理器
    pub fn memory_manager(mut self, manager: Arc<dyn MemoryManager>) -> Self {
        self.memory_manager = Some(manager);
        self
    }

    /// 设置内存限制
    pub fn memory_limit(mut self, limit: u64) -> Self {
        self.memory_limit = Some(limit);
        self
    }

    /// 构建结果
    pub fn build(self) -> Result {
        let value = match self.value {
            Some(v) => v,
            None => Value::Null(NullType::Null),
        };

        // 使用完整构造方法创建结果
        let result = Result::with_components(
            value,
            self.state,
            self.msg,
            self.iterator,
            self.memory_stats,
            self.check_memory,
            self.memory_manager,
            self.memory_limit,
        );

        result
    }

    /// 从现有结果构建
    pub fn from_result(result: &Result) -> Self {
        Self {
            check_memory: result.check_memory(),
            state: result.state().clone(),
            msg: result.msg().to_string(),
            value: Some(result.value().clone()),
            iterator: result.iterator().cloned(),
            memory_stats: result.memory_stats().clone(),
            memory_manager: None, // 不复制内存管理器
            memory_limit: result.get_memory_limit(),
        }
    }
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_result_builder() {
        let value = Value::Int(42);
        let result = ResultBuilder::new()
            .value(value.clone())
            .state(ResultState::Success)
            .msg("Test message".to_string())
            .build();

        assert_eq!(result.value(), &value);
        assert_eq!(result.state(), &ResultState::Success);
        assert_eq!(result.msg(), "Test message");
    }

    #[test]
    fn test_result_builder_default() {
        let result = ResultBuilder::default().build();
        assert_eq!(result.state(), &ResultState::Success);
        assert_eq!(result.msg(), "");
        assert!(!result.check_memory());
    }

    #[test]
    fn test_result_builder_from_result() {
        let original = Result::new(Value::String("test".to_string()), ResultState::Success);
        let builder = ResultBuilder::from_result(&original);
        let rebuilt = builder.build();

        assert_eq!(rebuilt.value(), original.value());
        assert_eq!(rebuilt.state(), original.state());
        assert_eq!(rebuilt.msg(), original.msg());
    }

    #[test]
    fn test_result_builder_with_memory_stats() {
        let mut stats = MemoryStats::new();
        stats.update_value_bytes(100);

        let result = ResultBuilder::new()
            .value(Value::Int(42))
            .memory_stats(stats)
            .build();

        assert_eq!(result.memory_stats().value_bytes(), 100);
    }
}
