//! 数据访问执行器构建器
//!
//! 负责创建数据访问类型的执行器，处理顶点和边相关的执行器

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_access::{
    GetNeighborsExecutor, GetVerticesExecutor, ScanEdgesExecutor, IndexScanExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::parsers::{parse_vertex_ids, parse_edge_direction};
use crate::query::planner::plan::core::nodes::{
    ScanVerticesNode, ScanEdgesNode, GetVerticesNode, GetNeighborsNode,
    EdgeIndexScanNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据访问执行器构建器
pub struct DataAccessBuilder<S: StorageClient + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> DataAccessBuilder<S> {
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
            node.id(),
            storage,
            node.space_id(),
            node.edge_type()
                .chars()
                .fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
            node.index_name()
                .chars()
                .fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)),
            node.scan_type().as_str(),
            node.scan_limits().to_vec(),
            node.filter().and_then(|f| f.get_expression()),
            node.return_columns().to_vec(),
            node.limit().map(|l| l as usize),
            true,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::IndexScan(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataAccessBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
