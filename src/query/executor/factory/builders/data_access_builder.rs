//! 数据访问执行器构建器
//!
//! 负责创建数据访问类型的执行器（ScanVertices, ScanEdges, GetVertices, GetNeighbors, IndexScan, GetEdges）

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, ExecutorConfig, IndexScanConfig};
use crate::query::executor::data_access::{
    GetEdgesExecutor, GetNeighborsExecutor, GetVerticesExecutor, IndexScanExecutor,
    ScanEdgesExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::parsers::{parse_edge_direction, parse_vertex_ids};
use crate::query::planning::plan::core::nodes::access::IndexScanNode;
use crate::query::planning::plan::core::nodes::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据访问执行器构建器
pub struct DataAccessBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataAccessBuilder<S> {
    /// 创建新的数据访问构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 ScanVertices 执行器
    pub fn build_scan_vertices(
        &self,
        node: &ScanVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = GetVerticesExecutor::new(
            node.id(),
            storage,
            None,
            None,
            node.vertex_filter().and_then(|f| f.get_expression()),
            node.limit().map(|l| l as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }

    /// 构建 ScanEdges 执行器
    pub fn build_scan_edges(
        &self,
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

    /// 构建 GetVertices 执行器
    pub fn build_get_vertices(
        &self,
        node: &GetVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let vertex_ids = parse_vertex_ids(node.src_vids());
        let executor = GetVerticesExecutor::new(
            node.id(),
            storage,
            if vertex_ids.is_empty() {
                None
            } else {
                Some(vertex_ids)
            },
            None,
            node.expression().and_then(|e| e.get_expression()),
            node.limit().map(|l| l as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }

    /// 构建 GetNeighbors 执行器
    pub fn build_get_neighbors(
        &self,
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

    /// 构建 EdgeIndexScan 执行器
    pub fn build_edge_index_scan(
        &self,
        node: &EdgeIndexScanNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = IndexScanExecutor::new(
            ExecutorConfig::new(node.id(), storage, context.expression_context().clone()),
            IndexScanConfig {
                space_id: node.space_id(),
                tag_id: node.edge_type()
                    .chars()
                    .fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
                index_id: node.index_name()
                    .chars()
                    .fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
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

    /// 构建 GetEdges 执行器
    pub fn build_get_edges(
        &self,
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

    /// 构建 IndexScan 执行器（用于标签索引扫描）
    pub fn build_index_scan(
        &self,
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
                scan_type: node.scan_type().as_str().to_string(),
                scan_limits: node.scan_limits().to_vec(),
                filter: node.filter().and_then(|f| f.get_expression()),
                return_columns: node.return_columns().to_vec(),
                limit: node.limit().map(|l| l as usize),
                is_edge: false, // is_edge = false，这是标签索引扫描
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
