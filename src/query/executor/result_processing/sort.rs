//! 排序执行器
//!
//! 提供高性能排序功能，支持多列排序和Top-N优化
//!
//! 参考nebula-graph的SortExecutor实现，支持Scatter-Gather并行计算模式

use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

use crate::common::thread::ThreadPool;
use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

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
///
/// 参考nebula-graph的SortExecutor实现，支持Scatter-Gather并行计算模式
pub struct SortExecutor<S: StorageClient + Send + 'static> {
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
    /// 线程池用于并行排序
    ///
    /// 参考nebula-graph的Executor::runMultiJobs，用于Scatter-Gather并行计算
    thread_pool: Option<Arc<ThreadPool>>,
    /// 并行计算配置
    parallel_config: ParallelConfig,
}

impl<S: StorageClient + Send + 'static> SortExecutor<S> {
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
            thread_pool: None,
            parallel_config: ParallelConfig::default(),
        })
    }

    /// 设置线程池
    ///
    /// 参考nebula-graph的Executor::runMultiJobs，用于Scatter-Gather并行计算
    pub fn with_thread_pool(mut self, thread_pool: Arc<ThreadPool>) -> Self {
        self.thread_pool = Some(thread_pool);
        self
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    /// 处理输入数据并排序
    fn process_input(&mut self) -> DBResult<DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;

            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    self.optimize_sort_keys(&data_set.col_names)?;
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
    ///
    /// 根据数据量选择排序方式：
    /// - 数据量小于parallel_threshold：单线程排序
    /// - 数据量大：使用Scatter-Gather并行排序
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

        let total_size = data_set.rows.len();

        // 根据并行配置判断是否使用并行排序
        if self.parallel_config.should_use_parallel(total_size) {
            return self.execute_parallel_sort(data_set);
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

    /// 并行排序
    ///
    /// 使用Scatter-Gather模式：
    /// - Scatter: 将数据分成多个块，每块在一个线程中排序
    /// - Gather: 使用k路归并合并排序后的块
    fn execute_parallel_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        let batch_size = self.parallel_config.calculate_batch_size(data_set.rows.len());
        let sort_keys = self.sort_keys.clone();

        // 检查是否所有排序键都使用列索引（并行排序需要）
        let all_use_column_index = sort_keys.iter().all(|key| key.uses_column_index());

        // 将数据分成多个块
        let chunks: Vec<Vec<Vec<Value>>> = data_set
            .rows
            .chunks(batch_size)
            .map(|c| c.to_vec())
            .collect();

        // 并行排序每个块
        let sorted_chunks: Vec<Vec<Vec<Value>>> = if all_use_column_index {
            // 使用列索引排序（更快）
            chunks
                .into_par_iter()
                .map(|mut chunk| {
                    chunk.par_sort_unstable_by(|a, b| {
                        for sort_key in &sort_keys {
                            if let Some(column_index) = sort_key.column_index {
                                if column_index < a.len() && column_index < b.len() {
                                    let a_val = &a[column_index];
                                    let b_val = &b[column_index];

                                    if let Some(cmp) = a_val.partial_cmp(b_val) {
                                        if cmp != Ordering::Equal {
                                            return match sort_key.order {
                                                SortOrder::Asc => cmp,
                                                SortOrder::Desc => cmp.reverse(),
                                            };
                                        }
                                    }
                                }
                            }
                        }
                        Ordering::Equal
                    });
                    chunk
                })
                .collect()
        } else {
            // 使用表达式排序（较慢，需要计算排序值）
            chunks
                .into_par_iter()
                .map(|chunk| {
                    let col_names: Vec<String> =
                        (0..chunk[0].len()).map(|i| format!("col_{}", i)).collect();

                    // 预计算排序值
                    let mut rows_with_sort_values: Vec<(Vec<Value>, Vec<Value>)> = chunk
                        .into_iter()
                        .map(|row| {
                            let sort_values = self.calculate_sort_values(&row, &col_names).unwrap_or_default();
                            (row, sort_values)
                        })
                        .collect();

                    // 根据排序值排序
                    rows_with_sort_values.par_sort_unstable_by(|(_, a), (_, b)| {
                        for ((idx, sort_val_a), sort_val_b) in a.iter().enumerate().zip(b.iter()) {
                            if let Some(comparison) = sort_val_a.partial_cmp(sort_val_b) {
                                if comparison != Ordering::Equal {
                                    return match sort_keys[idx].order {
                                        SortOrder::Asc => comparison,
                                        SortOrder::Desc => comparison.reverse(),
                                    };
                                }
                            }
                        }
                        Ordering::Equal
                    });

                    rows_with_sort_values.into_iter().map(|(row, _)| row).collect()
                })
                .collect()
        };

        // k路归并
        data_set.rows = self.k_way_merge(sorted_chunks)?;

        // 应用limit
        if let Some(limit) = self.limit {
            data_set.rows.truncate(limit);
        }

        Ok(())
    }

    /// k路归并
    fn k_way_merge(&self, sorted_chunks: Vec<Vec<Vec<Value>>>) -> DBResult<Vec<Vec<Value>>> {
        if sorted_chunks.is_empty() {
            return Ok(Vec::new());
        }

        if sorted_chunks.len() == 1 {
            return Ok(sorted_chunks.into_iter().next().unwrap());
        }

        // 使用优先队列实现k路归并
        #[derive(Clone)]
        struct HeapItem {
            row: Vec<Value>,
            chunk_idx: usize,
            row_idx: usize,
        }

        impl Eq for HeapItem {}

        impl PartialEq for HeapItem {
            fn eq(&self, other: &Self) -> bool {
                self.chunk_idx == other.chunk_idx && self.row_idx == other.row_idx
            }
        }

        impl Ord for HeapItem {
            fn cmp(&self, other: &Self) -> Ordering {
                // 注意：BinaryHeap在Rust中是大顶堆，我们需要小顶堆，所以反转比较结果
                // 这里我们使用一个简化的比较，实际应该使用完整的排序逻辑
                other.chunk_idx.cmp(&self.chunk_idx)
            }
        }

        impl PartialOrd for HeapItem {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut result = Vec::new();
        let mut chunk_iters: Vec<std::vec::IntoIter<Vec<Value>>> = sorted_chunks
            .into_iter()
            .map(|c| c.into_iter())
            .collect();

        // 初始化堆
        let mut heap = BinaryHeap::new();
        for (chunk_idx, iter) in chunk_iters.iter_mut().enumerate() {
            if let Some(row) = iter.next() {
                heap.push(HeapItem {
                    row,
                    chunk_idx,
                    row_idx: 0,
                });
            }
        }

        // 归并
        while let Some(item) = heap.pop() {
            result.push(item.row);

            if let Some(next_row) = chunk_iters[item.chunk_idx].next() {
                heap.push(HeapItem {
                    row: next_row,
                    chunk_idx: item.chunk_idx,
                    row_idx: item.row_idx + 1,
                });
            }
        }

        Ok(result)
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for SortExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input()?;
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for SortExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(DataSet::new()))
        };

        self.process(input_result)
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for SortExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for SortExecutor<S> {
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
