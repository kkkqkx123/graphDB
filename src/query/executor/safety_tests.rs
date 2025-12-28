//! 执行器安全测试
//!
//! 测试执行器的安全机制，包括递归检测、循环保护、对象池等

use crate::query::executor::recursion_detector::{RecursionDetector, ExecutorSafetyValidator, ExecutorSafetyConfig};
use crate::query::executor::object_pool::{ExecutorObjectPool, ObjectPoolConfig, ThreadSafeExecutorPool};
use crate::core::error::{DBError, DBResult};
use crate::query::executor::traits::{Executor, ExecutionResult};
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// 模拟存储引擎
struct MockStorage;

impl StorageEngine for MockStorage {
    fn insert_node(
        &mut self,
        _vertex: crate::core::vertex_edge_path::Vertex,
    ) -> Result<crate::core::Value, crate::storage::StorageError> {
        Ok(crate::core::Value::Null(crate::core::value::NullType::NaN))
    }

    fn get_node(
        &self,
        _id: &crate::core::Value,
    ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
    {
        Ok(None)
    }

    fn update_node(
        &mut self,
        _vertex: crate::core::vertex_edge_path::Vertex,
    ) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }

    fn delete_node(
        &mut self,
        _id: &crate::core::Value,
    ) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }

    fn scan_all_vertices(
        &self,
    ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
    {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(
        &self,
        _tag: &str,
    ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
    {
        Ok(Vec::new())
    }

    fn insert_edge(
        &mut self,
        _edge: crate::core::vertex_edge_path::Edge,
    ) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }

    fn get_edge(
        &self,
        _src: &crate::core::Value,
        _dst: &crate::core::Value,
        _edge_type: &str,
    ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
    {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _node_id: &crate::core::Value,
        _direction: crate::core::vertex_edge_path::Direction,
    ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
    {
        Ok(Vec::new())
    }

    fn delete_edge(
        &mut self,
        _src: &crate::core::Value,
        _dst: &crate::core::Value,
        _edge_type: &str,
    ) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
        Ok(1)
    }

    fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }

    fn rollback_transaction(
        &mut self,
        _tx_id: u64,
    ) -> Result<(), crate::storage::StorageError> {
        Ok(())
    }
}

/// 模拟简单执行器
struct SimpleExecutor {
    id: i64,
    name: String,
    storage: Arc<Mutex<MockStorage>>,
    is_open: bool,
}

impl SimpleExecutor {
    fn new(id: i64, name: &str, storage: Arc<Mutex<MockStorage>>) -> Self {
        Self {
            id,
            name: name.to_string(),
            storage,
            is_open: false,
        }
    }
}

#[async_trait]
impl Executor<MockStorage> for SimpleExecutor {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "SimpleExecutor"
    }
}

#[cfg(test)]
mod safety_tests {
    use super::*;

    #[test]
    fn test_recursion_detector_prevents_infinite_loop() {
        let mut detector = RecursionDetector::new(10);

        // 模拟循环引用
        detector.validate_executor(1, "Executor1").unwrap();
        detector.validate_executor(2, "Executor2").unwrap();
        detector.validate_executor(3, "Executor3").unwrap();

        // 尝试回到第一个执行器 - 应该失败
        let result = detector.validate_executor(1, "Executor1");
        assert!(result.is_err());
        if let Err(DBError::Query(err)) = result {
            assert!(err.to_string().contains("循环引用"));
        }
    }

    #[test]
    fn test_recursion_detector_prevents_stack_overflow() {
        let mut detector = RecursionDetector::new(5);

        // 正常深度
        for i in 1..=5 {
            assert!(detector.validate_executor(i, &format!("Executor{}", i)).is_ok());
        }

        // 超过最大深度 - 应该失败
        let result = detector.validate_executor(6, "Executor6");
        assert!(result.is_err());
        if let Err(DBError::Query(err)) = result {
            assert!(err.to_string().contains("调用深度超过最大限制"));
        }
    }

    #[test]
    fn test_safety_validator_loop_config() {
        let config = ExecutorSafetyConfig {
            max_loop_iterations: 1000,
            max_expand_depth: 100,
            max_recursion_depth: 50,
        };
        let validator = ExecutorSafetyValidator::new(config);

        // 正常配置
        assert!(validator.validate_loop_config(Some(100)).is_ok());
        assert!(validator.validate_loop_config(Some(1000)).is_ok());

        // 超过限制
        assert!(validator.validate_loop_config(Some(1001)).is_err());
    }

    #[test]
    fn test_safety_validator_expand_config() {
        let config = ExecutorSafetyConfig {
            max_loop_iterations: 1000,
            max_expand_depth: 100,
            max_recursion_depth: 50,
        };
        let validator = ExecutorSafetyValidator::new(config);

        // 正常配置
        assert!(validator.validate_expand_config(Some(50)).is_ok());
        assert!(validator.validate_expand_config(Some(100)).is_ok());

        // 超过限制
        assert!(validator.validate_expand_config(Some(101)).is_err());
    }

