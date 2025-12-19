use crate::query::context::managers::r#impl::{MemorySchemaManager, MemoryIndexManager, MemoryMetaClient, MemoryStorageClient};
use crate::query::context::validate::ValidateContext;
use crate::query::context::QueryContext;
use crate::query::executor_factory::ExecutorFactory;
use crate::query::optimizer::Optimizer;
use crate::query::parser::cypher::CypherStatement;
use crate::query::parser::parser::Parser;
use crate::query::planner::Planner;
use crate::query::types::{QueryError, QueryResult};
use crate::query::validator::{Validator, ValidatorFactory};
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
    storage: Arc<Mutex<S>>,
    parser: Parser,
    validator_factory: ValidatorFactory,
    planner: Box<dyn Planner>,
    optimizer: Optimizer,
    executor_factory: ExecutorFactory<S>,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> QueryPipelineManager<S> {
    /// 创建新的查询管道管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let executor_factory = ExecutorFactory::new(Arc::clone(&storage));

        Self {
            storage,
            parser: Parser::new(""),
            validator_factory: ValidatorFactory::new(),
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
    pub async fn execute_query(&mut self, query_text: &str) -> Result<QueryResult, QueryError> {
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
    fn create_query_context(&self, query_text: &str) -> Result<QueryContext, QueryError> {
        Ok(QueryContext::new(
            "default_session".to_string(),
            "default_user".to_string(),
            std::sync::Arc::new(MemorySchemaManager::default()),
            std::sync::Arc::new(MemoryIndexManager::default()),
            std::sync::Arc::new(MemoryMetaClient::default()),
            std::sync::Arc::new(MemoryStorageClient::default()),
        ))
    }

    /// 解析查询文本为AST
    fn parse_query(
        &mut self,
        _query_context: &mut QueryContext,
        query_text: &str,
    ) -> Result<CypherStatement, QueryError> {
        let mut parser = crate::query::parser::cypher::CypherParser::new(query_text.to_string());
        let statement = parser.parse_statement()
            .map_err(|e| QueryError::InvalidQuery(format!("解析失败: {}", e)))?;

        Ok(statement)
    }

    /// 验证查询的语义正确性
    fn validate_query(
        &mut self,
        query_context: &mut QueryContext,
        statement: &CypherStatement,
    ) -> Result<(), QueryError> {
        // 从 CypherStatement 创建具体的验证器
        let validator = self.validator_factory.create_validator(
            statement,
            std::sync::Arc::new(query_context.clone()),
        ).map_err(|e| QueryError::InvalidQuery(format!("验证器创建失败: {}", e)))?;

        // 验证查询
        validator
            .validate()
            .map_err(|e| QueryError::InvalidQuery(format!("验证失败: {}", e)))
    }

    /// 生成执行计划
    fn generate_execution_plan(
        &mut self,
        _query_context: &mut QueryContext,
        statement: &CypherStatement,
    ) -> Result<crate::query::planner::plan::ExecutionPlan, QueryError> {
        // 临时实现：创建一个空的执行计划
        // 在实际实现中，这里应该调用planner.transform(statement)
        let plan = crate::query::planner::plan::ExecutionPlan::new(None);

        Ok(plan)
    }

    /// 优化执行计划
    fn optimize_execution_plan(
        &mut self,
        _query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<crate::query::planner::plan::ExecutionPlan, QueryError> {
        // 临时实现：直接返回原计划
        // 在实际实现中，这里应该调用optimizer.find_best_plan()
        Ok(plan)
    }

    /// 执行优化后的计划
    async fn execute_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<QueryResult, QueryError> {
        // 调用执行器工厂执行计划
        self.executor_factory
            .execute_plan(query_context, plan)
            .await
    }
}
