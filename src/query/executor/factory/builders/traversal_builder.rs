//! Graph Traversal Executor Builder
//!
//! Responsible for creating executors of graph traversal types (Expand, ExpandAll, Traverse, AllPaths, ShortestPath, MultiShortestPath)

use crate::core::error::QueryError;
use crate::core::types::EdgeDirection;
use crate::query::executor::base::{
    AllPathsConfig, ExecutionContext, ExecutorConfig, MultiShortestPathConfig, ShortestPathConfig,
};
use crate::query::executor::data_processing::graph_traversal::algorithms::bfs_shortest::BfsShortestPathConfig;
use crate::query::executor::data_processing::graph_traversal::algorithms::MultiShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::{
    AllPathsExecutor, ExpandAllExecutor, ExpandExecutor, ShortestPathExecutor, TraverseExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::traversal::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
use crate::query::planning::plan::core::nodes::{ExpandAllNode, ExpandNode, TraverseNode};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Graph Traversal Executor Builder
pub struct TraversalBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> TraversalBuilder<S> {
    /// Create a new graph traversal builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Constructing the Expand executor
    pub fn build_expand(
        &self,
        node: &ExpandNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Parameters of ExpandExecutor::new: id, storage, edge_direction, edge_types, max_depth, expr_context
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

    /// Building the ExpandAll executor
    pub fn build_expand_all(
        &self,
        node: &ExpandAllNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Parameters of ExpandAllExecutor::new: id, storage, edge_direction, edge_types, any_edge_type, max_depth, expr_context
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

    /// Building the Traverse executor
    pub fn build_traverse(
        &self,
        node: &TraverseNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // TraverseExecutor::new parameters: id, storage, edge_direction, edge_types, max_depth, conditions, expr_context
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

    /// Building the AllPaths executor
    pub fn build_all_paths(
        &self,
        node: &AllPathsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = AllPathsExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            AllPathsConfig {
                left_start_ids: Vec::new(),  // Please provide the text you would like to have translated.
                right_start_ids: Vec::new(), // Please provide the text you would like to have translated.
                max_hops: node.max_hop(),
                edge_types: Some(node.edge_types().to_vec()),
                direction: EdgeDirection::Out,
            },
        );
        Ok(ExecutorEnum::AllPaths(executor))
    }

    /// Building the ShortestPath executor
    pub fn build_shortest_path(
        &self,
        node: &ShortestPathNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_processing::graph_traversal::algorithms::ShortestPathAlgorithmType;

        //  Obtain the start and end ID from the input.
        let start_vertex_ids: Vec<crate::core::Value> = Vec::new();
        let _end_vertex_ids: Vec<crate::core::Value> = Vec::new();

        let executor = ShortestPathExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            ShortestPathConfig {
                start_vertex_ids,
                direction: EdgeDirection::Out,
                edge_types: Some(node.edge_types().to_vec()),
            },
            ShortestPathAlgorithmType::BFS,
        );
        Ok(ExecutorEnum::ShortestPath(executor))
    }

    /// Building the BFSShortest executor
    pub fn build_bfs_shortest(
        &self,
        node: &BFSShortestNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::Value;
        use crate::query::executor::data_processing::graph_traversal::algorithms::BFSShortestExecutor;

        // BFSShortestExecutor::new parameters: id, storage, steps, edge_types, with_cycle, max_depth, single_shortest, limit, start_vertex, end_vertex, expr_context
        let executor = BFSShortestExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            BfsShortestPathConfig {
                steps: node.steps(),
                edge_types: node.edge_types().to_vec(),
                with_cycle: node.with_cycle(),
                max_depth: Some(node.steps()),
                single_shortest: false,
                limit: usize::MAX,
                start_vertex: Value::Null(crate::core::NullType::Null),
                end_vertex: Value::Null(crate::core::NullType::Null),
            },
        );
        Ok(ExecutorEnum::BFSShortest(executor))
    }

    /// Constructing the MultiShortestPath executor
    pub fn build_multi_shortest_path(
        &self,
        node: &MultiShortestPathNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        //  Obtain the starting and ending point IDs from the input.
        let start_vids: Vec<crate::core::Value> = Vec::new();
        let _end_vids: Vec<crate::core::Value> = Vec::new();

        let executor = MultiShortestPathExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            MultiShortestPathConfig {
                start_vids,
                direction: EdgeDirection::Out,
                edge_types: None,
                max_steps: node.steps(),
            },
        );
        Ok(ExecutorEnum::MultiShortestPath(executor))
    }
}

impl<S: StorageClient + 'static> Default for TraversalBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
