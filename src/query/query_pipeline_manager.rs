//! 查询管道管理器
//!
//! 负责协调整个查询处理流程：
//! 1. 管理查询处理的全生命周期
//! 2. 协调各个处理阶段（解析→验证→规划→优化→执行）
//! 3. 处理错误和异常
//! 4. 管理查询上下文和性能监控
//!
//! ## 与 OptimizerEngine 的关系
//!
//! `QueryPipelineManager` 通过引用使用 `OptimizerEngine`，而不是直接创建优化器组件。
//! `OptimizerEngine` 是全局实例，与数据库实例同生命周期，负责所有查询优化相关的功能。
//!
//! ```rust
//! // 创建方式
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
use crate::query::optimizer::OptimizerEngine;
use crate::query::parser::Parser;
use crate::query::planner::{ParameterizedQueryHandler, PlanCacheConfig, QueryPlanCache};
use crate::query::query_request_context::QueryRequestContext;
use crate::query::validator::{ValidatedStatement, ValidationInfo};
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// 查询管道管理器
///
/// 负责协调整查询处理流程，通过引用 `OptimizerEngine` 使用优化功能。
pub struct QueryPipelineManager<S: StorageClient + 'static> {
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
    /// 优化器引擎（全局实例的引用）
    optimizer_engine: Arc<OptimizerEngine>,
    /// 查询计划缓存
    plan_cache: Arc<QueryPlanCache>,
    /// 参数化查询处理器
    param_handler: ParameterizedQueryHandler,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    /// 使用指定的优化器引擎创建
    ///
    /// 这是推荐的生产环境使用方式，可以共享全局的 OptimizerEngine 实例。
    ///
    /// # 参数
    /// - `storage`: 存储客户端
    /// - `stats_manager`: 统计信息管理器
    /// - `optimizer_engine`: 优化器引擎（全局实例）
    pub fn with_optimizer(
        storage: Arc<Mutex<S>>,
        stats_manager: Arc<StatsManager>,
        optimizer_engine: Arc<OptimizerEngine>,
    ) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let plan_cache = Arc::new(QueryPlanCache::default());
        let param_handler = ParameterizedQueryHandler::default();

        log::info!("查询管道管理器已创建，使用优化器引擎和查询计划缓存");

        Self {
            executor_factory,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
        }
    }

    /// 使用指定的优化器引擎和计划缓存配置创建
    ///
    /// # 参数
    /// - `storage`: 存储客户端
    /// - `stats_manager`: 统计信息管理器
    /// - `optimizer_engine`: 优化器引擎（全局实例）
    /// - `plan_cache_config`: 查询计划缓存配置
    pub fn with_optimizer_and_cache(
        storage: Arc<Mutex<S>>,
        stats_manager: Arc<StatsManager>,
        optimizer_engine: Arc<OptimizerEngine>,
        plan_cache_config: PlanCacheConfig,
    ) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let plan_cache = Arc::new(QueryPlanCache::new(plan_cache_config));
        let param_handler = ParameterizedQueryHandler::default();

        log::info!("查询管道管理器已创建，使用优化器引擎和自定义查询计划缓存");

        Self {
            executor_factory,
            stats_manager,
            optimizer_engine,
            plan_cache,
            param_handler,
        }
    }

    /// 获取优化器引擎
    pub fn optimizer_engine(&self) -> &OptimizerEngine {
        &self.optimizer_engine
    }

    /// 获取查询计划缓存
    pub fn plan_cache(&self) -> &QueryPlanCache {
        &self.plan_cache
    }

    /// 获取查询计划缓存统计
    pub fn plan_cache_stats(&self) -> crate::query::planner::PlanCacheStats {
        self.plan_cache.stats()
    }

    /// 清空查询计划缓存
    pub fn clear_plan_cache(&self) {
        self.plan_cache.clear();
        log::info!("查询计划缓存已清空");
    }

    pub fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        self.execute_query_with_space(query_text, None)
    }

    pub fn execute_query_with_space(
        &mut self,
        query_text: &str,
        space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        // 1. 首先创建 QueryContext（贯穿整个查询生命周期）
        let rctx = Arc::new(QueryRequestContext::new(query_text.to_string()));
        let mut query_context = QueryContext::new(rctx);

        // 设置空间信息
        if let Some(ref space) = space_info {
            query_context.set_space_info(space.clone());
        }

        let query_context = Arc::new(query_context);

        // 2. 检查查询计划缓存
        if let Some(cached_plan) = self.plan_cache.get(query_text) {
            log::debug!("查询计划缓存命中");
            let execute_start = Instant::now();
            let result = self.execute_plan(query_context, cached_plan.plan.clone())?;
            let execution_time_ms = execute_start.elapsed().as_millis() as f64;
            self.plan_cache.record_execution(query_text, execution_time_ms);
            return Ok(result);
        }

        // 3. 解析查询
        let parser_result = self.parse_into_context(query_text)?;

        // 4. 验证查询（复用已创建的 QueryContext）
        let validation_info = self.validate_query_with_context(
            parser_result.ast.clone(),
            query_context.clone(),
        )?;

        // 创建验证后的语句（使用 Arc<Ast> 共享所有权）
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        // 5. 生成执行计划
        let execution_plan = self.generate_execution_plan(query_context.clone(), &validated)?;

        // 6. 优化执行计划
        let optimized_plan = self.optimize_execution_plan(execution_plan)?;

        // 7. 执行计划
        let execute_start = Instant::now();
        let result = self.execute_plan(query_context, optimized_plan.clone())?;
        let execution_time_ms = execute_start.elapsed().as_millis() as f64;

        // 8. 缓存查询计划
        let param_positions = self.param_handler.extract_params(query_text);
        self.plan_cache.put(query_text, optimized_plan, param_positions);
        self.plan_cache.record_execution(query_text, execution_time_ms);

        Ok(result)
    }

    /// 使用 QueryRequestContext 执行查询
    ///
    /// 这个方法允许 api 层传递完整的会话信息到 query 层
    pub fn execute_query_with_request(
        &mut self,
        query_text: &str,
        rctx: Arc<crate::query::query_request_context::QueryRequestContext>,
        space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        // 1. 首先创建 QueryContext（贯穿整个查询生命周期）
        let mut query_context = QueryContext::new(rctx);

        // 设置空间信息（在 Arc 包装之前）
        if let Some(ref space) = space_info {
            query_context.set_space_info(space.clone());
        }

        let query_context = Arc::new(query_context);

        // 2. 解析查询
        let parser_result = self.parse_into_context(query_text)?;

        // 3. 验证查询（复用已创建的 QueryContext）
        let validation_info = self.validate_query_with_context(
            parser_result.ast.clone(),
            query_context.clone(),
        )?;

        // 创建验证后的语句（使用 Arc<Ast> 共享所有权）
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        // 4. 生成执行计划
        let execution_plan = self.generate_execution_plan(query_context.clone(), &validated)?;

        // 5. 优化执行计划
        let optimized_plan = self.optimize_execution_plan(execution_plan)?;

        // 6. 执行计划
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

        // 1. 首先创建 QueryContext（贯穿整个查询生命周期）
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
        let validation_info = match self.validate_query_with_context(
            parser_result.ast.clone(),
            query_context.clone(),
        ) {
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

        // 创建验证后的语句（使用 Arc<Ast> 共享所有权）
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

    /// 验证查询并返回验证信息（使用传入的 QueryContext）
    ///
    /// 此方法复用已创建的 QueryContext，避免临时上下文的创建和丢弃
    /// 确保整个查询生命周期使用统一的上下文
    ///
    /// # 参数
    /// - `ast`: 抽象语法树
    /// - `qctx`: 查询上下文（贯穿整个查询生命周期）
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

        // 使用传入的 QueryContext 进行验证
        // 避免创建临时上下文，确保资源（ID生成器、对象池等）的一致性
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

    /// 使用验证后的语句生成执行计划
    fn generate_execution_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        validated: &ValidatedStatement,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 直接使用 Arc<Ast> 创建规划器，消除 SentenceKind 字符串匹配
        let plan = if let Some(mut planner_enum) =
            crate::query::planner::planner::PlannerEnum::from_ast(&validated.ast)
        {
            let sub_plan = planner_enum
                .transform(validated, query_context)
                .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?;
            crate::query::planner::plan::ExecutionPlan::new(sub_plan.root().clone())
        } else {
            return Err(DBError::from(QueryError::pipeline_planning_error(
                crate::query::planner::planner::PlannerError::NoSuitablePlanner(
                    "No suitable planner found".to_string(),
                ),
            )));
        };

        Ok(plan)
    }

    fn optimize_execution_plan(
        &mut self,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 使用 planner rewrite 规则进行优化
        use crate::query::planner::rewrite::rewrite_plan;

        let rewritten_plan = rewrite_plan(plan)
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))?;

        // 使用优化器的分析器分析重写后的计划
        if let Some(ref root) = rewritten_plan.root {
            // 引用计数分析 - 识别被多次引用的子计划
            let ref_analysis = self
                .optimizer_engine
                .reference_count_analyzer()
                .analyze(root);
            if ref_analysis.repeated_count() > 0 {
                log::debug!(
                    "发现 {} 个被多次引用的子计划",
                    ref_analysis.repeated_count()
                );
            }
        }

        Ok(rewritten_plan)
    }

    fn execute_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        self.executor_factory
            .execute_plan(query_context, plan)
            .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
    }
}
