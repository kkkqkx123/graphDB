//! 排序执行器
//!
//! 提供高性能排序功能，支持多列排序和Top-N优化

use async_trait::async_trait;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 排序顺序枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 排序键定义
#[derive(Debug, Clone)]
pub struct SortKey {
    pub expression: Expression,
    pub order: SortOrder,
    /// 优化后的列索引（如果表达式可以解析为列索引）
    pub column_index: Option<usize>,
}

impl SortKey {
    pub fn new(expression: Expression, order: SortOrder) -> Self {
        Self {
            expression,
            order,
            column_index: None,
        }
    }

    /// 创建基于列索引的排序键
    pub fn from_column_index(column_index: usize, order: SortOrder) -> Self {
        Self {
            expression: Expression::Literal(Value::Int(column_index as i64)),
            order,
            column_index: Some(column_index),
        }
    }

    /// 检查是否使用列索引排序
    pub fn uses_column_index(&self) -> bool {
        self.column_index.is_some()
    }
}

/// 排序配置
#[derive(Debug, Clone)]
pub struct SortConfig {
    /// 内存限制（字节），用于大数据集处理
    pub memory_limit: usize,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            memory_limit: 100 * 1024 * 1024, // 100MB 默认内存限制
        }
    }
}

/// 优化的排序执行器
pub struct SortExecutor<S: StorageEngine + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 排序键列表
    sort_keys: Vec<SortKey>,
    /// 限制数量
    limit: Option<usize>,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 排序配置
    config: SortConfig,
}

