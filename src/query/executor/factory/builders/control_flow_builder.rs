//! 控制流执行器构建器
//!
//! 负责创建控制流类型的执行器（Loop, Select, Argument, PassThrough, DataCollect）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::logic::{LoopExecutor, SelectExecutor};
use crate::query::executor::pipeline_executors::{
    ArgumentExecutor, DataCollectExecutor, PassThroughExecutor,
};
use crate::query::planning::plan::core::nodes::{
    ArgumentNode, DataCollectNode, LoopNode, PassThroughNode, SelectNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Create executor function type alias
type CreateExecutorFn<S> = dyn FnMut(
    &crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
    Arc<Mutex<S>>,
    &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError>;

/// 控制流执行器构建器
pub struct ControlFlowBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> ControlFlowBuilder<S> {
    /// 创建新的控制流构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 Loop 执行器
    pub fn build_loop(
        &self,
        node: &LoopNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        create_executor_fn: &mut CreateExecutorFn<S>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let body = node
            .body()
            .as_ref()
            .ok_or_else(|| QueryError::ExecutionError("Loop节点缺少body".to_string()))?;

        let body_executor = create_executor_fn(body, storage.clone(), context)?;

        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone());

        let executor = LoopExecutor::new(
            node.id(),
            storage,
            condition,
            body_executor,
            None,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Loop(executor))
    }

    /// 构建 Select 执行器
    pub fn build_select(
        &self,
        node: &SelectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        create_executor_fn: &mut CreateExecutorFn<S>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone())
            .unwrap_or_else(|| crate::core::Expression::Literal(crate::core::Value::Bool(true)));

        let if_branch = node
            .if_branch()
            .as_ref()
            .ok_or_else(|| QueryError::ExecutionError("Select节点缺少if_branch".to_string()))?;

        let if_executor = create_executor_fn(if_branch, storage.clone(), context)?;

        let else_executor = node
            .else_branch()
            .as_ref()
            .map(|branch| create_executor_fn(branch, storage.clone(), context))
            .transpose()?;

        let executor = SelectExecutor::new(
            node.id(),
            storage,
            condition,
            if_executor,
            else_executor,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Select(executor))
    }

    /// 构建 Argument 执行器
    pub fn build_argument(
        &self,
        node: &ArgumentNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ArgumentExecutor::new(
            node.id(),
            storage,
            node.var(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Argument(executor))
    }

    /// 构建 PassThrough 执行器
    pub fn build_pass_through(
        &self,
        node: &PassThroughNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor =
            PassThroughExecutor::new(node.id(), storage, context.expression_context().clone());
        Ok(ExecutorEnum::PassThrough(executor))
    }

    /// 构建 DataCollect 执行器
    pub fn build_data_collect(
        &self,
        node: &DataCollectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor =
            DataCollectExecutor::new(node.id(), storage, context.expression_context().clone());
        Ok(ExecutorEnum::DataCollect(executor))
    }
}

impl<S: StorageClient + 'static> Default for ControlFlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
