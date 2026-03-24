use std::sync::Arc;

use crate::query::executor::base::{ExecutorConfig, ShortestPathConfig};
use crate::query::executor::data_processing::graph_traversal::algorithms::ShortestPathAlgorithmType;
use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
use crate::query::executor::data_processing::graph_traversal::expand_all::ExpandAllExecutor;
use crate::query::executor::data_processing::graph_traversal::shortest_path::ShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor;
use crate::query::validator::context::ExpressionAnalysisContext;
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
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> ExpandExecutor<S> {
        ExpandExecutor::new(
            id,
            storage,
            edge_direction,
            edge_types,
            max_depth,
            expr_context,
        )
    }

    /// 创建ExpandAllExecutor
    pub fn create_expand_all_executor<S: crate::storage::StorageClient + std::marker::Send>(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        any_edge_type: bool,
        max_depth: Option<usize>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> ExpandAllExecutor<S> {
        ExpandAllExecutor::new(
            id,
            storage,
            edge_direction,
            edge_types,
            any_edge_type,
            max_depth,
            expr_context,
        )
    }

    /// 创建TraverseExecutor
    pub fn create_traverse_executor<S: crate::storage::StorageClient>(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        conditions: Option<String>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> TraverseExecutor<S> {
        TraverseExecutor::new(
            id,
            storage,
            edge_direction,
            edge_types,
            max_depth,
            conditions,
            expr_context,
        )
    }

    /// 创建ShortestPathExecutor
    pub fn create_shortest_path_executor<S: crate::storage::StorageClient>(
        id: i64,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
        config: ShortestPathConfig,
        algorithm: ShortestPathAlgorithmType,
    ) -> ShortestPathExecutor<S> {
        let base_config = ExecutorConfig::new(id, storage, expr_context);
        ShortestPathExecutor::new(base_config, config, algorithm)
    }
}
