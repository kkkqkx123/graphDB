//! Query Pipeline Manager
//!
//! Responsible for coordinating the entire query processing workflow:
//! 1. Managing the entire lifecycle of query processing
//! 2. Coordinate the various processing stages (parsing → validation → planning → optimization → execution)
//! 3. Handling errors and exceptions
//! 4. Managing query context and performance monitoring
//!
//! ## The relationship with OptimizerEngine
//!
//! The `QueryPipelineManager` uses the `OptimizerEngine` by reference, rather than directly creating the optimizer component.
//! `OptimizerEngine` is a global instance that has the same lifecycle as the database instance and is responsible for all functions related to query optimization.
//!
//! ```rust
// Method of creation
//! let optimizer_engine = Arc::new(OptimizerEngine::default());
//! let pipeline = QueryPipelineManager::with_optimizer(
//!     storage,
//!     stats_manager,
//!     optimizer_engine,
//! );
//! ```

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::{ErrorInfo, ErrorType, QueryMetrics, QueryPhase, QueryProfile, StatsManager};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor};
use crate::query::executor::explain::{ExplainExecutor, ExplainMode, ProfileExecutor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::utils::object_pool::{ObjectPoolConfig, ThreadSafeExecutorPool};
use crate::query::metadata::{CachedMetadataProvider, MetadataContext, MetadataProvider};
use crate::query::optimizer::OptimizerEngine;
use crate::query::parser::ast::stmt::{ExplainStmt, ProfileStmt};
use crate::query::parser::Parser;
use crate::query::planning::{ParameterizedQueryHandler, PlanCacheConfig, QueryPlanCache};
use crate::query::query_request_context::QueryRequestContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::{ValidatedStatement, ValidationInfo};
use crate::query::QueryContext;
use crate::storage::metadata::redb_schema_manager::RedbSchemaManager;
use crate::storage::StorageClient;
use crate::sync::SyncManager;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Query Pipeline Manager
///
/// Responsible for coordinating the overall query processing workflow, and utilizing optimization features by leveraging the `OptimizerEngine`.
pub struct QueryPipelineManager<S: StorageClient + 'static> {
    executor_factory: ExecutorFactory<S>,
    object_pool: Arc<ThreadSafeExecutorPool<S>>,
    stats_manager: Arc<StatsManager>,
    /// Optimizer engine (reference to the global instance)
    optimizer_engine: Arc<OptimizerEngine>,
    /// Query plan cache
    plan_cache: Arc<QueryPlanCache>,
    /// Parameterized Query Processor
    param_handler: ParameterizedQueryHandler,
    /// Schema manager for validation
    schema_manager: Option<Arc<RedbSchemaManager>>,
    /// Metadata provider for pre-resolving metadata during planning
    metadata_provider: Option<Arc<dyn MetadataProvider>>,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    /// Create using the specified optimizer engine.
    ///
    /// This is the recommended way to use the production environment, which allows for the sharing of a global OptimizerEngine instance.
    ///
    /// # Parameters
    /// `storage`: The storage component for the client side.
    /// `stats_manager`: A manager for statistical information.
    /// `optimizer_engine`: The optimizer engine (global instance).
    pub fn with_optimizer(
        storage: Arc<Mutex<S>>,
        stats_manager: Arc<StatsManager>,
        optimizer_engine: Arc<OptimizerEngine>,
    ) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let object_pool = Arc::new(ThreadSafeExecutorPool::new(ObjectPoolConfig::default()));
        let plan_cache = Arc::new(QueryPlanCache::default());
        let param_handler = ParameterizedQueryHandler::default();

        log::info!("Query pipeline manager created, using optimizer engine and query plan cache");

        Self {
            executor_factory,
            object_pool,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
            schema_manager: None,
            metadata_provider: None,
        }
    }

    /// Create using the specified optimizer engine and planning cache configuration.
    ///
    /// # Parameters
    /// - `storage`: Storage client
    /// - `stats_manager`: Statistics information manager
    /// - `optimizer_engine`: Optimizer engine (global instance)
    /// - `plan_cache_config`: Query plan cache configuration
    pub fn with_optimizer_and_cache(
        storage: Arc<Mutex<S>>,
        stats_manager: Arc<StatsManager>,
        optimizer_engine: Arc<OptimizerEngine>,
        plan_cache_config: PlanCacheConfig,
    ) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let object_pool = Arc::new(ThreadSafeExecutorPool::new(ObjectPoolConfig::default()));
        let plan_cache = Arc::new(QueryPlanCache::new(plan_cache_config));
        let param_handler = ParameterizedQueryHandler::default();

        log::info!(
            "Query pipeline manager created, using optimizer engine and custom query plan cache"
        );

        Self {
            executor_factory,
            object_pool,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
            schema_manager: None,
            metadata_provider: None,
        }
    }

    /// Obtaining the optimizer engine
    pub fn optimizer_engine(&self) -> &OptimizerEngine {
        &self.optimizer_engine
    }

    /// Set schema manager for validation
    pub fn with_schema_manager(mut self, schema_manager: Arc<RedbSchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// Set metadata provider for pre-resolving metadata during planning
    pub fn with_metadata_provider(mut self, metadata_provider: Arc<dyn MetadataProvider>) -> Self {
        self.metadata_provider = Some(metadata_provider);
        self
    }

    /// Create and set a cached metadata provider with the specified cache size
    pub fn with_cached_metadata_provider(
        mut self,
        inner_provider: Arc<dyn MetadataProvider>,
        _cache_size: usize,
    ) -> Self {
        let cached_provider = Arc::new(CachedMetadataProvider::new(inner_provider));
        self.metadata_provider = Some(cached_provider);
        self
    }

    /// Obtaining the query plan cache
    pub fn plan_cache(&self) -> &QueryPlanCache {
        &self.plan_cache
    }

    /// Obtain metrics on the query plan cache
    pub fn plan_cache_metrics(&self) -> std::sync::Arc<crate::query::cache::PlanCacheMetrics> {
        self.plan_cache.metrics()
    }

    /// Clear query plan cache.
    pub fn clear_plan_cache(&self) {
        self.plan_cache.clear();
        log::info!("Query plan cache cleared");
    }

    /// Obtain object pool statistics.
    pub fn object_pool_stats(&self) -> crate::query::executor::utils::object_pool::PoolStats {
        self.object_pool.stats()
    }

    /// Clear object pool.
    pub fn clear_object_pool(&self) {
        self.object_pool.clear();
        log::info!("Object pool cleared");
    }

    /// Set sync manager for executor factory
    pub fn with_sync_manager(mut self, sync_manager: Arc<SyncManager>) -> Self {
        self.executor_factory.set_sync_manager(sync_manager);
        self
    }

    pub fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        self.execute_query_with_space(query_text, None)
    }

    pub fn execute_query_with_space(
        &mut self,
        query_text: &str,
        space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        // 1. First, create a QueryContext (which persists throughout the entire lifecycle of the query).
        let rctx = Arc::new(QueryRequestContext::new(query_text.to_string()));
        let mut query_context = QueryContext::new(rctx);

        // Setting spatial information
        if let Some(ref space) = space_info {
            query_context.set_space_info(space.clone());
        }

        let query_context = Arc::new(query_context);

        // 2. Check the query plan cache.
        if let Some(cached_plan) = self.plan_cache.get(query_text) {
            log::debug!("Query plan cache hit");
            let execute_start = Instant::now();
            let result = self.execute_plan(query_context, cached_plan.plan.clone())?;
            let execution_time_ms = execute_start.elapsed().as_millis() as f64;
            self.plan_cache
                .record_execution(query_text, execution_time_ms);
            return Ok(result);
        }

        // 3. Analyzing the query
        let parser_result = self.parse_into_context(query_text)?;

        // 4. Verify the query (reusing the already created QueryContext)
        let validation_info =
            self.validate_query_with_context(parser_result.ast.clone(), query_context.clone())?;

        // Create a verified statement (using Arc<Ast> to share ownership).
        let validated = ValidatedStatement::new(parser_result.ast.clone(), validation_info);

        // Check for EXPLAIN/PROFILE statements and route accordingly
        match validated.ast.stmt() {
            crate::query::parser::ast::Stmt::Explain(explain_stmt) => {
                return self.execute_explain(explain_stmt, query_context);
            }
            crate::query::parser::ast::Stmt::Profile(profile_stmt) => {
                return self.execute_profile(profile_stmt, query_context);
            }
            _ => {}
        }

        // 5. Generate an execution plan.
        let execution_plan = self.generate_execution_plan(query_context.clone(), &validated)?;

        // 6. Optimizing the execution plan
        let optimized_plan = self.optimize_execution_plan(execution_plan)?;

        // 7. Execution Plan
        let execute_start = Instant::now();
        let result = self.execute_plan(query_context, optimized_plan.clone())?;
        let execution_time_ms = execute_start.elapsed().as_millis() as f64;

        // 8. Caching of query plans
        // Skip caching for INSERT statements as they contain literal values
        let should_cache = !matches!(
            validated.ast.stmt(),
            crate::query::parser::ast::Stmt::Insert(_)
        );
        if should_cache {
            let param_positions = self.param_handler.extract_params(query_text);
            self.plan_cache
                .put(query_text, optimized_plan, param_positions);
            self.plan_cache
                .record_execution(query_text, execution_time_ms);
        }

        Ok(result)
    }

    /// Execute the query using QueryRequestContext.
    ///
    /// This method allows the API layer to pass the complete session information to the query layer.
    pub fn execute_query_with_request(
        &mut self,
        query_text: &str,
        rctx: Arc<crate::query::query_request_context::QueryRequestContext>,
        space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        // 1. First, create a QueryContext (which persists throughout the entire lifecycle of the query).
        let mut query_context = QueryContext::new(rctx);

        // Setting spatial information (before packaging in the Arc format)
        if let Some(ref space) = space_info {
            query_context.set_space_info(space.clone());
        }

        let query_context = Arc::new(query_context);

        // 2. Analyze the query
        let parser_result = self.parse_into_context(query_text)?;

        // 3. Verify the query (reusing the already created QueryContext)
        let validation_info =
            self.validate_query_with_context(parser_result.ast.clone(), query_context.clone())?;

        // Create a verified statement (using Arc<Ast> to share ownership)
        let validated = ValidatedStatement::new(parser_result.ast.clone(), validation_info);

        // Check for EXPLAIN/PROFILE statements and route accordingly
        match validated.ast.stmt() {
            crate::query::parser::ast::Stmt::Explain(explain_stmt) => {
                return self.execute_explain(explain_stmt, query_context);
            }
            crate::query::parser::ast::Stmt::Profile(profile_stmt) => {
                return self.execute_profile(profile_stmt, query_context);
            }
            _ => {}
        }

        // 4. Generate an execution plan.
        let execution_plan = self.generate_execution_plan(query_context.clone(), &validated)?;

        // 5. Optimizing the execution plan
        let optimized_plan = self.optimize_execution_plan(execution_plan)?;

        // 6. Execution of the plan
        self.execute_plan(query_context, optimized_plan)
    }

    pub fn execute_query_with_metrics(
        &mut self,
        query_text: &str,
    ) -> DBResult<(ExecutionResult, QueryMetrics)> {
        self.execute_query_with_session(query_text, 0)
            .map(|(result, metrics, _)| (result, metrics))
    }

    pub fn execute_query_with_session(
        &mut self,
        query_text: &str,
        session_id: i64,
    ) -> DBResult<(ExecutionResult, QueryMetrics, QueryProfile)> {
        self.execute_query_with_profile(query_text, session_id)
    }

    pub fn execute_query_with_profile(
        &mut self,
        query_text: &str,
        session_id: i64,
    ) -> DBResult<(ExecutionResult, QueryMetrics, QueryProfile)> {
        let total_start = Instant::now();
        let mut metrics = QueryMetrics::new();
        let mut profile = QueryProfile::new(session_id, query_text.to_string());

        // 1. First, create a QueryContext (which persists throughout the entire lifecycle of the query).
        let rctx = Arc::new(QueryRequestContext::new(query_text.to_string()));
        let query_context = Arc::new(QueryContext::new(rctx));

        let parse_start = Instant::now();
        let parser_result = match self.parse_into_context(query_text) {
            Ok(result) => {
                profile.stages.parse_us = parse_start.elapsed().as_micros() as u64;
                metrics.record_parse_time(parse_start.elapsed());
                result
            }
            Err(e) => {
                profile.stages.parse_us = parse_start.elapsed().as_micros() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ParseError,
                    QueryPhase::Parse,
                    e.to_string(),
                ));
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        let validate_start = Instant::now();
        let validation_info = match self
            .validate_query_with_context(parser_result.ast.clone(), query_context.clone())
        {
            Ok(info) => info,
            Err(e) => {
                profile.stages.validate_us = validate_start.elapsed().as_micros() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ValidationError,
                    QueryPhase::Validate,
                    e.to_string(),
                ));
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        profile.stages.validate_us = validate_start.elapsed().as_micros() as u64;
        metrics.record_validate_time(validate_start.elapsed());

        // Create a verified statement (using Arc<Ast> to share ownership).
        let validated = ValidatedStatement::new(parser_result.ast.clone(), validation_info);

        // Check for EXPLAIN/PROFILE statements and route accordingly
        match validated.ast.stmt() {
            crate::query::parser::ast::Stmt::Explain(explain_stmt) => {
                let result = self.execute_explain(explain_stmt, query_context)?;
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                metrics.record_total_time(total_start.elapsed());
                return Ok((result, metrics, profile));
            }
            crate::query::parser::ast::Stmt::Profile(profile_stmt) => {
                let result = self.execute_profile(profile_stmt, query_context)?;
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                metrics.record_total_time(total_start.elapsed());
                return Ok((result, metrics, profile));
            }
            _ => {}
        }

        let plan_start = Instant::now();
        let execution_plan = match self.generate_execution_plan(query_context.clone(), &validated) {
            Ok(plan) => {
                profile.stages.plan_us = plan_start.elapsed().as_micros() as u64;
                metrics.set_plan_node_count(plan.node_count());
                metrics.record_plan_time(plan_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.plan_us = plan_start.elapsed().as_micros() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::PlanningError,
                    QueryPhase::Plan,
                    e.to_string(),
                ));
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        let optimize_start = Instant::now();
        let optimized_plan = match self.optimize_execution_plan(execution_plan) {
            Ok(plan) => {
                profile.stages.optimize_us = optimize_start.elapsed().as_micros() as u64;
                metrics.record_optimize_time(optimize_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.optimize_us = optimize_start.elapsed().as_micros() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::OptimizationError,
                    QueryPhase::Optimize,
                    e.to_string(),
                ));
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        let execute_start = Instant::now();
        let result = match self.execute_plan(query_context, optimized_plan) {
            Ok(result) => {
                profile.stages.execute_us = execute_start.elapsed().as_micros() as u64;
                profile.result_count = result.count();
                metrics.set_result_row_count(result.count());
                metrics.record_execute_time(execute_start.elapsed());
                result
            }
            Err(e) => {
                profile.stages.execute_us = execute_start.elapsed().as_micros() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ExecutionError,
                    QueryPhase::Execute,
                    e.to_string(),
                ));
                profile.total_duration_us = total_start.elapsed().as_micros() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        profile.total_duration_us = total_start.elapsed().as_micros() as u64;
        metrics.record_total_time(total_start.elapsed());

        self.stats_manager.record_query_metrics(&metrics);
        self.stats_manager.record_query_profile(profile.clone());

        Ok((result, metrics, profile))
    }

    fn parse_into_context(
        &mut self,
        query_text: &str,
    ) -> DBResult<crate::query::parser::ParserResult> {
        let mut parser = Parser::new(query_text);
        parser
            .parse()
            .map_err(|e| DBError::from(QueryError::pipeline_parse_error(e)))
    }

    /// Verify the query and return the verification information (using the provided QueryContext).
    ///
    /// This method reuses the already created QueryContext, thereby avoiding the creation and subsequent disposal of temporary contexts.
    /// Ensure that a consistent context is used throughout the entire lifecycle of the query.
    ///
    /// # Parameters
    /// `ast`: Abstract Syntax Tree
    /// `qctx`: Query context (persists throughout the entire lifecycle of the query).
    fn validate_query_with_context(
        &mut self,
        ast: Arc<crate::query::parser::ast::stmt::Ast>,
        qctx: Arc<QueryContext>,
    ) -> DBResult<ValidationInfo> {
        let mut validator =
            crate::query::validator::Validator::create_from_ast(&ast).ok_or_else(|| {
                DBError::from(QueryError::InvalidQuery(format!(
                    "Unsupported statement type: {:?}",
                    ast.stmt
                )))
            })?;

        // Set schema manager if available
        if let Some(ref schema_manager) = self.schema_manager {
            validator.set_schema_manager(schema_manager.clone());
        }

        // Perform verification using the provided QueryContext.
        // Avoid creating temporary contexts and ensure the consistency of resources (such as ID generators, object pools, etc.).
        let validation_result = validator.validate(ast.clone(), qctx);

        if validation_result.success {
            Ok(validation_result.info.unwrap_or_default())
        } else {
            let error_msg = validation_result
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(DBError::from(QueryError::InvalidQuery(error_msg)))
        }
    }

    /// Generate an execution plan using the verified statements.
    fn generate_execution_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        validated: &ValidatedStatement,
    ) -> DBResult<crate::query::planning::plan::ExecutionPlan> {
        // Create the planner directly using Arc<Ast>, eliminating the need for string matching of the SentenceKind type.
        let plan = if let Some(mut planner_enum) =
            crate::query::planning::planner::PlannerEnum::from_ast(&validated.ast)
        {
            // Build metadata context if metadata provider is available
            let metadata_context = if let Some(ref metadata_provider) = self.metadata_provider {
                Some(self.build_metadata_context(
                    validated,
                    query_context.clone(),
                    metadata_provider,
                )?)
            } else {
                None
            };

            // Transform with metadata context if available
            let sub_plan = if let Some(ref ctx) = metadata_context {
                planner_enum
                    .transform_with_metadata(validated, query_context, ctx)
                    .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?
            } else {
                planner_enum
                    .transform(validated, query_context)
                    .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?
            };

            let root = sub_plan.root().clone();
            crate::query::planning::plan::ExecutionPlan::new(root)
        } else {
            return Err(DBError::from(QueryError::pipeline_planning_error(
                crate::query::planning::planner::PlannerError::NoSuitablePlanner(
                    "No suitable planner found".to_string(),
                ),
            )));
        };

        Ok(plan)
    }

    /// Build metadata context for the given statement
    ///
    /// This method pre-resolves metadata (indexes, tags, edge types) during the planning phase,
    /// similar to PostgreSQL's FDW fdw_private mechanism.
    fn build_metadata_context(
        &self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
        metadata_provider: &Arc<dyn MetadataProvider>,
    ) -> DBResult<MetadataContext> {
        use crate::query::metadata::provider::MetadataProviderError;
        use crate::query::parser::ast::Stmt;

        let space_id = qctx.space_id().unwrap_or(0);
        let mut context = MetadataContext::new();
        let stmt = validated.stmt();

        // Pre-resolve metadata based on statement type
        match stmt {
            Stmt::SearchVector(search) => {
                // Pre-resolve vector index metadata
                match metadata_provider.get_index_metadata(space_id, &search.index_name) {
                    Ok(index_metadata) => {
                        context.set_index_metadata(search.index_name.clone(), index_metadata);
                    }
                    Err(MetadataProviderError::NotFound(msg)) => {
                        return Err(DBError::from(QueryError::InvalidQuery(format!(
                            "Vector index not found: {}",
                            msg
                        ))));
                    }
                    Err(e) => {
                        return Err(DBError::from(QueryError::InvalidQuery(format!(
                            "Failed to get index metadata: {}",
                            e
                        ))));
                    }
                }
            }
            Stmt::LookupVector(lookup) => {
                // Pre-resolve index metadata for lookup
                let index_name = &lookup.index_name;
                match metadata_provider.get_index_metadata(space_id, index_name) {
                    Ok(index_metadata) => {
                        context.set_index_metadata(index_name.clone(), index_metadata);
                    }
                    Err(MetadataProviderError::NotFound(msg)) => {
                        return Err(DBError::from(QueryError::InvalidQuery(format!(
                            "Vector index not found: {}",
                            msg
                        ))));
                    }
                    Err(e) => {
                        return Err(DBError::from(QueryError::InvalidQuery(format!(
                            "Failed to get index metadata: {}",
                            e
                        ))));
                    }
                }
            }
            Stmt::MatchVector(_match_stmt) => {
                // MatchVector doesn't have a direct index_name, it uses pattern matching
                // Metadata resolution happens at executor time for now
                log::debug!("MatchVector metadata resolution deferred to executor");
            }
            Stmt::Match(_match_stmt) => {
                // Pre-resolve tag and index metadata for MATCH statements
                let referenced_tags = &validated.validation_info.semantic_info.referenced_tags;
                let referenced_edges = &validated.validation_info.semantic_info.referenced_edges;

                // Resolve tag metadata and their indexes
                for tag_name in referenced_tags {
                    match metadata_provider.get_tag_metadata(space_id, tag_name) {
                        Ok(tag_metadata) => {
                            context.set_tag_metadata(tag_name.clone(), tag_metadata);
                        }
                        Err(e) => {
                            log::warn!("Failed to get tag metadata for '{}': {}", tag_name, e);
                        }
                    }
                }

                // Resolve edge type metadata
                for edge_type in referenced_edges {
                    match metadata_provider.get_edge_type_metadata(space_id, edge_type) {
                        Ok(edge_metadata) => {
                            context.set_edge_type_metadata(edge_type.clone(), edge_metadata);
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to get edge type metadata for '{}': {}",
                                edge_type,
                                e
                            );
                        }
                    }
                }

                // Resolve all indexes for the space
                match metadata_provider.list_indexes(space_id) {
                    Ok(indexes) => {
                        for index in indexes {
                            context.set_index_metadata(index.index_name.clone(), index);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to list indexes for space {}: {}", space_id, e);
                    }
                }
            }
            // For other statement types, we can extend metadata resolution as needed
            _ => {
                // No specific metadata resolution for other statement types yet
                log::debug!("No metadata resolution for statement type: {:?}", stmt);
            }
        }

        Ok(context)
    }

    fn optimize_execution_plan(
        &mut self,
        plan: crate::query::planning::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planning::plan::ExecutionPlan> {
        // Use the unified optimization interface from OptimizerEngine
        self.optimizer_engine
            .optimize(plan)
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))
    }

    fn execute_plan(
        &mut self,
        _query_context: Arc<QueryContext>,
        plan: crate::query::planning::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        use crate::query::executor::factory::engine::PlanExecutor;
        let mut plan_executor =
            PlanExecutor::with_object_pool(self.executor_factory.clone(), self.object_pool.clone());

        // Get expression context from query context
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());

        let storage = self.executor_factory.storage.clone().ok_or_else(|| {
            DBError::from(QueryError::ExecutionError(
                "Storage not available".to_string(),
            ))
        })?;

        plan_executor
            .execute_plan(&plan, storage, expr_ctx)
            .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
    }

    /// Execute EXPLAIN statement
    pub fn execute_explain(
        &mut self,
        explain_stmt: &ExplainStmt,
        qctx: Arc<QueryContext>,
    ) -> DBResult<ExecutionResult> {
        // 1. Get inner statement execution plan (without executing)
        let inner_ast = &explain_stmt.statement;
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let validation_info = self.validate_query_with_context(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx.clone(),
            )),
            qctx.clone(),
        )?;
        let inner_validated = ValidatedStatement::new(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx,
            )),
            validation_info,
        );
        let inner_plan = self.generate_execution_plan(qctx.clone(), &inner_validated)?;
        let optimized_plan = self.optimize_execution_plan(inner_plan)?;

        // 2. Create ExplainExecutor
        let storage = self.executor_factory.storage.clone().ok_or_else(|| {
            DBError::from(QueryError::ExecutionError(
                "Storage not available".to_string(),
            ))
        })?;

        let base = BaseExecutor::new(
            -1,
            "ExplainExecutor".to_string(),
            storage,
            Arc::new(ExpressionAnalysisContext::new()),
        );

        let mut explain_executor = ExplainExecutor::new(
            base,
            optimized_plan,
            explain_stmt.format.clone(),
            ExplainMode::PlanOnly,
        );

        // 3. Execute Explain
        explain_executor
            .execute()
            .map_err(|e| DBError::from(QueryError::ExecutionError(e.to_string())))
    }

    /// Execute EXPLAIN ANALYZE statement
    pub fn execute_explain_analyze(
        &mut self,
        explain_stmt: &ExplainStmt,
        qctx: Arc<QueryContext>,
    ) -> DBResult<ExecutionResult> {
        // 1. Get inner statement execution plan
        let inner_ast = &explain_stmt.statement;
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let validation_info = self.validate_query_with_context(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx.clone(),
            )),
            qctx.clone(),
        )?;
        let inner_validated = ValidatedStatement::new(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx,
            )),
            validation_info,
        );
        let inner_plan = self.generate_execution_plan(qctx.clone(), &inner_validated)?;
        let optimized_plan = self.optimize_execution_plan(inner_plan)?;

        // 2. Create ExplainExecutor with Analyze mode
        let storage = self.executor_factory.storage.clone().ok_or_else(|| {
            DBError::from(QueryError::ExecutionError(
                "Storage not available".to_string(),
            ))
        })?;

        let base = BaseExecutor::new(
            -1,
            "ExplainExecutor".to_string(),
            storage,
            Arc::new(ExpressionAnalysisContext::new()),
        );

        let mut explain_executor = ExplainExecutor::new(
            base,
            optimized_plan,
            explain_stmt.format.clone(),
            ExplainMode::Analyze,
        );

        // 3. Execute Explain Analyze
        explain_executor
            .execute()
            .map_err(|e| DBError::from(QueryError::ExecutionError(e.to_string())))
    }

    /// Execute PROFILE statement
    pub fn execute_profile(
        &mut self,
        profile_stmt: &ProfileStmt,
        qctx: Arc<QueryContext>,
    ) -> DBResult<ExecutionResult> {
        // 1. Get inner statement execution plan
        let inner_ast = &profile_stmt.statement;
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let validation_info = self.validate_query_with_context(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx.clone(),
            )),
            qctx.clone(),
        )?;
        let inner_validated = ValidatedStatement::new(
            Arc::new(crate::query::parser::ast::stmt::Ast::new(
                (**inner_ast).clone(),
                expr_ctx,
            )),
            validation_info,
        );
        let inner_plan = self.generate_execution_plan(qctx.clone(), &inner_validated)?;
        let optimized_plan = self.optimize_execution_plan(inner_plan)?;

        // 2. Create ProfileExecutor
        let storage = self.executor_factory.storage.clone().ok_or_else(|| {
            DBError::from(QueryError::ExecutionError(
                "Storage not available".to_string(),
            ))
        })?;

        let base = BaseExecutor::new(
            -1,
            "ProfileExecutor".to_string(),
            storage,
            Arc::new(ExpressionAnalysisContext::new()),
        );

        let mut profile_executor =
            ProfileExecutor::new(base, optimized_plan, profile_stmt.format.clone());

        // 3. Execute Profile
        profile_executor
            .execute()
            .map_err(|e| DBError::from(QueryError::ExecutionError(e.to_string())))
    }
}
