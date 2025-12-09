//! Join执行器的基础结构和公共功能
//!
//! 提供所有join操作的基础实现，包括哈希表构建、探测等核心功能

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, ExecutionResult};
use crate::query::executor::data_processing::join::hash_table::JoinKey;
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// Join执行器的基础结构
pub struct BaseJoinExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    /// 左侧输入变量名
    left_var: String,
    /// 右侧输入变量名
    right_var: String,
    /// 连接键表达式列表
    hash_keys: Vec<String>,
    /// 探测键表达式列表
    probe_keys: Vec<String>,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否交换左右输入（优化用）
    exchange: bool,
    /// 右侧输出列索引（用于自然连接）
    rhs_output_col_idxs: Option<Vec<usize>>,
}

impl<S: StorageEngine> BaseJoinExecutor<S> {
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
            base: BaseExecutor::new(id, "BaseJoinExecutor".to_string(), storage),
            left_var,
            right_var,
            hash_keys,
            probe_keys,
            col_names,
            exchange: false,
            rhs_output_col_idxs: None,
        }
    }

    /// 检查输入数据集
    pub fn check_input_datasets(&mut self) -> Result<(DataSet, DataSet), QueryError> {
        // 从执行上下文获取左右输入数据集
        let left_result = self
            .base
            .context
            .get_result(&self.left_var)
            .ok_or_else(|| {
                QueryError::ExecutionError(format!("找不到左输入变量: {}", self.left_var))
            })?;

        let right_result = self
            .base
            .context
            .get_result(&self.right_var)
            .ok_or_else(|| {
                QueryError::ExecutionError(format!("找不到右输入变量: {}", self.right_var))
            })?;

        let left_dataset = match left_result {
            ExecutionResult::Values(values) => {
                // 将Values转换为DataSet
                DataSet {
                    col_names: vec![],
                    rows: vec![values.clone()],
                }
            }
            _ => {
                return Err(QueryError::ExecutionError(
                    "左输入不是有效的数据集".to_string(),
                ))
            }
        };

        let right_dataset = match right_result {
            ExecutionResult::Values(values) => {
                // 将Values转换为DataSet
                DataSet {
                    col_names: vec![],
                    rows: vec![values.clone()],
                }
            }
            _ => {
                return Err(QueryError::ExecutionError(
                    "右输入不是有效的数据集".to_string(),
                ))
            }
        };

        Ok((left_dataset, right_dataset))
    }

    /// 构建单键哈希表
    pub fn build_single_key_hash_table(
        hash_key: &str,
        dataset: &DataSet,
        hash_table: &mut HashMap<Value, Vec<Vec<Value>>>,
    ) -> Result<(), QueryError> {
        for row in &dataset.rows {
            // 简化实现：假设hash_key是列索引
            let key_idx = hash_key
                .parse::<usize>()
                .map_err(|_| QueryError::ExecutionError("无效的键索引".to_string()))?;

            if key_idx < row.len() {
                let key = row[key_idx].clone();
                hash_table
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(row.clone());
            }
        }
        Ok(())
    }

    /// 构建多键哈希表
    pub fn build_multi_key_hash_table(
        hash_keys: &[String],
        dataset: &DataSet,
        hash_table: &mut HashMap<JoinKey, Vec<Vec<Value>>>,
    ) -> Result<(), QueryError> {
        for row in &dataset.rows {
            let mut key_values = Vec::new();
            for hash_key in hash_keys {
                let key_idx = hash_key
                    .parse::<usize>()
                    .map_err(|_| QueryError::ExecutionError("无效的键索引".to_string()))?;

                if key_idx < row.len() {
                    key_values.push(row[key_idx].clone());
                } else {
                    return Err(QueryError::ExecutionError("键索引超出范围".to_string()));
                }
            }

            let join_key = JoinKey::new(key_values);
            hash_table
                .entry(join_key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }
        Ok(())
    }

    /// 创建新行（连接左右两行）
    pub fn new_row(&self, left_row: Vec<Value>, right_row: Vec<Value>) -> Vec<Value> {
        let mut new_row = left_row;

        if let Some(ref col_idxs) = self.rhs_output_col_idxs {
            // 自然连接：只添加非重复的右侧行
            for &idx in col_idxs {
                if idx < right_row.len() {
                    new_row.push(right_row[idx].clone());
                }
            }
        } else {
            // 普通连接：添加整个右侧行
            new_row.extend(right_row);
        }

        new_row
    }

    /// 决定是否交换左右输入以优化性能
    pub fn should_exchange(&self, left_size: usize, right_size: usize) -> bool {
        left_size > right_size
    }

    /// 计算右侧输出列索引（用于自然连接）
    pub fn calculate_rhs_output_col_idxs(
        &mut self,
        left_col_names: &[String],
        right_col_names: &[String],
    ) {
        let mut rhs_output_col_idxs = Vec::new();

        for (i, right_col) in right_col_names.iter().enumerate() {
            if !left_col_names.contains(right_col) {
                rhs_output_col_idxs.push(i);
            }
        }

        if !rhs_output_col_idxs.is_empty() && rhs_output_col_idxs.len() != right_col_names.len() {
            self.rhs_output_col_idxs = Some(rhs_output_col_idxs);
        }
    }

    /// 获取列名
    pub fn get_col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    /// 获取哈希键
    pub fn get_hash_keys(&self) -> &Vec<String> {
        &self.hash_keys
    }

    /// 获取探测键
    pub fn get_probe_keys(&self) -> &Vec<String> {
        &self.probe_keys
    }

    /// 获取基础执行器
    pub fn get_base(&self) -> &BaseExecutor<S> {
        &self.base
    }

    /// 获取可变的基础执行器
    pub fn get_base_mut(&mut self) -> &mut BaseExecutor<S> {
        &mut self.base
    }
}

/// Join操作的通用trait
pub trait JoinOperation<S: StorageEngine> {
    /// 执行join操作
    fn execute_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError>;
}

/// 内连接操作
pub struct InnerJoinOperation;

impl InnerJoinOperation {
    pub fn new() -> Self {
        Self
    }
}

impl<S: StorageEngine> JoinOperation<S> for InnerJoinOperation {
    fn execute_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 简化实现：执行基本的内连接
        let mut result = DataSet::new();

        // 这里应该实现具体的内连接逻辑
        // 暂时返回空结果集
        Ok(result)
    }
}

/// 左连接操作
pub struct LeftJoinOperation;

impl LeftJoinOperation {
    pub fn new() -> Self {
        Self
    }
}

impl<S: StorageEngine> JoinOperation<S> for LeftJoinOperation {
    fn execute_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 简化实现：执行基本的左连接
        let mut result = DataSet::new();

        // 这里应该实现具体的左连接逻辑
        // 暂时返回空结果集
        Ok(result)
    }
}

/// 笛卡尔积操作
pub struct CartesianProductOperation;

impl CartesianProductOperation {
    pub fn new() -> Self {
        Self
    }
}

impl<S: StorageEngine> JoinOperation<S> for CartesianProductOperation {
    fn execute_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        let mut result = DataSet::new();

        // 执行笛卡尔积
        for left_row in &left_dataset.rows {
            for right_row in &right_dataset.rows {
                let mut new_row = left_row.clone();
                new_row.extend(right_row.clone());
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }
}
