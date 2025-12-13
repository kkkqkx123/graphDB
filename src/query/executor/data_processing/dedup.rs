//! DedupExecutor - 去重执行器
//!
//! 实现数据去重功能，支持基于指定键的去重策略

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::{Edge, Value, Vertex};
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::traits::{ExecutorCore, ExecutorLifecycle, ExecutorMetadata, ExecutionResult, DBResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 去重策略
#[derive(Debug, Clone, PartialEq)]
pub enum DedupStrategy {
    /// 完全去重，基于整个对象的值
    Full,
    /// 基于指定键去重
    ByKeys(Vec<String>),
    /// 基于顶点ID去重（仅对顶点有效）
    ByVertexId,
    /// 基于边的源、目标和类型去重（仅对边有效）
    ByEdgeKey,
}

/// DedupExecutor - 去重执行器
///
/// 实现数据去重功能，支持多种去重策略
#[derive(Debug)]
pub struct DedupExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<dyn crate::query::executor::traits::Executor<S>>>,
    strategy: DedupStrategy,
    memory_limit: usize, // 内存限制（字节）
    current_memory_usage: usize,
}

impl<S: StorageEngine + Send + 'static> DedupExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        strategy: DedupStrategy,
        memory_limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DedupExecutor".to_string(), storage),
            input_executor: None,
            strategy,
            memory_limit: memory_limit.unwrap_or(100 * 1024 * 1024), // 默认100MB
            current_memory_usage: 0,
        }
    }

    /// 执行去重操作
    async fn execute_dedup(
        &mut self,
        input: ExecutionResult,
    ) -> Result<ExecutionResult, QueryError> {
        match input {
            ExecutionResult::Values(values) => {
                let deduped_values = self.dedup_values(values).await?;
                Ok(ExecutionResult::Values(deduped_values))
            }
            ExecutionResult::Vertices(vertices) => {
                let deduped_vertices = self.dedup_vertices(vertices).await?;
                Ok(ExecutionResult::Vertices(deduped_vertices))
            }
            ExecutionResult::Edges(edges) => {
                let deduped_edges = self.dedup_edges(edges).await?;
                Ok(ExecutionResult::Edges(deduped_edges))
            }
            _ => Ok(input),
        }
    }

    /// 值去重
    async fn dedup_values(&mut self, values: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                self.hash_based_dedup(values, |value| format!("{:?}", value))
                    .await
            }
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor =
                    move |value: &Value| Self::extract_keys_from_value_static(value, &keys_clone);
                self.hash_based_dedup(values, key_extractor).await
            }
            _ => {
                // 对于值，其他策略退化为完全去重
                self.hash_based_dedup(values, |value| format!("{:?}", value))
                    .await
            }
        }
    }

    /// 顶点去重
    async fn dedup_vertices(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Vertex>, QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex))
                    .await
            }
            DedupStrategy::ByVertexId => {
                self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex.vid))
                    .await
            }
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor = move |vertex: &Vertex| {
                    Self::extract_keys_from_vertex_static(vertex, &keys_clone)
                };
                self.hash_based_dedup(vertices, key_extractor).await
            }
            _ => {
                // 默认基于顶点ID去重
                self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex.vid))
                    .await
            }
        }
    }

    /// 边去重
    async fn dedup_edges(&mut self, edges: Vec<Edge>) -> Result<Vec<Edge>, QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                self.hash_based_dedup(edges, |edge| format!("{:?}", edge))
                    .await
            }
            DedupStrategy::ByEdgeKey => {
                self.hash_based_dedup(edges, |edge| {
                    format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
                })
                .await
            }
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor =
                    move |edge: &Edge| Self::extract_keys_from_edge_static(edge, &keys_clone);
                self.hash_based_dedup(edges, key_extractor).await
            }
            _ => {
                // 默认基于边的关键信息去重
                self.hash_based_dedup(edges, |edge| {
                    format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
                })
                .await
            }
        }
    }

    /// 基于哈希的去重
    async fn hash_based_dedup<T, F>(
        &mut self,
        items: Vec<T>,
        key_extractor: F,
    ) -> Result<Vec<T>, QueryError>
    where
        T: Clone + Send + 'static,
        F: Fn(&T) -> String + Send + Sync,
    {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut memory_usage = 0;

        for item in items {
            let key = key_extractor(&item);

            if !seen.contains(&key) {
                // 估算内存使用
                let item_size = std::mem::size_of::<T>() + key.len();
                memory_usage += item_size;

                // 检查内存限制
                if self.current_memory_usage + memory_usage > self.memory_limit {
                    return Err(QueryError::ExecutionError("内存限制超出".to_string()));
                }

                seen.insert(key);
                result.push(item);
            }
        }

        self.current_memory_usage += memory_usage;
        Ok(result)
    }

    /// 从值中提取键（静态方法）
    fn extract_keys_from_value_static(value: &Value, keys: &[String]) -> String {
        match value {
            Value::Map(map) => keys
                .iter()
                .filter_map(|key| map.get(key))
                .map(|v| format!("{:?}", v))
                .collect::<Vec<_>>()
                .join("|"),
            _ => format!("{:?}", value),
        }
    }

    /// 从顶点中提取键（静态方法）
    fn extract_keys_from_vertex_static(vertex: &Vertex, keys: &[String]) -> String {
        let mut key_values = Vec::new();

        for key in keys {
            if key == "id" {
                key_values.push(format!("{:?}", vertex.vid));
            } else {
                // 在顶点的标签中查找属性
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(key) {
                        key_values.push(format!("{:?}", value));
                        break;
                    }
                }
            }
        }

        if key_values.is_empty() {
            format!("{:?}", vertex.vid)
        } else {
            key_values.join("|")
        }
    }

    /// 从边中提取键（静态方法）
    fn extract_keys_from_edge_static(edge: &Edge, keys: &[String]) -> String {
        let mut key_values = Vec::new();

        for key in keys {
            match key.as_str() {
                "src" => key_values.push(format!("{:?}", edge.src)),
                "dst" => key_values.push(format!("{:?}", edge.dst)),
                "type" => key_values.push(edge.edge_type.clone()),
                "ranking" => key_values.push(format!("{:?}", edge.ranking)),
                _ => {
                    if let Some(value) = edge.props.get(key.as_str()) {
                        key_values.push(format!("{:?}", value));
                    }
                }
            }
        }

        if key_values.is_empty() {
            format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
        } else {
            key_values.join("|")
        }
    }

    /// 获取当前内存使用量
    pub fn current_memory_usage(&self) -> usize {
        self.current_memory_usage
    }

    /// 重置内存使用量
    pub fn reset_memory_usage(&mut self) {
        self.current_memory_usage = 0;
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for DedupExecutor<S> {
    fn set_input(&mut self, input: Box<dyn crate::query::executor::traits::Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn crate::query::executor::traits::Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for DedupExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 重置内存使用量
        self.reset_memory_usage();

        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Values(Vec::new())
        };

        // 执行去重操作
        self.execute_dedup(input_result).await.map_err(|e| crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for DedupExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化去重所需的任何资源
        self.reset_memory_usage();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        self.reset_memory_usage();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for DedupExecutor<S> {
    fn id(&self) -> usize {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for DedupExecutor<S> {
    fn storage(&self) -> &S {
        self.base.storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // 模拟存储引擎
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            Ok(crate::core::Value::Null(NullType::NaN))
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

    #[tokio::test]
    async fn test_dedup_executor_full_strategy() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = DedupExecutor::new(1, storage, DedupStrategy::Full, None);

        // 设置测试数据（包含重复值）
        let test_data = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(1), // 重复
            Value::Int(3),
            Value::Int(2), // 重复
        ];

        let input_result = ExecutionResult::Values(test_data);

        // 创建模拟输入执行器
        struct MockInputExecutor {
            result: ExecutionResult,
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> ExecutorCore for MockInputExecutor {
            async fn execute(&mut self) -> DBResult<ExecutionResult> {
                Ok(self.result.clone())
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for MockInputExecutor {
            fn open(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn close(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn is_open(&self) -> bool {
                true
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorMetadata for MockInputExecutor {
            fn id(&self) -> usize {
                0
            }
            fn name(&self) -> &str {
                "MockInputExecutor"
            }
            fn description(&self) -> &str {
                "Mock input executor for testing"
            }
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for MockInputExecutor {
            fn storage(&self) -> &S {
                panic!("MockInputExecutor does not have storage")
            }
        }

        let input_executor = MockInputExecutor {
            result: input_result,
        };

        executor.set_input(Box::new(input_executor));

        // 执行去重
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 3); // 应该去重为3个值
                let mut sorted_values = values.clone();
                sorted_values.sort_by(|a, b| match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                });
                assert_eq!(
                    sorted_values,
                    vec![Value::Int(1), Value::Int(2), Value::Int(3),]
                );
            }
            _ => panic!("Expected Values result"),
        }
    }

    #[tokio::test]
    async fn test_dedup_executor_by_keys_strategy() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = DedupExecutor::new(
            1,
            storage,
            DedupStrategy::ByKeys(vec!["id".to_string()]),
            None,
        );

        // 设置测试数据（包含相同ID的不同对象）
        let test_data = vec![
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(1)),
                ("name".to_string(), Value::String("Alice".to_string())),
            ])),
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(2)),
                ("name".to_string(), Value::String("Bob".to_string())),
            ])),
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(1)), // 重复ID
                ("name".to_string(), Value::String("Alice2".to_string())),
            ])),
        ];

        let input_result = ExecutionResult::Values(test_data);

        // 创建模拟输入执行器
        struct MockInputExecutor {
            result: ExecutionResult,
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> ExecutorCore for MockInputExecutor {
            async fn execute(&mut self) -> DBResult<ExecutionResult> {
                Ok(self.result.clone())
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for MockInputExecutor {
            fn open(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn close(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn is_open(&self) -> bool {
                true
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorMetadata for MockInputExecutor {
            fn id(&self) -> usize {
                0
            }
            fn name(&self) -> &str {
                "MockInputExecutor"
            }
            fn description(&self) -> &str {
                "Mock input executor for testing"
            }
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for MockInputExecutor {
            fn storage(&self) -> &S {
                panic!("MockInputExecutor does not have storage")
            }
        }

        let input_executor = MockInputExecutor {
            result: input_result,
        };

        executor.set_input(Box::new(input_executor));

        // 执行去重
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 2); // 应该基于ID去重为2个值
            }
            _ => panic!("Expected Values result"),
        }
    }
}
