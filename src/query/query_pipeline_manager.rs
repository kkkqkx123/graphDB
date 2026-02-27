//! 查询管道管理器
//!
//! 负责协调整个查询处理流程：
//! 1. 管理查询处理的全生命周期
//! 2. 协调各个处理阶段（解析→验证→规划→优化→执行）
//! 3. 处理错误和异常
//! 4. 管理查询上下文和性能监控
//! 5. 查询优化决策缓存

use crate::core::{QueryMetrics, QueryProfile, StatsManager, ErrorInfo, ErrorType, QueryPhase};
use crate::query::QueryContext;
use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::base::ExecutionResult;
use crate::query::parser::Parser;
use crate::query::optimizer::decision::{
    DecisionCache, DecisionCacheConfig, DecisionCacheKey, OptimizationDecision,
};
use crate::query::planner::planner::SentenceKind;
use crate::query::planner::template_extractor::TemplateExtractor;
use crate::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

pub struct QueryPipelineManager<S: StorageClient + 'static> {
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
    decision_cache: Option<DecisionCache>,
    /// 统计信息版本（用于决策缓存失效）
    stats_version: u64,
    /// 索引版本（用于决策缓存失效）
    index_version: u64,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    pub fn new(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        // 尝试创建决策缓存
        let decision_cache = match DecisionCache::with_default_config() {
            Ok(cache) => {
                log::info!("查询优化决策缓存已启用");
                Some(cache)
            }
            Err(e) => {
                log::warn!("无法创建查询优化决策缓存: {}", e);
                None
            }
        };

        Self {
            executor_factory,
            stats_manager,
            decision_cache,
            stats_version: 1,
            index_version: 1,
        }
    }

    pub fn with_config(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        // 尝试创建决策缓存
        let decision_cache = match DecisionCache::with_default_config() {
            Ok(cache) => {
                log::info!("查询优化决策缓存已启用");
                Some(cache)
            }
            Err(e) => {
                log::warn!("无法创建查询优化决策缓存: {}", e);
                None
            }
        };

        Self {
            executor_factory,
            stats_manager,
            decision_cache,
            stats_version: 1,
            index_version: 1,
        }
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

        // 获取或计算优化决策
        let decision = if let Some(ref cache) = self.decision_cache {
            // 构建决策缓存键
            let template = TemplateExtractor::extract(stmt);
            let template_hash = DecisionCacheKey::hash_template(&template);
            let decision_key = DecisionCacheKey::new(
                template_hash,
                query_context.space_id().map(|id| id as i32),
                kind,
                Self::generate_pattern_fingerprint(stmt),
            );

            // 尝试从缓存获取或使用计算函数
            match cache.get_or_compute(
                decision_key,
                self.stats_version,
                self.index_version,
                || self.compute_decision(stmt, query_context.clone(), kind),
            ) {
                Ok(decision) => {
                    log::debug!("优化决策缓存命中或使用新决策");
                    Some(decision)
                }
                Err(e) => {
                    log::warn!("决策缓存操作失败: {}", e);
                    None
                }
            }
        } else {
            None
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

    /// 计算优化决策
    fn compute_decision(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
        kind: SentenceKind,
    ) -> Result<OptimizationDecision, crate::query::optimizer::decision::DecisionCacheError> {
        use crate::query::optimizer::decision::{
            TraversalStartDecision, AccessPath, EntityType,
            IndexSelectionDecision, JoinOrderDecision,
        };
        use crate::query::optimizer::{TraversalStartSelector, CostCalculator, SelectivityEstimator};
        use crate::query::optimizer::stats::StatisticsManager;

        // 创建统计信息管理器和代价计算器
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager.clone()));
        let selectivity_estimator = Arc::new(SelectivityEstimator::new(stats_manager.clone()));

        // 根据语句类型计算决策
        match kind {
            SentenceKind::Match => {
                // 对于 MATCH 语句，计算遍历起点决策
                let selector = TraversalStartSelector::new(
                    cost_calculator,
                    selectivity_estimator,
                );

                // 提取模式并选择起点
                if let crate::query::parser::ast::Stmt::Match(match_stmt) = stmt {
                    // 简化实现：使用第一个模式
                    if let Some(pattern) = match_stmt.patterns.first() {
                        if let Some(candidate) = selector.select_start_node(pattern) {
                            let access_path = Self::convert_selection_reason_to_access_path(
                                &candidate.reason,
                            );

                            let variable_name = candidate.node_pattern.variable.clone()
                                .unwrap_or_else(|| "n".to_string());

                            let traversal_decision = TraversalStartDecision::new(
                                variable_name,
                                access_path,
                                candidate.estimated_start_nodes as f64 / 10000.0, // 简化的选择性计算
                                candidate.estimated_cost,
                            );

                            return Ok(OptimizationDecision::new(
                                traversal_decision,
                                IndexSelectionDecision::empty(),
                                JoinOrderDecision::empty(),
                                self.stats_version,
                                self.index_version,
                            ));
                        }
                    }
                }

                // 默认决策
                Ok(OptimizationDecision::new(
                    TraversalStartDecision::new(
                        "n".to_string(),
                        AccessPath::FullScan {
                            entity_type: EntityType::Vertex { tag_name: None },
                        },
                        1.0,
                        1000.0,
                    ),
                    IndexSelectionDecision::empty(),
                    JoinOrderDecision::empty(),
                    self.stats_version,
                    self.index_version,
                ))
            }
            _ => {
                // 其他语句类型使用默认决策
                Ok(OptimizationDecision::new(
                    TraversalStartDecision::new(
                        "default".to_string(),
                        AccessPath::FullScan {
                            entity_type: EntityType::Vertex { tag_name: None },
                        },
                        1.0,
                        1000.0,
                    ),
                    IndexSelectionDecision::empty(),
                    JoinOrderDecision::empty(),
                    self.stats_version,
                    self.index_version,
                ))
            }
        }
    }

    /// 将选择原因转换为访问路径
    fn convert_selection_reason_to_access_path(
        reason: &crate::query::optimizer::strategy::SelectionReason,
    ) -> crate::query::optimizer::decision::AccessPath {
        use crate::query::optimizer::strategy::SelectionReason;
        use crate::query::optimizer::decision::{AccessPath, EntityType};

        match reason {
            SelectionReason::ExplicitVid => AccessPath::ExplicitVid {
                vid_description: "explicit".to_string(),
            },
            SelectionReason::HighSelectivityIndex { .. } => AccessPath::IndexScan {
                index_name: "auto".to_string(),
                property_name: "unknown".to_string(),
                predicate_description: "high_selectivity".to_string(),
            },
            SelectionReason::TagIndex { .. } => AccessPath::TagIndex {
                tag_name: "default".to_string(),
            },
            SelectionReason::FullScan { .. } => AccessPath::FullScan {
                entity_type: EntityType::Vertex { tag_name: None },
            },
            SelectionReason::VariableBinding { variable_name } => AccessPath::VariableBinding {
                source_variable: variable_name.clone(),
            },
        }
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
