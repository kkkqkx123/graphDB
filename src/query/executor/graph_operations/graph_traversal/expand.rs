use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{DBError, DBResult};
use crate::core::Value;
use crate::query::executor::base::ExecutorEnum;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::base::{ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::DataSet;
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// ExpandExecutor – An executor for path expansion (i.e., the process of extending or modifying paths in a given context)
///
/// Expand from the current node in the specified direction along the given edge type to obtain the adjacent nodes.
/// Supports multi-step expansion and sampling, and is commonly used for graph traversal and path querying.
pub struct ExpandExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,        // Maximum expansion depth
    pub step_limits: Option<Vec<usize>>, // Expansion limits for each step
    pub sample: bool,                    // Should sampling be enabled?
    pub sample_limit: Option<usize>,     // Sampling limitations
    pub with_loop: bool,                 // Are self-loop edges allowed?
    input_executor: Option<Box<ExecutorEnum<S>>>,
    // The cache stores the nodes that have been accessed, in order to avoid loops.
    pub visited_nodes: HashSet<Value>,
    // Adjacency relationship cache
    adjacency_cache: HashMap<Value, Vec<Value>>,
    // Current number of expansion steps
    current_step: usize,
}

// Manual Debug implementation for ExpandExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageClient> std::fmt::Debug for ExpandExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpandExecutor")
            .field("base", &"BaseExecutor")
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("visited_nodes", &self.visited_nodes)
            .field("adjacency_cache", &"HashMap<Value, Vec<Value>>")
            .finish()
    }
}

impl<S: StorageClient> ExpandExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ExpandExecutor".to_string(), storage, expr_context),
            edge_direction,
            edge_types,
            max_depth,
            step_limits: None,
            sample: false,
            sample_limit: None,
            with_loop: false,
            input_executor: None,
            visited_nodes: HashSet::new(),
            adjacency_cache: HashMap::new(),
            current_step: 0,
        }
    }

    /// Set the expansion limits for each step
    pub fn with_step_limits(mut self, step_limits: Vec<usize>) -> Self {
        self.step_limits = Some(step_limits);
        self
    }

    /// Enable sampling
    pub fn with_sampling(mut self, sample_limit: usize) -> Self {
        self.sample = true;
        self.sample_limit = Some(sample_limit);
        self
    }

    /// Set whether to allow self-looping edges.
    pub fn with_loop(mut self, with_loop: bool) -> Self {
        self.with_loop = with_loop;
        self
    }

    fn expand_multi_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        let max_steps = self.max_depth.unwrap_or(1);
        let mut current_nodes = input_nodes;
        let mut all_expanded = HashSet::new();

        for step in 0..max_steps {
            self.current_step = step;

            // Check the restrictions for each step.
            if let Some(ref step_limits) = self.step_limits {
                if step < step_limits.len() && current_nodes.len() > step_limits[step] {
                    // Application sampling
                    current_nodes = self.apply_sampling(&current_nodes, step_limits[step])?;
                }
            }

            // Execute single-step expansion.
            current_nodes = self.expand_step(current_nodes)?;

            // Check whether there are any additional nodes that can be expanded.
            if current_nodes.is_empty() {
                break;
            }

            // Record the extended nodes.
            for node in &current_nodes {
                all_expanded.insert(node.clone());
            }

            // Update statistical information
            self.base.get_stats_mut().add_stat(
                format!("step_{}_count", step),
                current_nodes.len().to_string(),
            );
        }

        Ok(all_expanded.into_iter().collect())
    }

    /// Application of the reservoir sampling algorithm
    fn apply_sampling(&self, nodes: &[Value], limit: usize) -> Result<Vec<Value>, QueryError> {
        if nodes.len() <= limit {
            return Ok(nodes.to_vec());
        }

        // Use the reservoir sampling algorithm
        let mut sampled = Vec::with_capacity(limit);
        for (i, node) in nodes.iter().enumerate() {
            if i < limit {
                sampled.push(node.clone());
            } else {
                let j = rand::random::<usize>() % (i + 1);
                if j < limit {
                    sampled[j] = node.clone();
                }
            }
        }

        Ok(sampled)
    }

    fn get_neighbors(&self, node_id: &Value) -> Result<Vec<Value>, QueryError> {
        let storage = self.base.get_storage().clone();
        super::traversal_utils::get_neighbors(
            &storage,
            node_id,
            self.edge_direction,
            &self.edge_types,
            self.with_loop,
        )
        .map_err(|e| QueryError::storage_error(e.to_string()))
    }

    /// Execute single-step expansion.
    fn expand_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        let mut expanded_nodes = Vec::new();

        for node_id in input_nodes {
            // Check whether the node has been accessed before.
            if self.visited_nodes.contains(&node_id) {
                continue;
            }

            // Marked as visited
            self.visited_nodes.insert(node_id.clone());

            // Obtaining neighbor nodes
            let neighbors = self.get_neighbors(&node_id)?;

            // Cache of adjacency relationships
            self.adjacency_cache
                .insert(node_id.clone(), neighbors.clone());

            // Add unvisited neighbor nodes
            for neighbor in neighbors {
                if !self.visited_nodes.contains(&neighbor) {
                    expanded_nodes.push(neighbor);
                }
            }
        }

        Ok(expanded_nodes)
    }

    /// Construct the extended result.
    fn build_expansion_result(&self, expanded_nodes: Vec<Value>) -> ExecutionResult {
        // Convert the node ID into a vertex object.
        let mut vertices = Vec::new();
        let storage = self.get_storage().lock();

        for node_id in expanded_nodes {
            if let Ok(Some(vertex)) = storage.get_vertex("default", &node_id) {
                vertices.push(vertex);
            }
        }

        let rows: Vec<Vec<Value>> = vertices
            .into_iter()
            .map(|v| vec![Value::Vertex(Box::new(v))])
            .collect();
        let dataset = DataSet::from_rows(rows, vec!["vertex".to_string()]);
        ExecutionResult::DataSet(dataset)
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ExpandExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for ExpandExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();

        // First, execute the input executor (if it exists).
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            // If no actuator is specified, return an empty result.
            ExecutionResult::DataSet(DataSet::new())
        };

        // Extract the input node.
        let input_nodes = match input_result {
            ExecutionResult::DataSet(dataset) => dataset
                .rows
                .into_iter()
                .flat_map(|row| row.into_iter())
                .filter_map(|v| match v {
                    Value::Vertex(vertex) => Some(*vertex.vid),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        // Perform the expansion operation.
        let expanded_nodes = if self.max_depth.unwrap_or(1) > 1 {
            self.expand_multi_step(input_nodes).map_err(DBError::from)?
        } else {
            self.expand_step(input_nodes).map_err(DBError::from)?
        };

        // Build the results.
        let result = self.build_expansion_result(expanded_nodes);

        // Update statistical information
        self.base.get_stats_mut().add_row(result.count());
        self.base.get_stats_mut().add_exec_time(start.elapsed());
        self.base.get_stats_mut().add_total_time(start.elapsed());

        Ok(result)
    }

    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources required for the extension.
        self.visited_nodes.clear();
        self.adjacency_cache.clear();
        self.current_step = 0;

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up resources
        self.visited_nodes.clear();
        self.adjacency_cache.clear();
        self.current_step = 0;

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
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

impl<S: StorageClient + Send> HasStorage<S> for ExpandExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ExpandExecutor storage should be set")
    }
}
