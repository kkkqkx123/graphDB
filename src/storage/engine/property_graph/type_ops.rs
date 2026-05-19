//! Type Operations
//!
//! Contains vertex and edge type creation/deletion operations for PropertyGraph.
//! This module handles low-level type management at the storage engine level.

use std::sync::atomic::Ordering;

use crate::core::types::LabelId;
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::{EdgeSchema, EdgeStrategy, EdgeTable};
use crate::storage::storage_types::StoragePropertyDef;
use crate::storage::vertex::{VertexSchema, VertexTable};

use super::super::edge_params::CreateEdgeTypeParams;
use super::PropertyGraph;

pub fn create_vertex_type(
    graph: &PropertyGraph,
    name: &str,
    properties: Vec<StoragePropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let mut vertex_label_names = graph.data_store.vertex_label_names.write();
    if vertex_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }
    
    let mut vertex_label_counter = graph.data_store.vertex_label_counter.write();
    let label_id = *vertex_label_counter;
    *vertex_label_counter += 1;
    
    let primary_key_index = properties
        .iter()
        .position(|p| p.name == primary_key)
        .ok_or_else(|| StorageError::property_not_found(primary_key.to_string()))?;

    let schema = VertexSchema {
        label_id,
        label_name: name.to_string(),
        properties,
        primary_key_index,
    };

    let table = VertexTable::new(label_id, name.to_string(), schema);
    graph.data_store.vertex_tables.write().insert(label_id, table);
    vertex_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_vertex_type_with_id(
    graph: &PropertyGraph,
    name: &str,
    label_id: LabelId,
    properties: Vec<StoragePropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let mut vertex_label_names = graph.data_store.vertex_label_names.write();
    if vertex_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }
    
    if graph.data_store.vertex_tables.read().contains_key(&label_id) {
        return Err(StorageError::label_already_exists(format!(
            "label_id {}",
            label_id
        )));
    }

    let mut vertex_label_counter = graph.data_store.vertex_label_counter.write();
    if label_id >= *vertex_label_counter {
        *vertex_label_counter = label_id + 1;
    }

    let primary_key_index = properties
        .iter()
        .position(|p| p.name == primary_key)
        .ok_or_else(|| StorageError::property_not_found(primary_key.to_string()))?;

    let schema = VertexSchema {
        label_id,
        label_name: name.to_string(),
        properties,
        primary_key_index,
    };

    let table = VertexTable::new(label_id, name.to_string(), schema);
    graph.data_store.vertex_tables.write().insert(label_id, table);
    vertex_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_edge_type(
    graph: &PropertyGraph,
    name: &str,
    src_label: LabelId,
    dst_label: LabelId,
    properties: Vec<StoragePropertyDef>,
    oe_strategy: EdgeStrategy,
    ie_strategy: EdgeStrategy,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    if !graph.data_store.vertex_tables.read().contains_key(&src_label) {
        return Err(StorageError::label_not_found(format!(
            "source label {}",
            src_label
        )));
    }
    if !graph.data_store.vertex_tables.read().contains_key(&dst_label) {
        return Err(StorageError::label_not_found(format!(
            "destination label {}",
            dst_label
        )));
    }

    let mut edge_label_names = graph.data_store.edge_label_names.write();
    if edge_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }

    let mut edge_label_counter = graph.data_store.edge_label_counter.write();
    let label_id = *edge_label_counter;
    *edge_label_counter += 1;

    let schema = EdgeSchema {
        label_id,
        label_name: name.to_string(),
        src_label,
        dst_label,
        properties,
        oe_strategy,
        ie_strategy,
    };

    let table = EdgeTable::new(schema)?;
    let key = (src_label, dst_label, label_id);
    graph.data_store.edge_tables.write().insert(key, table);
    edge_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_edge_type_with_id(
    graph: &PropertyGraph,
    params: CreateEdgeTypeParams,
    label_id: LabelId,
) -> StorageResult<LabelId> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    if !graph.data_store.vertex_tables.read().contains_key(&params.src_label) {
        return Err(StorageError::label_not_found(format!(
            "source label {}",
            params.src_label
        )));
    }
    if !graph.data_store.vertex_tables.read().contains_key(&params.dst_label) {
        return Err(StorageError::label_not_found(format!(
            "destination label {}",
            params.dst_label
        )));
    }

    let mut edge_label_names = graph.data_store.edge_label_names.write();
    if edge_label_names.contains_key(params.name) {
        return Err(StorageError::label_already_exists(params.name.to_string()));
    }

    let mut edge_label_counter = graph.data_store.edge_label_counter.write();
    if label_id >= *edge_label_counter {
        *edge_label_counter = label_id + 1;
    }

    let schema = EdgeSchema {
        label_id,
        label_name: params.name.to_string(),
        src_label: params.src_label,
        dst_label: params.dst_label,
        properties: params.properties,
        oe_strategy: params.oe_strategy,
        ie_strategy: params.ie_strategy,
    };

    let table = EdgeTable::new(schema)?;
    let key = (params.src_label, params.dst_label, label_id);
    graph.data_store.edge_tables.write().insert(key, table);
    edge_label_names.insert(params.name.to_string(), label_id);

    Ok(label_id)
}

pub fn drop_vertex_type(graph: &PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let label_id = {
        let mut vertex_label_names = graph.data_store.vertex_label_names.write();
        vertex_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?
    };
    
    graph.data_store.vertex_tables.write().remove(&label_id);
    graph.data_store.edge_tables.write().retain(|key, _table| {
        let (src, dst, _edge) = *key;
        src != label_id && dst != label_id
    });
    
    Ok(())
}

pub fn drop_edge_type(graph: &PropertyGraph, name: &str) -> StorageResult<()> {
    if !graph.is_open.load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }
    
    let label_id = {
        let mut edge_label_names = graph.data_store.edge_label_names.write();
        edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?
    };

    graph.data_store.edge_tables.write().retain(|_key, _table| {
        let (_, _, e) = *_key;
        e != label_id
    });

    Ok(())
}
