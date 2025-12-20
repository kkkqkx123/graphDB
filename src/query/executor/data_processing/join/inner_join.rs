//! 内连接执行器实现
//!
//! 实现基于哈希的内连接算法，支持单键和多键连接

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Value};
use crate::query::executor::data_processing::join::base_join::BaseJoinExecutor;
use crate::query::executor::data_processing::join::hash_table::{
    HashTableBuilder, HashTableProbe, MultiKeyHashTable, SingleKeyHashTable,
};
use crate::query::executor::traits::{
    ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata,
};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 内连接执行器
pub struct InnerJoinExecutor<S: StorageEngine> {
    base_executor: BaseJoinExecutor<S>,
    /// 哈希表（用于单键连接）
    single_key_hash_table: Option<SingleKeyHashTable>,
    /// 多键哈希表（用于多键连接）
    multi_key_hash_table: Option<MultiKeyHashTable>,
    /// 是否使用多键连接
    use_multi_key: bool,
}

// Manual Debug implementation for InnerJoinExecutor to avoid requiring Debug trait for BaseJoinExecutor
impl<S: StorageEngine> std::fmt::Debug for InnerJoinExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerJoinExecutor")
            .field("base_executor", &"BaseJoinExecutor<S>")
            .field("single_key_hash_table", &self.single_key_hash_table)
            .field("multi_key_hash_table", &self.multi_key_hash_table)
            .field("use_multi_key", &self.use_multi_key)
            .finish()
    }
}

