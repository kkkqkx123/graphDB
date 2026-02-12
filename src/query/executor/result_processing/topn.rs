//! TopN 执行器
//!
//! 实现高效的 TopN 查询，使用堆数据结构优化性能
//! CPU 密集型操作，使用 Rayon 进行并行化

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::types::OrderDirection;
use crate::core::{DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

/// 排序列定义
#[derive(Debug, Clone)]
pub struct SortColumn {
    /// 列索引
    pub column_index: usize,
    /// 数据类型
    pub data_type: crate::core::DataType,
    /// NULL 值是否排在前面
    pub nulls_first: bool,
}

impl SortColumn {
    pub fn new(column_index: usize, data_type: crate::core::DataType, nulls_first: bool) -> Self {
        Self {
            column_index,
            data_type,
            nulls_first,
        }
    }
}

/// TopN 错误类型
#[derive(Debug, thiserror::Error)]
pub enum TopNError {
    #[error("执行器已打开")]
    ExecutorAlreadyOpen,

    #[error("内存限制超出")]
    MemoryLimitExceeded,

    #[error("无效的列索引: {0}")]
    InvalidColumnIndex(usize),

    #[error("排序值提取失败: {0}")]
    SortValueExtractionFailed(String),

    #[error("堆操作失败: {0}")]
    HeapOperationFailed(String),

    #[error("输入执行器错误: {0}")]
    InputExecutorError(#[from] DBError),
}

/// TopNExecutor - TOP N 结果执行器
///
/// 返回排序后的前 N 个结果，是 Sort + Limit 的优化版本
/// 使用堆数据结构实现高效的 TopN 查询
/// CPU 密集型操作，使用 Rayon 进行并行化
pub struct TopNExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 返回的结果数量
    n: usize,
    /// 偏移量
    offset: usize,
    /// 排序键列表
    sort_keys: Vec<crate::query::executor::result_processing::sort::SortKey>,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 排序列定义
    sort_columns: Vec<SortColumn>,
    /// 排序方向
    sort_direction: OrderDirection,
    /// 堆数据结构（最大堆或最小堆）
    heap: Option<BinaryHeap<TopNItem>>,
    /// 是否已打开
    is_open: bool,
    /// 是否已关闭
    is_closed: bool,
    /// 已处理记录数
    processed_count: usize,
    /// 并行计算配置
    parallel_config: ParallelConfig,
}

impl<S: StorageClient> TopNExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        n: usize,
        sort_columns: Vec<String>,
        ascending: bool,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "TopNExecutor".to_string(),
            "Returns the top N results using optimized heap algorithm".to_string(),
            storage,
        );

        // 转换旧的排序列格式为新的排序键格式
        let sort_keys = sort_columns
            .into_iter()
            .map(|col| {
                let order = if ascending {
                    crate::query::executor::result_processing::sort::SortOrder::Asc
                } else {
                    crate::query::executor::result_processing::sort::SortOrder::Desc
                };
                crate::query::executor::result_processing::sort::SortKey::new(
                    Expression::Variable(col),
                    order,
                )
            })
            .collect();

        Self {
            base,
            n,
            offset: 0,
            sort_keys,
            input_executor: None,
            sort_columns: Vec::new(),
            sort_direction: if ascending {
                OrderDirection::Asc
            } else {
                OrderDirection::Desc
            },
            heap: None,
            is_open: false,
            is_closed: false,
            processed_count: 0,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 创建带有排序列定义的 TopN 执行器
    pub fn with_sort_columns(
        id: i64,
        storage: Arc<Mutex<S>>,
        n: usize,
        sort_columns: Vec<SortColumn>,
        sort_direction: OrderDirection,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "TopNExecutor".to_string(),
            "Returns the top N results using optimized heap algorithm".to_string(),
            storage,
        );

        let sort_keys = sort_columns
            .iter()
            .map(|col| {
                let order = if sort_direction == OrderDirection::Asc {
                    crate::query::executor::result_processing::sort::SortOrder::Asc
                } else {
                    crate::query::executor::result_processing::sort::SortOrder::Desc
                };
                crate::query::executor::result_processing::sort::SortKey::new(
                    Expression::Variable(format!("col_{}", col.column_index)),
                    order,
                )
            })
            .collect();

        Self {
            base,
            n,
            offset: 0,
            sort_keys,
            input_executor: None,
            sort_columns,
            sort_direction,
            heap: None,
            is_open: false,
            is_closed: false,
            processed_count: 0,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    /// 设置偏移量
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// 处理输入数据并执行 TopN
    fn process_input(&mut self) -> DBResult<ExecutionResult> {
        if let Some(input) = self.base.input.take() {
            match input {
                ExecutionResult::DataSet(dataset) => {
                    let topn_result = self.execute_topn_dataset(dataset)?;
                    Ok(ExecutionResult::DataSet(topn_result))
                }
                ExecutionResult::Vertices(vertices) => {
                    let topn_result = self.execute_topn_vertices(vertices)?;
                    Ok(ExecutionResult::Vertices(topn_result))
                }
                ExecutionResult::Edges(edges) => {
                    let topn_result = self.execute_topn_edges(edges)?;
                    Ok(ExecutionResult::Edges(topn_result))
                }
                ExecutionResult::Values(values) => {
                    let topn_result = self.execute_topn_values(values)?;
                    Ok(ExecutionResult::Values(topn_result))
                }
                _ => Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "TopN executor expects supported input type".to_string(),
                    ),
                )),
            }
        } else if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;

            match input_result {
                ExecutionResult::DataSet(dataset) => {
                    let topn_result = self.execute_topn_dataset(dataset)?;
                    Ok(ExecutionResult::DataSet(topn_result))
                }
                ExecutionResult::Vertices(vertices) => {
                    let topn_result = self.execute_topn_vertices(vertices)?;
                    Ok(ExecutionResult::Vertices(topn_result))
                }
                ExecutionResult::Edges(edges) => {
                    let topn_result = self.execute_topn_edges(edges)?;
                    Ok(ExecutionResult::Edges(topn_result))
                }
                ExecutionResult::Values(values) => {
                    let topn_result = self.execute_topn_values(values)?;
                    Ok(ExecutionResult::Values(topn_result))
                }
                _ => Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "TopN executor expects supported input type".to_string(),
                    ),
                )),
            }
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "TopN executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 对数据集执行 TopN
    ///
    /// 根据数据量选择执行方式：
    /// - 数据量小于阈值：单线程堆排序
    /// - 数据量大：使用 Rayon 并行处理
    fn execute_topn_dataset(&self, dataset: DataSet) -> DBResult<DataSet> {
        if self.sort_keys.is_empty() {
            return self.apply_limit_and_offset(dataset);
        }

        let total_size = dataset.rows.len();

        if self.parallel_config.should_use_parallel(total_size) {
            self.execute_topn_dataset_parallel(dataset)
        } else {
            self.execute_topn_dataset_sequential(dataset)
        }
    }

    /// 顺序执行的 TopN（使用堆排序）
    fn execute_topn_dataset_sequential(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        if self.is_ascending() {
            self.heap_ascending(&mut dataset, self.n + self.offset)?;
        } else {
            self.heap_descending(&mut dataset, self.n + self.offset)?;
        }

        Ok(dataset)
    }

    /// 并行执行的 TopN（使用 Rayon）
    ///
    /// 使用两阶段策略：
    /// 1. 并行计算每行的排序键值
    /// 2. 使用 Rayon 分区排序 + 选择 N 个元素
    fn execute_topn_dataset_parallel(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        let _heap_size = self.n + self.offset;
        let sort_keys = self.sort_keys.clone();
        let col_names = dataset.col_names.clone();
        let is_ascending = self.is_ascending();

        let rows_with_values: Vec<(Vec<Value>, Vec<Value>)> = dataset
            .rows
            .into_par_iter()
            .map(|row| {
                let sort_value = Self::calculate_sort_value_parallel(&row, &col_names, &sort_keys);
                (sort_value, row)
            })
            .collect();

        let target_count = self.n + self.offset;

        if is_ascending {
            let mut items: Vec<TopNItemParallel> = rows_with_values
                .into_iter()
                .map(|(sort_value, row)| TopNItemParallel {
                    sort_value,
                    row,
                })
                .collect();

            if items.len() > target_count {
                items.select_nth_unstable_by(target_count, |a, b| {
                    a.sort_value.partial_cmp(&b.sort_value).unwrap_or(Ordering::Equal)
                });
                items.truncate(target_count);
            }

            items.sort_by(|a, b| {
                a.sort_value.partial_cmp(&b.sort_value).unwrap_or(Ordering::Equal)
            });
            items.truncate(self.n);

            dataset.rows = items.into_iter().skip(self.offset).map(|item| item.row).collect();
        } else {
            let mut items: Vec<TopNItemParallel> = rows_with_values
                .into_iter()
                .map(|(sort_value, row)| TopNItemParallel {
                    sort_value,
                    row,
                })
                .collect();

            if items.len() > target_count {
                items.select_nth_unstable_by(target_count, |a, b| {
                    b.sort_value.partial_cmp(&a.sort_value).unwrap_or(Ordering::Equal)
                });
                items.truncate(target_count);
            }

            items.sort_by(|a, b| {
                b.sort_value.partial_cmp(&a.sort_value).unwrap_or(Ordering::Equal)
            });
            items.truncate(self.n);

            dataset.rows = items.into_iter().skip(self.offset).map(|item| item.row).collect();
        }

        Ok(dataset)
    }

    /// 并行计算行的排序值
    fn calculate_sort_value_parallel(
        row: &[Value],
        col_names: &[String],
        sort_keys: &[crate::query::executor::result_processing::sort::SortKey],
    ) -> Vec<Value> {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let mut sort_values = Vec::new();
        for sort_key in sort_keys {
            if let Some(column_index) = sort_key.column_index {
                if column_index < row.len() {
                    sort_values.push(row[column_index].clone());
                    continue;
                }
            }

            match ExpressionEvaluator::evaluate(&sort_key.expression, &mut context) {
                Ok(value) => sort_values.push(value),
                Err(_) => sort_values.push(Value::Null(crate::core::value::NullType::Null)),
            }
        }

        sort_values
    }

    /// 升序排序的堆实现
    fn heap_ascending(&self, dataset: &mut DataSet, heap_size: usize) -> DBResult<()> {
        let mut heap = BinaryHeap::with_capacity(heap_size);

        // 处理所有元素
        for (i, row) in dataset.rows.iter().enumerate() {
            let sort_value = self.calculate_sort_value(row, &dataset.col_names)?;
            let new_item = TopNItem {
                sort_value,
                _original_index: i,
                row: row.clone(),
            };

            if heap.len() < heap_size {
                heap.push(new_item);
            } else {
                // 对于升序（取最小的N个元素）：使用最大堆
                // 如果新元素小于堆顶（最大堆的堆顶是当前最大的元素），则替换
                if let Some(peeked) = heap.peek() {
                    if new_item < *peeked {
                        heap.pop();
                        heap.push(new_item);
                    }
                }
            }
        }

        // 提取并排序结果（升序）
        let mut items: Vec<TopNItem> = heap.into_iter().collect();
        items.sort_by(|a, b| a.sort_value.cmp(&b.sort_value));

        // 更新数据集
        dataset.rows = items
            .into_iter()
            .skip(self.offset)
            .map(|item| item.row)
            .collect();

        Ok(())
    }

    /// 降序排序的堆实现
    fn heap_descending(&self, dataset: &mut DataSet, heap_size: usize) -> DBResult<()> {
        // 对于降序（取最大的N个元素）：使用最小堆
        // 使用 TopNItemDesc 实现最小堆效果
        let mut heap = BinaryHeap::with_capacity(heap_size);

        // 处理所有元素
        for (i, row) in dataset.rows.iter().enumerate() {
            let sort_value = self.calculate_sort_value(row, &dataset.col_names)?;
            let new_item = TopNItemDesc {
                sort_value,
                _original_index: i,
                row: row.clone(),
            };

            if heap.len() < heap_size {
                heap.push(new_item);
            } else {
                // 对于降序TopN（取最大的N个元素）：
                // 使用最小堆，如果新元素大于堆顶（当前最小的元素），则替换
                if let Some(peeked) = heap.peek() {
                    // 注意：由于TopNItemDesc的比较逻辑是反向的，
                    // 所以这里应该使用小于比较，而不是大于
                    if new_item < *peeked {
                        heap.pop();
                        heap.push(new_item);
                    }
                }
            }
        }

        // 提取并排序结果（降序）
        let mut items: Vec<TopNItemDesc> = heap.into_iter().collect();
        items.sort_by(|a, b| b.sort_value.cmp(&a.sort_value));

        // 更新数据集
        dataset.rows = items
            .into_iter()
            .skip(self.offset)
            .map(|item| item.row)
            .collect();

        Ok(())
    }

    /// 计算行的排序值
    fn calculate_sort_value(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let mut sort_values = Vec::new();
        for sort_key in &self.sort_keys {
            let value =
                ExpressionEvaluator::evaluate(&sort_key.expression, &mut context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "Failed to evaluate sort expression: {}",
                        e
                    )))
                })?;
            sort_values.push(value);
        }

        Ok(sort_values)
    }

    /// 比较两个值
    #[allow(dead_code)]
    fn compare_values(
        &self,
        a: &Value,
        b: &Value,
        order: &crate::query::executor::result_processing::sort::SortOrder,
    ) -> Ordering {
        let comparison = a.partial_cmp(b).unwrap_or(Ordering::Equal);

        match order {
            crate::query::executor::result_processing::sort::SortOrder::Asc => comparison,
            crate::query::executor::result_processing::sort::SortOrder::Desc => {
                comparison.reverse()
            }
        }
    }

    /// 判断是否为升序
    fn is_ascending(&self) -> bool {
        self.sort_keys
            .first()
            .map(|key| {
                matches!(
                    key.order,
                    crate::query::executor::result_processing::sort::SortOrder::Asc
                )
            })
            .unwrap_or(true)
    }

    /// 应用限制和偏移
    fn apply_limit_and_offset(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        // 应用偏移量
        if self.offset > 0 {
            if self.offset < dataset.rows.len() {
                dataset.rows.drain(0..self.offset);
            } else {
                dataset.rows.clear();
            }
        }

        // 应用限制
        dataset.rows.truncate(self.n);

        Ok(dataset)
    }

    /// 对顶点列表执行 TopN
    ///
    /// 使用排序键对顶点进行排序，支持基于属性的复杂排序
    /// 参考nebula-graph的TopNExecutor实现，使用堆排序优化
    fn execute_topn_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        if vertices.is_empty() || self.n == 0 {
            return Ok(Vec::new());
        }

        let total_size = vertices.len();
        let heap_size = self.calculate_heap_size(total_size);

        if heap_size == 0 {
            return Ok(Vec::new());
        }

        // 计算maxCount：最终需要保留的元素数量
        let max_count = if total_size <= self.offset {
            0
        } else if total_size > self.offset + self.n {
            self.n
        } else {
            total_size - self.offset
        };

        if max_count == 0 {
            return Ok(Vec::new());
        }

        // 参考nebula-graph的TopNExecutor实现
        // 1. 先计算所有元素的排序值
        let mut vertices_with_sort_values: Vec<(Vec<Value>, crate::core::Vertex)> = vertices
            .into_iter()
            .map(|vertex| {
                let sort_values = self.calculate_vertex_sort_values(&vertex)?;
                Ok((sort_values, vertex))
            })
            .collect::<DBResult<Vec<_>>>()?;

        // 2. 使用select_nth_unstable优化TopN查询
        if vertices_with_sort_values.len() > heap_size {
            // 使用select_nth_unstable选择前heap_size个元素
            vertices_with_sort_values.select_nth_unstable_by(heap_size, |a, b| {
                self.compare_sort_values(&a.0, &b.0)
            });
            vertices_with_sort_values.truncate(heap_size);
        }

        // 3. 对选中的元素进行完整排序
        vertices_with_sort_values.sort_by(|a, b| self.compare_sort_values(&a.0, &b.0));

        // 4. 应用offset和limit
        let start = self.offset.min(vertices_with_sort_values.len());
        let end = (self.n + self.offset).min(vertices_with_sort_values.len());

        Ok(vertices_with_sort_values.into_iter().skip(start).take(end - start).map(|(_, v)| v).collect())
    }

    /// 对边列表执行 TopN
    ///
    /// 使用排序键对边进行排序，支持基于属性的复杂排序
    /// 参考nebula-graph的TopNExecutor实现，使用堆排序优化
    fn execute_topn_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<Vec<crate::core::Edge>> {
        if edges.is_empty() || self.n == 0 {
            return Ok(Vec::new());
        }

        let total_size = edges.len();
        let heap_size = self.calculate_heap_size(total_size);

        if heap_size == 0 {
            return Ok(Vec::new());
        }

        // 计算maxCount：最终需要保留的元素数量
        let max_count = if total_size <= self.offset {
            0
        } else if total_size > self.offset + self.n {
            self.n
        } else {
            total_size - self.offset
        };

        if max_count == 0 {
            return Ok(Vec::new());
        }

        // 参考nebula-graph的TopNExecutor实现
        // 1. 先计算所有元素的排序值
        let mut edges_with_sort_values: Vec<(Vec<Value>, crate::core::Edge)> = edges
            .into_iter()
            .map(|edge| {
                let sort_values = self.calculate_edge_sort_values(&edge)?;
                Ok((sort_values, edge))
            })
            .collect::<DBResult<Vec<_>>>()?;

        // 2. 使用select_nth_unstable优化TopN查询
        if edges_with_sort_values.len() > heap_size {
            // 使用select_nth_unstable选择前heap_size个元素
            edges_with_sort_values.select_nth_unstable_by(heap_size, |a, b| {
                self.compare_sort_values(&a.0, &b.0)
            });
            edges_with_sort_values.truncate(heap_size);
        }

        // 3. 对选中的元素进行完整排序
        edges_with_sort_values.sort_by(|a, b| self.compare_sort_values(&a.0, &b.0));

        // 4. 应用offset和limit
        let start = self.offset.min(edges_with_sort_values.len());
        let end = (self.n + self.offset).min(edges_with_sort_values.len());

        Ok(edges_with_sort_values.into_iter().skip(start).take(end - start).map(|(_, e)| e).collect())
    }

    /// 对值列表执行 TopN
    ///
    /// 使用排序键对值列表进行排序
    fn execute_topn_values(&self, values: Vec<Value>) -> DBResult<Vec<Value>> {
        if values.is_empty() || self.n == 0 {
            return Ok(Vec::new());
        }

        let total_size = values.len();
        let heap_size = self.calculate_heap_size(total_size);

        if heap_size == 0 {
            return Ok(Vec::new());
        }

        // 将Value包装为单行数据以便复用排序逻辑
        let mut rows: Vec<Vec<Value>> = values.into_iter().map(|v| vec![v]).collect();

        // 使用堆排序实现TopN
        let heap_size = self.n + self.offset;

        if rows.len() <= heap_size {
            // 数据量小于等于heap_size，直接排序
            rows.sort_by(|a, b| self.compare_rows(a, b).unwrap_or(Ordering::Equal));
        } else {
            // 使用TopN算法
            rows.select_nth_unstable_by(heap_size, |a, b| {
                self.compare_rows(a, b).unwrap_or(Ordering::Equal)
            });
            rows.truncate(heap_size);
            rows.sort_by(|a, b| self.compare_rows(a, b).unwrap_or(Ordering::Equal));
        }

        // 应用offset和limit
        let start = self.offset.min(rows.len());
        let end = (self.n + self.offset).min(rows.len());

        Ok(rows.into_iter().skip(start).take(end - start).map(|row| row.into_iter().next().unwrap()).collect())
    }

    /// 计算堆大小
    fn calculate_heap_size(&self, total_size: usize) -> usize {
        if total_size <= self.offset {
            0
        } else if total_size > self.offset + self.n {
            self.offset + self.n
        } else {
            total_size
        }
    }

    /// 计算顶点的排序值
    fn calculate_vertex_sort_values(&self, vertex: &crate::core::Vertex) -> DBResult<Vec<Value>> {
        let mut sort_values = Vec::with_capacity(self.sort_keys.len());

        for sort_key in &self.sort_keys {
            let value = self.extract_value_from_vertex(vertex, &sort_key.expression)?;
            sort_values.push(value);
        }

        Ok(sort_values)
    }

    /// 计算边的排序值
    fn calculate_edge_sort_values(&self, edge: &crate::core::Edge) -> DBResult<Vec<Value>> {
        let mut sort_values = Vec::with_capacity(self.sort_keys.len());

        for sort_key in &self.sort_keys {
            let value = self.extract_value_from_edge(edge, &sort_key.expression)?;
            sort_values.push(value);
        }

        Ok(sort_values)
    }

    /// 从顶点中提取值
    fn extract_value_from_vertex(&self, vertex: &crate::core::Vertex, expression: &Expression) -> DBResult<Value> {
        match expression {
            Expression::Variable(name) => {
                // 尝试从属性中获取
                if let Some(value) = vertex.get_property_any(name) {
                    Ok(value.clone())
                } else if name == "vid" || name == "_vid" {
                    Ok(*vertex.vid.clone())
                } else if name == "id" || name == "_id" {
                    Ok(Value::Int(vertex.id))
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Expression::Property { object, property } => {
                if object.as_ref() == &Expression::Variable("v".to_string())
                    || object.as_ref() == &Expression::Variable("vertex".to_string()) {
                    if let Some(value) = vertex.get_property_any(property) {
                        Ok(value.clone())
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Expression::Literal(value) => Ok(value.clone()),
            _ => {
                // 对于复杂表达式，使用表达式求值器
                let mut context = DefaultExpressionContext::new();
                context.set_variable("vid".to_string(), *vertex.vid.clone());
                context.set_variable("id".to_string(), Value::Int(vertex.id));

                // 添加所有标签属性到上下文
                for tag in &vertex.tags {
                    for (prop_name, prop_value) in &tag.properties {
                        context.set_variable(prop_name.clone(), prop_value.clone());
                    }
                }

                ExpressionEvaluator::evaluate(expression, &mut context)
                    .map_err(|e| DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    /// 从边中提取值
    fn extract_value_from_edge(&self, edge: &crate::core::Edge, expression: &Expression) -> DBResult<Value> {
        match expression {
            Expression::Variable(name) => {
                if name == "src" || name == "_src" {
                    Ok(*edge.src.clone())
                } else if name == "dst" || name == "_dst" {
                    Ok(*edge.dst.clone())
                } else if name == "ranking" || name == "_ranking" {
                    Ok(Value::Int(edge.ranking))
                } else if name == "edge_type" || name == "_type" {
                    Ok(Value::String(edge.edge_type.clone()))
                } else if let Some(value) = edge.get_property(name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Expression::Property { object, property } => {
                if object.as_ref() == &Expression::Variable("e".to_string())
                    || object.as_ref() == &Expression::Variable("edge".to_string()) {
                    if let Some(value) = edge.get_property(property) {
                        Ok(value.clone())
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Expression::Literal(value) => Ok(value.clone()),
            _ => {
                // 对于复杂表达式，使用表达式求值器
                let mut context = DefaultExpressionContext::new();
                context.set_variable("src".to_string(), *edge.src.clone());
                context.set_variable("dst".to_string(), *edge.dst.clone());
                context.set_variable("ranking".to_string(), Value::Int(edge.ranking));
                context.set_variable("edge_type".to_string(), Value::String(edge.edge_type.clone()));

                // 添加所有属性到上下文
                for (prop_name, prop_value) in &edge.props {
                    context.set_variable(prop_name.clone(), prop_value.clone());
                }

                ExpressionEvaluator::evaluate(expression, &mut context)
                    .map_err(|e| DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    /// 比较排序值
    fn compare_sort_values(&self, a: &[Value], b: &[Value]) -> Ordering {
        for (idx, (val_a, val_b)) in a.iter().zip(b.iter()).enumerate() {
            let order = if idx < self.sort_keys.len() {
                &self.sort_keys[idx].order
            } else {
                &crate::query::executor::result_processing::sort::SortOrder::Asc
            };

            let comparison = match val_a.partial_cmp(val_b) {
                Some(cmp) => cmp,
                None => continue,
            };

            if comparison != Ordering::Equal {
                return match order {
                    crate::query::executor::result_processing::sort::SortOrder::Asc => comparison,
                    crate::query::executor::result_processing::sort::SortOrder::Desc => comparison.reverse(),
                };
            }
        }
        Ordering::Equal
    }

    /// 比较两行数据（用于值列表排序）
    fn compare_rows(&self, a: &[Value], b: &[Value]) -> DBResult<Ordering> {
        // 创建虚拟列名
        let col_names: Vec<String> = (0..a.len().max(b.len())).map(|i| format!("col_{}", i)).collect();

        // 计算排序值
        let sort_values_a = self.calculate_sort_value(a, &col_names)?;
        let sort_values_b = self.calculate_sort_value(b, &col_names)?;

        Ok(self.compare_sort_values(&sort_values_a, &sort_values_b))
    }

    /// 提取排序值
    #[allow(dead_code)]
    fn extract_sort_values(&self, row: &[Value]) -> Result<Vec<Value>, TopNError> {
        let mut sort_values = Vec::with_capacity(self.sort_columns.len());

        for sort_col in &self.sort_columns {
            if sort_col.column_index >= row.len() {
                return Err(TopNError::InvalidColumnIndex(sort_col.column_index));
            }

            let value = &row[sort_col.column_index];

            // 处理 NULL 值排序
            let adjusted_value = if value.is_null() {
                if sort_col.nulls_first {
                    Value::Null(crate::core::value::NullType::Null)
                } else {
                    Value::Null(crate::core::value::NullType::Null)
                }
            } else {
                value.clone()
            };

            sort_values.push(adjusted_value);
        }

        Ok(sort_values)
    }

    /// 反转排序值（用于最大堆）
    #[allow(dead_code)]
    fn invert_sort_values(&self, mut sort_values: Vec<Value>) -> Result<Vec<Value>, TopNError> {
        for value in &mut sort_values {
            if !value.is_null() {
                *value = self.invert_value_for_sorting(value)?;
            }
        }
        Ok(sort_values)
    }

    /// 反转单个值的比较逻辑
    #[allow(dead_code)]
    fn invert_value_for_sorting(&self, value: &Value) -> Result<Value, TopNError> {
        match value {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            Value::String(s) => {
                let reversed: String = s.chars().rev().collect();
                Ok(Value::String(reversed))
            }
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Ok(value.clone()),
        }
    }

    /// 动态调整堆容量
    #[allow(dead_code)]
    fn optimize_heap_capacity(&mut self) {
        if let Some(ref mut heap) = self.heap {
            let current_capacity = heap.capacity();
            let ideal_capacity = self.n + 10;

            if current_capacity > ideal_capacity * 2 {
                let mut new_heap = BinaryHeap::with_capacity(ideal_capacity);
                while let Some(item) = heap.pop() {
                    new_heap.push(item);
                    if new_heap.len() >= self.n {
                        break;
                    }
                }
                self.heap = Some(new_heap);
            }
        }
    }

    /// 检查是否超出内存限制
    #[allow(dead_code)]
    fn exceeds_memory_limit(&self) -> bool {
        let estimated_memory = self.heap.as_ref().map_or(0, |h| h.len()) * 100;
        estimated_memory > 100 * 1024 * 1024
    }

    /// 获取堆大小
    pub fn get_heap_size(&self) -> usize {
        self.heap.as_ref().map_or(0, |h| h.len())
    }

    /// 获取已处理记录数
    pub fn get_processed_count(&self) -> usize {
        self.processed_count
    }

    /// 配置排序参数
    pub fn configure_sorting(
        &mut self,
        sort_columns: Vec<SortColumn>,
        sort_direction: OrderDirection,
    ) -> Result<(), TopNError> {
        if self.is_open {
            return Err(TopNError::ExecutorAlreadyOpen);
        }
        self.sort_columns = sort_columns;
        self.sort_direction = sort_direction;
        Ok(())
    }

    /// 推入堆中
    pub fn push_to_heap(&mut self, item: TopNItem) -> Result<(), TopNError> {
        if self.heap.is_none() {
            self.heap = Some(BinaryHeap::with_capacity(self.n + 1));
        }

        let heap = self.heap.as_mut().expect("heap should be initialized");

        heap.push(item);

        if heap.len() > self.n {
            heap.pop();
        }

        Ok(())
    }

    /// 从堆中弹出
    pub fn pop_from_heap(&mut self) -> Option<TopNItem> {
        self.heap.as_mut()?.pop()
    }
}

/// TopN 堆项
#[derive(Debug, Clone)]
pub struct TopNItem {
    sort_value: Vec<Value>,
    _original_index: usize,
    row: Vec<Value>,
}

impl PartialEq for TopNItem {
    fn eq(&self, other: &Self) -> bool {
        self.sort_value == other.sort_value
    }
}

impl Eq for TopNItem {}

impl PartialOrd for TopNItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopNItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // 正常比较，BinaryHeap 是最大堆
        self.sort_value.cmp(&other.sort_value)
    }
}

/// 用于降序排序的堆项（实现最小堆效果）
#[derive(Debug, Clone)]
struct TopNItemDesc {
    sort_value: Vec<Value>,
    _original_index: usize,
    row: Vec<Value>,
}

impl PartialEq for TopNItemDesc {
    fn eq(&self, other: &Self) -> bool {
        self.sort_value == other.sort_value
    }
}

impl Eq for TopNItemDesc {}

impl PartialOrd for TopNItemDesc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopNItemDesc {
    fn cmp(&self, other: &Self) -> Ordering {
        other.sort_value.cmp(&self.sort_value)
    }
}

/// 并行 TopN 使用的项（支持 partial_cmp）
#[derive(Debug, Clone)]
struct TopNItemParallel {
    sort_value: Vec<Value>,
    row: Vec<Value>,
}

impl PartialEq for TopNItemParallel {
    fn eq(&self, other: &Self) -> bool {
        self.sort_value == other.sort_value
    }
}

impl Eq for TopNItemParallel {}

impl PartialOrd for TopNItemParallel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sort_value.partial_cmp(&other.sort_value)
    }
}

impl Ord for TopNItemParallel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_value.cmp(&other.sort_value)
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for TopNExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for TopNExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        self.base.input = Some(input.clone());
        self.process_input()
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for TopNExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(DataSet::new()))
        };

        match input_result {
            ExecutionResult::DataSet(dataset) => {
                let topn_result = self.execute_topn_dataset(dataset)?;
                Ok(ExecutionResult::DataSet(topn_result))
            }
            ExecutionResult::Vertices(vertices) => {
                let topn_result = self.execute_topn_vertices(vertices)?;
                Ok(ExecutionResult::Vertices(topn_result))
            }
            ExecutionResult::Edges(edges) => {
                let topn_result = self.execute_topn_edges(edges)?;
                Ok(ExecutionResult::Edges(topn_result))
            }
            ExecutionResult::Values(values) => {
                let topn_result = self.execute_topn_values(values)?;
                Ok(ExecutionResult::Values(topn_result))
            }
            _ => Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "TopN executor expects supported input type".to_string(),
                ),
            )),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        if self.is_open {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Executor already open".to_string(),
                ),
            ));
        }

        if self.input_executor.is_none() {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Missing input executor".to_string(),
                ),
            ));
        }

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }

        self.is_open = true;
        self.is_closed = false;
        self.heap = Some(BinaryHeap::with_capacity(self.n + 1));
        self.processed_count = 0;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if !self.is_open || self.is_closed {
            return Ok(());
        }

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }

        self.heap = None;
        self.is_closed = true;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.id > 0
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

impl<S: StorageClient + Send + Sync + 'static> TopNExecutor<S> {
    pub fn execute_with_recovery(&mut self) -> DBResult<ExecutionResult> {
        match self.execute() {
            Ok(result) => Ok(result),
            Err(DBError::Query(crate::core::error::QueryError::ExecutionError(ref msg)))
                if msg.contains("memory") || msg.contains("limit") =>
            {
                self.fallback_to_external_sort()
            }
            Err(e) => Err(e),
        }
    }

    fn fallback_to_external_sort(&mut self) -> DBResult<ExecutionResult> {
        Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
            "Memory limit exceeded, consider reducing the dataset size or N value".to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[tokio::test]
    async fn test_topn_executor_basic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "score".to_string()];
        for i in 1..=10 {
            dataset.rows.push(vec![
                Value::String(format!("User{}", i)),
                Value::Int(i * 10),
            ]);
        }

        // 创建 TopN 执行器 (取前3名，按分数降序)
        let mut executor = TopNExecutor::new(1, storage, 3, vec!["score".to_string()], false);

        // 执行 TopN
        let result = executor
            .process(ExecutionResult::DataSet(dataset))
            .expect("TopN executor should process successfully");

        // 验证结果
        match result {
            ExecutionResult::DataSet(topn_dataset) => {
                assert_eq!(topn_dataset.rows.len(), 3);
                // 验证按分数降序排列
                assert_eq!(topn_dataset.rows[0][1], Value::Int(100)); // User10
                assert_eq!(topn_dataset.rows[1][1], Value::Int(90)); // User9
                assert_eq!(topn_dataset.rows[2][1], Value::Int(80)); // User8
            }
            _ => panic!("Expected DataSet result"),
        }
    }

    #[tokio::test]
    async fn test_topn_executor_with_offset() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let values: Vec<Value> = (1..=10).map(|i| Value::Int(i)).collect();

        // 创建 TopN 执行器 (取3-5名，按数值升序)
        let mut executor =
            TopNExecutor::new(1, storage, 3, vec!["value".to_string()], true).with_offset(2);

        // 执行 TopN
        let result = executor
            .process(ExecutionResult::Values(values))
            .expect("TopN executor should process successfully");

        // 验证结果
        match result {
            ExecutionResult::Values(topn_values) => {
                assert_eq!(topn_values.len(), 3);
                assert_eq!(topn_values[0], Value::Int(3)); // 跳过前2个，取3-5
                assert_eq!(topn_values[1], Value::Int(4));
                assert_eq!(topn_values[2], Value::Int(5));
            }
            _ => panic!("Expected Values result"),
        }
    }
}
