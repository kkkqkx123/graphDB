use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::query::executor::base::{Executor, ExecutionResult};
use crate::query::executor::data_processing::join::{
    base_join::BaseJoinExecutor,
    hash_table::JoinKey,
};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 右外连接执行器
/// 实现右外连接操作：保留右表的所有记录，左表没有匹配的部分用NULL填充
pub struct RightJoinExecutor<S: StorageEngine + Send + 'static> {
    base: BaseJoinExecutor<S>,
}

impl<S: StorageEngine + Send + 'static> RightJoinExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
        output_columns: Vec<String>,
    ) -> Self {
        Self {
            base: BaseJoinExecutor::new(id, storage, left_var, right_var, left_keys, right_keys, output_columns),
        }
    }

    /// 执行右外连接
    async fn execute_right_join(&mut self) -> Result<ExecutionResult, QueryError> {
        // 获取左右输入结果
        let left_result = self.base.base.context.get_result(self.base.left_var())
            .ok_or_else(|| QueryError::ExecutionError(format!("Left input variable '{}' not found", self.base.left_var())))?
            .clone();

        let right_result = self.base.base.context.get_result(self.base.right_var())
            .ok_or_else(|| QueryError::ExecutionError(format!("Right input variable '{}' not found", self.base.right_var())))?
            .clone();

        // 转换为数据集
        let left_dataset = match left_result {
            ExecutionResult::DataSet(ds) => ds,
            _ => return Err(QueryError::ExecutionError("Left input must be a DataSet".to_string())),
        };

        let right_dataset = match right_result {
            ExecutionResult::DataSet(ds) => ds,
            _ => return Err(QueryError::ExecutionError("Right input must be a DataSet".to_string())),
        };

        // 构建左表哈希表：以左表连接键作为键，行索引作为值
        let mut left_hash_table: HashMap<JoinKey, Vec<usize>> = HashMap::new();

        for (idx, row) in left_dataset.rows.iter().enumerate() {
            let mut key_parts = Vec::new();

            // 根据连接键提取值
            for key_idx in 0..self.base.hash_keys().len() {
                if let Some(key_pos) = left_dataset.col_names.iter().position(|r| r == &self.base.hash_keys()[key_idx]) {
                    if key_pos < row.len() {
                        key_parts.push(row[key_pos].clone());
                    } else {
                        key_parts.push(Value::Null(crate::core::value::NullType::Null));
                    }
                } else if let Ok(key_pos) = self.base.hash_keys()[key_idx].parse::<usize>() {
                    if key_pos < row.len() {
                        key_parts.push(row[key_pos].clone());
                    } else {
                        key_parts.push(Value::Null(crate::core::value::NullType::Null));
                    }
                } else {
                    key_parts.push(Value::Null(crate::core::value::NullType::Null));
                }
            }

            let key = JoinKey::new(key_parts);
            left_hash_table.entry(key).or_insert_with(Vec::new).push(idx);
        }

        // 构建结果数据集
        let mut result_dataset = DataSet {
            col_names: self.base.col_names().clone(),
            rows: Vec::new(),
        };

        // 处理右表的每一行
        for (right_idx, right_row) in right_dataset.rows.iter().enumerate() {
            let mut right_key_parts = Vec::new();

            // 根据连接键提取值
            for key_idx in 0..self.base.probe_keys().len() {
                if let Some(key_pos) = right_dataset.col_names.iter().position(|r| r == &self.base.probe_keys()[key_idx]) {
                    if key_pos < right_row.len() {
                        right_key_parts.push(right_row[key_pos].clone());
                    } else {
                        right_key_parts.push(Value::Null(crate::core::value::NullType::Null));
                    }
                } else if let Ok(key_pos) = self.base.probe_keys()[key_idx].parse::<usize>() {
                    if key_pos < right_row.len() {
                        right_key_parts.push(right_row[key_pos].clone());
                    } else {
                        right_key_parts.push(Value::Null(crate::core::value::NullType::Null));
                    }
                } else {
                    right_key_parts.push(Value::Null(crate::core::value::NullType::Null));
                }
            }

            let right_key = JoinKey::new(right_key_parts);

            // 查找匹配的左表行
            if let Some(left_indices) = left_hash_table.get(&right_key) {
                // 有匹配的左表行，进行连接
                for &left_idx in left_indices {
                    if left_idx < left_dataset.rows.len() {
                        let left_row = &left_dataset.rows[left_idx];
                        let mut joined_row = left_row.clone();
                        joined_row.extend_from_slice(right_row);
                        result_dataset.rows.push(joined_row);
                    }
                }
            } else {
                // 没有匹配的左表行，用NULL填充左表部分
                let mut null_left_row = Vec::new();
                for _ in 0..left_dataset.col_names.len() {
                    null_left_row.push(Value::Null(crate::core::value::NullType::Null));
                }

                let mut joined_row = null_left_row;
                joined_row.extend_from_slice(right_row);
                result_dataset.rows.push(joined_row);
            }
        }

        Ok(ExecutionResult::DataSet(result_dataset))
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for RightJoinExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        self.execute_right_join().await
    }

    fn open(&mut self) -> Result<(), QueryError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }
}