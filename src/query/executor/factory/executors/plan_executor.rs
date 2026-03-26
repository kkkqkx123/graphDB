//! Plan Executor
//!
//! Responsible for executing the execution plan and managing the lifecycle of the executor tree.

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::object_pool::ThreadSafeExecutorPool;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use std::sync::Arc;

/// Plan Executor
pub struct PlanExecutor<S: StorageClient + Send + 'static> {
    factory: ExecutorFactory<S>,
    object_pool: Option<Arc<ThreadSafeExecutorPool<S>>>,
}

impl<S: StorageClient + Send + 'static> PlanExecutor<S> {
    /// Create a new plan executor.
    pub fn new(factory: ExecutorFactory<S>) -> Self {
        Self {
            factory,
            object_pool: None,
        }
    }

    /// Create a new plan executor with object pool.
    pub fn with_object_pool(
        factory: ExecutorFactory<S>,
        object_pool: Arc<ThreadSafeExecutorPool<S>>,
    ) -> Self {
        Self {
            factory,
            object_pool: Some(object_pool),
        }
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

        // Try to get executor from pool first
        let executor_type = root_node.name();
        let mut executor = if let Some(pool) = &self.object_pool {
            if let Some(pooled_executor) = pool.acquire(executor_type) {
                log::debug!("从对象池获取执行器: {}", executor_type);
                pooled_executor
            } else {
                log::debug!("对象池未命中，创建新执行器: {}", executor_type);
                self.factory
                    .create_executor(root_node, storage, &execution_context)?
            }
        } else {
            self.factory
                .create_executor(root_node, storage, &execution_context)?
        };

        // Root Executor
        let result = executor
            .execute()
            .map_err(|e| QueryError::ExecutionError(format!("Executor execution failed: {}", e)))?;

        // Release executor back to pool
        if let Some(pool) = &self.object_pool {
            pool.release(executor_type, executor);
            log::debug!("执行器已释放回对象池: {}", executor_type);
        }

        // Return the execution result.
        Ok(result)
    }
}
