//! Plan Executor
//!
//! Responsible for executing the execution plan and managing the lifecycle of the executor tree.

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use std::sync::Arc;

/// Plan Executor
pub struct PlanExecutor<S: StorageClient + Send + 'static> {
    factory: ExecutorFactory<S>,
}

impl<S: StorageClient + Send + 'static> PlanExecutor<S> {
    /// Create a new plan executor.
    pub fn new(factory: ExecutorFactory<S>) -> Self {
        Self { factory }
    }

    /// Execute the execution plan.
    pub fn execute_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: ExecutionPlan,
    ) -> Result<ExecutionResult, QueryError> {
        // Obtaining the storage engine
        let storage = match &self.factory.storage {
            Some(storage) => storage.clone(),
            None => return Err(QueryError::ExecutionError("存储引擎未设置".to_string())),
        };

        // Obtain the root node
        let root_node = match plan.root() {
            Some(node) => node,
            None => return Err(QueryError::ExecutionError("执行计划没有根节点".to_string())),
        };

        // Analyzing the lifecycle and security of execution plans
        self.factory.analyze_plan_lifecycle(root_node)?;

        // Check whether the query was terminated.
        if query_context.is_killed() {
            return Err(QueryError::ExecutionError("查询已被终止".to_string()));
        }

        // Create an execution context.
        let expr_context =
            Arc::new(crate::query::validator::context::ExpressionAnalysisContext::new());
        let execution_context = ExecutionContext::new(expr_context);

        // Recursively construct the executor tree and execute it.
        let mut executor = self
            .factory
            .create_executor(root_node, storage, &execution_context)?;

        // Root Executor
        let result = executor
            .execute()
            .map_err(|e| QueryError::ExecutionError(format!("Executor execution failed: {}", e)))?;

        // Return the execution result.
        Ok(result)
    }
}
