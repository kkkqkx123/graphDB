//! Join执行器的基础结构和公共功能
//!
//! 提供所有join操作的基础实现，包括哈希表构建、探测等核心功能

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Expression, Value};
use crate::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::data_processing::join::hash_table::JoinKey;
use crate::query::executor::data_processing::join::join_key_evaluator::JoinKeyEvaluator;
use crate::query::executor::traits::ExecutionResult;
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// Join执行器的基础结构
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    /// 左侧输入变量名
    left_var: String,
    /// 右侧输入变量名
    right_var: String,
    /// 连接键表达式列表
    hash_keys: Vec<Expression>,
    /// 探测键表达式列表
    probe_keys: Vec<Expression>,
    /// 输出列名
    col_names: Vec<String>,
    /// 描述
    description: String,
    /// 是否交换左右输入（优化用）
    exchange: bool,
    /// 右侧输出列索引（用于自然连接）
    rhs_output_col_idxs: Option<Vec<usize>>,
}

impl<S: StorageEngine> BaseJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        Self::with_description(
            id,
            storage,
            left_var,
            right_var,
            hash_keys,
            probe_keys,
            col_names,
            String::new(),
        )
    }

    pub fn with_description(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
        description: String,
    ) -> Self {
        Self {
            base: BaseExecutor::with_description(
                id,
                "BaseJoinExecutor".to_string(),
                description.clone(),
                storage,
            ),
            left_var,
            right_var,
            hash_keys,
            probe_keys,
            col_names,
            description,
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
                // 检查Values中是否包含DataSet
                if values.len() == 1 {
                    if let Value::DataSet(dataset) = &values[0] {
                        dataset.clone()
                    } else {
                        // 单个值作为一行
                        DataSet {
                            col_names: vec![],
                            rows: vec![values.clone()],
                        }
                    }
                } else {
                    // 多个值，每个值作为一行
                    DataSet {
                        col_names: vec![],
                        rows: values.iter().map(|v| vec![v.clone()]).collect(),
                    }
                }
            }
            ExecutionResult::DataSet(dataset) => dataset.clone(),
            _ => {
                return Err(QueryError::ExecutionError(
                    "左输入不是有效的数据集".to_string(),
                ))
            }
        };

        let right_dataset = match right_result {
            ExecutionResult::Values(values) => {
                // 检查Values中是否包含DataSet
                if values.len() == 1 {
                    if let Value::DataSet(dataset) = &values[0] {
                        dataset.clone()
                    } else {
                        // 单个值作为一行
                        DataSet {
                            col_names: vec![],
                            rows: vec![values.clone()],
                        }
                    }
                } else {
                    // 多个值，每个值作为一行
                    DataSet {
                        col_names: vec![],
                        rows: values.iter().map(|v| vec![v.clone()]).collect(),
                    }
                }
            }
            ExecutionResult::DataSet(dataset) => dataset.clone(),
            _ => {
                return Err(QueryError::ExecutionError(
                    "右输入不是有效的数据集".to_string(),
                ))
            }
        };

        Ok((left_dataset, right_dataset))
    }

    /// 构建单键哈希表（使用JoinKeyEvaluator）
    pub fn build_single_key_hash_table_with_evaluator<C: ExpressionContext>(
        &self,
        dataset: &DataSet,
        hash_key_expr: &Expression,
        _evaluator: &JoinKeyEvaluator,
        context: &mut C,
    ) -> Result<HashMap<Value, Vec<Vec<Value>>>, QueryError> {
        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let key = JoinKeyEvaluator::evaluate_key(hash_key_expr, context)
                .map_err(|e| QueryError::ExecutionError(format!("键求值失败: {}", e)))?;

            hash_table
                .entry(key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        Ok(hash_table)
    }

    /// 构建多键哈希表（使用JoinKeyEvaluator）
    pub fn build_multi_key_hash_table_with_evaluator<C: ExpressionContext>(
        &self,
        dataset: &DataSet,
        hash_key_exprs: &[Expression],
        _evaluator: &JoinKeyEvaluator,
        context: &mut C,
    ) -> Result<HashMap<JoinKey, Vec<Vec<Value>>>, QueryError> {
        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let key_values = JoinKeyEvaluator::evaluate_keys(hash_key_exprs, context)
                .map_err(|e| QueryError::ExecutionError(format!("键求值失败: {}", e)))?;

            let join_key = JoinKey::new(key_values);
            hash_table
                .entry(join_key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        Ok(hash_table)
    }

    /// 探测单键哈希表（使用JoinKeyEvaluator）
    pub fn probe_single_key_hash_table_with_evaluator<C: ExpressionContext>(
        &self,
        probe_dataset: &DataSet,
        hash_table: &HashMap<Value, Vec<Vec<Value>>>,
        probe_key_expr: &Expression,
        _evaluator: &JoinKeyEvaluator,
        context: &mut C,
    ) -> Result<Vec<(Vec<Value>, Vec<Vec<Value>>)>, QueryError> {
        let mut results = Vec::new();

        for probe_row in &probe_dataset.rows {
            let key = JoinKeyEvaluator::evaluate_key(probe_key_expr, context)
                .map_err(|e| QueryError::ExecutionError(format!("探测键求值失败: {}", e)))?;

            if let Some(matching_rows) = hash_table.get(&key) {
                results.push((probe_row.clone(), matching_rows.clone()));
            }
        }

        Ok(results)
    }

    /// 探测多键哈希表（使用JoinKeyEvaluator）
    pub fn probe_multi_key_hash_table_with_evaluator<C: ExpressionContext>(
        &self,
        probe_dataset: &DataSet,
        hash_table: &HashMap<JoinKey, Vec<Vec<Value>>>,
        probe_key_exprs: &[Expression],
        _evaluator: &JoinKeyEvaluator,
        context: &mut C,
    ) -> Result<Vec<(Vec<Value>, Vec<Vec<Value>>)>, QueryError> {
        let mut results = Vec::new();

        for probe_row in &probe_dataset.rows {
            let key_values = JoinKeyEvaluator::evaluate_keys(probe_key_exprs, context)
                .map_err(|e| QueryError::ExecutionError(format!("探测键求值失败: {}", e)))?;

            let join_key = JoinKey::new(key_values);

            if let Some(matching_rows) = hash_table.get(&join_key) {
                results.push((probe_row.clone(), matching_rows.clone()));
            }
        }

        Ok(results)
    }

    /// 构建单键哈希表
    pub fn build_single_key_hash_table(
        hash_key: &str,
        dataset: &DataSet,
        hash_table: &mut HashMap<Value, Vec<Vec<Value>>>,
    ) -> Result<(), QueryError> {
        for row in &dataset.rows {
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
        let mut new_row = Vec::new();

        // 根据输出列名构建结果行
        // 输出列名格式：["id", "name", "age"]
        // 左表列名：["id", "name"]
        // 右表列名：["id", "age"]

        // 简化实现：假设输出列名已经指定了正确的顺序
        // 对于自然连接，重复的列应该只出现一次
        // 这里我们简单地将左表的所有列和右表的非重复列合并

        // 添加左表的所有列
        new_row.extend(left_row.clone());

        // 添加右表的非重复列（从第1列开始，跳过重复的id列）
        if right_row.len() > 1 {
            new_row.extend(right_row[1..].iter().cloned());
        }

        new_row
    }

    /// 决定是否交换左右输入以优化性能
    pub fn should_exchange(&self, left_size: usize, right_size: usize) -> bool {
        // 如果左表比右表大很多，交换以减少哈希表大小
        left_size > right_size * 2
    }

    /// 执行左右输入交换优化
    pub fn optimize_join_order(&mut self, left_dataset: &DataSet, right_dataset: &DataSet) {
        let left_size = left_dataset.rows.len();
        let right_size = right_dataset.rows.len();

        if self.should_exchange(left_size, right_size) {
            self.exchange = true;
        }
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

    /// 检查数据是否可以移动（避免不必要的拷贝）
    pub fn is_movable(&self, _var_name: &str) -> bool {
        // 检查变量是否不再被后续执行器使用
        // 简化实现：假设所有变量都可以移动
        // 实际实现需要检查执行计划中的变量生命周期
        true
    }

    /// 获取列名
    pub fn get_col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    /// 获取哈希键
    pub fn get_hash_keys(&self) -> &Vec<Expression> {
        &self.hash_keys
    }

    /// 获取探测键
    pub fn get_probe_keys(&self) -> &Vec<Expression> {
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

    /// 获取执行器ID
    pub fn id(&self) -> i64 {
        self.base.id
    }

    /// 获取执行器名称
    pub fn name(&self) -> &str {
        &self.base.name
    }

    /// 获取执行上下文的可变引用
    pub fn context_mut(&mut self) -> &mut crate::query::executor::base::ExecutionContext {
        &mut self.base.context
    }

    /// 获取左表变量名
    pub fn left_var(&self) -> &str {
        &self.left_var
    }

    /// 获取右表变量名
    pub fn right_var(&self) -> &str {
        &self.right_var
    }

    /// 获取哈希键列表
    pub fn hash_keys(&self) -> &Vec<Expression> {
        &self.hash_keys
    }

    /// 获取探测键列表
    pub fn probe_keys(&self) -> &Vec<Expression> {
        &self.probe_keys
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    /// 获取描述
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取是否交换了左右输入
    pub fn is_exchanged(&self) -> bool {
        self.exchange
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for BaseJoinExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("BaseJoinExecutor storage should be set")
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
        _left_dataset: &DataSet,
        _right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 简化实现：执行基本的内连接
        let result = DataSet::new();

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
        _left_dataset: &DataSet,
        _right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        // 简化实现：执行基本的左连接
        let result = DataSet::new();

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
