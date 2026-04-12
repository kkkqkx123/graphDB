//! Data Access Executor Builder
//!
//! Responsible for creating executors for different data access types (ScanVertices, ScanEdges, GetVertices, GetNeighbors, IndexScan, GetEdges)

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutorConfig, IndexScanConfig};
use crate::query::executor::data_access::{
    GetEdgesExecutor, GetNeighborsExecutor, GetVerticesExecutor, GetVerticesParams,
    IndexScanExecutor, ScanEdgesExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::param_parsing::{parse_edge_direction, parse_vertex_ids};
use crate::query::planning::plan::core::nodes::access::IndexScanNode;
use crate::query::planning::plan::core::nodes::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Data Access Executor Builder
pub struct DataAccessBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataAccessBuilder<S> {
    /// Create a new data access builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Building the ScanVertices executor
    pub fn build_scan_vertices(
        node: &ScanVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let params = GetVerticesParams {
            space_name: node.space_name().to_string(),
            vertex_ids: None,
            tag_filter: None,
            vertex_filter: node.vertex_filter().and_then(|f| f.get_expression()),
            limit: node.limit().map(|l| l as usize),
        };
        let executor = GetVerticesExecutor::new(
            node.id(),
            storage,
            params,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }

    /// Building the ScanEdges executor
    pub fn build_scan_edges(
        node: &ScanEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ScanEdgesExecutor::new(
            node.id(),
            storage,
            node.edge_type(),
            node.filter().and_then(|f| f.get_expression()),
            node.limit().map(|l| l as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ScanEdges(executor))
    }

    /// Constructing the GetVertices executor
    pub fn build_get_vertices(
        node: &GetVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let vertex_ids = parse_vertex_ids(node.src_vids());
        let params = GetVerticesParams {
            space_name: node.space_name().to_string(),
            vertex_ids: if vertex_ids.is_empty() {
                None
            } else {
                Some(vertex_ids)
            },
            tag_filter: None,
            vertex_filter: node.expression().and_then(|e| e.get_expression()),
            limit: node.limit().map(|l| l as usize),
        };
        let executor = GetVerticesExecutor::new(
            node.id(),
            storage,
            params,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }

    /// Constructing the GetNeighbors executor
    pub fn build_get_neighbors(
        node: &GetNeighborsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let vertex_ids = parse_vertex_ids(node.src_vids());
        let edge_direction = parse_edge_direction(node.direction());
        let edge_types = if node.edge_types().is_empty() {
            None
        } else {
            Some(node.edge_types().to_vec())
        };
        let executor = GetNeighborsExecutor::new(
            node.id(),
            storage,
            vertex_ids,
            edge_direction,
            edge_types,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetNeighbors(executor))
    }

    /// Building the EdgeIndexScan executor
    pub fn build_edge_index_scan(
        node: &EdgeIndexScanNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = IndexScanExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            IndexScanConfig {
                space_id: node.space_id(),
                tag_id: node
                    .edge_type()
                    .chars()
                    .fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
                index_id: node
                    .index_name()
                    .chars()
                    .fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
                index_name: node.index_name().to_string(),
                schema_name: node.schema_name().to_string(),
                scan_type: node.scan_type().as_str().to_string(),
                scan_limits: node.scan_limits().to_vec(),
                filter: node.filter().and_then(|f| f.get_expression()),
                return_columns: node.return_columns().to_vec(),
                limit: node.limit().map(|l| l as usize),
                is_edge: true,
            },
        );
        Ok(ExecutorEnum::IndexScan(executor))
    }

    /// Constructing the GetEdges executor
    pub fn build_get_edges(
        node: &GetEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let edge_type = if node.edge_type().is_empty() {
            None
        } else {
            Some(node.edge_type().to_string())
        };

        let executor = GetEdgesExecutor::new(
            node.id(),
            storage,
            edge_type,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetEdges(executor))
    }

    /// Building the IndexScan executor (for scanning tag indexes)
    pub fn build_index_scan(
        node: &IndexScanNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = IndexScanExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            IndexScanConfig {
                space_id: node.space_id(),
                tag_id: node.tag_id(),
                index_id: node.index_id(),
                index_name: node.index_name().to_string(),
                schema_name: node.schema_name().to_string(),
                scan_type: node.scan_type().as_str().to_string(),
                scan_limits: node.scan_limits().to_vec(),
                filter: node.filter().and_then(|f| f.get_expression()),
                return_columns: node.return_columns().to_vec(),
                limit: node.limit().map(|l| l as usize),
                is_edge: false,
            },
        );
        Ok(ExecutorEnum::IndexScan(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataAccessBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
