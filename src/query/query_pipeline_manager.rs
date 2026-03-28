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
use crate::query::executor::base::ExecutionResult;
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::object_pool::{ObjectPoolConfig, ThreadSafeExecutorPool};
use crate::query::optimizer::OptimizerEngine;
use crate::query::parser::Parser;
use crate::query::planning::{ParameterizedQueryHandler, PlanCacheConfig, QueryPlanCache};
use crate::query::query_request_context::QueryRequestContext;
use crate::query::validator::{ValidatedStatement, ValidationInfo};
use crate::query::QueryContext;
use crate::storage::StorageClient;
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

        log::info!("查询管道管理器已创建，使用优化器引擎和查询计划缓存");

        Self {
            executor_factory,
            object_pool,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
        }
    }

    /// Create using the specified optimizer engine and planning cache configuration.
    ///
    /// # 参数
    /// - `storage`: 存储客户端
    /// - `stats_manager`: 统计信息管理器
    /// - `optimizer_engine`: 优化器引擎（全局实例）
    /// - `plan_cache_config`: Queries the configuration of the plan cache.
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

        log::info!("查询管道管理器已创建，使用优化器引擎和自定义查询计划缓存");

        Self {
            executor_factory,
            object_pool,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
        }
    }

    /// Obtaining the optimizer engine
    pub fn optimizer_engine(&self) -> &OptimizerEngine {
        &self.optimizer_engine
    }

    /// Obtaining the query plan cache
    pub fn plan_cache(&self) -> &QueryPlanCache {
        &self.plan_cache
    }

    /// Obtain statistics on the query plan cache
    pub fn plan_cache_stats(&self) -> crate::query::planning::PlanCacheStats {
        self.plan_cache.stats()
    }

    /// Clear query plan cache.
    pub fn clear_plan_cache(&self) {
        self.plan_cache.clear();
        log::info!("查询计划缓存已清空");
    }

    /// Obtain object pool statistics.
    pub fn object_pool_stats(&self) -> crate::query::executor::object_pool::PoolStats {
        self.object_pool.stats()
    }

    /// Clear object pool.
    pub fn clear_object_pool(&self) {
        self.object_pool.clear();
        log::info!("对象池已清空");
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
            log::debug!("查询计划缓存命中");
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
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        // 5. Generate an execution plan.
        let execution_plan = self.generate_execution_plan(query_context.clone(), &validated)?;

        // 6. Optimizing the execution plan
        let optimized_plan = self.optimize_execution_plan(execution_plan)?;

        // 7. Execution Plan
        let execute_start = Instant::now();
        let result = self.execute_plan(query_context, optimized_plan.clone())?;
        let execution_time_ms = execute_start.elapsed().as_millis() as f64;

        // 8. Caching of query plans
        let param_positions = self.param_handler.extract_params(query_text);
        self.plan_cache
            .put(query_text, optimized_plan, param_positions);
        self.plan_cache
            .record_execution(query_text, execution_time_ms);

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
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

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
                profile.stages.parse_ms = parse_start.elapsed().as_millis() as u64;
                metrics.record_parse_time(parse_start.elapsed());
                result
            }
            Err(e) => {
                profile.stages.parse_ms = parse_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ParseError,
                    QueryPhase::Parse,
                    e.to_string(),
                ));
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
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
                profile.stages.validate_ms = validate_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ValidationError,
                    QueryPhase::Validate,
                    e.to_string(),
                ));
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        profile.stages.validate_ms = validate_start.elapsed().as_millis() as u64;
        metrics.record_validate_time(validate_start.elapsed());

        // Create a verified statement (using Arc<Ast> to share ownership).
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let plan_start = Instant::now();
        let execution_plan = match self.generate_execution_plan(query_context.clone(), &validated) {
            Ok(plan) => {
                profile.stages.plan_ms = plan_start.elapsed().as_millis() as u64;
                metrics.set_plan_node_count(plan.node_count());
                metrics.record_plan_time(plan_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.plan_ms = plan_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::PlanningError,
                    QueryPhase::Plan,
                    e.to_string(),
                ));
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        let optimize_start = Instant::now();
        let optimized_plan = match self.optimize_execution_plan(execution_plan) {
            Ok(plan) => {
                profile.stages.optimize_ms = optimize_start.elapsed().as_millis() as u64;
                metrics.record_optimize_time(optimize_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.optimize_ms = optimize_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::OptimizationError,
                    QueryPhase::Optimize,
                    e.to_string(),
                ));
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        let execute_start = Instant::now();
        let result = match self.execute_plan(query_context, optimized_plan) {
            Ok(result) => {
                profile.stages.execute_ms = execute_start.elapsed().as_millis() as u64;
                profile.result_count = result.count();
                metrics.set_result_row_count(result.count());
                metrics.record_execute_time(execute_start.elapsed());
                result
            }
            Err(e) => {
                profile.stages.execute_ms = execute_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(ErrorInfo::new(
                    ErrorType::ExecutionError,
                    QueryPhase::Execute,
                    e.to_string(),
                ));
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };

        profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
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
    /// # 参数
    /// AST (Abstract Syntax Tree)
    /// `qctx`: Query context ( persists throughout the entire lifecycle of the query).
    fn validate_query_with_context(
        &mut self,
        ast: Arc<crate::query::parser::ast::stmt::Ast>,
        qctx: Arc<QueryContext>,
    ) -> DBResult<ValidationInfo> {
        let mut validator =
            crate::query::validator::Validator::create_from_ast(&ast).ok_or_else(|| {
                DBError::from(QueryError::InvalidQuery(format!(
                    "不支持的语句类型: {:?}",
                    ast.stmt
                )))
            })?;

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
            let sub_plan = planner_enum
                .transform(validated, query_context)
                .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?;
            crate::query::planning::plan::ExecutionPlan::new(sub_plan.root().clone())
        } else {
            return Err(DBError::from(QueryError::pipeline_planning_error(
                crate::query::planning::planner::PlannerError::NoSuitablePlanner(
                    "No suitable planner found".to_string(),
                ),
            )));
        };

        Ok(plan)
    }

    fn optimize_execution_plan(
        &mut self,
        plan: crate::query::planning::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planning::plan::ExecutionPlan> {
        use crate::query::optimizer::OptimizationContext;
        use crate::query::optimizer::strategy::{MaterializationOptimizer, StrategyChain};
        use crate::query::planning::rewrite::rewrite_plan;

        // Create optimization context from OptimizerEngine
        let mut ctx = OptimizationContext::from(&self.optimizer_engine);

        // Optimize using the planner rewrite rule.
        let rewritten_plan = rewrite_plan(plan)
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))?;

        // Apply optimization strategies using StrategyChain
        if let Some(root) = rewritten_plan.root {
            // Perform batch plan analysis
            let batch_analyzer = self.optimizer_engine.batch_plan_analyzer();
            let batch_analysis = batch_analyzer.analyze(&root);
            ctx.set_batch_plan_analysis(batch_analysis);

            // Create materialization optimizer
            let stats_manager = ctx.stats_manager();
            let materialization_optimizer = MaterializationOptimizer::new(stats_manager.as_ref());

            // Create strategy chain with materialization optimizer
            let chain = StrategyChain::new()
                .add_strategy(Box::new(materialization_optimizer));

            // Apply strategies to the plan root
            let optimized_root = chain.apply(root, &ctx)
                .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))?;

            // Check for repeated subplans
            if let Some(analysis) = ctx.batch_plan_analysis() {
                if analysis.reference_count.repeated_count() > 0 {
                    log::debug!(
                        "发现 {} 个被多次引用的子计划",
                        analysis.reference_count.repeated_count()
                    );
                }
            }

            Ok(crate::query::planning::plan::ExecutionPlan::new(Some(optimized_root)))
        } else {
            Ok(rewritten_plan)
        }
    }

    fn execute_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: crate::query::planning::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        use crate::query::executor::factory::executors::plan_executor::PlanExecutor;
        let mut plan_executor =
            PlanExecutor::with_object_pool(self.executor_factory.clone(), self.object_pool.clone());
        plan_executor
            .execute_plan(query_context, plan)
            .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
    }
}
