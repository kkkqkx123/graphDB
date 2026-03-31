//! Plan Executor
//!
//! Responsible for executing the execution plan and managing the lifecycle of the executor tree.

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutionResult, Executor, InputExecutor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::object_pool::ThreadSafeExecutorPool;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
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

    /// Recursively build an executor chain from a plan tree node.
    ///
    /// For `SingleInputNode` plan nodes (e.g. Project, Filter), this creates the executor
    /// for the node itself, then recursively builds its child executor and connects it
    /// via `set_input`. For `BinaryInputNode` plan nodes (e.g. Join), both children are built.
    /// For `ZeroInputNode` plan nodes (leaf nodes), only the executor itself is created.
    fn build_executor_chain(
        &mut self,
        plan_node: &crate::query::planning::plan::PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<crate::query::executor::ExecutorEnum<S>, QueryError> {
        eprintln!(
            "[build_executor_chain] plan_node type: {}",
            plan_node.name()
        );

        let mut executor = self
            .factory
            .create_executor(plan_node, storage.clone(), context)?;

        let children = plan_node.children();
        eprintln!("[build_executor_chain] children count: {}", children.len());

        if !children.is_empty() {
            eprintln!("[build_executor_chain] building child executor...");
            let child_executor =
                self.build_executor_chain(children[0], storage.clone(), context)?;
            eprintln!("[build_executor_chain] setting input...");
            executor.set_input(child_executor);
            eprintln!("[build_executor_chain] input set");
        }

        eprintln!("[build_executor_chain] returning executor");
        Ok(executor)
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

        let executor_type = root_node.name();
        let is_stateful_executor = matches!(
            executor_type,
            "CreateSpace"
                | "DropSpace"
                | "DescSpace"
                | "CreateTag"
                | "AlterTag"
                | "DropTag"
                | "DescTag"
                | "CreateEdge"
                | "AlterEdge"
                | "DropEdge"
                | "DescEdge"
                | "CreateTagIndex"
                | "DropTagIndex"
                | "DescTagIndex"
                | "RebuildTagIndex"
                | "CreateEdgeIndex"
                | "DropEdgeIndex"
                | "DescEdgeIndex"
                | "RebuildEdgeIndex"
                | "CreateUser"
                | "AlterUser"
                | "DropUser"
                | "GrantRole"
                | "RevokeRole"
                | "ChangePassword"
                | "ShowSpaces"
                | "ShowTags"
                | "ShowEdges"
                | "ShowStats"
                | "ShowTagIndexes"
                | "ShowEdgeIndexes"
                | "SwitchSpace"
                | "ClearSpace"
                | "InsertVertices"
                | "InsertEdges"
                | "DeleteVertices"
                | "DeleteEdges"
                | "Remove"
                | "Update"
                | "UpdateVertices"
                | "UpdateEdges"
                | "Project"
                | "Filter"
                | "Limit"
                | "Sort"
                | "TopN"
                | "Aggregate"
                | "Dedup"
                | "Sample"
                | "GetVertices"
                | "GetEdges"
                | "GetNeighbors"
                | "ScanVertices"
                | "ScanEdges"
                | "Expand"
                | "ExpandAll"
                | "Traverse"
                | "ShortestPath"
                | "AllPaths"
                | "BFSShortest"
                | "MultiShortestPath"
                | "Materialize"
                | "Loop"
                | "Select"
                | "Union"
                | "Minus"
                | "Intersect"
                | "InnerJoin"
                | "LeftJoin"
                | "CrossJoin"
                | "DataCollect"
                | "Unwind"
                | "Assign"
                | "RollupApply"
                | "PatternApply"
                | "AppendVertices"
                | "IndexScan"
                | "EdgeIndexScan"
                | "HashInnerJoin"
                | "HashLeftJoin"
                | "FullOuterJoin"
        );

        let mut executor = if is_stateful_executor || self.object_pool.is_none() {
            self.build_executor_chain(root_node, storage, &execution_context)?
        } else if let Some(pool) = &self.object_pool {
            if let Some(pooled_executor) = pool.acquire(executor_type) {
                log::debug!("从对象池获取执行器: {}", executor_type);
                pooled_executor
            } else {
                log::debug!("对象池未命中，创建新执行器: {}", executor_type);
                self.build_executor_chain(root_node, storage, &execution_context)?
            }
        } else {
            self.build_executor_chain(root_node, storage, &execution_context)?
        };

        let result = executor
            .execute()
            .map_err(|e| QueryError::ExecutionError(format!("Executor execution failed: {}", e)))?;

        eprintln!(
            "[execute_plan] Executor executed, result type: {:?}",
            std::mem::discriminant(&result)
        );

        if let Some(pool) = &self.object_pool {
            if !is_stateful_executor {
                pool.release(executor_type, executor);
                log::debug!("执行器已释放回对象池: {}", executor_type);
            } else {
                log::debug!("Stateful执行器不释放到对象池: {}", executor_type);
            }
        }

        Ok(result)
    }
}
