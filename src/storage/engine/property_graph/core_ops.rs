//! Core CRUD Operations
//!
//! Contains vertex and edge CRUD operations for PropertyGraph.

use crate::core::types::{EdgeId, LabelId, Timestamp};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;

use super::super::edge::{EdgeOperationParams, EdgeTraversalParams};
use super::{InsertEdgeParams, PropertyGraph, PropertyGraphUpdateEdgePropertyParams};

pub fn insert_vertex(
    graph: &mut PropertyGraph,
    label: LabelId,
    external_id: &str,
    properties: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<u32> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.insert_vertex(label, external_id, properties, ts)
}

pub fn get_vertex(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: &str,
    ts: Timestamp,
) -> Option<VertexRecord> {
    if !graph.is_open {
        return None;
    }

    let internal_id = graph
        .cache_manager
        .get_cached_vertex_id(label, external_id)
        .or_else(|| {
            let id = graph.schema_ops.get_vertex_internal_id(label, external_id, ts)?;
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

    let record = graph
        .schema_ops
        .get_vertex_by_internal_id(label, internal_id, ts)?;

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
    if !graph.is_open {
        return None;
    }

    if let Some(cached) = graph.cache_manager.get_cached_vertex(label, internal_id) {
        return Some(VertexRecord {
            internal_id: cached.internal_id,
            vid: cached.internal_id as u64,
            properties: cached.properties,
        });
    }

    let record = graph
        .schema_ops
        .get_vertex_by_internal_id(label, internal_id, ts)?;

    graph.cache_manager.cache_vertex(
        label,
        internal_id,
        String::new(),
        record.properties.clone(),
    );

    Some(record)
}

pub fn delete_vertex(
    graph: &mut PropertyGraph,
    label: LabelId,
    external_id: &str,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph.schema_ops.delete_vertex(label, external_id, ts)
}

pub fn update_vertex_property(
    graph: &mut PropertyGraph,
    label: LabelId,
    external_id: &str,
    property_name: &str,
    value: &Value,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    graph
        .schema_ops
        .update_vertex_property(label, external_id, property_name, value, ts)
}

pub fn insert_edge(graph: &mut PropertyGraph, params: InsertEdgeParams) -> StorageResult<EdgeId> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    let op_params = EdgeOperationParams {
        edge_label: params.edge_label,
        src_label: params.src_label,
        src_id: params.src_id,
        dst_label: params.dst_label,
        dst_id: params.dst_id,
    };
    graph.edge_ops.insert_edge(
        op_params,
        params.properties,
        params.ts,
        &graph.schema_ops.vertex_tables,
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
    if !graph.is_open {
        return None;
    }
    let params = EdgeOperationParams {
        edge_label,
        src_label,
        src_id,
        dst_label,
        dst_id,
    };
    graph.edge_ops.get_edge(params, ts, &graph.schema_ops.vertex_tables)
}

pub fn delete_edge(
    graph: &mut PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    src_id: &str,
    dst_label: LabelId,
    dst_id: &str,
    ts: Timestamp,
) -> StorageResult<bool> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    let params = EdgeOperationParams {
        edge_label,
        src_label,
        src_id,
        dst_label,
        dst_id,
    };
    graph.edge_ops.delete_edge(params, ts, &graph.schema_ops.vertex_tables)
}

pub fn update_edge_property(
    graph: &mut PropertyGraph,
    params: PropertyGraphUpdateEdgePropertyParams,
) -> StorageResult<bool> {
    if !graph.is_open {
        return Err(StorageError::storage_not_open());
    }
    let op_params = EdgeOperationParams {
        edge_label: params.edge_label,
        src_label: params.src_label,
        src_id: params.src_id,
        dst_label: params.dst_label,
        dst_id: params.dst_id,
    };
    graph.edge_ops.update_edge_property(
        op_params,
        params.prop_name,
        params.value,
        params.ts,
        &graph.schema_ops.vertex_tables,
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
    if !graph.is_open {
        return None;
    }
    let params = EdgeTraversalParams {
        edge_label,
        src_label,
        dst_label,
    };
    graph.edge_ops.out_edges(params, src_id, ts, &graph.schema_ops.vertex_tables)
}

pub fn in_edges(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    dst_label: LabelId,
    dst_id: &str,
    ts: Timestamp,
) -> Option<Vec<EdgeRecord>> {
    if !graph.is_open {
        return None;
    }
    let params = EdgeTraversalParams {
        edge_label,
        src_label,
        dst_label,
    };
    graph.edge_ops.in_edges(params, dst_id, ts, &graph.schema_ops.vertex_tables)
}
