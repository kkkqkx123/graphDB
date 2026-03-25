//! Query Execution API – Core Layer
//!
//! Provides transport layer independent query execution

use crate::api::core::{CoreError, CoreResult, ExecutionMetadata, QueryRequest, QueryResult, Row};
use crate::core::StatsManager;
use crate::query::{OptimizerEngine, QueryPipelineManager};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Universal Query API – Core Layer
pub struct QueryApi<S: StorageClient + 'static> {
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryApi<S> {
    /// Create a new QueryApi instance
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer_engine = Arc::new(OptimizerEngine::default());
        Self {
            pipeline_manager: QueryPipelineManager::with_optimizer(
                storage,
                stats_manager,
                optimizer_engine,
            ),
        }
    }

    /// Please provide the text you would like to have translated.
    ///
    /// # Parameters
    /// `query`: The query statement
    /// - `ctx`: query request
    ///
    /// # Return
    /// Structured Search Results
    pub fn execute(&mut self, query: &str, ctx: QueryRequest) -> CoreResult<QueryResult> {
        let start_time = Instant::now();

        // Constructing a QueryRequestContext
        let rctx = Arc::new(
            crate::query::query_request_context::QueryRequestContext::new(query.to_string()),
        );

        // Building spatial information
        let space_info = ctx.space_id.map(|id| crate::core::types::SpaceInfo {
            space_id: id,
            space_name: String::new(),
            vid_type: crate::core::DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        });

        // Execute the query (using the new execute_query_with_request method).
        let execution_result = self
            .pipeline_manager
            .execute_query_with_request(query, rctx, space_info)
            .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;

        // Conversion to structured results
        let mut result = Self::convert_to_query_result(execution_result)?;
        result.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    /// Execute a parameterized query
    pub fn execute_with_params(
        &mut self,
        query: &str,
        params: std::collections::HashMap<String, crate::core::Value>,
        ctx: QueryRequest,
    ) -> CoreResult<QueryResult> {
        let mut ctx = ctx;
        ctx.parameters = Some(params);
        self.execute(query, ctx)
    }

    /// Convert execution results to structured query results
    fn convert_to_query_result(
        execution: crate::query::executor::base::ExecutionResult,
    ) -> CoreResult<QueryResult> {
        match execution {
            crate::query::executor::base::ExecutionResult::DataSet(data) => {
                // Processing the results of a dataset: The DataSet uses `col_names` instead of `columns`.
                let columns = data.col_names.clone();
                let mut rows = Vec::new();

                for row_data in &data.rows {
                    let mut row = Row::with_capacity(columns.len());
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(value) = row_data.get(i) {
                            row.insert(col.clone(), value.clone());
                        }
                    }
                    rows.push(row);
                }

                let metadata = ExecutionMetadata {
                    execution_time_ms: 0,
                    rows_scanned: data.rows.len() as u64,
                    rows_returned: data.rows.len() as u64,
                    cache_hit: false,
                };

                Ok(QueryResult {
                    columns,
                    rows,
                    metadata,
                })
            }
            crate::query::executor::base::ExecutionResult::Values(values) => {
                // Processing value list results
                let column = "value".to_string();
                let rows: Vec<Row> = values
                    .into_iter()
                    .map(|v| {
                        let mut row = Row::new();
                        row.insert(column.clone(), v);
                        row
                    })
                    .collect();

                Ok(QueryResult {
                    columns: vec![column],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Vertices(vertices) => {
                // Processing vertex results: The Value::Vertex type requires a Box(Vertex> object.
                let rows: Vec<Row> = vertices
                    .into_iter()
                    .map(|v| {
                        let mut row = Row::new();
                        row.insert(
                            "vertex".to_string(),
                            crate::core::Value::Vertex(Box::new(v)),
                        );
                        row
                    })
                    .collect();

                Ok(QueryResult {
                    columns: vec!["vertex".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Edges(edges) => {
                // Processing Edge Results - Value::Edge does not require a Box.
                let rows: Vec<Row> = edges
                    .into_iter()
                    .map(|e| {
                        let mut row = Row::new();
                        row.insert("edge".to_string(), crate::core::Value::Edge(e));
                        row
                    })
                    .collect();

                Ok(QueryResult {
                    columns: vec!["edge".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Result(core_result) => {
                // 处理 CoreResult - 使用 col_names() 方法和 rows() 方法
                let columns: Vec<String> = core_result.col_names().to_vec();
                let mut rows = Vec::new();

                for row_data in core_result.rows().iter() {
                    let mut row = Row::with_capacity(columns.len());
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(value) = row_data.get(i) {
                            row.insert(col.clone(), value.clone());
                        }
                    }
                    rows.push(row);
                }

                Ok(QueryResult {
                    columns,
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Empty
            | crate::query::executor::base::ExecutionResult::Success => {
                // Empty result
                Ok(QueryResult {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Count(count) => {
                // Counting results
                let mut row = Row::new();
                row.insert("count".to_string(), crate::core::Value::Int(count as i64));

                Ok(QueryResult {
                    columns: vec!["count".to_string()],
                    rows: vec![row],
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Paths(paths) => {
                // Path result – The Value::Path type does not require the use of a Box.
                let rows: Vec<Row> = paths
                    .into_iter()
                    .map(|p| {
                        let mut row = Row::new();
                        row.insert("path".to_string(), crate::core::Value::Path(p));
                        row
                    })
                    .collect();

                Ok(QueryResult {
                    columns: vec!["path".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Error(e) => {
                Err(CoreError::QueryExecutionFailed(e))
            }
        }
    }
}
