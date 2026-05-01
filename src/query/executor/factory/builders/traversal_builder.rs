//! Graph Traversal Executor Builder
//!
//! Responsible for creating executors of graph traversal types (Expand, ExpandAll, Traverse, AllPaths, ShortestPath, MultiShortestPath)

use crate::core::error::QueryError;
use crate::core::types::EdgeDirection;
use crate::query::executor::base::ExecutorEnum;
use crate::query::executor::base::{
    AllPathsConfig, ExecutionContext, ExecutorConfig, MultiShortestPathConfig, ShortestPathConfig,
};
use crate::query::executor::graph_operations::graph_traversal::algorithms::bfs_shortest::BfsShortestPathConfig;
use crate::query::executor::graph_operations::graph_traversal::algorithms::MultiShortestPathExecutor;
use crate::query::executor::graph_operations::graph_traversal::{
    AllPathsExecutor, ExpandAllExecutor, ExpandExecutor, ShortestPathExecutor, TraverseExecutor,
};
use crate::query::planning::plan::core::nodes::base::plan_node_traits::{
    MultipleInputNode, PlanNode,
};
use crate::query::planning::plan::core::nodes::traversal::{
    AllPathsNode, BFSShortestNode, BiExpandNode, BiTraverseNode, MultiShortestPathNode,
    ShortestPathNode,
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
        node: &ExpandAllNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Parameters of ExpandAllExecutor::new: id, storage, edge_direction, edge_types, any_edge_type, max_depth, expr_context
        // ExpandAllNode 的 direction() 返回 &str，需要转换为 EdgeDirection
        let edge_direction = EdgeDirection::from(node.direction());
        
        // Get space name from storage using space_id
        let space_name = {
            let storage_guard = storage.lock();
            match storage_guard.get_space_by_id(node.space_id()) {
                Ok(Some(space_info)) => space_info.space_name,
                _ => "default".to_string(), // Fallback to default if space not found
            }
        };
        
        let mut executor = ExpandAllExecutor::with_context(
            node.id(),
            storage,
            edge_direction,
            Some(node.edge_types().to_vec()),
            false, // any_edge_type
            node.step_limit().map(|s| s as usize),
            context.clone(),
            node.space_id(),
            space_name,
        )
        .with_src_vids(node.src_vids().to_vec())
        .with_include_empty_paths(node.include_empty_paths())
        .with_filter(node.filter().cloned());

        // If input_var is set, use it to get input from ExecutionContext
        if let Some(input_var) = node.get_input_var() {
            executor = executor.with_input_var(input_var.to_string());
        } else {
            // If there are input nodes, get the input variable name from the first input node
            let inputs = node.inputs();
            if !inputs.is_empty() {
                if let Some(input_var) = inputs[0].output_var() {
                    executor = executor.with_input_var(input_var.to_string());
                }
            }
        }

        // Set column names from the node configuration
        // This allows custom dst column names for variable binding in multi-hop queries
        if !node.col_names().is_empty() {
            executor = executor.with_col_names(node.col_names().to_vec());
        }

        Ok(ExecutorEnum::ExpandAll(executor))
    }

    /// Building the Traverse executor
    pub fn build_traverse(
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
        node: &AllPathsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = AllPathsExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            AllPathsConfig {
                left_start_ids: node.start_vertex_ids().to_vec(),
                right_start_ids: node.end_vertex_ids().to_vec(),
                max_hops: node.max_hop(),
                edge_types: Some(node.edge_types().to_vec()),
                direction: EdgeDirection::Out,
            },
        );
        Ok(ExecutorEnum::AllPaths(executor))
    }

    /// Building the ShortestPath executor
    pub fn build_shortest_path(
        node: &ShortestPathNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::graph_operations::graph_traversal::algorithms::ShortestPathAlgorithmType;

        let start_vertex_ids = node.start_vertex_ids().to_vec();
        let end_vertex_ids = node.end_vertex_ids().to_vec();

        let mut executor = ShortestPathExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            ShortestPathConfig {
                start_vertex_ids,
                direction: EdgeDirection::Out,
                edge_types: Some(node.edge_types().to_vec()),
            },
            ShortestPathAlgorithmType::BFS,
        );
        executor.set_end_vertex_ids(end_vertex_ids);
        executor.max_depth = Some(node.max_step());
        Ok(ExecutorEnum::ShortestPath(executor))
    }

    /// Building the BFSShortest executor
    pub fn build_bfs_shortest(
        node: &BFSShortestNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::Value;
        use crate::query::executor::graph_operations::graph_traversal::algorithms::BFSShortestExecutor;

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

    /// Constructing the BiExpand executor
    /// Bidirectional expand from two input sources meeting at common vertices
    pub fn build_bi_expand(
        node: &BiExpandNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ExpandExecutor::new(
            node.id(),
            storage,
            node.left_direction(),
            Some(node.edge_types().to_vec()),
            Some(node.max_hops()),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::BiExpand(executor))
    }

    /// Constructing the BiTraverse executor
    /// Bidirectional traverse from two input sources meeting at common vertices
    pub fn build_bi_traverse(
        node: &BiTraverseNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ExpandExecutor::new(
            node.id(),
            storage,
            node.left_direction(),
            Some(node.edge_types().to_vec()),
            Some(node.max_hops()),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::BiTraverse(executor))
    }
}

impl<S: StorageClient + 'static> Default for TraversalBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
