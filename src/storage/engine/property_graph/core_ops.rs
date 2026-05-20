//! Core CRUD Operations
//!
//! Contains vertex and edge CRUD operations for PropertyGraph.

use crate::core::types::{LabelId, Timestamp, VertexId};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::EdgeRecord;
use crate::storage::storage_types::EdgeOffset;
use crate::storage::vertex::VertexRecord;

use super::{InsertEdgeParams, InsertEdgeParamsByI64, PropertyGraph, PropertyGraphUpdateEdgePropertyParams};

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
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    let internal_id = table.insert(external_id, properties, ts)?;

    graph.cache_manager.cache_vertex_id(label, external_id, internal_id, ts);

    Ok(internal_id)
}

pub fn insert_vertex_by_i64(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: i64,
    properties: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<u32> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    let internal_id = table.insert_by_i64(external_id, properties, ts)?;

    graph.cache_manager.cache_vertex_id(label, &external_id.to_string(), internal_id, ts);

    Ok(internal_id)
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
        .get_cached_vertex_id(label, external_id, ts)
        .or_else(|| {
            let id = {
                let vertex_tables = graph.data_store.vertex_tables.read();
                vertex_tables.get(&label)?.get_internal_id(external_id, ts)
            };
            if let Some(id) = id {
                graph.cache_manager.cache_vertex_id(label, external_id, id, ts);
            }
            id
        })?;

    if let Some(cached) = graph.cache_manager.get_cached_vertex(label, internal_id) {
        return Some(VertexRecord {
            internal_id: cached.internal_id,
            vid: VertexId::from_u64(cached.internal_id as u64),
            properties: cached.properties,
        });
    }

    let record = {
        let vertex_tables = graph.data_store.vertex_tables.read();
        vertex_tables.get(&label)?.get_by_internal_id(internal_id, ts)?
    };

    graph.cache_manager.cache_vertex(
        label,
        internal_id,
        external_id.to_string(),
        record.properties.clone(),
    );

    Some(record)
}

pub fn get_vertex_by_i64(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: i64,
    ts: Timestamp,
) -> Option<VertexRecord> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }

    let external_id_str = external_id.to_string();
    let internal_id = graph
        .cache_manager
        .get_cached_vertex_id(label, &external_id_str, ts)
        .or_else(|| {
            let id = {
                let vertex_tables = graph.data_store.vertex_tables.read();
                vertex_tables.get(&label)?.get_internal_id_by_i64(external_id, ts)
            };
            if let Some(id) = id {
                graph.cache_manager.cache_vertex_id(label, &external_id_str, id, ts);
            }
            id
        })?;

    if let Some(cached) = graph.cache_manager.get_cached_vertex(label, internal_id) {
        return Some(VertexRecord {
            internal_id: cached.internal_id,
            vid: VertexId::from_int64(external_id),
            properties: cached.properties,
        });
    }

    let record = {
        let vertex_tables = graph.data_store.vertex_tables.read();
        vertex_tables.get(&label)?.get_by_internal_id(internal_id, ts)?
    };

    graph.cache_manager.cache_vertex(
        label,
        internal_id,
        external_id_str,
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
            vid: VertexId::from_u64(cached.internal_id as u64),
            properties: cached.properties,
        });
    }

    let record = {
        let vertex_tables = graph.data_store.vertex_tables.read();
        vertex_tables.get(&label)?.get_by_internal_id(internal_id, ts)?
    };

    // Retrieve external_id from storage for proper caching
    let external_id = {
        let vertex_tables = graph.data_store.vertex_tables.read();
        vertex_tables
            .get(&label)?
            .get_external_id(internal_id, ts)
            .map(|k| k.to_string())
            .unwrap_or_default()
    };

    if !external_id.is_empty() {
        graph.cache_manager.cache_vertex_id(label, &external_id, internal_id, ts);
    }

    graph
        .cache_manager
        .cache_vertex(label, internal_id, external_id, record.properties.clone());

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
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    let internal_id = table.get_internal_id(external_id, ts);

    table.delete(external_id, ts)?;

    // Invalidate cache entries after successful deletion
    graph.cache_manager.remove_cached_vertex_id(label, external_id);
    if let Some(id) = internal_id {
        graph.cache_manager.remove_cached_vertex(label, id);
    }

    Ok(())
}

