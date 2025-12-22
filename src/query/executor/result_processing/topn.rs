//! TopN 执行器
//!
//! 实现高效的 TopN 查询，使用堆数据结构优化性能

use async_trait::async_trait;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Value};
use crate::core::context::expression::ExpressionContextCore;
use crate::core::context::expression::{ExpressionContext, DefaultExpressionContext};
use crate::core::{Expression, ExpressionEvaluator};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{
    ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata,
};
use crate::storage::StorageEngine;

/// TopNExecutor - TOP N 结果执行器
///
/// 返回排序后的前 N 个结果，是 Sort + Limit 的优化版本
/// 使用堆数据结构实现高效的 TopN 查询
pub struct TopNExecutor<S: StorageEngine> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 返回的结果数量
    n: usize,
    /// 偏移量
    offset: usize,
    /// 排序键列表
    sort_keys: Vec<crate::query::executor::result_processing::sort::SortKey>,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 是否使用堆优化
    use_heap_optimization: bool,
}

impl<S: StorageEngine> TopNExecutor<S> {
    pub fn new(
        id: usize,
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
                    Expression::Property {
                        object: Box::new(Expression::Variable("row".to_string())),
                        property: col,
                    },
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
            use_heap_optimization: true,
        }
    }

    /// 设置偏移量
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// 启用/禁用堆优化
    pub fn with_heap_optimization(mut self, enable: bool) -> Self {
        self.use_heap_optimization = enable;
        self
    }

    /// 处理输入数据并执行 TopN
    async fn process_input(&mut self) -> DBResult<ExecutionResult> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

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

        if self.use_heap_optimization && dataset.rows.len() > self.n + self.offset {
            self.heap_optimized_topn_dataset(&mut dataset)?;
        } else {
            // 对于小数据集，直接排序
            self.sort_dataset(&mut dataset)?;
        }

        self.apply_limit_and_offset(dataset)
    }

    /// 使用堆优化的 TopN 算法处理数据集
    fn heap_optimized_topn_dataset(&self, dataset: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;
        let heap_size = self.n + self.offset;

        // 创建最小堆（用于升序）或最大堆（用于降序）
        let mut heap = BinaryHeap::with_capacity(heap_size);

        // 处理前 heap_size 个元素
        for (i, row) in dataset.rows.iter().enumerate().take(heap_size) {
            let sort_value = self.calculate_sort_value(row, &dataset.col_names, &evaluator)?;
            heap.push(TopNItem {
                sort_value,
                original_index: i,
                row: row.clone(),
            });
        }

        // 处理剩余元素
        for (i, row) in dataset.rows.iter().enumerate().skip(heap_size) {
            let sort_value = self.calculate_sort_value(row, &dataset.col_names, &evaluator)?;
            let new_item = TopNItem {
                sort_value,
                original_index: i,
                row: row.clone(),
            };

            // 根据排序方向决定是否替换堆顶元素
            if let Some(peeked) = heap.peek() {
                let should_replace = if self.is_ascending() {
                    // 升序：维护最大堆，如果新元素小于堆顶，则替换
                    new_item < *peeked
                } else {
                    // 降序：维护最小堆，如果新元素大于堆顶，则替换
                    new_item > *peeked
                };

                if should_replace {
                    heap.pop();
                    heap.push(new_item);
                }
            }
        }

        // 提取并排序结果
        let mut items: Vec<TopNItem> = heap.into_iter().collect();
        items.sort_by(|a, b| {
            if self.is_ascending() {
                a.sort_value.cmp(&b.sort_value)
            } else {
                b.sort_value.cmp(&a.sort_value)
            }
        });

        // 更新数据集
        dataset.rows = items
            .into_iter()
            .skip(self.offset)
            .map(|item| item.row)
            .collect();

        Ok(())
    }

    /// 计算行的排序值
    fn calculate_sort_value(
        &self,
        row: &[Value],
        col_names: &[String],
        evaluator: &ExpressionEvaluator,
    ) -> DBResult<Vec<Value>> {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let mut sort_values = Vec::new();
        for sort_key in &self.sort_keys {
            let value = evaluator
                .evaluate(&sort_key.expression, &context)
                .map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "Failed to evaluate sort expression: {}",
                        e
                    )))
                })?;
            sort_values.push(value);
        }

        Ok(sort_values)
    }

    /// 对数据集进行排序
    fn sort_dataset(&self, dataset: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;

        dataset.rows.sort_by(|a, b| {
            let sort_a = match self.calculate_sort_value(a, &dataset.col_names, &evaluator) {
                Ok(val) => val,
                Err(_) => return Ordering::Less, // 如果计算排序值失败，将此行放在前面
            };
            let sort_b = match self.calculate_sort_value(b, &dataset.col_names, &evaluator) {
                Ok(val) => val,
                Err(_) => return Ordering::Greater, // 如果计算排序值失败，将此行放在后面
            };

            // 逐个比较排序键
            for ((idx, sort_val_a), sort_val_b) in sort_a.iter().enumerate().zip(sort_b.iter()) {
                let comparison =
                    self.compare_values(sort_val_a, sort_val_b, &self.sort_keys[idx].order);
                if !comparison.is_eq() {
                    return comparison;
                }
            }
            Ordering::Equal
        });

        Ok(())
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
}

/// TopN 堆项
#[derive(Debug, Clone)]
struct TopNItem {
    sort_value: Vec<Value>,
    original_index: usize,
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
        // 反向比较，以便 BinaryHeap 作为最大堆使用
        other.sort_value.cmp(&self.sort_value)
    }
}

impl<S: StorageEngine> InputExecutor<S> for TopNExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for TopNExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        self.base.input = Some(input.clone());
        self.process_input().await
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
        self.base.memory_usage = 0;
        self.base.input = None;
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for TopNExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，使用设置的输入数据
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result).await
    }
}

impl<S: StorageEngine + Send> ExecutorLifecycle for TopNExecutor<S> {
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
        self.base.id > 0 // 简单的状态检查
    }
}

impl<S: StorageEngine> ExecutorMetadata for TopNExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for TopNExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;

    // 模拟存储引擎
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            Ok(crate::core::Value::Null(NullType::NaN))
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            Ok(1)
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn scan_all_vertices(
            &self,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(
            &self,
            _tag: &str,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }
    }

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

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::DataSet(dataset));

        // 执行 TopN
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
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

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::Values(values));

        // 执行 TopN
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
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