impl<S: StorageEngine> InnerJoinExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<String>,
        probe_keys: Vec<String>,
        col_names: Vec<String>,
    ) -> Self {
        let use_multi_key = hash_keys.len() > 1;
        Self {
            base_executor: BaseJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
            single_key_hash_table: None,
            multi_key_hash_table: None,
            use_multi_key,
        }
    }

    /// 执行单键内连接
    fn execute_single_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 解析键索引
        let left_key_idx = self.base_executor.get_hash_keys()[0]
            .parse::<usize>()
            .map_err(|_| QueryError::ExecutionError("无效的左键索引".to_string()))?;
        let right_key_idx = self.base_executor.get_probe_keys()[0]
            .parse::<usize>()
            .map_err(|_| QueryError::ExecutionError("无效的右键索引".to_string()))?;

        // 决定是否交换左右输入以优化性能
        let (build_dataset, probe_dataset, build_key_idx, probe_key_idx, exchange) = if self
            .base_executor
            .should_exchange(left_dataset.rows.len(), right_dataset.rows.len())
        {
            // 交换：右表作为构建表，左表作为探测表
            (
                right_dataset,
                left_dataset,
                right_key_idx,
                left_key_idx,
                true,
            )
        } else {
            // 不交换：左表作为构建表，右表作为探测表
            (
                left_dataset,
                right_dataset,
                left_key_idx,
                right_key_idx,
                false,
            )
        };

        // 构建哈希表
        let hash_table = HashTableBuilder::build_single_key_table(build_dataset, build_key_idx)
            .map_err(|e| QueryError::ExecutionError(format!("构建哈希表失败: {}", e)))?;

        // 探测哈希表
        let probe_results =
            HashTableProbe::probe_single_key(&hash_table, probe_dataset, probe_key_idx);

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        for (probe_row, matching_rows) in probe_results {
            for build_row in matching_rows {
                let new_row = if exchange {
                    // 交换了，探测行是左，构建行是右
                    self.base_executor.new_row(probe_row.clone(), build_row)
                } else {
                    // 未交换，构建行是左，探测行是右
                    self.base_executor.new_row(build_row, probe_row.clone())
                };
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }

    /// 执行多键内连接
    fn execute_multi_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 解析键索引
        let mut left_key_indices = Vec::new();
        let mut right_key_indices = Vec::new();

        for key in self.base_executor.get_hash_keys() {
            let idx = key
                .parse::<usize>()
                .map_err(|_| QueryError::ExecutionError("无效的左键索引".to_string()))?;
            left_key_indices.push(idx);
        }

        for key in self.base_executor.get_probe_keys() {
            let idx = key
                .parse::<usize>()
                .map_err(|_| QueryError::ExecutionError("无效的右键索引".to_string()))?;
            right_key_indices.push(idx);
        }

        // 决定是否交换左右输入以优化性能
        let (build_dataset, probe_dataset, build_key_indices, probe_key_indices, exchange) = if self
            .base_executor
            .should_exchange(left_dataset.rows.len(), right_dataset.rows.len())
        {
            // 交换：右表作为构建表，左表作为探测表
            (
                right_dataset,
                left_dataset,
                &right_key_indices,
                &left_key_indices,
                true,
            )
        } else {
            // 不交换：左表作为构建表，右表作为探测表
            (
                left_dataset,
                right_dataset,
                &left_key_indices,
                &right_key_indices,
                false,
            )
        };

        // 构建哈希表
        let hash_table = HashTableBuilder::build_multi_key_table(build_dataset, build_key_indices)
            .map_err(|e| QueryError::ExecutionError(format!("构建多键哈希表失败: {}", e)))?;

        // 探测哈希表
        let probe_results =
            HashTableProbe::probe_multi_key(&hash_table, probe_dataset, probe_key_indices);

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        for (probe_row, matching_rows) in probe_results {
            for build_row in matching_rows {
                let new_row = if exchange {
                    // 交换了，探测行是左，构建行是右
                    self.base_executor.new_row(probe_row.clone(), build_row)
                } else {
                    // 未交换，构建行是左，探测行是右
                    self.base_executor.new_row(build_row, probe_row.clone())
                };
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for InnerJoinExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 检查输入数据集
        let (left_dataset, right_dataset) = self
            .base_executor
            .check_input_datasets()
            .map_err(DBError::from)?;

        // 处理空集情况
        if left_dataset.rows.is_empty() || right_dataset.rows.is_empty() {
            let empty_result = DataSet {
                col_names: self.base_executor.get_col_names().clone(),
                rows: Vec::new(),
            };
            return Ok(ExecutionResult::Values(vec![Value::DataSet(empty_result)]));
        }

        // 根据键的数量选择连接算法
        let result = if self.use_multi_key {
            self.execute_multi_key_join(&left_dataset, &right_dataset)
                .map_err(DBError::from)?
        } else {
            self.execute_single_key_join(&left_dataset, &right_dataset)
                .map_err(DBError::from)?
        };

        Ok(ExecutionResult::Values(vec![Value::DataSet(result)]))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for InnerJoinExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化资源
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        self.single_key_hash_table = None;
        self.multi_key_hash_table = None;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base_executor.get_base().is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for InnerJoinExecutor<S> {
    fn id(&self) -> usize {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        &self.base_executor.get_base().name
    }

    fn description(&self) -> &str {
        &self.base_executor.get_base().description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for InnerJoinExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base_executor.get_base().storage
    }
}

/// 哈希内连接执行器（并行版本）
#[derive(Debug)]
pub struct HashInnerJoinExecutor<S: StorageEngine> {
    inner: InnerJoinExecutor<S>,
}

impl<S: StorageEngine> HashInnerJoinExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<String>,
        probe_keys: Vec<String>,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            inner: InnerJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for HashInnerJoinExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 目前与普通内连接相同，后续可以添加并行处理逻辑
        self.inner.execute().await
    }
}

impl<S: StorageEngine> ExecutorLifecycle for HashInnerJoinExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for HashInnerJoinExecutor<S> {
    fn id(&self) -> usize {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "HashInnerJoinExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for HashInnerJoinExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.inner.storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    // 模拟存储引擎
    struct MockStorage;

    impl crate::storage::StorageEngine for MockStorage {
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
    }

    #[tokio::test]
    async fn test_inner_join_single_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = InnerJoinExecutor::new(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec!["0".to_string()], // 左表第0列作为键
            vec!["0".to_string()], // 右表第0列作为键
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        // 设置执行上下文
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "age".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::Int(25)],
                vec![Value::Int(3), Value::Int(35)],
            ],
        };

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行连接
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                println!("连接结果: {}个值", values.len());
                if let Some(Value::DataSet(dataset)) = values.first() {
                    println!("数据集行数: {}", dataset.rows.len());
                    for (i, row) in dataset.rows.iter().enumerate() {
                        println!("行{}: {:?}", i, row);
                    }
                    assert_eq!(dataset.rows.len(), 1); // 只有一个匹配
                    assert_eq!(
                        dataset.rows[0],
                        vec![
                            Value::Int(1),
                            Value::String("Alice".to_string()),
                            Value::Int(25),
                        ]
                    );
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }
}
