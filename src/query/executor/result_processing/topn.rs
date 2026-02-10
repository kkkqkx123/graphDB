//! TopN 执行器
//!
//! 实现高效的 TopN 查询，使用堆数据结构优化性能

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::types::OrderDirection;
use crate::core::{DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
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
        }
    }

    /// 设置偏移量
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// 处理输入数据并执行 TopN
    async fn process_input(&mut self) -> DBResult<ExecutionResult> {
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
    fn execute_topn_dataset(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        if self.sort_keys.is_empty() {
            // 如果没有排序键，直接应用限制
            return self.apply_limit_and_offset(dataset);
        }

        // 统一使用堆排序算法
        let heap_size = self.n + self.offset;
        
        if self.is_ascending() {
            // 升序：使用最大堆，保留最小的N个元素
            self.heap_ascending(&mut dataset, heap_size)?;
        } else {
            // 降序：使用最小堆，保留最大的N个元素
            self.heap_descending(&mut dataset, heap_size)?;
        }

        Ok(dataset)
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
    fn execute_topn_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        // 简化实现：按顶点ID排序
        let mut vertices = vertices;
        if self.is_ascending() {
            vertices.sort_by(|a, b| a.vid.cmp(&b.vid));
        } else {
            vertices.sort_by(|a, b| b.vid.cmp(&a.vid));
        }

        let start = self.offset.min(vertices.len());
        let end = (self.n + self.offset).min(vertices.len());

        Ok(vertices[start..end].to_vec())
    }

    /// 对边列表执行 TopN
    fn execute_topn_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<Vec<crate::core::Edge>> {
        // 简化实现：按源顶点ID排序
        let mut edges = edges;
        if self.is_ascending() {
            edges.sort_by(|a, b| a.src.cmp(&b.src));
        } else {
            edges.sort_by(|a, b| b.src.cmp(&a.src));
        }

        let start = self.offset.min(edges.len());
        let end = (self.n + self.offset).min(edges.len());

        Ok(edges[start..end].to_vec())
    }

    /// 对值列表执行 TopN
    fn execute_topn_values(&self, values: Vec<Value>) -> DBResult<Vec<Value>> {
        // 简化实现：直接排序
        let mut values = values;
        if self.is_ascending() {
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        } else {
            values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
        }

        let start = self.offset.min(values.len());
        let end = (self.n + self.offset).min(values.len());

        Ok(values[start..end].to_vec())
    }

    /// 提取排序值
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
    fn invert_sort_values(&self, mut sort_values: Vec<Value>) -> Result<Vec<Value>, TopNError> {
        for value in &mut sort_values {
            if !value.is_null() {
                *value = self.invert_value_for_sorting(value)?;
            }
        }
        Ok(sort_values)
    }

    /// 反转单个值的比较逻辑
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
        // 对于降序TopN（取最大的N个元素），我们需要最小堆
        // 所以比较时让较小的元素排在前面（在BinaryHeap中，较大的元素会排在堆顶）
        // 但我们需要的是最小堆效果，所以应该让较小的元素有更高的优先级
        // 正确的实现：让较小的元素排在前面，这样在最大堆中，较小的元素会被放在堆顶
        other.sort_value.cmp(&self.sort_value)
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

    fn process_input_batch(&mut self, batch_size: usize) -> DBResult<ExecutionResult> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;

            match input_result {
                ExecutionResult::DataSet(dataset) => {
                    let mut batch = Vec::with_capacity(batch_size);
                    let mut all_results = Vec::new();

                    for row in dataset.rows.into_iter() {
                        batch.push(row);

                        if batch.len() >= batch_size {
                            let processed = self.process_batch(&batch)?;
                            all_results.extend(processed);
                            batch.clear();
                        }
                    }

                    if !batch.is_empty() {
                        let processed = self.process_batch(&batch)?;
                        all_results.extend(processed);
                    }

                    let result_dataset = DataSet {
                        col_names: dataset.col_names,
                        rows: all_results,
                    };
                    Ok(ExecutionResult::DataSet(result_dataset))
                }
                _ => Ok(input_result),
            }
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "TopN executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 处理批量数据
    fn process_batch(&mut self, batch: &[Vec<Value>]) -> DBResult<Vec<Vec<Value>>> {
        let mut results = Vec::new();

        for row in batch {
            if let Some(sort_value) = self.calculate_sort_value(row, &[]).ok() {
                let item = TopNItem {
                    sort_value,
                    _original_index: self.processed_count,
                    row: row.clone(),
                };

                if self.heap.is_none() {
                    self.heap = Some(BinaryHeap::with_capacity(self.n + 1));
                }

                if let Some(ref mut heap) = self.heap {
                    heap.push(item);
                    self.processed_count += 1;

                    if heap.len() > self.n {
                        heap.pop();
                    }
                }
            }
        }

        if let Some(ref mut heap) = self.heap {
            let mut items: Vec<TopNItem> = heap.drain().collect();
            items.sort_by(|a, b| {
                if self.is_ascending() {
                    a.sort_value.cmp(&b.sort_value)
                } else {
                    b.sort_value.cmp(&a.sort_value)
                }
            });

            results = items.into_iter().map(|item| item.row).collect();
        }

        Ok(results)
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
            .await
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
            .await
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
