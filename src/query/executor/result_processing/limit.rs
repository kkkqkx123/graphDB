//! 限制执行器
//!
//! 实现对查询结果的数量限制和偏移功能，支持 LIMIT 和 OFFSET 操作

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Value};
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

/// 限制执行器 - 实现LIMIT和OFFSET功能
pub struct LimitExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 限制数量
    limit: Option<usize>,
    /// 偏移量
    offset: usize,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageClient + Send + 'static> LimitExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, limit: Option<usize>, offset: usize) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "LimitExecutor".to_string(),
            "Limits query results with LIMIT and OFFSET".to_string(),
            storage,
        );

        Self {
            base,
            limit,
            offset,
            input_executor: None,
        }
    }

    /// 仅设置LIMIT
    pub fn with_limit(id: i64, storage: Arc<Mutex<S>>, limit: usize) -> Self {
        Self::new(id, storage, Some(limit), 0)
    }

    /// 仅设置OFFSET
    pub fn with_offset(id: i64, storage: Arc<Mutex<S>>, offset: usize) -> Self {
        Self::new(id, storage, None, offset)
    }

    /// 处理输入数据并应用限制
    fn process_input(&mut self) -> DBResult<DataSet> {
        // 优先使用 input_executor
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;
            self.apply_limits_to_input(input_result)
        } else if let Some(input) = &self.base.input {
            // 使用 base.input 作为备选
            self.apply_limits_to_input(input.clone())
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Limit executor requires input".to_string(),
                ),
            ))
        }
    }

    /// 对输入应用限制
    fn apply_limits_to_input(&self, input: ExecutionResult) -> DBResult<DataSet> {
        match input {
            ExecutionResult::DataSet(mut data_set) => {
                self.apply_limits(&mut data_set)?;
                Ok(data_set)
            }
            ExecutionResult::Values(values) => {
                let dataset = self.apply_values_limit(values)?;
                Ok(dataset)
            }
            ExecutionResult::Vertices(vertices) => {
                let dataset = self.apply_vertices_limit(vertices)?;
                Ok(dataset)
            }
            ExecutionResult::Edges(edges) => {
                let dataset = self.apply_edges_limit(edges)?;
                Ok(dataset)
            }
            _ => Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Limit executor expects DataSet, Values, Vertices, or Edges input"
                        .to_string(),
                ),
            )),
        }
    }

    /// 对数据集应用限制
    fn apply_limits(&self, data_set: &mut DataSet) -> DBResult<()> {
        // 应用偏移量
        if self.offset > 0 {
            if self.offset < data_set.rows.len() {
                data_set.rows.drain(0..self.offset);
            } else {
                data_set.rows.clear();
            }
        }

        // 应用限制
        if let Some(limit) = self.limit {
            data_set.rows.truncate(limit);
        }

        Ok(())
    }

    /// 对值列表应用限制
    fn apply_values_limit(&self, mut values: Vec<Value>) -> DBResult<DataSet> {
        // 应用偏移量
        if self.offset > 0 {
            if self.offset < values.len() {
                values.drain(0..self.offset);
            } else {
                values.clear();
            }
        }

        // 应用限制
        if let Some(limit) = self.limit {
            values.truncate(limit);
        }

        Ok(DataSet {
            col_names: vec!["_value".to_string()], // 单列数据
            rows: values.into_iter().map(|v| vec![v]).collect(),
        })
    }

    /// 对顶点列表应用限制
    fn apply_vertices_limit(&self, mut vertices: Vec<crate::core::Vertex>) -> DBResult<DataSet> {
        // 应用偏移量
        if self.offset > 0 {
            if self.offset < vertices.len() {
                vertices.drain(0..self.offset);
            } else {
                vertices.clear();
            }
        }

        // 应用限制
        if let Some(limit) = self.limit {
            vertices.truncate(limit);
        }

        // 将顶点转换为数据集
        let rows: Vec<Vec<Value>> = vertices
            .into_iter()
            .map(|v| vec![Value::Vertex(Box::new(v))])
            .collect();

        Ok(DataSet {
            col_names: vec!["_vertex".to_string()],
            rows,
        })
    }

    /// 对边列表应用限制
    fn apply_edges_limit(&self, mut edges: Vec<crate::core::Edge>) -> DBResult<DataSet> {
        // 应用偏移量
        if self.offset > 0 {
            if self.offset < edges.len() {
                edges.drain(0..self.offset);
            } else {
                edges.clear();
            }
        }

        // 应用限制
        if let Some(limit) = self.limit {
            edges.truncate(limit);
        }

        // 将边转换为数据集
        let rows: Vec<Vec<Value>> = edges.into_iter().map(|e| vec![Value::Edge(e)]).collect();

        Ok(DataSet {
            col_names: vec!["_edge".to_string()],
            rows,
        })
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for LimitExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        if self.input_executor.is_none() && self.base.input.is_none() {
            ResultProcessor::set_input(self, input);
        }
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for LimitExecutor<S> {
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for LimitExecutor<S> {
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

    #[tokio::test]
    async fn test_limit_executor_basic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        for i in 1..=10 {
            dataset.rows.push(vec![
                Value::String(format!("User{}", i)),
                Value::Int(i * 10),
            ]);
        }

        // 创建限制执行器 (LIMIT 5 OFFSET 2)
        let mut executor = LimitExecutor::new(1, storage, Some(5), 2);

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::DataSet(dataset));

        // 执行限制
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process limit");

        // 验证结果
        match result {
            ExecutionResult::DataSet(limited_dataset) => {
                assert_eq!(limited_dataset.rows.len(), 5);
                // 验证跳过了前2行，取了5行
                assert_eq!(limited_dataset.rows[0][1], Value::Int(30)); // User3
                assert_eq!(limited_dataset.rows[4][1], Value::Int(70)); // User7
            }
            _ => panic!("Expected DataSet result"),
        }
    }

    #[tokio::test]
    async fn test_limit_executor_only_limit() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let values: Vec<Value> = (1..=10).map(|i| Value::Int(i)).collect();

        // 创建限制执行器 (仅 LIMIT 3)
        let mut executor = LimitExecutor::with_limit(1, storage, 3);

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::Values(values));

        // 执行限制
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process limit");

        // 验证结果
        match result {
            ExecutionResult::DataSet(limited_dataset) => {
                assert_eq!(limited_dataset.rows.len(), 3);
                assert_eq!(limited_dataset.col_names, vec!["_value"]);
                assert_eq!(limited_dataset.rows[0][0], Value::Int(1));
                assert_eq!(limited_dataset.rows[2][0], Value::Int(3));
            }
            _ => panic!("Expected DataSet result"),
        }
    }
}
