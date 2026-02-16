use std::sync::Arc;

use crate::query::executor::data_processing::graph_traversal::algorithms::ShortestPathAlgorithmType;
use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
use crate::query::executor::data_processing::graph_traversal::expand_all::ExpandAllExecutor;
use crate::query::executor::data_processing::graph_traversal::shortest_path::ShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor;
use parking_lot::Mutex;

/// 图遍历执行器工厂
pub struct GraphTraversalExecutorFactory;

impl GraphTraversalExecutorFactory {
    /// 创建ExpandExecutor
    pub fn create_expand_executor<S: crate::storage::StorageClient>(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> ExpandExecutor<S> {
        ExpandExecutor::new(id, storage, edge_direction, edge_types, max_depth)
    }

    /// 创建ExpandAllExecutor
    pub fn create_expand_all_executor<S: crate::storage::StorageClient + std::marker::Send>(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> ExpandAllExecutor<S> {
        ExpandAllExecutor::new(id, storage, edge_direction, edge_types, max_depth)
    }

    /// 创建TraverseExecutor
    pub fn create_traverse_executor<S: crate::storage::StorageClient>(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        conditions: Option<String>,
    ) -> TraverseExecutor<S> {
        TraverseExecutor::new(
            id,
            storage,
            edge_direction,
            edge_types,
            max_depth,
            conditions,
        )
    }

    /// 创建ShortestPathExecutor
    pub fn create_shortest_path_executor<S: crate::storage::StorageClient>(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vertex_ids: Vec<crate::core::Value>,
        end_vertex_ids: Vec<crate::core::Value>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        algorithm: ShortestPathAlgorithmType,
    ) -> ShortestPathExecutor<S> {
        ShortestPathExecutor::new(
            id,
            storage,
            start_vertex_ids,
            end_vertex_ids,
            edge_direction,
            edge_types,
            max_depth,
            algorithm,
        )
    }
}
