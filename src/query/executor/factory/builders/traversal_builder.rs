//! 图遍历执行器构建器
//!
//! 负责创建图遍历类型的执行器（Expand, ExpandAll, Traverse）

use crate::core::error::QueryError;
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
        // ExpandNode 没有 input_var 方法，使用 output_var 或生成默认值
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = ExpandExecutor::new(
            node.id(),
            storage,
            input_var,
            node.edge_types().to_vec(),
            node.direction().into(),
            node.step_limit().map(|s| s as usize),
            node.col_names().to_vec(),
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
        // ExpandAllNode 没有 input_var 方法，使用 output_var 或生成默认值
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = ExpandAllExecutor::new(
            node.id(),
            storage,
            input_var,
            node.edge_types().to_vec(),
            node.direction().into(),
            node.step_limit().map(|s| s as usize),
            node.col_names().to_vec(),
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
        // TraverseNode 没有 input_var 方法，使用 output_var 或生成默认值
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = TraverseExecutor::new(
            node.id(),
            storage,
            input_var,
            node.edge_types().to_vec(),
            node.direction().into(),
            node.max_steps() as usize,
            node.col_names().to_vec(),
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
