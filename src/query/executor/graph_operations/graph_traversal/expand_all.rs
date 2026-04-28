use std::collections::HashSet;
use std::sync::Arc;

use crate::core::error::DBResult;
use crate::core::types::ContextualExpression;
use crate::core::{Edge, Expression, NPath, Path, Value, Vertex};
use crate::query::executor::base::ExecutorEnum;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::base::{ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::DataSet;
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// ExpandAllExecutor – An executor that performs full-path expansion
///
/// Return all possible paths starting from the current node, not just the next-hop node.
/// Usually used in path exploration queries
pub struct ExpandAllExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub any_edge_type: bool,
    pub max_depth: Option<usize>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    // Use the NPath cache to store intermediate results and reduce the amount of memory copying.
    npath_cache: Vec<Arc<NPath>>,
    // Path caching (converted during the final output process)
    path_cache: Vec<Path>,
    // Set of visited nodes, used to avoid loops.
    pub visited_nodes: HashSet<Value>,
    // Source vertex IDs for starting the expansion (from GO FROM clause)
    pub src_vids: Vec<Value>,
    // Whether to include empty paths (paths with no edges) in the result
    pub include_empty_paths: bool,
    // Input variable name for getting input from ExecutionContext
    pub input_var: Option<String>,
    // Column names for the output DataSet
    pub col_names: Vec<String>,
    // Space ID for the query
    pub space_id: u64,
    // Space name for storage operations
    pub space_name: String,
    // Filter condition pushed down from FilterNode
    filter: Option<Expression>,
}

// Manual Debug implementation for ExpandAllExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageClient> std::fmt::Debug for ExpandAllExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpandAllExecutor")
            .field("base", &"BaseExecutor")
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("path_cache", &self.path_cache)
            .field("visited_nodes", &self.visited_nodes)
            .finish()
    }
}

