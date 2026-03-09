//! 计划执行器
//!
//! 负责执行执行计划，管理执行器树的生命周期

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutionResult};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use std::sync::Arc;

/// 计划执行器
pub struct PlanExecutor<S: StorageClient + Send + 'static> {
    factory: ExecutorFactory<S>,
}

impl<S: StorageClient + Send + 'static> PlanExecutor<S> {
    /// 创建新的计划执行器
    pub fn new(factory: ExecutorFactory<S>) -> Self {
        Self { factory }
    }

    /// 执行执行计划
    pub fn execute_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: ExecutionPlan,
    ) -> Result<ExecutionResult, QueryError> {
        // 获取存储引擎
        let storage = match &self.factory.storage {
            Some(storage) => storage.clone(),
            None => return Err(QueryError::ExecutionError("存储引擎未设置".to_string())),
        };

        // 获取根节点
        let root_node = match plan.root() {
            Some(node) => node,
            None => return Err(QueryError::ExecutionError("执行计划没有根节点".to_string())),
        };

        // 分析执行计划的生命周期和安全性
        self.factory.analyze_plan_lifecycle(root_node)?;

        // 检查查询是否被终止
        if query_context.is_killed() {
            return Err(QueryError::ExecutionError("查询已被终止".to_string()));
        }

        // 创建执行上下文
        let expr_context = Arc::new(crate::query::validator::context::ExpressionAnalysisContext::new());
        let execution_context = ExecutionContext::new(expr_context);

        // 递归构建执行器树并执行
        let mut executor =
            self.factory.create_executor(root_node, storage, &execution_context)?;

        // 执行根执行器
        let result = executor
            .execute()
            .map_err(|e| QueryError::ExecutionError(format!("Executor execution failed: {}", e)))?;

        // 返回执行结果
        Ok(result)
    }
}
