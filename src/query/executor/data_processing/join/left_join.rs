//! 左外连接执行器实现
//!
//! 实现基于哈希的左外连接算法，支持单键和多键连接

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Expression, NullType, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::data_processing::join::{
    base_join::BaseJoinExecutor, hash_table::{build_hash_table, extract_key_values, JoinKey},
};
use crate::query::executor::traits::{
    ExecutionResult, Executor, HasStorage,
};
use crate::storage::StorageEngine;

/// 左外连接执行器
pub struct LeftJoinExecutor<S: StorageEngine> {
    base_executor: BaseJoinExecutor<S>,
    /// 右侧数据集的列数（用于填充NULL值）
    right_col_size: usize,
    /// 是否使用多键连接
    use_multi_key: bool,
}

impl<S: StorageEngine> LeftJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        let use_multi_key = hash_keys.len() > 1;
        Self {
            base_executor: BaseJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
            right_col_size: 0,
            use_multi_key,
        }
    }

    /// 执行单键左外连接
    fn execute_single_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> DBResult<DataSet> {
        // 记录右侧数据集的列数
        self.right_col_size = right_dataset.col_names.len();

        // 左外连接总是以左表为驱动表，右表构建哈希表
        let build_dataset = right_dataset;

        // 构建哈希表
        let hash_table =
            build_hash_table(build_dataset, self.base_executor.get_probe_keys()).map_err(
                |e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "构建哈希表失败: {}",
                        e
                    )))
                },
            )?;

        // 构建列名到索引的映射
        let left_col_map: std::collections::HashMap<&str, usize> = left_dataset
            .col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        // 记录已匹配的左表行索引
        let mut matched_rows = std::collections::HashSet::new();

        // 处理左表的每一行
        for left_row in &left_dataset.rows {
            let left_key_parts = extract_key_values(
                left_row,
                &left_dataset.col_names,
                self.base_executor.get_hash_keys(),
                &left_col_map,
            );

            let left_key = JoinKey::new(left_key_parts);

            // 查找匹配的右表行
            if let Some(right_indices) = hash_table.get(&left_key) {
                matched_rows.insert(left_row.clone()); // 标记为已匹配

                for &right_idx in right_indices {
                    if right_idx < build_dataset.rows.len() {
                        let right_row = &build_dataset.rows[right_idx];
                        let new_row = self.base_executor.new_row(left_row.clone(), right_row.clone());
                        result.rows.push(new_row);
                    }
                }
            }
        }

        // 处理未匹配的左表行（填充NULL）
        for left_row in &left_dataset.rows {
            if !matched_rows.contains(left_row) {
                let mut new_row = left_row.clone();
                // 为右侧列填充NULL值
                for _ in 0..self.right_col_size {
                    new_row.push(Value::Null(NullType::Null));
                }
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }

    /// 执行多键左外连接
    fn execute_multi_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> DBResult<DataSet> {
        // 记录右侧数据集的列数
        self.right_col_size = right_dataset.col_names.len();

        // 左外连接总是以左表为驱动表，右表构建哈希表
        let build_dataset = right_dataset;

        // 构建哈希表
        let hash_table =
            build_hash_table(build_dataset, self.base_executor.get_probe_keys()).map_err(
                |e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "构建多键哈希表失败: {}",
                        e
                    )))
                },
            )?;

        // 构建列名到索引的映射
        let left_col_map: std::collections::HashMap<&str, usize> = left_dataset
            .col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        // 记录已匹配的左表行索引
        let mut matched_rows = std::collections::HashSet::new();

        // 处理左表的每一行
        for left_row in &left_dataset.rows {
            let left_key_parts = extract_key_values(
                left_row,
                &left_dataset.col_names,
                self.base_executor.get_hash_keys(),
                &left_col_map,
            );

            let left_key = JoinKey::new(left_key_parts);

            // 查找匹配的右表行
            if let Some(right_indices) = hash_table.get(&left_key) {
                matched_rows.insert(left_row.clone()); // 标记为已匹配

                for &right_idx in right_indices {
                    if right_idx < build_dataset.rows.len() {
                        let right_row = &build_dataset.rows[right_idx];
                        let new_row = self.base_executor.new_row(left_row.clone(), right_row.clone());
                        result.rows.push(new_row);
                    }
                }
            }
        }

        // 处理未匹配的左表行（填充NULL）
        for left_row in &left_dataset.rows {
            if !matched_rows.contains(left_row) {
                let mut new_row = left_row.clone();
                // 为右侧列填充NULL值
                for _ in 0..self.right_col_size {
                    new_row.push(Value::Null(NullType::Null));
                }
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }

    /// 创建填充NULL的右侧行
    
    fn create_null_right_row(&self) -> Vec<Value> {
        vec![Value::Null(NullType::Null); self.right_col_size]
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for LeftJoinExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 检查输入数据集
        let (left_dataset, right_dataset) = self.base_executor.check_input_datasets()?;

        // 处理空集情况
        if left_dataset.rows.is_empty() {
            let empty_result = DataSet {
                col_names: self.base_executor.get_col_names().clone(),
                rows: Vec::new(),
            };
            return Ok(ExecutionResult::Values(vec![Value::DataSet(empty_result)]));
        }

        if right_dataset.rows.is_empty() {
            // 右表为空，左表所有行都填充NULL
            let mut result = DataSet::new();
            result.col_names = self.base_executor.get_col_names().clone();
            self.right_col_size = right_dataset.col_names.len();

            for left_row in &left_dataset.rows {
                let mut new_row = left_row.clone();
                for _ in 0..self.right_col_size {
                    new_row.push(Value::Null(NullType::Null));
                }
                result.rows.push(new_row);
            }

            return Ok(ExecutionResult::Values(vec![Value::DataSet(result)]));
        }

        // 根据键的数量选择连接算法
        let result = if self.use_multi_key {
            self.execute_multi_key_join(&left_dataset, &right_dataset)?
        } else {
            self.execute_single_key_join(&left_dataset, &right_dataset)?
        };

        Ok(ExecutionResult::Values(vec![Value::DataSet(result)]))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for LeftJoinExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化资源
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base_executor.get_base().is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for LeftJoinExecutor<S> {
    fn id(&self) -> i64 {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        &self.base_executor.get_base().name
    }

    fn description(&self) -> &str {
        &self.base_executor.get_base().description
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for LeftJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base_executor.get_base().storage.as_ref().expect("LeftJoinExecutor storage should be set")
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for LeftJoinExecutor<S> {
}

/// 哈希左外连接执行器（并行版本）
pub struct HashLeftJoinExecutor<S: StorageEngine> {
    inner: LeftJoinExecutor<S>,
}

impl<S: StorageEngine> HashLeftJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            inner: LeftJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for HashLeftJoinExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 目前与普通左连接相同，后续可以添加并行处理逻辑
        self.inner.execute().await
    }
}

impl<S: StorageEngine> ExecutorLifecycle for HashLeftJoinExecutor<S> {
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

impl<S: StorageEngine> ExecutorMetadata for HashLeftJoinExecutor<S> {
    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "HashLeftJoinExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for HashLeftJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for HashLeftJoinExecutor<S> {
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
    async fn test_left_join_single_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = LeftJoinExecutor::new(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![crate::core::Expression::Variable("0".to_string())], // 左表第0列作为键
            vec![crate::core::Expression::Variable("0".to_string())], // 右表第0列作为键
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        // 设置执行上下文
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
                vec![Value::Int(3), Value::String("Charlie".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "age".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::Int(25)],
                vec![Value::Int(2), Value::Int(30)],
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
        let result = executor.execute().await.expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 3); // 三行结果（包括未匹配的）

                    // 第一行：Alice匹配
                    assert_eq!(
                        dataset.rows[0],
                        vec![
                            Value::Int(1),
                            Value::String("Alice".to_string()),
                            Value::Int(25),
                        ]
                    );

                    // 第二行：Bob匹配
                    assert_eq!(
                        dataset.rows[1],
                        vec![
                            Value::Int(2),
                            Value::String("Bob".to_string()),
                            Value::Int(30),
                        ]
                    );

                    // 第三行：Charlie未匹配，age为NULL
                    assert_eq!(dataset.rows[2][0], Value::Int(3));
                    assert_eq!(dataset.rows[2][1], Value::String("Charlie".to_string()));
                    assert_eq!(dataset.rows[2][2], Value::Null(NullType::Null));
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_left_join_empty_right() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = LeftJoinExecutor::new(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![Expression::Variable("0".to_string())],
            vec![Expression::Variable("0".to_string())],
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
            rows: Vec::new(), // 空右表
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
        let result = executor.execute().await.expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 2); // 两行结果，都填充NULL

                    // 所有行的age都应该是NULL
                    for row in &dataset.rows {
                        assert_eq!(row[2], Value::Null(NullType::Null));
                    }
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }
}
