//! 图遍历执行器构建器
//!
//! 负责创建图遍历类型的执行器（Expand, ExpandAll, Traverse, AllPaths, ShortestPath, MultiShortestPath）

use crate::core::error::QueryError;
use crate::core::types::EdgeDirection;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::graph_traversal::algorithms::MultiShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::{
    AllPathsExecutor, ExpandAllExecutor, ExpandExecutor, ShortestPathExecutor, TraverseExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::traversal::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
use crate::query::planner::plan::core::nodes::{ExpandAllNode, ExpandNode, TraverseNode};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 图遍历执行器构建器
pub struct TraversalBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> TraversalBuilder<S> {
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

    /// 构建 AllPaths 执行器
    pub fn build_all_paths(
        &self,
        node: &AllPathsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // AllPathsExecutor::new 参数: id, storage, left_start_ids, right_start_ids, edge_direction, edge_types, max_steps, expr_context
        let executor = AllPathsExecutor::new(
            node.id(),
            storage,
            Vec::new(), // left_start_ids - 需要从输入获取
            Vec::new(), // right_start_ids - 需要从输入获取
            EdgeDirection::Out,
            Some(node.edge_types().to_vec()),
            node.max_hop(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AllPaths(executor))
    }

    /// 构建 ShortestPath 执行器
    pub fn build_shortest_path(
        &self,
        node: &ShortestPathNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_processing::graph_traversal::algorithms::ShortestPathAlgorithmType;

        // 从输入获取起点和终点ID
        let start_vertex_ids = Vec::new();
        let end_vertex_ids = Vec::new();

        let executor = ShortestPathExecutor::new(
            node.id(),
            storage,
            start_vertex_ids,
            end_vertex_ids,
            EdgeDirection::Out, // 默认向外扩展
            Some(node.edge_types().to_vec()),
            Some(node.max_step()),
            ShortestPathAlgorithmType::BFS,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShortestPath(executor))
    }

    /// 构建 BFSShortest 执行器
    pub fn build_bfs_shortest(
        &self,
        node: &BFSShortestNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::Value;
        use crate::query::executor::data_processing::graph_traversal::algorithms::BFSShortestExecutor;

        // BFSShortestExecutor::new 参数: id, storage, steps, edge_types, with_cycle, max_depth, single_shortest, limit, start_vertex, end_vertex, expr_context
        let executor = BFSShortestExecutor::new(
            node.id(),
            storage,
            node.steps(),
            node.edge_types().to_vec(),
            node.with_cycle(),
            Some(node.steps()),
            false,                                    // single_shortest
            usize::MAX,                               // limit
            Value::Null(crate::core::NullType::Null), // start_vertex - 需要从输入获取
            Value::Null(crate::core::NullType::Null), // end_vertex - 需要从输入获取
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::BFSShortest(executor))
    }

    /// 构建 MultiShortestPath 执行器
    pub fn build_multi_shortest_path(
        &self,
        node: &MultiShortestPathNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 从输入获取起点和终点ID
        let start_vids = Vec::new();
        let end_vids = Vec::new();

        let executor = MultiShortestPathExecutor::new(
            node.id(),
            storage,
            start_vids,
            end_vids,
            EdgeDirection::Out,
            None, // edge_types
            node.steps(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::MultiShortestPath(executor))
    }
}

impl<S: StorageClient + 'static> Default for TraversalBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
