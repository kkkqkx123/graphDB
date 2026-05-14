//! Type Operations
//!
//! Contains vertex and edge type creation/deletion operations for PropertyGraph.
//! This module handles low-level type management at the storage engine level.

use std::sync::atomic::Ordering;

use crate::core::types::LabelId;
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::{EdgeStrategy, PropertyDef as EdgePropertyDef};
use crate::storage::vertex::PropertyDef as VertexPropertyDef;

use super::super::edge::CreateEdgeTypeParams;
use super::PropertyGraph;

pub fn create_vertex_type(
    graph: &PropertyGraph,
    name: &str,
    properties: Vec<VertexPropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.write().create_vertex_type(name, properties, primary_key)
}

pub fn create_vertex_type_with_id(
    graph: &PropertyGraph,
    name: &str,
    label_id: LabelId,
    properties: Vec<VertexPropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph
        .schema_ops
        .write()
        .create_vertex_type_with_id(name, label_id, properties, primary_key)
}

pub fn create_edge_type(
    graph: &PropertyGraph,
    name: &str,
    src_label: LabelId,
    dst_label: LabelId,
    properties: Vec<EdgePropertyDef>,
    oe_strategy: EdgeStrategy,
    ie_strategy: EdgeStrategy,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
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
    let schema = graph.schema_ops.read();
    graph.edge_ops.write().create_edge_type(params, &schema.vertex_tables)
}

pub fn create_edge_type_with_id(
    graph: &PropertyGraph,
    params: CreateEdgeTypeParams,
    label_id: LabelId,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let schema = graph.schema_ops.read();
    graph.edge_ops.write().create_edge_type_with_id(params, label_id, &schema.vertex_tables)
}

pub fn drop_vertex_type(graph: &PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let label_id = {
        let schema = graph.schema_ops.read();
        schema
            .vertex_label_names
            .get(name)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?
    };
    graph.schema_ops.write().drop_vertex_type(name)?;
    graph.edge_ops.write().drop_edges_for_vertex_label(label_id);
    Ok(())
}

pub fn drop_edge_type(graph: &PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph.edge_ops.write().drop_edge_type(name)
}
