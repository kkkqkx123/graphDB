//! Index MVCC Operations
//!
//! Contains index-related MVCC operations for PropertyGraph.
//! This module handles low-level index updates with MVCC support.

use crate::core::types::Timestamp;
use crate::core::{StorageResult, Value};
use crate::storage::index::secondary::IndexDataManager;

use super::PropertyGraph;

pub fn update_vertex_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    vertex_id: &Value,
    index_name: &str,
    props: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<()> {
    graph
        .index_data_manager
        .write()
        .update_vertex_indexes_mvcc(space_id, vertex_id, index_name, props, ts)
}

pub fn delete_vertex_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    vertex_id: &Value,
    ts: Timestamp,
) -> StorageResult<()> {
    graph
        .index_data_manager
        .write()
        .delete_vertex_indexes_mvcc(space_id, vertex_id, ts)
}

pub fn update_edge_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    src: &Value,
    dst: &Value,
    index_name: &str,
    props: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<()> {
    graph
        .index_data_manager
        .write()
        .update_edge_indexes_mvcc(space_id, src, dst, index_name, props, ts)
}

pub fn delete_edge_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    src: &Value,
    dst: &Value,
    index_names: &[String],
    ts: Timestamp,
) -> StorageResult<()> {
    graph
        .index_data_manager
        .write()
        .delete_edge_indexes_mvcc(space_id, src, dst, index_names, ts)
}
