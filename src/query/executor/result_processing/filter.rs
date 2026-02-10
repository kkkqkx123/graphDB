//! 过滤执行器
//!
//! 实现对查询结果的条件过滤功能，支持 HAVING 子句
//! CPU 密集型操作，使用 Rayon 进行并行化

use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use crate::core::error::{DBError, DBResult};
use crate::core::value::DataSet;
use crate::core::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

/// FilterExecutor - 过滤执行器
///
/// 实现对查询结果的条件过滤功能
/// CPU 密集型操作，使用 Rayon 进行并行化
pub struct FilterExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 过滤条件表达式
    condition: Expression,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 并行计算配置
    parallel_config: ParallelConfig,
}

impl<S: StorageClient + Send + 'static> FilterExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, condition: Expression) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "FilterExecutor".to_string(),
            "Filters query results based on specified conditions".to_string(),
            storage,
        );

        Self {
            base,
            condition,
            input_executor: None,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    /// 处理输入数据并应用过滤条件
    fn process_input(&mut self) -> DBResult<ExecutionResult> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;
            self.filter_input(input_result)
        } else if let Some(input) = &self.base.input {
            self.filter_input(input.clone())
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Filter executor requires input".to_string(),
                ),
            ))
        }
    }

    /// 过滤输入数据
    fn filter_input(&self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        match input {
            ExecutionResult::DataSet(mut dataset) => {
                self.apply_filter(&mut dataset)?;
                Ok(ExecutionResult::DataSet(dataset))
            }
            ExecutionResult::Values(values) => {
                let filtered_values = self.filter_values(values)?;
                Ok(ExecutionResult::Values(filtered_values))
            }
            ExecutionResult::Vertices(vertices) => {
                let filtered_vertices = self.filter_vertices(vertices)?;
                Ok(ExecutionResult::Vertices(filtered_vertices))
            }
            ExecutionResult::Edges(edges) => {
                let filtered_edges = self.filter_edges(edges)?;
                Ok(ExecutionResult::Edges(filtered_edges))
            }
            _ => Ok(input),
        }
    }

    /// 对数据集应用过滤条件
    ///
    /// 根据配置选择过滤方式：
    /// - 数据量小于阈值：单线程处理
    /// - 数据量大：使用 Rayon 并行处理
    fn apply_filter(&self, dataset: &mut DataSet) -> DBResult<()> {
        let total_size = dataset.rows.len();

        // 根据并行配置判断是否使用并行计算
        if !self.parallel_config.should_use_parallel(total_size) {
            // 数据量小或禁用并行，使用单线程处理
            self.apply_filter_single(dataset)
        } else {
            // 数据量大，使用 Rayon 并行处理
            let batch_size = self.parallel_config.calculate_batch_size(total_size);
            self.apply_filter_parallel(dataset, batch_size)
        }
    }

    /// 计算批量大小
    ///
    /// 使用并行配置的 calculate_batch_size 方法
    fn calculate_batch_size(&self, total_size: usize) -> usize {
        self.parallel_config.calculate_batch_size(total_size)
    }

    /// 单线程过滤
    fn apply_filter_single(&self, dataset: &mut DataSet) -> DBResult<()> {
        let mut filtered_rows = Vec::new();

        for row in &dataset.rows {
            let mut context = DefaultExpressionContext::new();

            // 设置列名作为变量
            for (i, col_name) in dataset.col_names.iter().enumerate() {
                if i < row.len() {
                    context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 设置 row 变量（包含整行数据）
            let row_map: std::collections::HashMap<String, crate::core::Value> = dataset
                .col_names
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if i < row.len() {
                        Some((name.clone(), row[i].clone()))
                    } else {
                        None
                    }
                })
                .collect();
            context.set_variable("row".to_string(), crate::core::Value::Map(row_map));

            let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
                .map_err(|e| {
                    DBError::Expression(crate::core::error::ExpressionError::function_error(
                        format!("Failed to evaluate filter condition: {}", e),
                    ))
                })?;

            if let crate::core::Value::Bool(true) = condition_result {
                filtered_rows.push(row.clone());
            }
        }

        dataset.rows = filtered_rows;
        Ok(())
    }

    /// 并行过滤
    fn apply_filter_parallel(&self, dataset: &mut DataSet, batch_size: usize) -> DBResult<()> {
        let col_names = dataset.col_names.clone();
        let condition = self.condition.clone();

        let filtered_rows: Vec<Vec<Value>> = dataset
            .rows
            .par_chunks(batch_size)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .filter_map(|row| {
                        let mut context = DefaultExpressionContext::new();
                        for (i, col_name) in col_names.iter().enumerate() {
                            if i < row.len() {
                                context.set_variable(col_name.clone(), row[i].clone());
                            }
                        }

                        match ExpressionEvaluator::evaluate(&condition, &mut context) {
                            Ok(crate::core::Value::Bool(true)) => Some(row.clone()),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        dataset.rows = filtered_rows;
        Ok(())
    }

    /// 过滤值列表
    fn filter_values(&self, values: Vec<crate::core::Value>) -> DBResult<Vec<crate::core::Value>> {
        let mut filtered_values = Vec::new();

        for value in values {
            // 构建表达式上下文
            let mut context = DefaultExpressionContext::new();
            context.set_variable("value".to_string(), value.clone());

            // 评估过滤条件
            let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
                .map_err(|e| {
                    DBError::Expression(crate::core::error::ExpressionError::function_error(
                        format!("Failed to evaluate filter condition: {}", e),
                    ))
                })?;

            // 如果条件为真，保留该值
            if let crate::core::Value::Bool(true) = condition_result {
                filtered_values.push(value);
            }
        }

        Ok(filtered_values)
    }

    /// 过滤顶点列表
    fn filter_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        let mut filtered_vertices = Vec::new();

        for vertex in vertices {
            // 构建表达式上下文
            let mut context = DefaultExpressionContext::new();
            // 设置顶点信息
            context.set_variable(
                "_vertex".to_string(),
                Value::Vertex(Box::new(vertex.clone())),
            );

            // 评估过滤条件
            let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
                .map_err(|e| {
                    DBError::Expression(crate::core::error::ExpressionError::function_error(
                        format!("Failed to evaluate filter condition: {}", e),
                    ))
                })?;

            // 如果条件为真，保留该顶点
            if let crate::core::Value::Bool(true) = condition_result {
                filtered_vertices.push(vertex);
            }
        }

        Ok(filtered_vertices)
    }

    /// 过滤边列表
    fn filter_edges(&self, edges: Vec<crate::core::Edge>) -> DBResult<Vec<crate::core::Edge>> {
        let mut filtered_edges = Vec::new();

        for edge in edges {
            // 构建表达式上下文
            let mut context = DefaultExpressionContext::new();
            // 设置边信息
            context.set_variable("_edge".to_string(), Value::Edge(edge.clone()));

            // 评估过滤条件
            let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
                .map_err(|e| {
                    DBError::Expression(crate::core::error::ExpressionError::function_error(
                        format!("Failed to evaluate filter condition: {}", e),
                    ))
                })?;

            // 如果条件为真，保留该边
            if let crate::core::Value::Bool(true) = condition_result {
                filtered_edges.push(edge);
            }
        }

        Ok(filtered_edges)
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for FilterExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        if self.input_executor.is_none() && self.base.input.is_none() {
            <Self as ResultProcessor<S>>::set_input(self, input.clone());
        }
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for FilterExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        let result = self.process(input_result);
        
        if let Ok(ref exec_result) = result {
            self.base.get_stats_mut().add_row(exec_result.count());
        }
        
        result
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for FilterExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_filter_executor_basic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        dataset.rows.push(vec![
            crate::core::Value::String("Alice".to_string()),
            crate::core::Value::Int(30),
        ]);
        dataset.rows.push(vec![
            crate::core::Value::String("Bob".to_string()),
            crate::core::Value::Int(25),
        ]);
        dataset.rows.push(vec![
            crate::core::Value::String("Charlie".to_string()),
            crate::core::Value::Int(35),
        ]);

        // 创建过滤执行器 (age > 25)
        let condition = Expression::Binary {
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("row".to_string())),
                property: "age".to_string(),
            }),
            op: crate::core::BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(crate::core::Value::Int(25))),
        };

        let mut executor = FilterExecutor::new(1, storage, condition);

        // 设置输入数据
        <FilterExecutor<MockStorage> as ResultProcessor<MockStorage>>::set_input(
            &mut executor,
            ExecutionResult::DataSet(dataset),
        );

        // 执行过滤
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to get next");

        // 验证结果
        match result {
            ExecutionResult::DataSet(filtered_dataset) => {
                assert_eq!(filtered_dataset.rows.len(), 2); // Alice 和 Charlie
                                                            // 验证年龄都大于25
                for row in &filtered_dataset.rows {
                    if let crate::core::Value::Int(age) = &row[1] {
                        assert!(age > &25);
                    }
                }
            }
            _ => panic!("Expected DataSet result"),
        }
    }
}
