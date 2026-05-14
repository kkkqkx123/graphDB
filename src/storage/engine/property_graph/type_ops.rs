//! Type Operations
//!
//! Contains vertex and edge type creation/deletion operations for PropertyGraph.
//! This module handles low-level type management at the storage engine level.

use crate::core::types::LabelId;
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::{EdgeStrategy, PropertyDef as EdgePropertyDef};
use crate::storage::vertex::PropertyDef as VertexPropertyDef;

use super::super::edge::CreateEdgeTypeParams;
use super::PropertyGraph;

pub fn create_vertex_type(
    graph: &mut PropertyGraph,
    name: &str,
    properties: Vec<VertexPropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.create_vertex_type(name, properties, primary_key)
}

pub fn create_vertex_type_with_id(
    graph: &mut PropertyGraph,
    name: &str,
    label_id: LabelId,
    properties: Vec<VertexPropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph
        .schema_ops
        .create_vertex_type_with_id(name, label_id, properties, primary_key)
}

pub fn create_edge_type(
    graph: &mut PropertyGraph,
    name: &str,
    src_label: LabelId,
    dst_label: LabelId,
    properties: Vec<EdgePropertyDef>,
    oe_strategy: EdgeStrategy,
    ie_strategy: EdgeStrategy,
) -> StorageResult<LabelId> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    let params = CreateEdgeTypeParams {
        name,
        src_label,
        dst_label,
        properties,
        oe_strategy,
        ie_strategy,
    };
    graph
        .edge_ops
        .create_edge_type(params, &graph.schema_ops.vertex_tables)
}

pub fn create_edge_type_with_id(
    graph: &mut PropertyGraph,
    params: CreateEdgeTypeParams,
    label_id: LabelId,
) -> StorageResult<LabelId> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph
        .edge_ops
        .create_edge_type_with_id(params, label_id, &graph.schema_ops.vertex_tables)
}

pub fn drop_vertex_type(graph: &mut PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    let label_id = graph
        .schema_ops
        .vertex_label_names
        .get(name)
        .copied()
        .ok_or_else(|| StorageError::label_not_found(name.to_string()))?;
    graph.schema_ops.drop_vertex_type(name)?;
    graph.edge_ops.drop_edges_for_vertex_label(label_id);
    Ok(())
}

pub fn drop_edge_type(graph: &mut PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph.edge_ops.drop_edge_type(name)
}
