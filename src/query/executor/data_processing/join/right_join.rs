use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Expression, Value};
use crate::query::executor::data_processing::join::{
    base_join::BaseJoinExecutor,
    hash_table::{build_hash_table, extract_key_values, JoinKey},
};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 右外连接执行器
/// 实现右外连接操作：保留右表的所有记录，左表没有匹配的部分用NULL填充
pub struct RightJoinExecutor<S: StorageEngine + Send + 'static> {
    base: BaseJoinExecutor<S>,
}

impl<S: StorageEngine + Send + 'static> RightJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
        output_columns: Vec<String>,
    ) -> Self {
        let hash_keys: Vec<Expression> = left_keys.into_iter().map(Expression::Variable).collect();
        let probe_keys: Vec<Expression> =
            right_keys.into_iter().map(Expression::Variable).collect();
        Self {
            base: BaseJoinExecutor::with_description(
                id,
                storage,
                left_var,
                right_var,
                hash_keys,
                probe_keys,
                output_columns,
                "Right join executor - performs right outer join".to_string(),
            ),
        }
    }

    /// 执行右外连接
    async fn execute_right_join(&mut self) -> DBResult<ExecutionResult> {
        // 获取左右输入结果
        let left_result = self
            .base
            .base
            .context
            .get_result(self.base.left_var())
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Left input variable '{}' not found",
                    self.base.left_var()
                )))
            })?
            .clone();

        let right_result = self
            .base
            .base
            .context
            .get_result(self.base.right_var())
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Right input variable '{}' not found",
                    self.base.right_var()
                )))
            })?
            .clone();

        // 转换为数据集
        let left_dataset = match left_result {
            ExecutionResult::DataSet(ds) => ds,
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Left input must be a DataSet".to_string(),
                    ),
                ))
            }
        };

        let right_dataset = match right_result {
            ExecutionResult::DataSet(ds) => ds,
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Right input must be a DataSet".to_string(),
                    ),
                ))
            }
        };

        // 预先构建列名到索引的映射
        let right_col_map: HashMap<&str, usize> = right_dataset
            .col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 构建左表哈希表：以左表连接键作为键，行索引作为值
        let left_hash_table =
            build_hash_table(&left_dataset, self.base.hash_keys()).map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Failed to build hash table: {}",
                    e
                )))
            })?;

        // 构建结果数据集
        let mut result_dataset = DataSet {
            col_names: self.base.col_names().clone(),
            rows: Vec::new(),
        };

        // 处理右表的每一行
        for (_right_idx, right_row) in right_dataset.rows.iter().enumerate() {
            let right_key_parts = extract_key_values(
                right_row,
                &right_dataset.col_names,
                self.base.probe_keys(),
                &right_col_map,
            );

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
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.execute_right_join().await
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for RightJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.base.get_storage()
    }
}
