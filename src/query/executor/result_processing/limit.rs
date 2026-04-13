//! Limit the execution of the actuator
//!
//! Implementing a limit on the number of query results and an offset function, supporting the LIMIT and OFFSET operations.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult};
use crate::query::DataSet;
use crate::core::Value;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::base::{BaseResultProcessor, ResultProcessor, ResultProcessorContext};
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageClient;

/// Limiting actuators – Implementing the LIMIT and OFFSET functions
pub struct LimitExecutor<S: StorageClient + Send + 'static> {
    /// Basic processor
    base: BaseResultProcessor<S>,
    /// Limit the quantity
    limit: Option<usize>,
    /// Offset
    offset: usize,
    /// Input actuator
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

    /// Only set the LIMIT
    pub fn with_limit(id: i64, storage: Arc<Mutex<S>>, limit: usize) -> Self {
        Self::new(id, storage, Some(limit), 0)
    }

    /// Only set the OFFSET.
    pub fn with_offset(id: i64, storage: Arc<Mutex<S>>, offset: usize) -> Self {
        Self::new(id, storage, None, offset)
    }

    /// Process the input data and apply the relevant restrictions.
    fn process_input(&mut self) -> DBResult<DataSet> {
        // Give priority to using the `inputExecutor`.
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

    /// Apply restrictions to the input.
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
                    "Limit executor expects DataSet, Values, Vertices, or Edges input".to_string(),
                ),
            )),
        }
    }

    /// Applying restrictions to the dataset
    fn apply_limits(&self, data_set: &mut DataSet) -> DBResult<()> {
        // Application offset
        if self.offset > 0 {
            if self.offset < data_set.rows.len() {
                data_set.rows.drain(0..self.offset);
            } else {
                data_set.rows.clear();
            }
        }

        // Application restrictions
        if let Some(limit) = self.limit {
            data_set.rows.truncate(limit);
        }

        Ok(())
    }

    /// Applying restrictions to a list of values
    fn apply_values_limit(&self, mut values: Vec<Value>) -> DBResult<DataSet> {
        // Application offset
        if self.offset > 0 {
            if self.offset < values.len() {
                values.drain(0..self.offset);
            } else {
                values.clear();
            }
        }

        // Application restrictions
        if let Some(limit) = self.limit {
            values.truncate(limit);
        }

        Ok(DataSet {
            col_names: vec!["_value".to_string()], // Single-column data
            rows: values.into_iter().map(|v| vec![v]).collect(),
        })
    }

    /// Apply restrictions to the list of vertices.
    fn apply_vertices_limit(&self, mut vertices: Vec<crate::core::Vertex>) -> DBResult<DataSet> {
        // Application offset
        if self.offset > 0 {
            if self.offset < vertices.len() {
                vertices.drain(0..self.offset);
            } else {
                vertices.clear();
            }
        }

        // Application restrictions
        if let Some(limit) = self.limit {
            vertices.truncate(limit);
        }

        // Convert the vertices into a dataset.
        let rows: Vec<Vec<Value>> = vertices
            .into_iter()
            .map(|v| vec![Value::Vertex(Box::new(v))])
            .collect();

        Ok(DataSet {
            col_names: vec!["_vertex".to_string()],
            rows,
        })
    }

    /// Apply restrictions to the list of opposite sides.
    fn apply_edges_limit(&self, mut edges: Vec<crate::core::Edge>) -> DBResult<DataSet> {
        // Application offset
        if self.offset > 0 {
            if self.offset < edges.len() {
                edges.drain(0..self.offset);
            } else {
                edges.clear();
            }
        }

        // Application restrictions
        if let Some(limit) = self.limit {
            edges.truncate(limit);
        }

        // Convert the edges into a dataset.
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
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

    #[test]
    fn test_limit_executor_basic() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));

        // Create test data
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        for i in 1..=10 {
            dataset.rows.push(vec![
                Value::String(format!("User{}", i)),
                Value::Int(i * 10),
            ]);
        }

        // Create a limit executor (LIMIT 5 OFFSET 2)
        let mut executor = LimitExecutor::new(1, storage, Some(5), 2);

        // Setting the input data
        ResultProcessor::set_input(&mut executor, ExecutionResult::DataSet(dataset));

        // Enforce the restrictions.
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process limit");

        // Verification results
        match result {
            ExecutionResult::DataSet(limited_dataset) => {
                assert_eq!(limited_dataset.rows.len(), 5);
                // The validation process skipped the first 2 lines and selected the 5th line.
                assert_eq!(limited_dataset.rows[0][1], Value::Int(30)); // User3
                assert_eq!(limited_dataset.rows[4][1], Value::Int(70)); // User7
            }
            _ => panic!("Expected DataSet result"),
        }
    }

    #[test]
    fn test_limit_executor_only_limit() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));

        // Create test data
        let values: Vec<Value> = (1..=10).map(Value::Int).collect();

        // Create a limit executor (only allowing the execution of the LIMIT command 3 times).
        let mut executor = LimitExecutor::with_limit(1, storage, 3);

        // Setting the input data
        ResultProcessor::set_input(&mut executor, ExecutionResult::Values(values));

        // Enforce the restrictions.
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process limit");

        // Verification results
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
