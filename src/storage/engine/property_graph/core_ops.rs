//! Core CRUD Operations
//!
//! Contains vertex and edge CRUD operations for PropertyGraph.

use crate::core::types::{EdgeId, LabelId, Timestamp};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;

use super::super::edge::{EdgeOperationParams, EdgeTraversalParams};
use super::{InsertEdgeParams, PropertyGraph, PropertyGraphUpdateEdgePropertyParams};

use std::sync::atomic::Ordering;

pub fn insert_vertex(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: &str,
    properties: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<u32> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.write().insert_vertex(label, external_id, properties, ts)
}

pub fn get_vertex(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: &str,
    ts: Timestamp,
) -> Option<VertexRecord> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }

    let internal_id = graph
        .cache_manager
        .get_cached_vertex_id(label, external_id)
        .or_else(|| {
            let id = graph.schema_ops.read().get_vertex_internal_id(label, external_id, ts)?;
            graph.cache_manager.cache_vertex_id(label, external_id, id);
            Some(id)
        })?;

    if let Some(cached) = graph.cache_manager.get_cached_vertex(label, internal_id) {
        return Some(VertexRecord {
            internal_id: cached.internal_id,
            vid: cached.internal_id as u64,
            properties: cached.properties,
        });
    }

    let record = {
        let schema = graph.schema_ops.read();
        schema.get_vertex_by_internal_id(label, internal_id, ts)?
    };

    graph.cache_manager.cache_vertex(
        label,
        internal_id,
        external_id.to_string(),
        record.properties.clone(),
    );

    Some(record)
}

pub fn get_vertex_by_internal_id(
    graph: &PropertyGraph,
    label: LabelId,
    internal_id: u32,
    ts: Timestamp,
) -> Option<VertexRecord> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }

    if let Some(cached) = graph.cache_manager.get_cached_vertex(label, internal_id) {
        return Some(VertexRecord {
            internal_id: cached.internal_id,
            vid: cached.internal_id as u64,
            properties: cached.properties,
        });
    }

    let record = {
        let schema = graph.schema_ops.read();
        schema.get_vertex_by_internal_id(label, internal_id, ts)?
    };

    graph.cache_manager.cache_vertex(
        label,
        internal_id,
        String::new(),
        record.properties.clone(),
    );

    Some(record)
}

pub fn delete_vertex(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: &str,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.write().delete_vertex(label, external_id, ts)
}

pub fn update_vertex_property(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: &str,
    property_name: &str,
    value: &Value,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    graph
        .schema_ops
        .write()
        .update_vertex_property(label, external_id, property_name, value, ts)
}

pub fn insert_edge(graph: &PropertyGraph, params: InsertEdgeParams) -> StorageResult<EdgeId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let op_params = EdgeOperationParams {
        edge_label: params.edge_label,
        src_label: params.src_label,
        src_id: params.src_id,
        dst_label: params.dst_label,
        dst_id: params.dst_id,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.write().insert_edge(
        op_params,
        params.properties,
        params.ts,
        &schema.vertex_tables,
    )
}

pub fn get_edge(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    src_id: &str,
    dst_label: LabelId,
    dst_id: &str,
    ts: Timestamp,
) -> Option<EdgeRecord> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }
    let params = EdgeOperationParams {
        edge_label,
        src_label,
        src_id,
        dst_label,
        dst_id,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.read().get_edge(params, ts, &schema.vertex_tables)
}

pub fn delete_edge(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    src_id: &str,
    dst_label: LabelId,
    dst_id: &str,
    ts: Timestamp,
) -> StorageResult<bool> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let params = EdgeOperationParams {
        edge_label,
        src_label,
        src_id,
        dst_label,
        dst_id,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.write().delete_edge(params, ts, &schema.vertex_tables)
}

pub fn update_edge_property(
    graph: &PropertyGraph,
    params: PropertyGraphUpdateEdgePropertyParams,
) -> StorageResult<bool> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let op_params = EdgeOperationParams {
        edge_label: params.edge_label,
        src_label: params.src_label,
        src_id: params.src_id,
        dst_label: params.dst_label,
        dst_id: params.dst_id,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.write().update_edge_property(
        op_params,
        params.prop_name,
        params.value,
        params.ts,
        &schema.vertex_tables,
    )
}

pub fn out_edges(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    dst_label: LabelId,
    src_id: &str,
    ts: Timestamp,
) -> Option<Vec<EdgeRecord>> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }
    let params = EdgeTraversalParams {
        edge_label,
        src_label,
        dst_label,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.read().out_edges(params, src_id, ts, &schema.vertex_tables)
}

pub fn in_edges(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    dst_label: LabelId,
    dst_id: &str,
    ts: Timestamp,
) -> Option<Vec<EdgeRecord>> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }
    let params = EdgeTraversalParams {
        edge_label,
        src_label,
        dst_label,
    };
    let schema = graph.schema_ops.read();
    graph.edge_ops.read().in_edges(params, dst_id, ts, &schema.vertex_tables)
}