pub fn delete_vertex_by_i64(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: i64,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    let internal_id = table.get_internal_id_by_i64(external_id, ts);

    table.delete_by_i64(external_id, ts)?;

    // Invalidate cache entries after successful deletion
    let external_id_str = external_id.to_string();
    graph.cache_manager.remove_cached_vertex_id(label, &external_id_str);
    if let Some(id) = internal_id {
        graph.cache_manager.remove_cached_vertex(label, id);
    }

    Ok(())
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
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;
    
    let internal_id = table
        .get_internal_id(external_id, ts)
        .ok_or(StorageError::vertex_not_found())?;
    
    table.update_property(internal_id, property_name, value, ts)?;

    // Property change invalidates cached vertex data
    graph.cache_manager.remove_cached_vertex(label, internal_id);

    Ok(())
}

pub fn update_vertex_property_by_i64(
    graph: &PropertyGraph,
    label: LabelId,
    external_id: i64,
    property_name: &str,
    value: &Value,
    ts: Timestamp,
) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    let mut vertex_tables = graph.data_store.vertex_tables.write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;
    
    let internal_id = table
        .get_internal_id_by_i64(external_id, ts)
        .ok_or(StorageError::vertex_not_found())?;
    
    table.update_property(internal_id, property_name, value, ts)?;

    // Property change invalidates cached vertex data
    graph.cache_manager.remove_cached_vertex(label, internal_id);

    Ok(())
}

pub fn insert_edge(graph: &PropertyGraph, params: InsertEdgeParams) -> StorageResult<EdgeOffset> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
        StorageError::label_not_found(format!("source vertex label {}", params.src_label))
    })?;
    let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
        StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
    })?;

    let src_internal = src_table
        .get_internal_id(params.src_id, params.ts)
        .ok_or(StorageError::vertex_not_found())?;
    let dst_internal = dst_table
        .get_internal_id(params.dst_id, params.ts)
        .ok_or(StorageError::vertex_not_found())?;
    drop(vertex_tables);

    let key = (params.src_label, params.dst_label, params.edge_label);
    let mut edge_tables = graph.data_store.edge_tables.write();
    let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
        StorageError::label_not_found(format!("edge label {}", params.edge_label))
    })?;

    edge_table.insert_edge(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        params.properties,
        params.ts,
    )
}

pub fn insert_edge_by_i64(graph: &PropertyGraph, params: InsertEdgeParamsByI64) -> StorageResult<EdgeOffset> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
        StorageError::label_not_found(format!("source vertex label {}", params.src_label))
    })?;
    let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
        StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
    })?;

    let src_internal = src_table
        .get_internal_id_by_i64(params.src_id, params.ts)
        .ok_or(StorageError::vertex_not_found())?;
    let dst_internal = dst_table
        .get_internal_id_by_i64(params.dst_id, params.ts)
        .ok_or(StorageError::vertex_not_found())?;
    drop(vertex_tables);

    let key = (params.src_label, params.dst_label, params.edge_label);
    let mut edge_tables = graph.data_store.edge_tables.write();
    let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
        StorageError::label_not_found(format!("edge label {}", params.edge_label))
    })?;

    edge_table.insert_edge(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        params.properties,
        params.ts,
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
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&src_label)?;
    let dst_table = vertex_tables.get(&dst_label)?;

    let src_internal = src_table.get_internal_id(src_id, ts)?;
    let dst_internal = dst_table.get_internal_id(dst_id, ts)?;
    drop(vertex_tables);

    let key = (src_label, dst_label, edge_label);
    let edge_tables = graph.data_store.edge_tables.read();
    let edge_table = edge_tables.get(&key)?;

    edge_table.get_edge(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        ts,
    )
}

