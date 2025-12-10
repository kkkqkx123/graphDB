use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use graphdb::query::executor::{Executor, ExecutionResult};
use graphdb::query::scheduler::{AsyncMsgNotifyBasedScheduler, ExecutionPlan, QueryScheduler};
use graphdb::query::QueryError;
use graphdb::storage::StorageEngine;
use graphdb::storage::StorageError;
use graphdb::core::{Value, Vertex, Edge, Direction};

// 创建一个简单的模拟存储引擎
struct MockStorage {
    data: HashMap<String, Value>,
}

impl MockStorage {
    fn new() -> Self {
        let mut data = HashMap::new();
        data.insert("test".to_string(), Value::String("test_value".to_string()));
        Self { data }
    }
}

impl StorageEngine for MockStorage {
    fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, StorageError> {
        Ok(Value::String("mock_id".to_string()))
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

    fn insert_edge(&mut self, _edge: Edge) -> Result<(), StorageError> {
        Ok(())
    }

    fn get_edge(&self, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<Option<Edge>, StorageError> {
        Ok(None)
    }

    fn get_node_edges(&self, _node_id: &Value, _direction: Direction) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn delete_edge(&mut self, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<(), StorageError> {
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<u64, StorageError> {
        Ok(1)
    }

    fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }
}

// 创建一个简单的模拟执行器
struct MockExecutor {
    id: usize,
    name: String,
    result: ExecutionResult,
}

impl MockExecutor {
    fn new(id: usize, name: String, result: ExecutionResult) -> Self {
        Self { id, name, result }
    }
}

#[async_trait]
impl Executor<MockStorage> for MockExecutor {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 模拟执行
        Ok(self.result.clone())
    }

    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::test]
async fn test_execution_plan_basic() {
    let mut plan = ExecutionPlan::<MockStorage>::new(0);
    
    // 添加执行器
    let executor1 = Box::new(MockExecutor::new(0, "executor1".to_string(), ExecutionResult::Success));
    let executor2 = Box::new(MockExecutor::new(1, "executor2".to_string(), ExecutionResult::Success));
    
    plan.add_executor(executor1);
    plan.add_executor(executor2);
    
    // 添加依赖关系
    plan.add_dependency(0, 1).unwrap();
    
    // 验证计划
    assert!(plan.validate().is_ok());
    
    // 测试获取可执行执行器
    let completed = HashMap::new();
    let executable = plan.get_executable_executors(&completed);
    assert_eq!(executable.len(), 1); // 只有根执行器可执行
    assert_eq!(executable[0], 0);
    
    // 测试依赖检查
    assert!(plan.are_dependencies_satisfied(0, &completed));
    assert!(!plan.are_dependencies_satisfied(1, &completed));
}

#[tokio::test]
async fn test_scheduler_basic() {
    let storage = Arc::new(Mutex::new(MockStorage::new()));
    let mut scheduler = AsyncMsgNotifyBasedScheduler::new(storage);
    
    // 创建执行计划
    let mut plan = ExecutionPlan::<MockStorage>::new(0);
    let executor1 = Box::new(MockExecutor::new(0, "executor1".to_string(), ExecutionResult::Success));
    plan.add_executor(executor1);
    
    // 执行计划
    let result = scheduler.schedule(plan).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scheduler_with_dependencies() {
    let storage = Arc::new(Mutex::new(MockStorage::new()));
    let mut scheduler = AsyncMsgNotifyBasedScheduler::new(storage);
    
    // 创建执行计划
    let mut plan = ExecutionPlan::<MockStorage>::new(0);
    let executor1 = Box::new(MockExecutor::new(0, "executor1".to_string(), ExecutionResult::Success));
    let executor2 = Box::new(MockExecutor::new(1, "executor2".to_string(), ExecutionResult::Success));
    
    plan.add_executor(executor1);
    plan.add_executor(executor2);
    plan.add_dependency(0, 1).unwrap();
    
    // 执行计划
    let result = scheduler.schedule(plan).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scheduler_cycle_detection() {
    let mut plan = ExecutionPlan::<MockStorage>::new(0);
    let executor1 = Box::new(MockExecutor::new(0, "executor1".to_string(), ExecutionResult::Success));
    let executor2 = Box::new(MockExecutor::new(1, "executor2".to_string(), ExecutionResult::Success));
    
    plan.add_executor(executor1);
    plan.add_executor(executor2);
    
    // 创建循环依赖
    plan.add_dependency(0, 1).unwrap();
    plan.add_dependency(1, 0).unwrap();
    
    // 验证应该检测到循环
    assert!(plan.validate().is_err());
}

#[tokio::test]
async fn test_execution_state() {
    use graphdb::query::scheduler::async_scheduler::ExecutionState;
    
    let mut state = ExecutionState::new();
    
    // 初始状态
    assert!(!state.has_failure());
    assert!(!state.is_executor_executing(0));
    assert!(!state.is_executor_completed(0));
    
    // 标记执行器正在执行
    state.executing_executors.insert(0);
    assert!(state.is_executor_executing(0));
    assert!(!state.is_executor_completed(0));
    
    // 完成执行
    state.executing_executors.remove(&0);
    state.execution_results.insert(0, ExecutionResult::Success);
    assert!(!state.is_executor_executing(0));
    assert!(state.is_executor_completed(0));
    
    // 测试失败状态
    state.set_failure(QueryError::InvalidQuery("test error".to_string()));
    assert!(state.has_failure());
    assert!(state.take_failure().is_some());
    assert!(!state.has_failure());
}