impl<S: StorageEngine + Send + 'static> SortExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        sort_keys: Vec<SortKey>,
        limit: Option<usize>,
        config: SortConfig,
    ) -> DBResult<Self> {
        let base = BaseResultProcessor::new(
            id,
            "SortExecutor".to_string(),
            "High-performance sorting".to_string(),
            storage,
        );

        Ok(Self {
            base,
            sort_keys,
            limit,
            input_executor: None,
            config,
        })
    }

    /// 处理输入数据并排序
    async fn process_input(&mut self) -> DBResult<DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    // 优化排序键（将表达式解析为列索引）
                    self.optimize_sort_keys(&data_set.col_names)?;

                    // 根据数据集大小和配置选择合适的排序算法
                    self.execute_sort(&mut data_set)?;
                    Ok(data_set)
                }
                _ => Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Sort executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Sort executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 优化排序键，将表达式解析为列索引
    fn optimize_sort_keys(&mut self, col_names: &[String]) -> DBResult<()> {
        // 先收集需要解析的表达式
        let mut expressions_to_parse = Vec::new();
        for (i, sort_key) in self.sort_keys.iter().enumerate() {
            if sort_key.column_index.is_none() {
                expressions_to_parse.push((i, sort_key.expression.clone()));
            }
        }

        // 解析表达式为列索引
        for (i, expression) in expressions_to_parse {
            if let Some(column_index) = self.parse_expression_to_column_index(&expression, col_names)? {
                self.sort_keys[i].column_index = Some(column_index);
            }
        }

        Ok(())
    }

    /// 将表达式解析为列索引
    fn parse_expression_to_column_index(
        &self,
        expression: &Expression,
        col_names: &[String],
    ) -> DBResult<Option<usize>> {
        match expression {
            Expression::Property { object: _, property } => {
                // 查找属性名对应的列索引
                for (index, col_name) in col_names.iter().enumerate() {
                    if col_name == property {
                        return Ok(Some(index));
                    }
                }
                Ok(None)
            }
            Expression::Literal(Value::Int(index)) => {
                // 直接使用列索引
                let idx = *index as usize;
                if idx < col_names.len() {
                    Ok(Some(idx))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None), // 其他表达式类型暂时不支持优化
        }
    }

    /// 执行排序算法
    fn execute_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        if self.sort_keys.is_empty() || data_set.rows.is_empty() {
            return Ok(());
        }

        // 检查内存使用是否超过限制
        let estimated_memory = self.estimate_memory_usage(data_set);
        if estimated_memory > self.config.memory_limit {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(format!(
                    "排序操作内存使用超出限制: {} > {}",
                    estimated_memory, self.config.memory_limit
                )),
            ));
        }

        // 检查是否所有排序键都使用列索引
        let all_use_column_index = self.sort_keys.iter().all(|key| key.uses_column_index());

        if all_use_column_index {
            // 使用列索引进行排序
            return self.execute_column_index_sort(data_set);
        }

        // 如果有LIMIT且数据量很大，使用Top-N算法
        if let Some(limit) = self.limit {
            if data_set.rows.len() > limit * 10 {
                return self.execute_top_n_sort(data_set, limit);
            }
        }

        // 使用标准库排序（简化实现）
        self.execute_standard_sort(data_set)
    }

    /// 使用列索引进行排序
    fn execute_column_index_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        // 验证所有列索引都在有效范围内
        for sort_key in &self.sort_keys {
            if let Some(column_index) = sort_key.column_index {
                if data_set.rows.iter().any(|row| column_index >= row.len()) {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(format!(
                            "列索引超出范围: {} (最大索引: {})",
                            column_index,
                            data_set.rows[0].len() - 1
                        )),
                    ));
                }
            }
        }

        // 直接使用标准库排序，性能最优
        data_set.rows.sort_unstable_by(|a, b| {
            self.compare_by_column_indices(a, b)
                .unwrap_or(Ordering::Equal)
        });

        // 应用limit
        if let Some(limit) = self.limit {
            data_set.rows.truncate(limit);
        }

        Ok(())
    }

    /// 使用标准库排序（简化实现）
    fn execute_standard_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        // 使用标准库排序
        data_set
            .rows
            .sort_unstable_by(|a, b| self.compare_rows(a, b).unwrap_or(Ordering::Equal));

        // 应用limit
        if let Some(limit) = self.limit {
            data_set.rows.truncate(limit);
        }

        Ok(())
    }

    /// 基于列索引比较两个数据行
    fn compare_by_column_indices(&self, a: &[Value], b: &[Value]) -> DBResult<Ordering> {
        for sort_key in &self.sort_keys {
            if let Some(column_index) = sort_key.column_index {
                if column_index >= a.len() || column_index >= b.len() {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(format!(
                            "列索引超出范围: {} (最大索引: {})",
                            column_index,
                            a.len().min(b.len()) - 1
                        )),
                    ));
                }

                let a_val = &a[column_index];
                let b_val = &b[column_index];

                let cmp = a_val.partial_cmp(b_val).ok_or_else(|| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "值比较失败，类型不匹配: {:?} 和 {:?}",
                        a_val, b_val
                    )))
                })?;
                if cmp != Ordering::Equal {
                    return Ok(match sort_key.order {
                        SortOrder::Asc => cmp,
                        SortOrder::Desc => cmp.reverse(),
                    });
                }
            }
        }
        Ok(Ordering::Equal)
    }

    /// 估算数据集内存使用
    fn estimate_memory_usage(&self, data_set: &DataSet) -> usize {
        if data_set.rows.is_empty() {
            return 0;
        }

        // 估算每行的内存使用
        let sample_row = &data_set.rows[0];
        let mut row_size = std::mem::size_of::<Vec<Value>>();

        // 估算每个值的内存使用
        for value in sample_row {
            row_size += self.estimate_value_size(value);
        }

        // 估算排序键的内存使用
        let sort_key_size = self.sort_keys.len() * std::mem::size_of::<Value>();

        // 总内存使用 = 行数 × (行大小 + 排序键大小)
        data_set.rows.len() * (row_size + sort_key_size)
    }

    /// 估算单个值的内存使用
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::String(s) => std::mem::size_of::<String>() + s.len(),
            Value::Int(_) => std::mem::size_of::<i64>(),
            Value::Float(_) => std::mem::size_of::<f64>(),
            Value::Bool(_) => std::mem::size_of::<bool>(),
            Value::Null(_) => 0,
            Value::List(list) => {
                std::mem::size_of::<Vec<Value>>()
                    + list
                        .iter()
                        .map(|v| self.estimate_value_size(v))
                        .sum::<usize>()
            }
            Value::Map(map) => {
                std::mem::size_of::<std::collections::HashMap<String, Value>>()
                    + map
                        .iter()
                        .map(|(k, v)| k.len() + self.estimate_value_size(v))
                        .sum::<usize>()
            }
            _ => std::mem::size_of::<Value>(), // 默认大小
        }
    }

    /// 执行Top-N排序（使用select_nth_unstable优化）
    fn execute_top_n_sort(&mut self, data_set: &mut DataSet, n: usize) -> DBResult<()> {
        if n == 0 || data_set.rows.is_empty() {
            return Ok(());
        }

        // 如果n大于等于数据集大小，直接排序整个数据集
        if n >= data_set.rows.len() {
            return self.execute_sort(data_set);
        }

        // 检查是否所有排序键都使用列索引
        let all_use_column_index = self.sort_keys.iter().all(|key| key.uses_column_index());

        if all_use_column_index {
            // 使用列索引进行Top-N排序
            return self.execute_column_index_top_n_sort(data_set, n);
        }

        // 获取所有排序键的方向
        let sort_orders: Vec<SortOrder> = self.sort_keys.iter().map(|key| key.order).collect();

        // 根据排序方向选择正确的比较逻辑
        let is_ascending = sort_orders[0] == SortOrder::Asc;

        // 使用最佳实践方法：直接对数据行进行操作
        if is_ascending {
            // 升序：选择前n个最小的元素
            let (left, _, _) = data_set.rows.select_nth_unstable_by(n, |a, b| {
                self.compare_rows(a, b).unwrap_or(Ordering::Equal)
            });
            left.sort_unstable_by(|a, b| self.compare_rows(a, b).unwrap_or(Ordering::Equal));
            data_set.rows.truncate(n);
        } else {
            // 降序：选择最大的n个元素
            // compare_rows方法内部已经根据SortOrder::Desc正确处理了降序比较
            // 因此直接使用compare_rows(a, b)即可
            data_set
                .rows
                .sort_unstable_by(|a, b| self.compare_rows(a, b).unwrap_or(Ordering::Equal));
            data_set.rows.truncate(n);
        }

        Ok(())
    }

    /// 使用列索引进行Top-N排序
    fn execute_column_index_top_n_sort(
        &mut self,
        data_set: &mut DataSet,
        n: usize,
    ) -> DBResult<()> {
        // 获取所有排序键的方向
        let sort_orders: Vec<SortOrder> = self.sort_keys.iter().map(|key| key.order).collect();
        let is_ascending = sort_orders[0] == SortOrder::Asc;

        if is_ascending {
            // 升序：选择前n个最小的元素
            let (left, _, _) = data_set.rows.select_nth_unstable_by(n, |a, b| {
                self.compare_by_column_indices(a, b)
                    .unwrap_or(Ordering::Equal)
            });
            left.sort_unstable_by(|a, b| {
                self.compare_by_column_indices(a, b)
                    .unwrap_or(Ordering::Equal)
            });
            data_set.rows.truncate(n);
        } else {
            // 降序：选择最大的n个元素
            data_set.rows.sort_unstable_by(|a, b| {
                self.compare_by_column_indices(a, b)
                    .unwrap_or(Ordering::Equal)
            });
            data_set.rows.truncate(n);
        }

        Ok(())
    }

    /// 计算行的排序键值
    fn calculate_sort_values(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
        let mut sort_values = Vec::new();

        for sort_key in &self.sort_keys {
            // 处理按列索引排序的特殊情况
            if let Expression::Literal(Value::Int(index)) = &sort_key.expression {
                let idx = *index as usize;
                if idx < row.len() {
                    sort_values.push(row[idx].clone());
                } else {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(format!(
                            "列索引{}超出范围，行长度:{}",
                            idx,
                            row.len()
                        )),
                    ));
                }
            } else {
                // 使用表达式求值器处理其他类型的表达式
                let mut expr_context = DefaultExpressionContext::new();
                for (i, col_name) in col_names.iter().enumerate() {
                    if i < row.len() {
                        expr_context.set_variable(col_name.clone(), row[i].clone());
                    }
                }

                let sort_value =
                    ExpressionEvaluator::evaluate(&sort_key.expression, &mut expr_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;
                sort_values.push(sort_value);
            }
        }

        Ok(sort_values)
    }

    /// 比较两个排序值向量
    fn compare_sort_items_vec(&self, a: &[Value], b: &[Value]) -> DBResult<Ordering> {
        for ((idx, sort_val_a), sort_val_b) in a.iter().enumerate().zip(b.iter()) {
            let comparison =
                self.compare_values(sort_val_a, sort_val_b, &self.sort_keys[idx].order)?;
            if !comparison.is_eq() {
                return Ok(comparison);
            }
        }
        Ok(Ordering::Equal)
    }

    /// 比较两个值，根据排序方向
    fn compare_values(&self, a: &Value, b: &Value, order: &SortOrder) -> DBResult<Ordering> {
        let comparison = a.partial_cmp(b).ok_or_else(|| {
            DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                "排序值比较失败，类型不匹配: {:?} 和 {:?}",
                a, b
            )))
        })?;

        Ok(match order {
            SortOrder::Asc => comparison,
            SortOrder::Desc => comparison.reverse(),
        })
    }

    /// 比较两个数据行，直接使用排序键表达式
    /// 注意：这个方法内部已经根据SortKey的order字段正确处理了排序方向
    fn compare_rows(&self, a: &[Value], b: &[Value]) -> DBResult<Ordering> {
        // 创建虚拟列名（因为排序键表达式应该能够直接访问行数据）
        let col_names: Vec<String> = (0..a.len()).map(|i| format!("col_{}", i)).collect();

        // 为每行计算排序值
        let sort_values_a = self.calculate_sort_values(a, &col_names)?;
        let sort_values_b = self.calculate_sort_values(b, &col_names)?;

        // 使用现有的比较逻辑
        self.compare_sort_items_vec(&sort_values_a, &sort_values_b)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for SortExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.base.memory_usage
    }

    fn reset(&mut self) {
        self.base.reset_state();
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for SortExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(DataSet::new()))
        };

        self.process(input_result).await
    }

    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.input.is_some()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for SortExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for SortExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::{DataSet, Value};
    use crate::storage::MockStorage;

    fn create_test_dataset() -> DataSet {
        let mut data_set = DataSet::new();
        data_set.col_names = vec!["name".to_string(), "age".to_string(), "score".to_string()];

        // 添加测试数据
        data_set.rows = vec![
            vec![
                Value::String("Alice".to_string()),
                Value::Int(25),
                Value::Float(85.5),
            ],
            vec![
                Value::String("Bob".to_string()),
                Value::Int(30),
                Value::Float(92.0),
            ],
            vec![
                Value::String("Charlie".to_string()),
                Value::Int(22),
                Value::Float(78.5),
            ],
            vec![
                Value::String("David".to_string()),
                Value::Int(28),
                Value::Float(88.0),
            ],
            vec![
                Value::String("Eve".to_string()),
                Value::Int(26),
                Value::Float(95.5),
            ],
        ];

        data_set
    }

    #[test]
    fn test_sort_key_column_index() {
        // 测试排序键列索引功能
        let sort_key = SortKey::new(Expression::Literal(Value::Int(1)), SortOrder::Asc);
        assert!(!sort_key.uses_column_index());

        // 测试基于列索引的排序键
        let column_index_sort_key = SortKey::from_column_index(1, SortOrder::Desc);
        assert!(column_index_sort_key.uses_column_index());
        assert_eq!(column_index_sort_key.column_index, Some(1));
    }

    #[test]
    fn test_column_index_sorting() {
        let mut data_set = create_test_dataset();

        // 使用列索引排序
        let sort_keys = vec![SortKey::from_column_index(2, SortOrder::Asc)]; // 按score列升序排序

        let config = SortConfig::default();

        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = SortExecutor::new(1, storage, sort_keys, None, config).expect("SortExecutor::new should succeed");

        // 执行排序
        executor.execute_sort(&mut data_set).expect("execute_sort should succeed");

        // 验证排序结果（升序：分数从低到高）
        assert_eq!(data_set.rows.len(), 5);
        assert_eq!(data_set.rows[0][2], Value::Float(78.5)); // Charlie (最低分)
        assert_eq!(data_set.rows[1][2], Value::Float(85.5)); // Alice
        assert_eq!(data_set.rows[2][2], Value::Float(88.0)); // David
        assert_eq!(data_set.rows[3][2], Value::Float(92.0)); // Bob
        assert_eq!(data_set.rows[4][2], Value::Float(95.5)); // Eve (最高分)
    }

    #[test]
    fn test_top_n_sort() {
        let mut data_set = create_test_dataset();
        let sort_keys = vec![SortKey::from_column_index(2, SortOrder::Desc)]; // 按score列降序排序

        let config = SortConfig::default();

        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = SortExecutor::new(1, storage, sort_keys, Some(3), config).expect("SortExecutor::new should succeed");

        // 执行排序
        executor.execute_sort(&mut data_set).expect("execute_sort should succeed");

        // 验证Top-N结果
        assert_eq!(data_set.rows.len(), 3); // 只保留前3个
        assert_eq!(data_set.rows[0][2], Value::Float(95.5)); // Eve (最高分)
        assert_eq!(data_set.rows[1][2], Value::Float(92.0)); // Bob
        assert_eq!(data_set.rows[2][2], Value::Float(88.0)); // David (第三高分)
    }

    #[test]
    fn test_column_index_top_n_sort() {
        let mut data_set = create_test_dataset();

        // 使用列索引排序
        let sort_keys = vec![SortKey::from_column_index(2, SortOrder::Desc)]; // 按score列降序排序

        let config = SortConfig::default();

        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = SortExecutor::new(1, storage, sort_keys, Some(2), config).expect("SortExecutor::new should succeed");

        // 执行排序
        executor.execute_sort(&mut data_set).expect("execute_sort should succeed");

        // 验证Top-N结果
        assert_eq!(data_set.rows.len(), 2); // 只保留前2个
        assert_eq!(data_set.rows[0][2], Value::Float(95.5)); // Eve (最高分)
        assert_eq!(data_set.rows[1][2], Value::Float(92.0)); // Bob (第二高分)
    }

    #[test]
    fn test_multi_column_sorting() {
        let mut data_set = create_test_dataset();

        // 多列排序：先按age升序，再按score降序
        let sort_keys = vec![
            SortKey::from_column_index(1, SortOrder::Asc), // age升序
            SortKey::from_column_index(2, SortOrder::Desc), // score降序
        ];

        let config = SortConfig::default();

        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = SortExecutor::new(1, storage, sort_keys, None, config).expect("SortExecutor::new should succeed");
        executor.execute_sort(&mut data_set).expect("execute_sort should succeed");

        // 验证多列排序结果
        assert_eq!(data_set.rows.len(), 5);

        // 第一列排序：年龄从低到高
        assert_eq!(data_set.rows[0][1], Value::Int(22)); // Charlie (最年轻)
        assert_eq!(data_set.rows[1][1], Value::Int(25)); // Alice
        assert_eq!(data_set.rows[2][1], Value::Int(26)); // Eve
        assert_eq!(data_set.rows[3][1], Value::Int(28)); // David
        assert_eq!(data_set.rows[4][1], Value::Int(30)); // Bob (最年长)

        // 对于相同年龄的行，按分数降序排序
        // 这里没有年龄相同的行，所以不需要额外验证
    }

    #[test]
    fn test_error_handling() {
        let mut data_set = create_test_dataset();

        // 测试无效的列索引
        let sort_keys = vec![SortKey::from_column_index(10, SortOrder::Asc)]; // 无效列索引

        let config = SortConfig::default();
        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = SortExecutor::new(1, storage, sort_keys, None, config).expect("SortExecutor::new should succeed");

        // 验证多列排序结果应该会返回错误，因为列索引超出范围
        let result = executor.execute_sort(&mut data_set);
        assert!(result.is_err());

        // 验证错误信息包含列索引信息
        let error = result.unwrap_err();
        assert!(format!("{:?}", error).contains("列索引"));
    }

    #[test]
    fn test_compare_by_column_indices() {
        let data_set = create_test_dataset();
        let sort_keys = vec![SortKey::from_column_index(2, SortOrder::Asc)];

        let config = SortConfig::default();

        let storage = Arc::new(Mutex::new(MockStorage));

        let executor = SortExecutor::new(1, storage, sort_keys, None, config).expect("SortExecutor::new should succeed");

        // 测试列索引比较功能
        let row1 = &data_set.rows[0]; // Alice: 85.5
        let row2 = &data_set.rows[1]; // Bob: 92.0

        let result = executor.compare_by_column_indices(row1, row2).expect("compare_by_column_indices should succeed");
        assert_eq!(result, Ordering::Less); // 85.5 < 92.0
    }
}
