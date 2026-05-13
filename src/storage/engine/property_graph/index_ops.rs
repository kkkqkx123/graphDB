//! Index Operations
//!
//! Contains index-related operations for PropertyGraph.

use crate::core::{StorageResult, Value};
use crate::storage::index::secondary::GcStats;
use crate::transaction::wal::types::Timestamp;

use super::PropertyGraph;

pub fn update_vertex_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    vertex_id: &Value,
    index_name: &str,
    props: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<()> {
    graph.index_data_manager.update_vertex_indexes_mvcc(
        space_id,
        vertex_id,
        index_name,
        props,
        ts,
    )
}

pub fn delete_vertex_indexes_mvcc(
    graph: &PropertyGraph,
    space_id: u64,
    vertex_id: &Value,
    ts: Timestamp,
) -> StorageResult<()> {
    graph
        .index_data_manager
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
    graph.index_data_manager.update_edge_indexes_mvcc(
        space_id,
        src,
        dst,
        index_name,
        props,
        ts,
    )
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
        .delete_edge_indexes_mvcc(space_id, src, dst, index_names, ts)
}

pub fn gc_index_tombstones(graph: &mut PropertyGraph, ts: Timestamp) -> StorageResult<GcStats> {
    graph.index_data_manager.gc_tombstones(ts)
}

pub fn gc_index_tombstones_incremental(
    graph: &PropertyGraph,
    ts: Timestamp,
    batch_size: usize,
) -> StorageResult<GcStats> {
    graph.index_data_manager.gc_tombstones_incremental(ts, batch_size)
}
