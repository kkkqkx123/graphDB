//! 左外连接执行器实现
//!
//! 实现基于哈希的左外连接算法，支持单键和多键连接

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet, NullType};
use crate::storage::StorageEngine;
use crate::query::executor::base::{Executor, ExecutionResult};
use crate::query::QueryError;
use crate::query::executor::data_processing::join::base_join::{BaseJoinExecutor, JoinOperation};
use crate::query::executor::data_processing::join::hash_table::{HashTableBuilder, HashTableProbe, SingleKeyHashTable, MultiKeyHashTable};

/// 左外连接执行器
pub struct LeftJoinExecutor<S: StorageEngine> {
    base_executor: BaseJoinExecutor<S>,
    /// 右侧数据集的列数（用于填充NULL值）
    right_col_size: usize,
    /// 哈希表（用于单键连接）
    single_key_hash_table: Option<SingleKeyHashTable>,
    /// 多键哈希表（用于多键连接）
    multi_key_hash_table: Option<MultiKeyHashTable>,
    /// 是否使用多键连接
    use_multi_key: bool,
}

impl<S: StorageEngine> LeftJoinExecutor<S> {
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
                id,
                storage,
                left_var,
                right_var,
                hash_keys,
                probe_keys,
                col_names,
            ),
            right_col_size: 0,
            single_key_hash_table: None,
            multi_key_hash_table: None,
            use_multi_key,
        }
    }

    /// 执行单键左外连接
    fn execute_single_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 记录右侧数据集的列数
        self.right_col_size = right_dataset.col_names.len();

        // 左外连接总是以左表为驱动表，右表构建哈希表
        let build_dataset = right_dataset;
        let probe_dataset = left_dataset;
        let build_key_idx = 0; // 右表键索引
        let probe_key_idx = 0; // 左表键索引

        // 构建哈希表
        let hash_table = HashTableBuilder::build_single_key_table(build_dataset, build_key_idx)
            .map_err(|e| QueryError::ExecutionError(format!("构建哈希表失败: {}", e)))?;

        // 探测哈希表
        let probe_results = HashTableProbe::probe_single_key(&hash_table, probe_dataset, probe_key_idx);

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        // 记录已匹配的左表行索引
        let mut matched_rows = std::collections::HashSet::new();

        // 处理匹配的行
        for (probe_row, matching_rows) in probe_results {
            matched_rows.insert(probe_row.clone()); // 标记为已匹配
            
            for build_row in matching_rows {
                let new_row = self.base_executor.new_row(probe_row.clone(), build_row);
                result.rows.push(new_row);
            }
        }

        // 处理未匹配的左表行（填充NULL）
        for probe_row in &probe_dataset.rows {
            if !matched_rows.contains(probe_row) {
                let mut new_row = probe_row.clone();
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
    ) -> Result<DataSet, QueryError> {
        // 记录右侧数据集的列数
        self.right_col_size = right_dataset.col_names.len();

        // 解析键索引
        let mut left_key_indices = Vec::new();
        let mut right_key_indices = Vec::new();

        for key in self.base_executor.get_hash_keys() {
            let idx = key.parse::<usize>()
                .map_err(|_| QueryError::ExecutionError("无效的左键索引".to_string()))?;
            left_key_indices.push(idx);
        }

        for key in self.base_executor.get_probe_keys() {
            let idx = key.parse::<usize>()
                .map_err(|_| QueryError::ExecutionError("无效的右键索引".to_string()))?;
            right_key_indices.push(idx);
        }

        // 左外连接总是以左表为驱动表，右表构建哈希表
        let build_dataset = right_dataset;
        let probe_dataset = left_dataset;
        let build_key_indices = &right_key_indices;
        let probe_key_indices = &left_key_indices;

        // 构建哈希表
        let hash_table = HashTableBuilder::build_multi_key_table(build_dataset, build_key_indices)
            .map_err(|e| QueryError::ExecutionError(format!("构建多键哈希表失败: {}", e)))?;

        // 探测哈希表
        let probe_results = HashTableProbe::probe_multi_key(&hash_table, probe_dataset, probe_key_indices);

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        // 记录已匹配的左表行索引
        let mut matched_rows = std::collections::HashSet::new();

        // 处理匹配的行
        for (probe_row, matching_rows) in probe_results {
            matched_rows.insert(probe_row.clone()); // 标记为已匹配
            
            for build_row in matching_rows {
                let new_row = self.base_executor.new_row(probe_row.clone(), build_row);
                result.rows.push(new_row);
            }
        }

        // 处理未匹配的左表行（填充NULL）
        for probe_row in &probe_dataset.rows {
            if !matched_rows.contains(probe_row) {
                let mut new_row = probe_row.clone();
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
impl<S: StorageEngine + Send + 'static> Executor<S> for LeftJoinExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
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

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化资源
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        self.single_key_hash_table = None;
        self.multi_key_hash_table = None;
        Ok(())
    }

    fn id(&self) -> usize {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        &self.base_executor.get_base().name
    }
}

/// 哈希左外连接执行器（并行版本）
pub struct HashLeftJoinExecutor<S: StorageEngine> {
    inner: LeftJoinExecutor<S>,
}

impl<S: StorageEngine> HashLeftJoinExecutor<S> {
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
            inner: LeftJoinExecutor::new(
                id,
                storage,
                left_var,
                right_var,
                hash_keys,
                probe_keys,
                col_names,
            ),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for HashLeftJoinExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 目前与普通左连接相同，后续可以添加并行处理逻辑
        self.inner.execute().await
    }

    fn open(&mut self) -> Result<(), QueryError> {
        self.inner.open()
    }

    fn close(&mut self) -> Result<(), QueryError> {
        self.inner.close()
    }

    fn id(&self) -> usize {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "HashLeftJoinExecutor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use std::collections::HashMap;
    use crate::query::executor::base::ExecutionContext;

    // 模拟存储引擎
    struct MockStorage;
    impl crate::storage::StorageEngine for MockStorage {}

    #[tokio::test]
    async fn test_left_join_single_key() {
        let storage = Arc::new(Mutex::new(MockStorage));
        
        // 创建执行器
        let mut executor = LeftJoinExecutor::new(
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

        executor.base_executor.base.context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.base.context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行连接
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 3); // 三行结果（包括未匹配的）
                    
                    // 第一行：Alice匹配
                    assert_eq!(dataset.rows[0], vec![
                        Value::Int(1),
                        Value::String("Alice".to_string()),
                        Value::Int(25),
                    ]);
                    
                    // 第二行：Bob匹配
                    assert_eq!(dataset.rows[1], vec![
                        Value::Int(2),
                        Value::String("Bob".to_string()),
                        Value::Int(30),
                    ]);
                    
                    // 第三行：Charlie未匹配，age为NULL
                    assert_eq!(dataset.rows[2][0], Value::Int(3));
                    assert_eq!(dataset.rows[2][1], Value::String("Charlie".to_string()));
                    assert_eq!(dataset.rows[2][2], Value::Null(NullType::Null));
                } else {
                    panic!("期望DataSet结果");
                }
            },
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
            vec!["0".to_string()],
            vec!["0".to_string()],
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

        executor.base_executor.base.context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.base.context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行连接
        let result = executor.execute().await.unwrap();

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
            },
            _ => panic!("期望Values结果"),
        }
    }
}