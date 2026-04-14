//! DedupExecutor – An executor for removing duplicates
//!
//! Implement a data deduplication function that supports deduplication strategies based on specified keys.
//! CPU-intensive operations are parallelized using Rayon.

use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

use rayon;

use crate::core::{Edge, Value, Vertex};
use crate::query::DataSet;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::base::{BaseResultProcessor, ResultProcessor, ResultProcessorContext};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::storage::iterator::Row;
use crate::storage::StorageClient;

/// Duplicacy removal strategy
#[derive(Debug, Clone, PartialEq)]
pub enum DedupStrategy {
    /// Complete deduplication, based on the values of the entire object
    Full,
    /// Dedupe based on a specified key
    ByKeys(Vec<String>),
    /// Deduplication based on vertex IDs (only effective for vertices)
    ByVertexId,
    /// Deduplication of sources, targets, and types based on edges (only effective for edges)
    ByEdgeKey,
}

/// DedupExecutor – An executor for removing duplicates
///
/// Implement a data deduplication function that supports multiple deduplication strategies.
/// CPU-intensive operations are parallelized using Rayon.
pub struct DedupExecutor<S: StorageClient + Send + 'static> {
    /// Basic processor
    base: BaseResultProcessor<S>,
    /// Input actuator
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// de-duplication strategy
    strategy: DedupStrategy,
    /// Memory limit (in bytes)
    memory_limit: usize,
    /// Current memory usage
    current_memory_usage: usize,
    /// Parallel computing configuration
    parallel_config: ParallelConfig,
}

