//! Query Execution API – Core Layer
//!
//! Provides transport layer independent query execution

use crate::api::core::error::{CoreError, CoreResult};
use crate::api::core::types::{ExecutionMetadata, QueryRequest, QueryResult, Row};
use crate::core::metadata::SchemaManager;
use crate::core::StatsManager;
use crate::query::metadata::{
    CachedMetadataProvider, CompositeMetadataProvider, FulltextIndexMetadataProvider,
    MetadataProvider, SchemaMetadataProvider,
};
#[cfg(feature = "qdrant")]
use crate::query::metadata::VectorIndexMetadataProvider;
use crate::query::{OptimizerEngine, QueryPipelineManager};
use crate::storage::StorageClient;
#[cfg(feature = "qdrant")]
use crate::sync::vector_sync::VectorSyncCoordinator;
use crate::sync::SyncManager;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
#[cfg(feature = "qdrant")]
use vector_client::{VectorClientConfig, VectorManager};

/// Universal Query API – Core Layer
pub struct QueryApi<S: StorageClient + 'static> {
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryApi<S> {
    /// Create a new QueryApi instance with external StatsManager
    pub fn new(storage: Arc<RwLock<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let optimizer_engine = Arc::new(OptimizerEngine::default());
        Self {
            pipeline_manager: QueryPipelineManager::with_optimizer(
                storage,
                stats_manager,
                optimizer_engine,
            ),
        }
    }

    /// Create a new QueryApi instance with sync manager support
    pub fn with_sync_manager(
        storage: Arc<RwLock<S>>,
        stats_manager: Arc<StatsManager>,
        sync_manager: Arc<SyncManager>,
    ) -> Self {
        let optimizer_engine = Arc::new(OptimizerEngine::default());
        Self {
            pipeline_manager: QueryPipelineManager::with_optimizer(
                storage,
                stats_manager,
                optimizer_engine,
            )
            .with_sync_manager(sync_manager),
        }
    }

    /// Create a new QueryApi instance with schema manager support
    pub fn with_schema_manager(
        storage: Arc<RwLock<S>>,
        stats_manager: Arc<StatsManager>,
        schema_manager: Arc<SchemaManager>,
    ) -> Self {
        let optimizer_engine = Arc::new(OptimizerEngine::default());

        let schema_provider = Arc::new(SchemaMetadataProvider::new(schema_manager.clone(), None));
        let cached_provider = Arc::new(CachedMetadataProvider::new(schema_provider));

        Self {
            pipeline_manager: QueryPipelineManager::with_optimizer(
                storage,
                stats_manager,
                optimizer_engine,
            )
            .with_schema_manager(schema_manager)
            .with_metadata_provider(cached_provider),
        }
    }

    /// Create a new QueryApi instance with both schema manager and sync manager support
    pub fn with_schema_and_sync_manager(
        storage: Arc<RwLock<S>>,
        stats_manager: Arc<StatsManager>,
        schema_manager: Arc<SchemaManager>,
        sync_manager: Arc<SyncManager>,
    ) -> Self {
        let optimizer_engine = Arc::new(OptimizerEngine::default());

        let schema_provider: Arc<dyn MetadataProvider> =
            Arc::new(SchemaMetadataProvider::new(schema_manager.clone(), None));

        let fulltext_provider: Arc<dyn MetadataProvider> =
            Arc::new(FulltextIndexMetadataProvider::new(
                sync_manager.fulltext_manager(),
            ));

        let mut providers: Vec<Arc<dyn MetadataProvider>> =
            vec![schema_provider, fulltext_provider];

        #[cfg(feature = "qdrant")]
        if let Some(vector_coordinator) = sync_manager.vector_coordinator() {
            providers.push(Arc::new(VectorIndexMetadataProvider::new(
                vector_coordinator.clone(),
            )));
        }

        let composite = Arc::new(CompositeMetadataProvider::new(providers));
        let cached_provider = Arc::new(CachedMetadataProvider::new(composite));

        Self {
            pipeline_manager: QueryPipelineManager::with_optimizer(
                storage,
                stats_manager,
                optimizer_engine,
            )
            .with_schema_manager(schema_manager)
            .with_metadata_provider(cached_provider)
            .with_sync_manager(sync_manager),
        }
    }

    /// Create a new QueryApi instance with vector search support
    #[cfg(feature = "qdrant")]
    pub async fn with_vector_search(
        storage: Arc<RwLock<S>>,
        stats_manager: Arc<StatsManager>,
        vector_config: VectorClientConfig,
        schema_manager: Option<Arc<SchemaManager>>,
    ) -> Result<Self, String> {
        let optimizer_engine = Arc::new(OptimizerEngine::default());

        // Create vector manager
        let vector_manager = Arc::new(
            VectorManager::new(vector_config)
                .await
                .map_err(|e| format!("Failed to create vector manager: {}", e))?,
        );

        // Create vector coordinator (embedding service is optional)
        let vector_coordinator = Arc::new(VectorSyncCoordinator::new(vector_manager, None));

        // Create metadata providers
        let vector_provider: Arc<dyn MetadataProvider> =
            Arc::new(VectorIndexMetadataProvider::new(vector_coordinator.clone()));

        // Compose with schema provider if schema_manager is available
        let mut pipeline_manager =
            QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer_engine);

        if let Some(sm) = schema_manager {
            let schema_provider = Arc::new(SchemaMetadataProvider::new(sm.clone(), None));
            let composite = Arc::new(CompositeMetadataProvider::new(vec![
                schema_provider,
                vector_provider,
            ]));
            let cached = Arc::new(CachedMetadataProvider::new(composite));
            pipeline_manager = pipeline_manager
                .with_schema_manager(sm)
                .with_metadata_provider(cached);
        } else {
            let cached = Arc::new(CachedMetadataProvider::new(vector_provider));
            pipeline_manager = pipeline_manager.with_metadata_provider(cached);
        }

        Ok(Self { pipeline_manager })
    }

    /// Execute a query with the given query request
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
        let rctx = Arc::new(crate::query::QueryRequestContext::new(query.to_string()));

        // Build space info from request context if space_id is provided
        let space_info = ctx.space_id.map(|id| {
            let space_name = ctx.space_name.clone().unwrap_or_default();
            let mut space_info = crate::core::types::SpaceInfo::new(space_name);
            space_info.space_id = id;
            space_info
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
        // Create new QueryRequest with parameters
        let new_ctx = QueryRequest {
            space_id: ctx.space_id,
            space_name: ctx.space_name,
            auto_commit: ctx.auto_commit,
            transaction_id: ctx.transaction_id,
            parameters: Some(params),
        };
        self.execute(query, new_ctx)
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
                    rows_scanned: data.row_count() as u64,
                    rows_returned: data.row_count() as u64,
                    cache_hit: false,
                };

                Ok(QueryResult {
                    columns,
                    rows,
                    metadata,
                })
            }
            crate::query::executor::base::ExecutionResult::Success => {
                // Successful execution with no data
                Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Empty => {
                // Empty result
                Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::SpaceSwitched(summary) => {
                // Space switched successfully
                let mut row = crate::api::core::types::Row::new();
                row.values.insert(
                    "space_name".to_string(),
                    crate::core::Value::String(summary.name.clone()),
                );
                row.values.insert(
                    "space_id".to_string(),
                    crate::core::Value::BigInt(summary.id as i64),
                );
                row.values.insert(
                    "vid_type".to_string(),
                    crate::core::Value::String(summary.vid_type.to_string()),
                );
                Ok(QueryResult {
                    columns: vec![
                        "space_name".to_string(),
                        "space_id".to_string(),
                        "vid_type".to_string(),
                    ],
                    rows: vec![row],
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Error(msg) => {
                // Error case - should be handled before this function
                Err(CoreError::Internal(msg))
            }
        }
    }
}
