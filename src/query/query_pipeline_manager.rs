//! 查询管道管理器
//!
//! 负责协调整个查询处理流程：
//! 1. 管理查询处理的全生命周期
//! 2. 协调各个处理阶段（解析→验证→规划→优化→执行）
//! 3. 处理错误和异常
//! 4. 管理查询上下文和性能监控
//! 5. 查询计划缓存

use crate::core::{QueryMetrics, QueryProfile, StatsManager, ErrorInfo, ErrorType, QueryPhase};
use crate::query::QueryContext;
use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::base::ExecutionResult;
use crate::query::parser::Parser;
use crate::query::planner::planner::{PlanCache, PlanCacheKey};
use crate::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

pub struct QueryPipelineManager<S: StorageClient + 'static> {
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
    plan_cache: Option<PlanCache>,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    pub fn new(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        // 尝试创建计划缓存
        let plan_cache = match PlanCache::with_default_config() {
            Ok(cache) => {
                log::info!("查询计划缓存已启用");
                Some(cache)
            }
            Err(e) => {
                log::warn!("无法创建查询计划缓存: {}", e);
                None
            }
        };

        Self {
            executor_factory,
            stats_manager,
            plan_cache,
        }
    }

    pub fn with_config(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        // 尝试创建计划缓存
        let plan_cache = match PlanCache::with_default_config() {
            Ok(cache) => {
                log::info!("查询计划缓存已启用");
                Some(cache)
            }
            Err(e) => {
                log::warn!("无法创建查询计划缓存: {}", e);
                None
            }
        };

        Self {
            executor_factory,
            stats_manager,
            plan_cache,
        }
    }

    pub async fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        self.execute_query_with_space(query_text, None).await
    }
    
    pub async fn execute_query_with_space(
        &mut self,
        query_text: &str,
        _space_info: Option<crate::core::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        let query_context = Arc::new(self.create_query_context(query_text)?);
        let stmt = self.parse_into_context(query_text)?;
        
        self.validate_query(query_context.clone(), &stmt)?;
        let execution_plan = self.generate_execution_plan(query_context.clone(), &stmt)?;
        let optimized_plan = self.optimize_execution_plan(query_context.clone(), execution_plan)?;
        self.execute_plan(query_context, optimized_plan).await
    }

    pub async fn execute_query_with_metrics(
        &mut self,
        query_text: &str,
    ) -> DBResult<(ExecutionResult, QueryMetrics)> {
        self.execute_query_with_session(query_text, 0).await.map(|(result, metrics, _)| (result, metrics))
    }

    pub async fn execute_query_with_session(
        &mut self,
        query_text: &str,
        session_id: i64,
    ) -> DBResult<(ExecutionResult, QueryMetrics, QueryProfile)> {
        self.execute_query_with_profile(query_text, session_id).await
    }

    pub async fn execute_query_with_profile(
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
        let result = match self.execute_plan(query_context, optimized_plan).await {
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

    fn create_query_context(&self, _query_text: &str) -> DBResult<QueryContext> {
        Ok(QueryContext::default())
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
        // 1. 尝试从缓存获取
        if let Some(ref cache) = self.plan_cache {
            match PlanCacheKey::from_stmt(stmt, query_context.space_id().map(|id| id as i32)) {
                Ok(cache_key) => {
                    match cache.get(&cache_key) {
                        Ok(Some(cached_plan)) => {
                            log::debug!("查询计划缓存命中");
                            return Ok(cached_plan);
                        }
                        Ok(None) => {
                            // 缓存未命中，继续生成计划
                        }
                        Err(e) => {
                            log::warn!("查询计划缓存访问失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("无法创建缓存键: {}", e);
                }
            }
        }

        // 2. 生成新计划
        let kind = crate::query::planner::planner::SentenceKind::from_stmt(stmt)
            .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?;
        
        let plan = if let Some(mut planner_enum) = crate::query::planner::planner::PlannerEnum::from_sentence_kind(kind) {
            let sub_plan = planner_enum.transform(stmt, query_context.clone())
                .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))?;
            crate::query::planner::plan::ExecutionPlan::new(sub_plan.root().clone())
        } else {
            return Err(DBError::from(QueryError::pipeline_planning_error(
                crate::query::planner::planner::PlannerError::NoSuitablePlanner(
                    "No suitable planner found".to_string()
                )
            )));
        };

        // 3. 存入缓存
        if let Some(ref cache) = self.plan_cache {
            if let Ok(cache_key) = PlanCacheKey::from_stmt(stmt, query_context.space_id().map(|id| id as i32)) {
                // 估算计划成本（使用节点数作为简单估算）
                let estimated_cost = plan.node_count() as f64 * 100.0;
                if let Err(e) = cache.insert(cache_key, plan.clone(), estimated_cost) {
                    log::warn!("无法缓存查询计划: {}", e);
                } else {
                    log::debug!("查询计划已缓存");
                }
            }
        }

        Ok(plan)
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

    async fn execute_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        self.executor_factory
            .execute_plan(query_context, plan)
            .await
            .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
    }
}
