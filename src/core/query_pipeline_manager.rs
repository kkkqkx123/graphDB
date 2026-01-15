use crate::core::context::query::QueryContext;
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
    _parser: Parser,
    validator: Validator,
    _planner: Box<dyn Planner>,
    _optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
}

impl<S: StorageEngine + 'static> QueryPipelineManager<S> {
    /// 创建新的查询管道管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let executor_factory = ExecutorFactory::with_storage(storage.clone());

        Self {
            _storage: storage,
            _parser: Parser::new(""),
            validator: Validator::new(crate::query::validator::ValidationContext::new()),
            _planner: Box::new(crate::query::planner::SequentialPlanner::new()),
            _optimizer: Optimizer::default(),
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

        // 2. 解析查询
        let ast = self.parse_query(&mut query_context, query_text)?;

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
    fn create_query_context(&self, query_text: &str) -> DBResult<QueryContext> {
        let session_info = crate::core::context::session::SessionInfo::new(
            "default_session".to_string(),
            "default_user".to_string(),
            vec![],
            "127.0.0.1".to_string(),
            8080,
            "default_client".to_string(),
            "default_connection".to_string(),
        );
        Ok(QueryContext::new(
            uuid::Uuid::new_v4().to_string(),
            crate::core::context::query::QueryType::DataQuery,
            query_text,
            session_info,
        ))
    }

    /// 解析查询文本为AST
    fn parse_query(
        &mut self,
        _query_context: &mut QueryContext,
        query_text: &str,
    ) -> DBResult<crate::query::context::ast::QueryAstContext> {
        let _parser = Parser::new(query_text);
        // 临时实现：返回一个空的AST上下文
        // 在实际实现中，这里应该调用parser.parse()并处理结果
        let ast = crate::query::context::ast::QueryAstContext::new(query_text);

        Ok(ast)
    }

    /// 验证查询的语义正确性
    fn validate_query(
        &mut self,
        _query_context: &mut QueryContext,
        _ast: &crate::query::context::ast::QueryAstContext,
    ) -> DBResult<()> {
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
        _ast: &crate::query::context::ast::QueryAstContext,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 临时实现：创建一个空的执行计划
        // 在实际实现中，这里应该调用planner.transform(ast)
        let mut plan = crate::query::planner::plan::ExecutionPlan::new(None);
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

    /// 优化执行计划
    fn optimize_execution_plan(
        &mut self,
        _query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
        // 临时实现：直接返回原计划
        // 在实际实现中，这里应该调用optimizer.find_best_plan()
        Ok(plan)
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
