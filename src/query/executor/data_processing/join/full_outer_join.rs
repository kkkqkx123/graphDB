use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Expression, Value};
use crate::query::executor::data_processing::join::{
    base_join::BaseJoinExecutor,
    hash_table::{build_hash_table, extract_key_values, JoinKey},
};
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

/// 全外连接执行器
/// 实现全外连接操作：保留左右表的所有记录，没有匹配的部分用NULL填充
pub struct FullOuterJoinExecutor<S: StorageClient + Send + 'static> {
    base: BaseJoinExecutor<S>,
}

impl<S: StorageClient + Send + 'static> FullOuterJoinExecutor<S> {
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
                "Full outer join executor - performs full outer join".to_string(),
            ),
        }
    }

    fn execute_full_outer_join(&mut self) -> DBResult<ExecutionResult> {
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
        let left_col_map: HashMap<&str, usize> = left_dataset
            .col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let _right_col_map: HashMap<&str, usize> = right_dataset
            .col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 构建左表哈希表：以左表连接键作为键，行索引作为值
        let left_hash_table_indices = build_hash_table(&left_dataset, self.base.hash_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Failed to build left hash table: {}",
                    e
                )))
            })?;

        // 转换为带匹配标志的哈希表
        let mut left_hash_table: HashMap<JoinKey, Vec<(usize, bool)>> = HashMap::new();
        for (key, indices) in left_hash_table_indices {
            left_hash_table
                .entry(key)
                .or_insert_with(Vec::new)
                .extend(indices.into_iter().map(|idx| (idx, false)));
        }

        // 构建右表哈希表：以右表连接键作为键，行索引作为值
        let right_hash_table_indices = build_hash_table(&right_dataset, self.base.probe_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Failed to build right hash table: {}",
                    e
                )))
            })?;

        // 转换为带匹配标志的哈希表
        let mut right_hash_table: HashMap<JoinKey, Vec<(usize, bool)>> = HashMap::new();
        for (key, indices) in right_hash_table_indices {
            right_hash_table
                .entry(key)
                .or_insert_with(Vec::new)
                .extend(indices.into_iter().map(|idx| (idx, false)));
        }

        // 构建结果数据集
        let mut result_dataset = DataSet {
            col_names: self.base.col_names().clone(),
            rows: Vec::new(),
        };

        // 处理左表的每一行
        for (_idx, row) in left_dataset.rows.iter().enumerate() {
            let key_parts = extract_key_values(
                row,
                &left_dataset.col_names,
                self.base.hash_keys(),
                &left_col_map,
            );

            let key = JoinKey::new(key_parts);

            // 如果右表有匹配的行
            if let Some(right_indices) = right_hash_table.get_mut(&key) {
                for (right_idx, ref mut matched) in right_indices {
                    *matched = true; // 标记为已匹配
                    if *right_idx < right_dataset.rows.len() {
                        let right_row = &right_dataset.rows[*right_idx];
                        let mut joined_row = row.clone();
                        joined_row.extend_from_slice(right_row);
                        result_dataset.rows.push(joined_row);
                    }
                }
            } else {
                // 没有匹配的右表行，用NULL填充右表部分
                let mut null_right_row = Vec::new();
                for _ in 0..right_dataset.col_names.len() {
                    null_right_row.push(Value::Null(crate::core::value::NullType::Null));
                }

                let mut joined_row = row.clone();
                joined_row.extend_from_slice(&null_right_row);
                result_dataset.rows.push(joined_row);
            }
        }

        // 添加右表中没有匹配的行
        for (key, right_entries) in &right_hash_table {
            for (right_idx, matched) in right_entries {
                if !matched {
                    // 找对应的左表键，如果存在未处理的左表行
                    if *right_idx < right_dataset.rows.len() {
                        let right_row = &right_dataset.rows[*right_idx];

                        // 检查是否有左表行匹配当前右表行的键
                        let has_left_match =
                            left_hash_table.get(key).map_or(false, |left_entries| {
                                left_entries.iter().any(|(_left_idx, matched)| !matched)
                            });

                        if !has_left_match {
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
                }
            }
        }

        Ok(ExecutionResult::DataSet(result_dataset))
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for FullOuterJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.execute_full_outer_join()
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

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_base().get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_base_mut().get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for FullOuterJoinExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .get_base()
            .storage
            .as_ref()
            .expect("FullOuterJoinExecutor storage should be set")
    }
}
