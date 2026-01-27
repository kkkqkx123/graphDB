use crate::query::context::execution::QueryContext;
use crate::core::error::{DBError, DBResult};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::traits::ExecutionResult;
use crate::query::optimizer::Optimizer;
use crate::query::parser::Parser;
use crate::query::planner::planner::{ConfigurablePlannerRegistry, Planner, PlannerConfig};
use crate::query::validator::Validator;
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

/// 查询管道管理器 - 负责协调整个查询处理流程
///
/// 这个类取代了原来的QueryConverter，现在负责：
/// 1. 管理查询处理的全生命周期
/// 2. 协调各个处理阶段（解析→验证→规划→优化→执行）
/// 3. 处理错误和异常
/// 4. 管理查询上下文
pub struct QueryPipelineManager<S: StorageEngine + 'static> {
    _storage: Arc<Mutex<S>>,
    validator: Validator,
    planner: ConfigurablePlannerRegistry,
    optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
}

impl<S: StorageEngine + 'static> QueryPipelineManager<S> {
    /// 创建新的查询管道管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = ConfigurablePlannerRegistry::new();

        Self::register_planners(&mut planner);

        Self {
            _storage: storage,
            validator: Validator::new(),
            planner,
            optimizer: Optimizer::default(),
            executor_factory,
        }
    }

    /// 创建带配置的查询管道管理器
    pub fn with_config(storage: Arc<Mutex<S>>, config: PlannerConfig) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());
        let mut planner = ConfigurablePlannerRegistry::with_config(config);

        Self::register_planners(&mut planner);

        Self {
            _storage: storage,
            validator: Validator::new(),
            planner,
            optimizer: Optimizer::default(),
            executor_factory,
        }
    }

    fn register_planners(planner: &mut ConfigurablePlannerRegistry) {
        // 注册新的 MATCH 语句规划器 (使用三层架构)
        planner.register_planner(
            crate::query::planner::planner::SentenceKind::Match,
            crate::query::planner::statements::match_statement_planner::MatchStatementPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::match_statement_planner::MatchStatementPlanner::new()) as Box<dyn Planner>,
            100,
        );
    }

    /// 执行查询的主要入口点
    ///
    /// # 参数
    /// * `query_text` - 查询文本
    ///
    /// # 返回
    /// * `Ok(ExecutionResult)` - 查询执行结果
    /// * `Err(QueryError)` - 查询处理过程中的错误
    pub async fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
        let mut query_context = self.create_query_context(query_text)?;
        let mut ast = self.parse_into_context(query_text)?;
        self.validate_query(&mut query_context, &mut ast)?;
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;
        self.execute_plan(&mut query_context, optimized_plan).await
    }

    /// 创建查询上下文
    fn create_query_context(&self, _query_text: &str) -> DBResult<QueryContext> {
        Ok(QueryContext::new())
    }

    /// 解析查询文本为 AST 上下文
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
            Err(e) => Err(DBError::Query(crate::core::error::QueryError::ParseError(
                format!("解析失败: {}", e),
            ))),
        }
    }

    /// 验证查询的语义正确性
    fn validate_query(
        &mut self,
        query_context: &mut QueryContext,
        ast: &mut crate::query::context::ast::AstContext,
    ) -> DBResult<()> {
        let _stmt = ast.sentence().ok_or_else(|| {
            DBError::Query(crate::core::error::QueryError::InvalidQuery(
                "AST 上下文中缺少语句".to_string(),
            ))
        })?;
        self.validator.validate_with_ast_context(Some(query_context), ast)
    }

    /// 生成执行计划
    fn generate_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        ast: &crate::query::context::ast::AstContext,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        self.planner
            .create_plan(query_context, ast)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::PlanningError(format!(
                    "规划失败: {}",
                    e
                )))
            })
    }

    /// 优化执行计划
    fn optimize_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        self.optimizer
            .find_best_plan(query_context, plan)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::OptimizationError(format!(
                    "优化失败: {}",
                    e
                )))
            })
    }

    /// 执行优化后的计划
    async fn execute_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<ExecutionResult> {
        self.executor_factory
            .execute_plan(query_context, plan)
            .await
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "执行失败: {}",
                    e
                )))
            })
    }

    /// 获取规划器配置
    pub fn planner_config(&self) -> &PlannerConfig {
        self.planner.config()
    }

    /// 更新规划器配置
    pub fn set_planner_config(&mut self, config: PlannerConfig) {
        self.planner.set_config(config);
    }

    /// 清空计划缓存
    pub fn clear_plan_cache(&mut self) {
        self.planner.clear_cache();
    }
}
