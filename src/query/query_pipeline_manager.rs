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

use crate::core::{QueryMetrics, QueryProfile, StatsManager, ErrorInfo, ErrorType, QueryPhase};
use crate::query::QueryContext;
use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::base::ExecutionResult;
use crate::query::parser::Parser;
use crate::query::optimizer::OptimizerEngine;
use crate::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

/// 查询管道管理器
///
/// 负责协调整查询处理流程，通过引用 `OptimizerEngine` 使用优化功能。
pub struct QueryPipelineManager<S: StorageClient + 'static> {
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
    /// 优化器引擎（全局实例的引用）
    optimizer_engine: Arc<OptimizerEngine>,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    /// 使用默认优化器引擎创建
    ///
    /// 注意：这会创建一个新的 OptimizerEngine 实例。在生产环境中，
    /// 建议使用 `with_optimizer` 方法传入全局共享的 OptimizerEngine。
    pub fn new(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let optimizer_engine = Arc::new(OptimizerEngine::default());
        Self::with_optimizer(storage, stats_manager, optimizer_engine)
    }

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

        log::info!("查询管道管理器已创建，使用优化器引擎");

        Self {
            executor_factory,
            stats_manager,
            optimizer_engine,
        }
    }

    /// 获取优化器引擎
    pub fn optimizer_engine(&self) -> &OptimizerEngine {
        &self.optimizer_engine
    }

    pub fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        self.execute_query_with_space(query_text, None)
    }

    pub fn execute_query_with_space(
        &mut self,
        query_text: &str,
        _space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        let query_context = Arc::new(self.create_query_context(query_text)?);
        let stmt = self.parse_into_context(query_text)?;

        self.validate_query(query_context.clone(), &stmt)?;
        let execution_plan = self.generate_execution_plan(query_context.clone(), &stmt)?;
        let optimized_plan = self.optimize_execution_plan(query_context.clone(), execution_plan)?;
        self.execute_plan(query_context, optimized_plan)
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
        let query_context = self.create_query_context_with_request(rctx)?;
        
        // 设置空间信息
        if let Some(space) = space_info {
            query_context.set_space_info(space);
        }
        
        let query_context = Arc::new(query_context);
        let stmt = self.parse_into_context(query_text)?;

        self.validate_query(query_context.clone(), &stmt)?;
        let execution_plan = self.generate_execution_plan(query_context.clone(), &stmt)?;
        let optimized_plan = self.optimize_execution_plan(query_context.clone(), execution_plan)?;
        self.execute_plan(query_context, optimized_plan)
    }

    pub fn execute_query_with_metrics(
        &mut self,
        query_text: &str,
    ) -> DBResult<(ExecutionResult, QueryMetrics)> {
        self.execute_query_with_session(query_text, 0).map(|(result, metrics, _)| (result, metrics))
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
        
        let query_context = Arc::new(self.create_query_context(query_text)?);
        
        let parse_start = Instant::now();
        let stmt = match self.parse_into_context(query_text) {
            Ok(stmt) => {
                profile.stages.parse_ms = parse_start.elapsed().as_millis() as u64;
                metrics.record_parse_time(parse_start.elapsed());
                stmt
            }
            Err(e) => {
                profile.stages.parse_ms = parse_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(
                    ErrorInfo::new(ErrorType::ParseError, QueryPhase::Parse, e.to_string())
                );
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };
        
        let validate_start = Instant::now();
        if let Err(e) = self.validate_query(query_context.clone(), &stmt) {
            profile.stages.validate_ms = validate_start.elapsed().as_millis() as u64;
            profile.mark_failed_with_info(
                ErrorInfo::new(ErrorType::ValidationError, QueryPhase::Validate, e.to_string())
            );
            profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
            self.stats_manager.record_query_profile(profile.clone());
            return Err(e);
        }
        profile.stages.validate_ms = validate_start.elapsed().as_millis() as u64;
        metrics.record_validate_time(validate_start.elapsed());
        
        let plan_start = Instant::now();
        let execution_plan = match self.generate_execution_plan(query_context.clone(), &stmt) {
            Ok(plan) => {
                profile.stages.plan_ms = plan_start.elapsed().as_millis() as u64;
                metrics.set_plan_node_count(plan.node_count());
                metrics.record_plan_time(plan_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.plan_ms = plan_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(
                    ErrorInfo::new(ErrorType::PlanningError, QueryPhase::Plan, e.to_string())
                );
                profile.total_duration_ms = total_start.elapsed().as_millis() as u64;
                self.stats_manager.record_query_profile(profile.clone());
                return Err(e);
            }
        };
        
        let optimize_start = Instant::now();
        let optimized_plan = match self.optimize_execution_plan(query_context.clone(), execution_plan) {
            Ok(plan) => {
                profile.stages.optimize_ms = optimize_start.elapsed().as_millis() as u64;
                metrics.record_optimize_time(optimize_start.elapsed());
                plan
            }
            Err(e) => {
                profile.stages.optimize_ms = optimize_start.elapsed().as_millis() as u64;
                profile.mark_failed_with_info(
                    ErrorInfo::new(ErrorType::OptimizationError, QueryPhase::Optimize, e.to_string())
                );
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
                profile.mark_failed_with_info(
                    ErrorInfo::new(ErrorType::ExecutionError, QueryPhase::Execute, e.to_string())
                );
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

    /// 创建查询上下文（使用默认配置）
    fn create_query_context(&self, query_text: &str) -> DBResult<QueryContext> {
        use crate::query::query_request_context::QueryRequestContext;
        let rctx = Arc::new(QueryRequestContext::new(query_text.to_string()));
        Ok(QueryContext::new(rctx))
    }

    /// 从 QueryRequestContext 创建查询上下文
    /// 
    /// 这个方法允许 api 层传递会话信息到 query 层
    pub fn create_query_context_with_request(
        &self,
        rctx: Arc<crate::query::query_request_context::QueryRequestContext>,
    ) -> DBResult<QueryContext> {
        Ok(QueryContext::new(rctx))
    }

    fn parse_into_context(
        &mut self,
        query_text: &str,
    ) -> DBResult<crate::query::parser::ast::Stmt> {
        let mut parser = Parser::new(query_text);
        parser.parse()
            .map_err(|e| DBError::from(QueryError::pipeline_parse_error(e)))
    }

    fn validate_query(
        &mut self,
        query_context: Arc<QueryContext>,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> DBResult<()> {
        let mut validator = crate::query::validator::Validator::from_stmt(stmt)
            .ok_or_else(|| {
                DBError::from(QueryError::InvalidQuery(format!(
                    "不支持的语句类型: {:?}",
                    stmt
                )))
            })?;

        validator.validate(stmt, query_context)
            .map(|_| ())
            .map_err(|e| DBError::from(QueryError::pipeline_validation_error(e)))
    }

    fn generate_execution_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 获取语句类型
        let kind = crate::query::planner::planner::SentenceKind::from_stmt(stmt)
            .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?;

        // 使用 OptimizerEngine 计算优化决策
        let decision = match self.optimizer_engine.compute_decision(stmt, kind) {
            Ok(decision) => {
                log::debug!("优化决策计算成功");
                Some(decision)
            }
            Err(e) => {
                log::warn!("优化决策计算失败: {}", e);
                None
            }
        };

        // 生成执行计划
        let plan = if let Some(mut planner_enum) = crate::query::planner::planner::PlannerEnum::from_sentence_kind(kind) {
            let sub_plan = if let Some(ref decision) = decision {
                // 使用预计算的决策生成计划
                planner_enum.transform_with_decision(stmt, query_context.clone(), decision)
                    .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?
            } else {
                // 不使用决策，直接生成计划
                planner_enum.transform(stmt, query_context.clone())
                    .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?
            };
            crate::query::planner::plan::ExecutionPlan::new(sub_plan.root().clone())
        } else {
            return Err(DBError::from(QueryError::pipeline_planning_error(
                crate::query::planner::planner::PlannerError::NoSuitablePlanner(
                    "No suitable planner found".to_string()
                )
            )));
        };

        Ok(plan)
    }

    /// 生成模式指纹
    fn generate_pattern_fingerprint(
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Option<String> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(m) => {
                let pattern_count = m.patterns.len();
                let has_where = m.where_clause.is_some();
                let has_return = m.return_clause.is_some();
                Some(format!("M:{}:W{}:R{}", pattern_count, has_where as u8, has_return as u8))
            }
            crate::query::parser::ast::Stmt::Go(g) => {
                let step_str = match &g.steps {
                    crate::query::parser::ast::Steps::Fixed(n) => format!("F{}", n),
                    crate::query::parser::ast::Steps::Range { min, max } => format!("R{}-{}", min, max),
                    crate::query::parser::ast::Steps::Variable(_) => "V".to_string(),
                };
                Some(format!("G:{}:S{}", step_str, g.over.as_ref().map(|_| "E").unwrap_or("N")))
            }
            _ => None,
        }
    }

    fn optimize_execution_plan(
        &mut self,
        _query_context: Arc<QueryContext>,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 使用 planner rewrite 规则进行优化
        use crate::query::planner::rewrite::rewrite_plan;
        
        let optimized_plan = rewrite_plan(plan)
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))?;
        
        Ok(optimized_plan)
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
