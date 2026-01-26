use crate::query::context::execution::QueryContext;
use crate::core::error::{DBError, DBResult};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::traits::ExecutionResult;
use crate::query::optimizer::Optimizer;
use crate::query::parser::Parser;
use crate::query::planner::Planner;
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
    planner: Box<dyn Planner>,
    optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
}

impl<S: StorageEngine + 'static> QueryPipelineManager<S> {
    /// 创建新的查询管道管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        Self {
            _storage: storage,
            validator: Validator::new(crate::query::validator::ValidationContext::new()),
            planner: Box::new(crate::query::planner::SequentialPlanner::new()),
            optimizer: Optimizer::default(),
            executor_factory,
        }
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
        // 1. 创建查询上下文
        let mut query_context = self.create_query_context(query_text)?;

        // 2. 解析查询并生成 AST 上下文
        let ast = self.parse_into_context(query_text)?;

        // 3. 验证查询
        self.validate_query(&mut query_context, &ast)?;

        // 4. 生成执行计划
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;

        // 5. 优化执行计划
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;

        // 6. 执行计划
        self.execute_plan(&mut query_context, optimized_plan).await
    }

    /// 创建查询上下文
    fn create_query_context(&self, _query_text: &str) -> DBResult<QueryContext> {
        Ok(QueryContext::new())
    }

    /// 解析查询文本为 AST 上下文
    ///
    /// 直接生成 QueryAstContext，Parser 输出的 Stmt 会自动设置到上下文中
    fn parse_into_context(
        &mut self,
        query_text: &str,
    ) -> DBResult<crate::query::context::ast::QueryAstContext> {
        let mut parser = Parser::new(query_text);
        match parser.parse() {
            Ok(stmt) => {
                let mut ast = crate::query::context::ast::QueryAstContext::new(query_text);
                ast.set_statement(stmt);
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
        _query_context: &mut QueryContext,
        ast: &crate::query::context::ast::QueryAstContext,
    ) -> DBResult<()> {
        let _stmt = ast.base_context().sentence().ok_or_else(|| {
            DBError::Query(crate::core::error::QueryError::InvalidQuery(
                "AST 上下文中缺少语句".to_string(),
            ))
        })?;
        self.validator.validate_unified().map_err(|e| {
            DBError::Query(crate::core::error::QueryError::InvalidQuery(format!(
                "验证失败: {}",
                e
            )))
        })
    }

    /// 生成执行计划
    fn generate_execution_plan(
        &mut self,
        _query_context: &mut QueryContext,
        ast: &crate::query::context::ast::QueryAstContext,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        let ast_ctx = ast.base_context();
        match self.planner.transform(ast_ctx) {
            Ok(sub_plan) => {
                let mut plan = crate::query::planner::plan::ExecutionPlan::new(sub_plan.root().clone());
                let uuid = uuid::Uuid::new_v4();
                let uuid_bytes = uuid.as_bytes();
                let id = i64::from_ne_bytes([
                    uuid_bytes[0],
                    uuid_bytes[1],
                    uuid_bytes[2],
                    uuid_bytes[3],
                    uuid_bytes[4],
                    uuid_bytes[5],
                    uuid_bytes[6],
                    uuid_bytes[7],
                ]);
                plan.set_id(id);
                Ok(plan)
            }
            Err(e) => Err(DBError::Query(crate::core::error::QueryError::PlanningError(
                format!("规划失败: {}", e),
            ))),
        }
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
        // 调用执行器工厂执行计划
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
}
