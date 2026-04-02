//! Plan Executor
//!
//! Responsible for executing the execution plan and managing the lifecycle of the executor tree.

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutionResult, Executor, InputExecutor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::object_pool::ThreadSafeExecutorPool;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::planning::plan::PlanNodeEnum;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Find the input_var from an ExpandAllNode in the plan tree
fn find_expand_all_input_var(node: &PlanNodeEnum) -> Option<String> {
    if let Some(expand_all) = node.as_expand_all() {
        expand_all.get_input_var().map(|v| v.to_string())
    } else {
        // Recursively check all children
        for child in node.children() {
            if let Some(var) = find_expand_all_input_var(child) {
                return Some(var);
            }
        }
        None
    }
}

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
    /// via `set_input`. For `BinaryInputNode` plan nodes (e.g. Join), both children are built
    /// and executed to store their results in the execution context.
    /// For `ZeroInputNode` plan nodes (leaf nodes), only the executor itself is created.
    fn build_executor_chain(
        &mut self,
        plan_node: &crate::query::planning::plan::PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<crate::query::executor::ExecutorEnum<S>, QueryError> {
        let mut executor = self
            .factory
            .create_executor(plan_node, storage.clone(), context)?;

        let children = plan_node.children();

        match children.len() {
            0 => {
                // ZeroInputNode: no child nodes to process
            }
            1 => {
                // SingleInputNode: build child and set as input
                let child_executor =
                    self.build_executor_chain(children[0], storage.clone(), context)?;
                executor.set_input(child_executor);
            }
            2 => {
                // BinaryInputNode (e.g., Join): build and execute both children
                let mut left_executor =
                    self.build_executor_chain(children[0], storage.clone(), context)?;
                let left_result = left_executor.execute().map_err(|e| {
                    QueryError::ExecutionError(format!("Left child execution failed: {}", e))
                })?;

                // Get left variable name from left child's output_var
                // This must match the variable name used by the join executor (from extract_join_vars)
                let left_var = children[0]
                    .output_var()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("left_{}", plan_node.id()));
                context.set_result(left_var.clone(), left_result.clone());

                // If right child (or its descendants) is ExpandAllNode with input_var,
                // also store the result under that variable name
                // This allows ExpandAllExecutor to find the input using its input_var
                if let Some(input_var) = find_expand_all_input_var(children[1]) {
                    if input_var != left_var {
                        context.set_result(input_var, left_result);
                    }
                }

                let mut right_executor =
                    self.build_executor_chain(children[1], storage.clone(), context)?;
                let right_result = right_executor.execute().map_err(|e| {
                    QueryError::ExecutionError(format!("Right child execution failed: {}", e))
                })?;

                // Get right variable name from node's output_var or use default
                let right_var = children[1]
                    .output_var()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("right_{}", plan_node.id()));
                context.set_result(right_var, right_result);
            }
            _ => {
                // MultipleInputNode (e.g., ExpandAllNode with multiple inputs):
                // Execute all children and store their results in ExecutionContext
                // The executor will use input_var to find the appropriate input
                for (i, child) in children.iter().enumerate() {
                    let mut child_executor =
                        self.build_executor_chain(child, storage.clone(), context)?;
                    let child_result = child_executor.execute().map_err(|e| {
                        QueryError::ExecutionError(format!("Child {} execution failed: {}", i, e))
                    })?;

                    // Store the result in ExecutionContext using the child's output_var
                    let child_var = child
                        .output_var()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("child_{}_{}", plan_node.id(), i));
                    context.set_result(child_var, child_result);
                }

                // If the plan node has an input_var, also store the first child's result under that name
                // This allows the executor to find the input using input_var
                if let Some(input_var) = find_expand_all_input_var(plan_node) {
                    if let Some(first_child) = children.first() {
                        let first_var = first_child
                            .output_var()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| format!("child_{}_0", plan_node.id()));
                        if let Some(result) = context.get_result(&first_var) {
                            context.set_result(input_var, result);
                        }
                    }
                }
            }
        }

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