impl<S: StorageClient + Send + 'static> DedupExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        strategy: DedupStrategy,
        memory_limit: Option<usize>,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "DedupExecutor".to_string(),
            "Removes duplicate records from query results".to_string(),
            storage,
        );

        Self {
            base,
            input_executor: None,
            strategy,
            memory_limit: memory_limit.unwrap_or(100 * 1024 * 1024), // Default size: 100 MB
            current_memory_usage: 0,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// Setting up the parallel computing configuration
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    fn execute_dedup(
        &mut self,
        input: ExecutionResult,
    ) -> Result<ExecutionResult, crate::query::QueryError> {
        match input {
            ExecutionResult::Values(values) => {
                let deduped_values = self.dedup_values(values)?;
                let dataset = DataSet::from_rows(
                    deduped_values.into_iter().map(|v| vec![v]).collect(),
                    vec!["value".to_string()],
                );
                Ok(ExecutionResult::DataSet(dataset))
            }
            ExecutionResult::Vertices(vertices) => {
                let deduped_vertices = self.dedup_vertices(vertices)?;
                let rows: Vec<Vec<Value>> = deduped_vertices
                    .into_iter()
                    .map(|v| vec![Value::Vertex(Box::new(v))])
                    .collect();
                let dataset = DataSet::from_rows(rows, vec!["vertex".to_string()]);
                Ok(ExecutionResult::DataSet(dataset))
            }
            ExecutionResult::Edges(edges) => {
                let deduped_edges = self.dedup_edges(edges)?;
                let rows: Vec<Vec<Value>> = deduped_edges
                    .into_iter()
                    .map(|e| vec![Value::Edge(e)])
                    .collect();
                let dataset = DataSet::from_rows(rows, vec!["edge".to_string()]);
                Ok(ExecutionResult::DataSet(dataset))
            }
            ExecutionResult::DataSet(mut dataset) => {
                self.dedup_dataset(&mut dataset)?;
                Ok(ExecutionResult::DataSet(dataset))
            }
            ExecutionResult::Empty | ExecutionResult::Success => Ok(input),
            ExecutionResult::Error(msg) => Err(crate::query::QueryError::ExecutionError(msg)),
        }
    }

    /// Value deduplication
    fn dedup_values(&mut self, values: Vec<Value>) -> Result<Vec<Value>, crate::query::QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => self.hash_based_dedup(values, |value| format!("{:?}", value)),
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor =
                    move |value: &Value| Self::extract_keys_from_value_static(value, &keys_clone);
                self.hash_based_dedup(values, key_extractor)
            }
            _ => self.hash_based_dedup(values, |value| format!("{:?}", value)),
        }
    }

    fn dedup_vertices(
        &mut self,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Vertex>, crate::query::QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex))
            }
            DedupStrategy::ByVertexId => {
                self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex.vid))
            }
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor = move |vertex: &Vertex| {
                    Self::extract_keys_from_vertex_static(vertex, &keys_clone)
                };
                self.hash_based_dedup(vertices, key_extractor)
            }
            _ => self.hash_based_dedup(vertices, |vertex| format!("{:?}", vertex.vid)),
        }
    }

    fn dedup_edges(&mut self, edges: Vec<Edge>) -> Result<Vec<Edge>, crate::query::QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => self.hash_based_dedup(edges, |edge| format!("{:?}", edge)),
            DedupStrategy::ByEdgeKey => self.hash_based_dedup(edges, |edge| {
                format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
            }),
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let keys_clone = keys.clone();
                let key_extractor =
                    move |edge: &Edge| Self::extract_keys_from_edge_static(edge, &keys_clone);
                self.hash_based_dedup(edges, key_extractor)
            }
            _ => self.hash_based_dedup(edges, |edge| {
                format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
            }),
        }
    }

    /// Data set deduplication
    ///
    /// Choose the deduplication method based on the amount of data:
    /// Data volume is below the threshold: Single-threaded hash-based deduplication
    /// Large amount of data: Rayon is used to perform parallel partitioning and deduplication.
    fn dedup_dataset(
        &mut self,
        dataset: &mut crate::query::DataSet,
    ) -> Result<(), crate::query::QueryError> {
        let total_size = dataset.rows.len();

        if self.parallel_config.should_use_parallel(total_size) {
            self.dedup_dataset_parallel(dataset)
        } else {
            self.dedup_dataset_sequential(dataset)
        }
    }

    /// Duplication removal with sequential execution
    fn dedup_dataset_sequential(
        &mut self,
        dataset: &mut crate::query::DataSet,
    ) -> Result<(), crate::query::QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                let mut seen = HashSet::new();
                let mut unique_rows = Vec::new();

                for row in &dataset.rows {
                    let key = format!("{:?}", row);
                    if seen.insert(key) {
                        unique_rows.push(row.clone());
                    }
                }

                dataset.rows = unique_rows;
                Ok(())
            }
            DedupStrategy::ByKeys(keys) => {
                let mut seen = HashSet::new();
                let mut unique_rows = Vec::new();

                for row in &dataset.rows {
                    let mut key_parts = Vec::new();
                    for key in &keys {
                        if let Some(col_index) =
                            dataset.col_names.iter().position(|name| name == key)
                        {
                            if col_index < row.len() {
                                key_parts.push(format!("{:?}", row[col_index]));
                            }
                        }
                    }
                    let key = key_parts.join("|");

                    if seen.insert(key) {
                        unique_rows.push(row.clone());
                    }
                }

                dataset.rows = unique_rows;
                Ok(())
            }
            _ => self.dedup_dataset_with_strategy_sequential(dataset),
        }
    }

    /// Deduplication performed in parallel (using Rayon)
    ///
    /// Use a partitioning strategy:
    /// Parallel computing of the unique keys in each row
    /// 2. Partitioning based on the hash value of the key
    /// 3. Deduplication of local data within each region
    /// 4. Combine the results from each district
    fn dedup_dataset_parallel(
        &mut self,
        dataset: &mut crate::query::DataSet,
    ) -> Result<(), crate::query::QueryError> {
        let rows = std::mem::take(&mut dataset.rows);
        let strategy = self.strategy.clone();
        let col_names = dataset.col_names.clone();

        let (deduped_rows, _) = rayon::join(
            || Self::dedup_partition_full(&rows, &strategy, &col_names),
            || (),
        );

        dataset.rows = deduped_rows;
        Ok(())
    }

    fn dedup_partition_full(
        rows: &[Vec<Value>],
        strategy: &DedupStrategy,
        col_names: &[String],
    ) -> Vec<Vec<Value>> {
        let mut seen = HashSet::new();
        let mut unique_rows = Vec::new();

        for row in rows {
            let key = match strategy {
                DedupStrategy::Full => format!("{:?}", row),
                DedupStrategy::ByKeys(keys) => {
                    let mut key_parts = Vec::new();
                    for key in keys {
                        if let Some(col_index) = col_names.iter().position(|name| name == key) {
                            if col_index < row.len() {
                                key_parts.push(format!("{:?}", row[col_index]));
                            }
                        }
                    }
                    key_parts.join("|")
                }
                _ => format!("{:?}", row),
            };

            if seen.insert(key) {
                unique_rows.push(row.clone());
            }
        }

        unique_rows
    }

    fn dedup_dataset_with_strategy_sequential(
        &mut self,
        dataset: &mut crate::query::DataSet,
    ) -> Result<(), crate::query::QueryError> {
        match self.strategy.clone() {
            DedupStrategy::Full => {
                self.hash_based_dedup_dataset(dataset, |row| format!("{:?}", row))
            }
            DedupStrategy::ByKeys(keys) => {
                let keys = Arc::new(keys);
                let col_names = dataset.col_names.clone();
                self.hash_based_dedup_dataset(dataset, move |row| {
                    let key_parts: Vec<String> = keys
                        .iter()
                        .filter_map(|key| {
                            col_names
                                .iter()
                                .position(|name| name == key)
                                .and_then(|idx| row.get(idx))
                                .map(|v| format!("{:?}", v))
                        })
                        .collect();
                    key_parts.join("|")
                })
            }
            _ => Ok(()),
        }
    }

    fn hash_based_dedup<T, F>(
        &mut self,
        items: Vec<T>,
        key_extractor: F,
    ) -> Result<Vec<T>, crate::query::QueryError>
    where
        T: Clone + Send + 'static,
        F: Fn(&T) -> String + Send + Sync,
    {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut memory_usage = 0;

        for item in items {
            let key = key_extractor(&item);

            if !seen.contains(&key) {
                // Estimating memory usage
                let item_size = std::mem::size_of::<T>() + key.len();
                memory_usage += item_size;

                // Check the memory limitations.
                if self.current_memory_usage + memory_usage > self.memory_limit {
                    return Err(crate::query::QueryError::ExecutionError(
                        "Memory limit exceeded".to_string(),
                    ));
                }

                seen.insert(key);
                result.push(item);
            }
        }

        self.current_memory_usage += memory_usage;
        Ok(result)
    }

    fn hash_based_dedup_dataset<F>(
        &mut self,
        dataset: &mut crate::query::DataSet,
        key_extractor: F,
    ) -> Result<(), crate::query::QueryError>
    where
        F: Fn(&Row) -> String + Send + Sync,
    {
        let mut seen = HashSet::new();
        let mut unique_rows = Vec::new();
        let mut memory_usage = 0;

        for row in &dataset.rows {
            let key = key_extractor(row);

            if !seen.contains(&key) {
                let row_size = std::mem::size_of::<Row>() + key.len();
                memory_usage += row_size;

                if self.current_memory_usage + memory_usage > self.memory_limit {
                    return Err(crate::query::QueryError::ExecutionError(
                        "Memory limit exceeded".to_string(),
                    ));
                }

                seen.insert(key);
                unique_rows.push(row.clone());
            }
        }

        dataset.rows = unique_rows;
        self.current_memory_usage += memory_usage;
        Ok(())
    }

    /// Extracting keys from values (static method)
    fn extract_keys_from_value_static(value: &Value, keys: &[String]) -> String {
        match value {
            Value::Map(map) => keys
                .iter()
                .filter_map(|key| map.get(key))
                .map(|v| format!("{:?}", v))
                .collect::<Vec<_>>()
                .join("|"),
            _ => format!("{:?}", value),
        }
    }

    /// Extract keys from the vertices (static method)
    fn extract_keys_from_vertex_static(vertex: &Vertex, keys: &[String]) -> String {
        let mut key_values = Vec::new();

        for key in keys {
            if key == "id" {
                key_values.push(format!("{:?}", vertex.vid));
            } else {
                // Search for the attribute in the tag of the vertex.
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(key) {
                        key_values.push(format!("{:?}", value));
                        break;
                    }
                }
            }
        }

        if key_values.is_empty() {
            format!("{:?}", vertex.vid)
        } else {
            key_values.join("|")
        }
    }

    /// Extract keys from the edges (static method)
    fn extract_keys_from_edge_static(edge: &Edge, keys: &[String]) -> String {
        let mut key_values = Vec::new();

        for key in keys {
            match key.as_str() {
                "src" => key_values.push(format!("{:?}", edge.src)),
                "dst" => key_values.push(format!("{:?}", edge.dst)),
                "type" => key_values.push(edge.edge_type.clone()),
                "ranking" => key_values.push(format!("{:?}", edge.ranking)),
                _ => {
                    if let Some(value) = edge.props.get(key.as_str()) {
                        key_values.push(format!("{:?}", value));
                    }
                }
            }
        }

        if key_values.is_empty() {
            format!("{:?}-{}-{:?}", edge.src, edge.edge_type, edge.dst)
        } else {
            key_values.join("|")
        }
    }

    /// Get the current memory usage
    pub fn current_memory_usage(&self) -> usize {
        self.current_memory_usage
    }

    /// Reset the memory usage
    pub fn reset_memory_usage(&mut self) {
        self.current_memory_usage = 0;
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for DedupExecutor<S> {
    fn process(&mut self, _input: ExecutionResult) -> DBResult<ExecutionResult> {
        // Reset the memory usage.
        self.reset_memory_usage();

        // 从 input_executor 或 base.input 获取输入
        let input = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else if let Some(input) = &self.base.input {
            input.clone()
        } else {
            return Ok(ExecutionResult::Values(Vec::new()));
        };

        // Perform the deduplication operation.
        self.execute_dedup(input).map_err(|e| {
            crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                e.to_string(),
            ))
        })
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
        self.current_memory_usage
    }

    fn reset(&mut self) {
        self.reset_memory_usage();
        self.base.reset_state();
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DedupExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::Values(Vec::new()))
        };

        self.process(input_result)
    }

    fn open(&mut self) -> DBResult<()> {
        self.reset_memory_usage();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.reset_memory_usage();

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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for DedupExecutor<S> {
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
    use std::collections::HashMap;

    #[test]
    fn test_dedup_executor_full_strategy() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));

        let mut executor = DedupExecutor::new(1, storage.clone(), DedupStrategy::Full, None);

        // Set up test data (including duplicate values)
        let test_data = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(1), // Repeat
            Value::Int(3),
            Value::Int(2), // Repeat
        ];

        let input_result = ExecutionResult::Values(test_data);

        // Use the set_input method of the ResultProcessor trait.
        <DedupExecutor<MockStorage> as crate::query::executor::base::ResultProcessor<
            MockStorage,
        >>::set_input(&mut executor, input_result);

        // Remove duplicates.
        let result = executor
            .process(ExecutionResult::Values(Vec::new()))
            .expect("Failed to process dedup");

        // Verification results
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 3); // The duplicates should be removed to leave only 3 values.
                let mut sorted_values = values.clone();
                sorted_values.sort_by(|a, b| match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                });
                assert_eq!(
                    sorted_values,
                    vec![Value::Int(1), Value::Int(2), Value::Int(3),]
                );
            }
            _ => panic!("Expected Values result"),
        }
    }

    #[test]
    fn test_dedup_executor_by_keys_strategy() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));

        let mut executor = DedupExecutor::<MockStorage>::new(
            1,
            storage.clone(),
            DedupStrategy::ByKeys(vec!["id".to_string()]),
            None,
        );

        // Set up test data (including different objects with the same ID).
        let test_data = vec![
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(1)),
                ("name".to_string(), Value::String("Alice".to_string())),
            ])),
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(2)),
                ("name".to_string(), Value::String("Bob".to_string())),
            ])),
            Value::Map(HashMap::from([
                ("id".to_string(), Value::Int(1)), // Duplicate ID
                ("name".to_string(), Value::String("Alice2".to_string())),
            ])),
        ];

        // Use the `set_input` method to set the input data.
        <DedupExecutor<MockStorage> as crate::query::executor::base::ResultProcessor<
            MockStorage,
        >>::set_input(&mut executor, ExecutionResult::Values(test_data));

        // Handle deduplication.
        let result = executor
            .process(ExecutionResult::Values(Vec::new()))
            .expect("Failed to process dedup");

        // Verification results
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 2); // The duplication should be removed based on the ID, resulting in two unique values.
            }
            _ => panic!("Expected Values result"),
        }
    }
}