    #[test]
    fn test_recursion_detector_with_leave() {
        let mut detector = RecursionDetector::new(10);

        // 进入执行器
        detector.validate_executor(1, "E1").unwrap();
        detector.validate_executor(2, "E2").unwrap();
        detector.validate_executor(3, "E3").unwrap();

        // 离开执行器
        detector.leave_executor();
        detector.leave_executor();

        // 现在可以重新进入
        assert!(detector.validate_executor(1, "E1").is_ok());
    }

    #[test]
    fn test_recursion_detector_reset() {
        let mut detector = RecursionDetector::new(5);

        // 进入多个执行器
        for i in 1..=5 {
            detector.validate_executor(i, &format!("E{}", i)).unwrap();
        }

        // 重置
        detector.reset();

        // 现在可以重新进入
        for i in 1..=5 {
            assert!(detector.validate_executor(i, &format!("E{}", i)).is_ok());
        }
    }

    #[test]
    fn test_recursion_path_tracking() {
        let mut detector = RecursionDetector::new(10);

        detector.validate_executor(1, "Executor1").unwrap();
        detector.validate_executor(2, "Executor2").unwrap();
        detector.validate_executor(3, "Executor3").unwrap();

        let path = detector.get_recursion_path();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "Executor1(1)");
        assert_eq!(path[1], "Executor2(2)");
        assert_eq!(path[2], "Executor3(3)");
    }

    #[test]
    fn test_recursion_detector_current_depth() {
        let mut detector = RecursionDetector::new(10);

        assert_eq!(detector.current_depth(), 0);

        detector.validate_executor(1, "E1").unwrap();
        assert_eq!(detector.current_depth(), 1);

        detector.validate_executor(2, "E2").unwrap();
        assert_eq!(detector.current_depth(), 2);

        detector.leave_executor();
        assert_eq!(detector.current_depth(), 1);
    }

    #[test]
    fn test_recursion_detector_visited_set() {
        let mut detector = RecursionDetector::new(10);

        assert_eq!(detector.visited_count(), 0);

        detector.validate_executor(1, "E1").unwrap();
        assert_eq!(detector.visited_count(), 1);

        detector.validate_executor(2, "E2").unwrap();
        assert_eq!(detector.visited_count(), 2);

        detector.leave_executor();
        assert_eq!(detector.visited_count(), 1);
    }

    #[test]
    fn test_safety_validator_default_config() {
        let validator = ExecutorSafetyValidator::default();

        // 验证默认配置
        assert!(validator.validate_loop_config(Some(1000)).is_ok());
        assert!(validator.validate_expand_config(Some(100)).is_ok());
    }

    #[test]
    fn test_recursion_detector_complex_scenario() {
        let mut detector = RecursionDetector::new(10);

        // 模拟复杂的执行器链
        detector.validate_executor(1, "Start").unwrap();
        detector.validate_executor(2, "Filter").unwrap();
        detector.validate_executor(3, "Project").unwrap();
        detector.validate_executor(4, "Expand").unwrap();
        detector.validate_executor(5, "Filter").unwrap();

        // 尝试创建循环 - 应该失败
        assert!(detector.validate_executor(3, "Project").is_err());

        // 离开一些执行器
        detector.leave_executor();
        detector.leave_executor();

        // 现在可以重新进入
        assert!(detector.validate_executor(3, "Project").is_ok());
    }

    #[tokio::test]
    async fn test_simple_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut executor = SimpleExecutor::new(1, "TestExecutor", storage);

        assert!(!executor.is_open());
        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "TestExecutor");

        executor.open().unwrap();
        assert!(executor.is_open());

        let result = executor.execute().await.unwrap();
        assert!(matches!(result, ExecutionResult::Success));

        executor.close().unwrap();
        assert!(!executor.is_open());
    }

    #[test]
    fn test_recursion_detector_edge_cases() {
        let mut detector = RecursionDetector::new(1);

        // 最大深度为1
        assert!(detector.validate_executor(1, "E1").is_ok());

        // 超过最大深度
        assert!(detector.validate_executor(2, "E2").is_err());

        // 重置后可以重新进入
        detector.reset();
        assert!(detector.validate_executor(1, "E1").is_ok());
    }

    #[test]
    fn test_safety_validator_boundary_conditions() {
        let config = ExecutorSafetyConfig {
            max_loop_iterations: 100,
            max_expand_depth: 50,
            max_recursion_depth: 25,
        };
        let validator = ExecutorSafetyValidator::new(config);

        // 边界条件 - 刚好等于限制
        assert!(validator.validate_loop_config(Some(100)).is_ok());
        assert!(validator.validate_expand_config(Some(50)).is_ok());

        // 边界条件 - 超过限制1
        assert!(validator.validate_loop_config(Some(101)).is_err());
        assert!(validator.validate_expand_config(Some(51)).is_err());
    }

    #[test]
    fn test_object_pool_acquire_release() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: true,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        // 初始状态 - 池为空
        assert!(pool.acquire("SimpleExecutor").is_none());

        // 创建并释放执行器
        let storage = Arc::new(Mutex::new(MockStorage));
        let executor1: Box<dyn Executor<MockStorage>> = Box::new(SimpleExecutor::new(1, "E1", storage.clone()));
        let executor2: Box<dyn Executor<MockStorage>> = Box::new(SimpleExecutor::new(2, "E2", storage.clone()));

        pool.release("SimpleExecutor", executor1);
        pool.release("SimpleExecutor", executor2);

        // 现在应该能够获取执行器
        let acquired = pool.acquire("SimpleExecutor");
        assert!(acquired.is_some());
        assert_eq!(acquired.unwrap().id(), 2);

        let acquired = pool.acquire("SimpleExecutor");
        assert!(acquired.is_some());
        assert_eq!(acquired.unwrap().id(), 1);

        // 池再次为空
        assert!(pool.acquire("SimpleExecutor").is_none());
    }

    #[test]
    fn test_object_pool_max_size() {
        let config = ObjectPoolConfig {
            max_pool_size: 2,
            enabled: true,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));

        // 释放3个执行器，但池最大为2
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(1, "E1", storage.clone())));
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(2, "E2", storage.clone())));
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(3, "E3", storage.clone())));

        // 只能获取2个执行器
        assert!(pool.acquire("SimpleExecutor").is_some());
        assert!(pool.acquire("SimpleExecutor").is_some());
        assert!(pool.acquire("SimpleExecutor").is_none());
    }

    #[test]
    fn test_object_pool_disabled() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: false,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));
        let executor: Box<dyn Executor<MockStorage>> = Box::new(SimpleExecutor::new(1, "E1", storage));

        // 释放执行器 - 应该被丢弃
        pool.release("SimpleExecutor", executor);

        // 获取执行器 - 应该返回None
        assert!(pool.acquire("SimpleExecutor").is_none());
    }

    #[test]
    fn test_object_pool_multiple_types() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: true,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));

        // 释放不同类型的执行器
        pool.release("TypeA", Box::new(SimpleExecutor::new(1, "A1", storage.clone())));
        pool.release("TypeA", Box::new(SimpleExecutor::new(2, "A2", storage.clone())));
        pool.release("TypeB", Box::new(SimpleExecutor::new(3, "B1", storage.clone())));
        pool.release("TypeB", Box::new(SimpleExecutor::new(4, "B2", storage.clone())));

        // 从不同类型获取执行器
        assert_eq!(pool.acquire("TypeA").unwrap().id(), 2);
        assert_eq!(pool.acquire("TypeA").unwrap().id(), 1);
        assert_eq!(pool.acquire("TypeB").unwrap().id(), 4);
        assert_eq!(pool.acquire("TypeB").unwrap().id(), 3);
    }

    #[test]
    fn test_object_pool_stats() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: true,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));

        // 初始统计
        let stats = pool.get_stats();
        assert_eq!(stats.total_acquires, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.total_releases, 0);

        // 释放执行器
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(1, "E1", storage.clone())));

        // 获取 - 命中
        pool.acquire("SimpleExecutor");
        let stats = pool.get_stats();
        assert_eq!(stats.total_acquires, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);

        // 获取 - 未命中
        pool.acquire("SimpleExecutor");
        let stats = pool.get_stats();
        assert_eq!(stats.total_acquires, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_object_pool_reset() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: true,
        };
        let mut pool = ExecutorObjectPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));

        // 添加一些执行器
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(1, "E1", storage.clone())));
        pool.release("SimpleExecutor", Box::new(SimpleExecutor::new(2, "E2", storage)));

        // 重置池
        pool.reset();

        // 池应该为空
        assert!(pool.acquire("SimpleExecutor").is_none());
    }

    #[test]
    fn test_thread_safe_pool_concurrent_access() {
        let config = ObjectPoolConfig {
            max_pool_size: 10,
            enabled: true,
        };
        let pool = Arc::new(Mutex::new(ExecutorObjectPool::<MockStorage>::new(config)));

        let mut handles = vec![];

        // 多个线程并发访问
        for i in 0..5 {
            let pool_clone = Arc::clone(&pool);
            let handle = std::thread::spawn(move || {
                let storage = Arc::new(Mutex::new(MockStorage));
                let executor: Box<dyn Executor<MockStorage>> = Box::new(SimpleExecutor::new(i, &format!("E{}", i), storage));

                let mut pool_guard = pool_clone.lock().unwrap();
                pool_guard.release("SimpleExecutor", executor);
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证池中的执行器数量
        let pool_guard = pool.lock().unwrap();
        let stats = pool_guard.get_stats();
        assert_eq!(stats.total_releases, 5);
    }

    #[test]
    fn test_thread_safe_pool_wrapper() {
        let config = ObjectPoolConfig {
            max_pool_size: 5,
            enabled: true,
        };
        let pool = ThreadSafeExecutorPool::<MockStorage>::new(config);

        let storage = Arc::new(Mutex::new(MockStorage));
        let executor: Box<dyn Executor<MockStorage>> = Box::new(SimpleExecutor::new(1, "E1", storage));

        // 释放执行器
        pool.release("SimpleExecutor", executor);

        // 获取执行器
        let acquired = pool.acquire("SimpleExecutor");
        assert!(acquired.is_some());
    }
}
