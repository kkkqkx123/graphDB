use crate::query::context::{QueryContext, RequestContext};
use crate::query::executor::ExecutorFactory;
use crate::query::optimizer::Optimizer;
use crate::query::parser::parser::Parser;
use crate::query::planner::Planner;
use crate::query::types::{QueryError, QueryResult};
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
#[derive(Debug)]
pub struct QueryPipelineManager<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    parser: Parser,
    validator: Validator,
    planner: Box<dyn Planner>,
    optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
}

impl<S: StorageEngine> QueryPipelineManager<S> {
    /// 创建新的查询管道管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let executor_factory = ExecutorFactory::new(Arc::clone(&storage));

        Self {
            storage,
            parser: Parser::new(""),
            validator: Validator::new(crate::query::validator::ValidateContext::new()),
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
    /// * `Ok(QueryResult)` - 查询执行结果
    /// * `Err(QueryError)` - 查询处理过程中的错误
    pub async fn execute_query(&self, query_text: &str) -> Result<QueryResult, QueryError> {
        // 1. 创建查询上下文
        let mut query_context = self.create_query_context(query_text)?;

        // 2. 解析查询
        let ast = self.parse_query(&mut query_context, query_text)?;

        // 3. 验证查询
        self.validate_query(&mut query_context, &ast)?;

        // 4. 生成执行计划
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;

        // 5. 优化执行计划
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;

        // 6. 执行计划
        self.execute_plan(&mut query_context, optimized_plan)
    }

    /// 创建查询上下文
    fn create_query_context(&self, query_text: &str) -> Result<QueryContext, QueryError> {
        let session_info = crate::query::context::request_context::SessionInfo::new(
            "default_session".to_string(),
            "default_user".to_string(),
            "localhost".to_string(),
            0,
        );
        let request_params =
            crate::query::context::request_context::RequestParams::new(query_text.to_string());
        let request_context = RequestContext::new(session_info, request_params);
        Ok(QueryContext::with_request_context(std::sync::Arc::new(
            request_context,
        )))
    }

    /// 解析查询文本为AST
    fn parse_query(
        &self,
        _query_context: &mut QueryContext,
        query_text: &str,
    ) -> Result<crate::query::context::ast::QueryAstContext, QueryError> {
        let mut parser = Parser::new(query_text);
        // 临时实现：返回一个空的AST上下文
        // 在实际实现中，这里应该调用parser.parse()并处理结果
        let ast = crate::query::context::ast::QueryAstContext::new(query_text);

        Ok(ast)
    }

    /// 验证查询的语义正确性
    fn validate_query(
        &self,
        query_context: &mut QueryContext,
        _ast: &crate::query::context::ast::QueryAstContext,
    ) -> Result<(), QueryError> {
        self.validator
            .validate()
            .map_err(|e| QueryError::InvalidQuery(format!("验证失败: {}", e)))
    }

    /// 生成执行计划
    fn generate_execution_plan(
        &self,
        query_context: &mut QueryContext,
        ast: &crate::query::context::ast::QueryAstContext,
    ) -> Result<crate::query::planner::plan::ExecutionPlan, QueryError> {
        // 临时实现：创建一个空的执行计划
        // 在实际实现中，这里应该调用planner.transform(ast)
        let mut plan = crate::query::planner::plan::ExecutionPlan::new(None);
        plan.set_id(query_context.gen_id());

        Ok(plan)
    }

    /// 优化执行计划
    fn optimize_execution_plan(
        &self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<crate::query::planner::plan::ExecutionPlan, QueryError> {
        // 临时实现：直接返回原计划
        // 在实际实现中，这里应该调用optimizer.find_best_plan()
        Ok(plan)
    }

    /// 执行优化后的计划
    async fn execute_plan(
        &self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<QueryResult, QueryError> {
        // 调用执行器工厂执行计划
        self.executor_factory.execute_plan(query_context, plan).await
    }
}
