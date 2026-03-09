//! 图遍历执行器构建器
//!
//! 负责创建图遍历类型的执行器（Expand, ExpandAll, Traverse）

use crate::core::error::QueryError;
use crate::core::types::EdgeDirection;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::graph_traversal::{
    ExpandAllExecutor, ExpandExecutor, TraverseExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::{
    ExpandAllNode, ExpandNode, TraverseNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 图遍历执行器构建器
pub struct TraversalBuilder<S: StorageClient + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> TraversalBuilder<S> {
    /// 创建新的图遍历构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 Expand 执行器
    pub fn build_expand(
        &self,
        node: &ExpandNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ExpandExecutor::new 参数: id, storage, edge_direction, edge_types, max_depth, expr_context
        // direction() 返回 EdgeDirection 值，直接传递即可
        let executor = ExpandExecutor::new(
            node.id(),
            storage,
            node.direction(),
            Some(node.edge_types().to_vec()),
            node.step_limit().map(|s| s as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Expand(executor))
    }

    /// 构建 ExpandAll 执行器
    pub fn build_expand_all(
        &self,
        node: &ExpandAllNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ExpandAllExecutor::new 参数: id, storage, edge_direction, edge_types, any_edge_type, max_depth, expr_context
        // ExpandAllNode 的 direction() 返回 &str，需要转换为 EdgeDirection
        let edge_direction = EdgeDirection::from(node.direction());
        let executor = ExpandAllExecutor::new(
            node.id(),
            storage,
            edge_direction,
            Some(node.edge_types().to_vec()),
            false, // any_edge_type
            node.step_limit().map(|s| s as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ExpandAll(executor))
    }

    /// 构建 Traverse 执行器
    pub fn build_traverse(
        &self,
        node: &TraverseNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // TraverseExecutor::new 参数: id, storage, edge_direction, edge_types, max_depth, conditions, expr_context
        let executor = TraverseExecutor::new(
            node.id(),
            storage,
            node.direction(),
            Some(node.edge_types().to_vec()),
            Some(node.max_steps() as usize),
            None, // conditions
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Traverse(executor))
    }
}

impl<S: StorageClient + 'static> Default for TraversalBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
