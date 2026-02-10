//! 左外连接执行器实现
//!
//! 实现基于哈希的左外连接算法，支持单键和多键连接

use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Expression, NullType, Value};
use crate::query::executor::data_processing::join::{
    base_join::BaseJoinExecutor,
    hash_table::{build_hash_table, extract_key_values, JoinKey},
};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 左外连接执行器
pub struct LeftJoinExecutor<S: StorageClient> {
    base_executor: BaseJoinExecutor<S>,
    /// 右侧数据集的列数（用于填充NULL值）
    right_col_size: usize,
    /// 是否使用多键连接
    use_multi_key: bool,
}

impl<S: StorageClient> LeftJoinExecutor<S> {
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
        let hash_table = build_hash_table(build_dataset, self.base_executor.get_probe_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建哈希表失败: {}",
                    e
                )))
            })?;

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
                        let new_row = self
                            .base_executor
                            .new_row(left_row.clone(), right_row.clone(), &left_dataset.col_names, &right_dataset.col_names);
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
        let hash_table = build_hash_table(build_dataset, self.base_executor.get_probe_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建多键哈希表失败: {}",
                    e
                )))
            })?;

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
                        let new_row = self
                            .base_executor
                            .new_row(left_row.clone(), right_row.clone(), &left_dataset.col_names, &right_dataset.col_names);
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
}

impl<S: StorageClient + Send + 'static> Executor<S> for LeftJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let (left_dataset, right_dataset) = self.base_executor.check_input_datasets()?;

        if left_dataset.rows.is_empty() {
            let empty_result = DataSet {
                col_names: self.base_executor.get_col_names().clone(),
                rows: Vec::new(),
            };
            return Ok(ExecutionResult::Values(vec![Value::DataSet(empty_result)]));
        }

        if right_dataset.rows.is_empty() {
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

        let result = if self.use_multi_key {
            self.execute_multi_key_join(&left_dataset, &right_dataset)?
        } else {
            self.execute_single_key_join(&left_dataset, &right_dataset)?
        };

        Ok(ExecutionResult::Values(vec![Value::DataSet(result)]))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base_executor.get_base().is_open()
    }

    fn id(&self) -> i64 {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        &self.base_executor.get_base().name
    }

    fn description(&self) -> &str {
        &self.base_executor.get_base().description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base_executor.get_base().get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base_executor.get_base_mut().get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for LeftJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base_executor
            .get_base()
            .storage
            .as_ref()
            .expect("LeftJoinExecutor storage should be set")
    }
}

/// 哈希左外连接执行器（并行版本）
pub struct HashLeftJoinExecutor<S: StorageClient> {
    inner: LeftJoinExecutor<S>,
}

impl<S: StorageClient> HashLeftJoinExecutor<S> {
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

impl<S: StorageClient + Send + 'static> Executor<S> for HashLeftJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.inner.execute()
    }

    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "HashLeftJoinExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.inner.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.inner.stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for HashLeftJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::storage::test_mock::MockStorage;

    #[tokio::test]
    async fn test_left_join_single_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = LeftJoinExecutor::new(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![crate::core::Expression::Variable("id".to_string())], // 左表id列作为键
            vec![crate::core::Expression::Variable("id".to_string())], // 右表id列作为键
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