pub fn get_edge_by_i64(
    graph: &PropertyGraph,
    edge_label: LabelId,
    src_label: LabelId,
    src_id: i64,
    dst_label: LabelId,
    dst_id: i64,
    ts: Timestamp,
) -> Option<EdgeRecord> {
    if !graph.is_open.load(Ordering::Acquire) {
        return None;
    }
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&src_label)?;
    let dst_table = vertex_tables.get(&dst_label)?;

    let src_internal = src_table.get_internal_id_by_i64(src_id, ts)?;
    let dst_internal = dst_table.get_internal_id_by_i64(dst_id, ts)?;
    drop(vertex_tables);

    let key = (src_label, dst_label, edge_label);
    let edge_tables = graph.data_store.edge_tables.read();
    let edge_table = edge_tables.get(&key)?;

    edge_table.get_edge(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        ts,
    )
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
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&src_label).ok_or_else(|| {
        StorageError::label_not_found(format!("source vertex label {}", src_label))
    })?;
    let dst_table = vertex_tables.get(&dst_label).ok_or_else(|| {
        StorageError::label_not_found(format!("destination vertex label {}", dst_label))
    })?;

    let src_internal = if let Ok(id) = src_id.parse::<i64>() {
        src_table.get_internal_id_by_i64(id, ts)
            .or_else(|| src_table.get_internal_id(src_id, ts))
    } else {
        src_table.get_internal_id(src_id, ts)
    }
    .ok_or(StorageError::vertex_not_found())?;
    
    let dst_internal = if let Ok(id) = dst_id.parse::<i64>() {
        dst_table.get_internal_id_by_i64(id, ts)
            .or_else(|| dst_table.get_internal_id(dst_id, ts))
    } else {
        dst_table.get_internal_id(dst_id, ts)
    }
    .ok_or(StorageError::vertex_not_found())?;
    
    drop(vertex_tables);

    let key = (src_label, dst_label, edge_label);
    let mut edge_tables = graph.data_store.edge_tables.write();
    let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
        StorageError::label_not_found(format!("edge label {}", edge_label))
    })?;

    edge_table.delete_edge(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        ts,
    )
}

pub fn update_edge_property(
    graph: &PropertyGraph,
    params: PropertyGraphUpdateEdgePropertyParams,
) -> StorageResult<bool> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
        StorageError::label_not_found(format!("source vertex label {}", params.src_label))
    })?;
    let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
        StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
    })?;

    let src_internal = if let Ok(id) = params.src_id.parse::<i64>() {
        src_table.get_internal_id_by_i64(id, params.ts)
            .or_else(|| src_table.get_internal_id(params.src_id, params.ts))
    } else {
        src_table.get_internal_id(params.src_id, params.ts)
    }
    .ok_or(StorageError::vertex_not_found())?;
    
    let dst_internal = if let Ok(id) = params.dst_id.parse::<i64>() {
        dst_table.get_internal_id_by_i64(id, params.ts)
            .or_else(|| dst_table.get_internal_id(params.dst_id, params.ts))
    } else {
        dst_table.get_internal_id(params.dst_id, params.ts)
    }
    .ok_or(StorageError::vertex_not_found())?;
    
    drop(vertex_tables);

    let key = (params.src_label, params.dst_label, params.edge_label);
    let mut edge_tables = graph.data_store.edge_tables.write();
    let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
        StorageError::label_not_found(format!("edge label {}", params.edge_label))
    })?;

    edge_table.update_edge_property(
        VertexId::from_int64(src_internal as i64),
        VertexId::from_int64(dst_internal as i64),
        params.prop_name,
        params.value,
        params.ts,
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
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let src_table = vertex_tables.get(&src_label)?;
    let src_internal = if let Ok(id) = src_id.parse::<i64>() {
        src_table.get_internal_id_by_i64(id, ts)
            .or_else(|| src_table.get_internal_id(src_id, ts))
    } else {
        src_table.get_internal_id(src_id, ts)
    }?;
    drop(vertex_tables);

    let key = (src_label, dst_label, edge_label);
    let edge_tables = graph.data_store.edge_tables.read();
    let edge_table = edge_tables.get(&key)?;

    Some(edge_table.out_edges(VertexId::from_int64(src_internal as i64), ts))
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
    
    let vertex_tables = graph.data_store.vertex_tables.read();
    let dst_table = vertex_tables.get(&dst_label)?;
    let dst_internal = if let Ok(id) = dst_id.parse::<i64>() {
        dst_table.get_internal_id_by_i64(id, ts)
            .or_else(|| dst_table.get_internal_id(dst_id, ts))
    } else {
        dst_table.get_internal_id(dst_id, ts)
    }?;
    drop(vertex_tables);

    let key = (src_label, dst_label, edge_label);
    let edge_tables = graph.data_store.edge_tables.read();
    let edge_table = edge_tables.get(&key)?;

    Some(edge_table.in_edges(VertexId::from_int64(dst_internal as i64), ts))
}