impl<S: StorageClient + Send> ExpandAllExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        any_edge_type: bool,
        max_depth: Option<usize>,
        expr_context: Arc<ExpressionAnalysisContext>,
        space_id: u64,
        space_name: String,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ExpandAllExecutor".to_string(), storage, expr_context),
            edge_direction,
            edge_types,
            any_edge_type,
            max_depth,
            input_executor: None,
            npath_cache: Vec::new(),
            path_cache: Vec::new(),
            visited_nodes: HashSet::new(),
            src_vids: Vec::new(),
            include_empty_paths: true, // Default to true for backward compatibility
            input_var: None,
            col_names: vec!["src".to_string(), "edge".to_string(), "dst".to_string()],
            space_id,
            space_name,
            filter: None,
        }
    }

    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        any_edge_type: bool,
        max_depth: Option<usize>,
        context: crate::query::executor::base::ExecutionContext,
        space_id: u64,
        space_name: String,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "ExpandAllExecutor".to_string(), storage, context),
            edge_direction,
            edge_types,
            any_edge_type,
            max_depth,
            input_executor: None,
            npath_cache: Vec::new(),
            path_cache: Vec::new(),
            visited_nodes: HashSet::new(),
            src_vids: Vec::new(),
            include_empty_paths: true,
            input_var: None,
            col_names: vec!["src".to_string(), "edge".to_string(), "dst".to_string()],
            space_id,
            space_name,
            filter: None,
        }
    }

    pub fn with_src_vids(mut self, src_vids: Vec<Value>) -> Self {
        self.src_vids = src_vids;
        self
    }

    pub fn with_include_empty_paths(mut self, include: bool) -> Self {
        self.include_empty_paths = include;
        self
    }

    pub fn with_input_var(mut self, input_var: String) -> Self {
        self.input_var = Some(input_var);
        self
    }

    pub fn with_col_names(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    pub fn with_filter(mut self, filter: Option<ContextualExpression>) -> Self {
        self.filter = filter.and_then(|ctx_expr| ctx_expr.expression().map(|meta| meta.inner().clone()));
        self
    }

    fn get_neighbors_with_edges(&self, node_id: &Value) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = self.base.get_storage().clone();
        let edge_types = if self.any_edge_type {
            None
        } else {
            self.edge_types.clone()
        };
        super::traversal_utils::get_neighbors_with_edges(
            &storage,
            node_id,
            self.edge_direction,
            &edge_types,
            false,
        )
        .map_err(|e| QueryError::StorageError(e.to_string()))
    }

    /// Recursive expansion of paths (synchronous version)
    fn expand_paths_recursive(
        &mut self,
        current_npath: &Arc<NPath>,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<Vec<Arc<NPath>>, QueryError> {
        // Get the last node of the current path.
        let current_node = &current_npath.vertex().vid;

        // Check whether the maximum depth has been reached.
        if current_depth >= max_depth {
            // Return to the current path
            return Ok(vec![current_npath.clone()]);
        }

        // Obtaining neighbor nodes and edges
        let neighbors_with_edges = self.get_neighbors_with_edges(current_node)?;

        if neighbors_with_edges.is_empty() {
            // There are no more neighbors; return to the current path.
            return Ok(vec![current_npath.clone()]);
        }

        let mut all_npaths: Vec<Arc<NPath>> = Vec::new();

        // Create a new path for each neighbor.
        for (neighbor_id, edge) in neighbors_with_edges {
            // Check whether the node has already been visited (to avoid loops).
            if self.visited_nodes.contains(&neighbor_id) {
                // Create a path that contains loops.
                let path_with_cycle = Arc::new(NPath::extend(
                    current_npath.clone(),
                    Arc::new(edge),
                    Arc::new(Vertex::new(neighbor_id.clone(), Vec::new())),
                ));
                all_npaths.push(path_with_cycle);
                continue;
            }

            // Obtain the complete information of the neighboring nodes.
            let neighbor_vertex = {
                let storage = self.get_storage().lock();
                storage
                    .get_vertex("default", &neighbor_id)
                    .map_err(|e| QueryError::StorageError(e.to_string()))?
            };

            // Create a vertex object: If the vertex already exists, use the actual vertex; otherwise, create a suspended vertex (with an empty Tag list).
            let vertex = match neighbor_vertex {
                Some(v) => v,
                None => {
                    // Suspension edge processing: Create a vertex for an empty Tag, while retaining the VID (Video Identifier).
                    Vertex::new(neighbor_id.clone(), Vec::new())
                }
            };

            // Using NPath expansion, O(1) operation
            let new_npath = Arc::new(NPath::extend(
                current_npath.clone(),
                Arc::new(edge),
                Arc::new(vertex),
            ));

            // Marked as visited
            self.visited_nodes.insert(neighbor_id.clone());

            // Recursive expansion (continuing to expand in order to obtain more edges, even if the vertex is "hanging"/not directly connected to other nodes in the graph).
            let mut expanded_npaths =
                self.expand_paths_recursive(&new_npath, current_depth + 1, max_depth)?;
            all_npaths.append(&mut expanded_npaths);

            // Unmark (allows access from other paths)
            self.visited_nodes.remove(&neighbor_id);
        }

        // Add the current path
        all_npaths.push(current_npath.clone());

        Ok(all_npaths)
    }

    /// Construct the extended result.
    ///
    /// Returns a DataSet with columns from self.col_names (typically ["src", "edge", "dst"] or
    /// with a custom dst column name like ["src", "edge", "b"]) for each path step.
    /// This format allows subsequent operations to easily access the source vertex,
    /// edge, and destination vertex separately.
    fn build_expansion_result(&self) -> ExecutionResult {
        // Convert NPath to Path for output.
        let paths: Vec<Path> = self.npath_cache.iter().map(|np| np.to_path()).collect();

        // Build a DataSet with separate columns for src, edge, and dst
        // Use the configured column names, which may include custom dst column names
        let mut dataset = crate::query::DataSet::new();
        dataset.col_names = self.col_names.clone();

        let target_depth = self.max_depth.unwrap_or(1);

        // Determine if we have additional columns beyond src/edge/dst
        // These are typically edge type aliases for property access (e.g., KNOWS.since)
        let has_edge_alias = self.col_names.len() > 3;
        let edge_alias_index = if has_edge_alias { Some(3) } else { None };

        for path in &paths {
            // Skip empty paths if include_empty_paths is false
            if !self.include_empty_paths && path.steps.is_empty() {
                continue;
            }

            // For GO queries (include_empty_paths is false), only return the last step of paths
            // with exactly the target depth
            // For other queries (include_empty_paths is true), return all steps
            if self.include_empty_paths {
                // For each step in the path, create a row with src, edge, dst
                for step in &path.steps {
                    let mut row = vec![
                        Value::Vertex(path.src.clone()),
                        Value::edge((*step.edge).clone()),
                        Value::Vertex(Box::new((*step.dst).clone())),
                    ];
                    // Add edge alias column if present (duplicates edge value for property access)
                    if let Some(idx) = edge_alias_index {
                        if idx < self.col_names.len() {
                            row.push(Value::edge((*step.edge).clone()));
                        }
                    }
                    
                    // Apply filter if present
                    if self.should_include_row(&row, &dataset.col_names) {
                        dataset.rows.push(row);
                    }
                }

                // If include_empty_paths is true and path has no steps, add a row with just src
                if path.steps.is_empty() {
                    let mut row = vec![
                        Value::Vertex(path.src.clone()),
                        Value::Null(crate::core::NullType::Null),
                        Value::Null(crate::core::NullType::Null),
                    ];
                    // Add null for edge alias column if present
                    if edge_alias_index.is_some() {
                        row.push(Value::Null(crate::core::NullType::Null));
                    }
                    
                    // Apply filter if present
                    if self.should_include_row(&row, &dataset.col_names) {
                        dataset.rows.push(row);
                    }
                }
            } else if path.steps.len() == target_depth {
                // For GO queries, only add the last step
                if let Some(last_step) = path.steps.last() {
                    let mut row = vec![
                        Value::Vertex(path.src.clone()),
                        Value::edge((*last_step.edge).clone()),
                        Value::Vertex(Box::new((*last_step.dst).clone())),
                    ];
                    // Add edge alias column if present (duplicates edge value for property access)
                    if let Some(idx) = edge_alias_index {
                        if idx < self.col_names.len() {
                            row.push(Value::edge((*last_step.edge).clone()));
                        }
                    }
                    
                    // Apply filter if present
                    if self.should_include_row(&row, &dataset.col_names) {
                        dataset.rows.push(row);
                    }
                }
            }
        }

        ExecutionResult::DataSet(dataset)
    }
    
    /// Check if a row should be included based on the filter condition
    fn should_include_row(&self, row: &[Value], col_names: &[String]) -> bool {
        if let Some(ref filter) = self.filter {
            let mut context = DefaultExpressionContext::new();
            
            // Set column values as variables
            for (i, col_name) in col_names.iter().enumerate() {
                if i < row.len() {
                    context.set_variable(col_name.clone(), row[i].clone());
                }
            }
            
            // Map GO query special variables: $$ -> dst, $^ -> src, target -> dst, edge -> edge
            if let Some(dst_idx) = col_names.iter().position(|c| c == "dst") {
                if dst_idx < row.len() {
                    context.set_variable("$$".to_string(), row[dst_idx].clone());
                    context.set_variable("target".to_string(), row[dst_idx].clone());
                }
            }
            if let Some(src_idx) = col_names.iter().position(|c| c == "src") {
                if src_idx < row.len() {
                    context.set_variable("$^".to_string(), row[src_idx].clone());
                }
            }
            if let Some(edge_idx) = col_names.iter().position(|c| c == "edge") {
                if edge_idx < row.len() {
                    context.set_variable("edge".to_string(), row[edge_idx].clone());
                    // Map edge type name to the edge value for GO queries like WHERE friend.strength > 5
                    if let Value::Edge(ref edge_val) = row[edge_idx] {
                        context.set_variable(edge_val.edge_type().to_string(), row[edge_idx].clone());
                    }
                }
            }
            
            // Evaluate the filter condition
            match ExpressionEvaluator::evaluate(filter, &mut context) {
                Ok(Value::Bool(true)) => true,
                _ => false,
            }
        } else {
            true
        }
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ExpandAllExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for ExpandAllExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // Clear caches to ensure fresh results for each execution
        // This prevents duplicate results when the executor is reused
        self.npath_cache.clear();
        self.path_cache.clear();
        self.visited_nodes.clear();

        // First, execute the input executor (if it exists).
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else if let Some(ref input_var) = self.input_var {
            // Try to get input from ExecutionContext
            self.base
                .context
                .get_result(input_var)
                .unwrap_or_else(|| ExecutionResult::DataSet(DataSet::new()))
        } else {
            // If no actuator is specified, return an empty result.
            ExecutionResult::DataSet(DataSet::new())
        };

        // Extract the input node.
        let mut input_nodes = match input_result {
            ExecutionResult::DataSet(dataset) => dataset
                .rows
                .into_iter()
                .flat_map(|row| row.into_iter())
                .filter_map(|v| match v {
                    Value::Vertex(vertex) => Some(*vertex),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        // If src_vids is set (from GO FROM clause), add those vertices as input nodes
        if !self.src_vids.is_empty() {
            let storage = self.get_storage().lock();
            for vid in &self.src_vids {
                match storage.get_vertex(&self.space_name, vid) {
                    Ok(Some(vertex)) => {
                        input_nodes.push(vertex);
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
        }

        // Determine the maximum depth.
        let max_depth = self.max_depth.unwrap_or(3); // The default depth is 3.

        // Generate a path for each input node.
        for vertex in input_nodes {
            // Reset the access status
            self.visited_nodes.clear();
            self.visited_nodes.insert((*vertex.vid).clone());

            // Create the initial NPath.
            let initial_npath = Arc::new(NPath::new(Arc::new(vertex)));

            // Recursive expansion of the path
            let mut expanded_npaths = self.expand_paths_recursive(&initial_npath, 0, max_depth)?;
            self.npath_cache.append(&mut expanded_npaths);
        }

        // Build the results.
        Ok(self.build_expansion_result())
    }

    fn open(&mut self) -> DBResult<()> {
        self.npath_cache.clear();
        self.path_cache.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.npath_cache.clear();
        self.path_cache.clear();
        self.visited_nodes.clear();

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

impl<S: StorageClient + Send> HasStorage<S> for ExpandAllExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ExpandAllExecutor storage should be set")
    }
}
