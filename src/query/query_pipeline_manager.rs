//! 查询管道管理器
//!
//! 负责协调整个查询处理流程：
//! 1. 管理查询处理的全生命周期
//! 2. 协调各个处理阶段（解析→验证→规划→优化→执行）
//! 3. 处理错误和异常
//! 4. 管理查询上下文和性能监控

use crate::api::service::stats_manager::{QueryMetrics, StatsManager};
use crate::query::context::execution::QueryContext;
use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::traits::ExecutionResult;
use crate::query::optimizer::{Optimizer, OptimizationConfig, RuleConfig};
use crate::query::parser::Parser;
use crate::query::planner::planner::{StaticConfigurablePlannerRegistry, PlannerConfig};
use crate::query::validator::Validator;
use crate::storage::StorageClient;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

pub struct QueryPipelineManager<S: StorageClient + 'static> {
    validator: Validator,
    planner: StaticConfigurablePlannerRegistry,
    optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    pub fn new(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = StaticConfigurablePlannerRegistry::new();

        Self::register_planners(&mut planner);

        Self {
            validator: Validator::new(),
            planner,
            optimizer: Optimizer::from_registry(),
            executor_factory,
            stats_manager,
        }
    }

    pub fn with_optimizer_config(storage: Arc<Mutex<S>>, rule_config: RuleConfig, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = StaticConfigurablePlannerRegistry::new();

        Self::register_planners(&mut planner);

        let config = OptimizationConfig::with_rule_config(rule_config);
        let optimizer = Optimizer::with_config(vec![], config);

        Self {
            validator: Validator::new(),
            planner,
            optimizer,
            executor_factory,
            stats_manager,
        }
    }

    pub fn from_config_file(storage: Arc<Mutex<S>>, config_path: &PathBuf, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = StaticConfigurablePlannerRegistry::new();

        Self::register_planners(&mut planner);

        let optimizer = match crate::query::optimizer::load_optimizer_config(config_path) {
            Ok(config_info) => {
                let rule_config = config_info.to_rule_config();
                let opt_config = OptimizationConfig {
                    max_iteration_rounds: config_info.max_iteration_rounds,
                    max_exploration_rounds: config_info.max_exploration_rounds,
                    enable_cost_model: config_info.enable_cost_model,
                    enable_multi_plan: config_info.enable_multi_plan,
                    enable_property_pruning: config_info.enable_property_pruning,
                    rule_config: Some(rule_config),
                    enable_adaptive_iteration: config_info.enable_adaptive_iteration,
                    stable_threshold: config_info.stable_threshold,
                    min_iteration_rounds: config_info.min_iteration_rounds,
                };
                Optimizer::with_config(vec![], opt_config)
            }
            Err(_) => {
                log::warn!("无法加载优化器配置，使用默认配置");
                Optimizer::from_registry()
            }
        };

        Self {
            validator: Validator::new(),
            planner,
            optimizer,
            executor_factory,
            stats_manager,
        }
    }

    pub fn with_config(storage: Arc<Mutex<S>>, config: PlannerConfig, stats_manager: Arc<StatsManager>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = StaticConfigurablePlannerRegistry::with_config(config);

        Self::register_planners(&mut planner);

        Self {
            validator: Validator::new(),
            planner,
            optimizer: Optimizer::default(),
            executor_factory,
            stats_manager,
        }
    }

    fn register_planners(planner: &mut StaticConfigurablePlannerRegistry) {
        planner.register(
            crate::query::planner::planner::SentenceKind::Match,
            crate::query::planner::planner::MatchAndInstantiateEnum::Match(
                crate::query::planner::statements::match_statement_planner::MatchStatementPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::Go,
            crate::query::planner::planner::MatchAndInstantiateEnum::Go(
                crate::query::planner::statements::go_planner::GoPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::Lookup,
            crate::query::planner::planner::MatchAndInstantiateEnum::Lookup(
                crate::query::planner::statements::lookup_planner::LookupPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::Path,
            crate::query::planner::planner::MatchAndInstantiateEnum::Path(
                crate::query::planner::statements::path_planner::PathPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::Subgraph,
            crate::query::planner::planner::MatchAndInstantiateEnum::Subgraph(
                crate::query::planner::statements::subgraph_planner::SubgraphPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::FetchVertices,
            crate::query::planner::planner::MatchAndInstantiateEnum::FetchVertices(
                crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::FetchEdges,
            crate::query::planner::planner::MatchAndInstantiateEnum::FetchEdges(
                crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::Maintain,
            crate::query::planner::planner::MatchAndInstantiateEnum::Maintain(
                crate::query::planner::statements::maintain_planner::MaintainPlanner::new()
            ),
        );
        planner.register(
            crate::query::planner::planner::SentenceKind::UserManagement,
            crate::query::planner::planner::MatchAndInstantiateEnum::UserManagement(
                crate::query::planner::statements::user_management_planner::UserManagementPlanner::new()
            ),
        );
    }

    pub async fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        self.execute_query_with_space(query_text, None).await
    }
    
    pub async fn execute_query_with_space(
        &mut self, 
        query_text: &str,
        space_info: Option<crate::query::context::validate::types::SpaceInfo>,
    ) -> DBResult<ExecutionResult> {
        let mut query_context = self.create_query_context(query_text)?;
        let mut ast = self.parse_into_context(query_text)?;
        
        // 如果提供了空间信息，设置到 AST 上下文中
        if let Some(space) = space_info {
            ast.set_space(space);
        }
        
        self.validate_query(&mut query_context, &mut ast)?;
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;
        self.execute_plan(&mut query_context, optimized_plan).await
    }

    pub async fn execute_query_with_metrics(
        &mut self,
        query_text: &str,
    ) -> DBResult<(ExecutionResult, QueryMetrics)> {
        let total_start = Instant::now();
        let mut metrics = QueryMetrics::new();
        
        let mut query_context = self.create_query_context(query_text)?;
        
        let parse_start = Instant::now();
        let mut ast = self.parse_into_context(query_text)?;
        metrics.record_parse_time(parse_start.elapsed());
        
        let validate_start = Instant::now();
        self.validate_query(&mut query_context, &mut ast)?;
        metrics.record_validate_time(validate_start.elapsed());
        
        let plan_start = Instant::now();
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;
        metrics.set_plan_node_count(execution_plan.node_count());
        metrics.record_plan_time(plan_start.elapsed());
        
        let optimize_start = Instant::now();
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;
        metrics.record_optimize_time(optimize_start.elapsed());
        
        let execute_start = Instant::now();
        let result = self.execute_plan(&mut query_context, optimized_plan).await?;
        metrics.set_result_row_count(result.count());
        metrics.record_execute_time(execute_start.elapsed());
        
        metrics.record_total_time(total_start.elapsed());
        
        self.stats_manager.record_query_metrics(&metrics);
        
        Ok((result, metrics))
    }

    fn create_query_context(&self, _query_text: &str) -> DBResult<QueryContext> {
        Ok(QueryContext::new())
    }

    fn parse_into_context(
        &mut self,
        query_text: &str,
    ) -> DBResult<crate::query::context::ast::AstContext> {
        let mut parser = Parser::new(query_text);
        match parser.parse() {
            Ok(stmt) => {
                let mut ast = crate::query::context::ast::AstContext::new(None, Some(stmt));
                ast.set_query_type_from_statement();
                Ok(ast)
            }
            Err(e) => Err(DBError::from(QueryError::pipeline_parse_error(e))),
        }
    }

    fn validate_query(
        &mut self,
        query_context: &mut QueryContext,
        ast: &mut crate::query::context::ast::AstContext,
    ) -> DBResult<()> {
        let _stmt = ast.sentence().ok_or_else(|| {
            DBError::from(QueryError::InvalidQuery("AST 上下文中缺少语句".to_string()))
        })?;
        self.validator.validate_with_ast_context(Some(query_context), ast)
    }

    fn generate_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        ast: &crate::query::context::ast::AstContext,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        self.planner
            .create_plan(query_context, ast)
            .map_err(|e| DBError::from(QueryError::pipeline_planning_error(e)))
    }

    fn optimize_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        self.optimizer
            .find_best_plan(query_context, plan)
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))
    }

    async fn execute_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        self.executor_factory
            .execute_plan(query_context, plan)
            .await
            .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
    }
}
