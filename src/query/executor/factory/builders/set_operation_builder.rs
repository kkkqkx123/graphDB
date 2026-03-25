//! Collection Operation Executor Builder
//!
//! Responsible for creating executors for set operation types (Union, Minus, Intersect)

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::set_operations::{
    IntersectExecutor, MinusExecutor, UnionExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::{IntersectNode, MinusNode, UnionNode};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Set Operation Executor Builder
pub struct SetOperationBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> SetOperationBuilder<S> {
    /// Create a new collection operation builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Building a Union executor
    pub fn build_union(
        &self,
        node: &UnionNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The UnionExecutor requires left_input_var and right_input_var.
        // Use `output_var` or generate a default value.
        let left_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = format!("right_{}", node.id());

        let executor = UnionExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Union(executor))
    }

    /// Building the Minus executor
    pub fn build_minus(
        &self,
        node: &MinusNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The `MinusExecutor` requires `left_input_var` and `right_input_var`.
        let left_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = format!("right_{}", node.id());

        let executor = MinusExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Minus(executor))
    }

    /// Constructing the Intersect executor
    pub fn build_intersect(
        &self,
        node: &IntersectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The IntersectExecutor requires the left_input_var and right_input_var parameters.
        let left_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = format!("right_{}", node.id());

        let executor = IntersectExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Intersect(executor))
    }
}

impl<S: StorageClient + 'static> Default for SetOperationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
