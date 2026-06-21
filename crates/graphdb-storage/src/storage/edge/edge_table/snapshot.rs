//! Snapshot export and time-travel query support.
//!
//! Enables consistent point-in-time snapshots of the edge table for
//! backup, replication, and time-travel queries.

use super::super::{Csr, Nbr, EdgeSchema, LabelId, VertexId, ImmutableNbr, CsrBase};
use super::segment::CsrSegment;
use crate::core::types::{Timestamp, EdgeId};
use crate::core::StorageResult;
use crate::storage::edge::PropertyTable;
use std::collections::HashMap;

/// Exported read-only snapshot of an edge table at a specific timestamp.
///
/// Suitable for:
/// - Backup and restore operations
/// - Time-travel queries (historical data)
/// - Cross-node replication
/// - Snapshot isolation in transactions
#[derive(Debug, Clone)]
pub struct ExportedEdgeSnapshot {
    /// Timestamp of this snapshot
    pub snapshot_ts: Timestamp,
    /// Edge label identifier
    pub label: LabelId,
    /// Read-only outgoing edges
    pub out_csr: Csr,
    /// Read-only incoming edges
    pub in_csr: Csr,
    /// Edge properties (cloned for independence)
    pub properties: PropertyTable,
    /// Edge schema metadata
    pub schema: EdgeSchema,
}

impl ExportedEdgeSnapshot {
    /// Get outgoing edges from a source vertex (snapshot isolation)
    ///
    /// Returns edges as they existed at snapshot_ts.
    /// No timestamp filtering needed - snapshot is already filtered.
    pub fn get_out_edges(&self, src: u32) -> Vec<Nbr> {
        self.out_csr.edges_of(src)
            .iter()
            .map(|edge| Nbr::new(edge.neighbor, edge.edge_id, edge.prop_offset, edge.timestamp))
            .collect()
    }

    /// Get incoming edges to a destination vertex (snapshot isolation)
    ///
    /// Returns edges as they existed at snapshot_ts.
    pub fn get_in_edges(&self, dst: u32) -> Vec<Nbr> {
        self.in_csr.edges_of(dst)
            .iter()
            .map(|edge| Nbr::new(edge.neighbor, edge.edge_id, edge.prop_offset, edge.timestamp))
            .collect()
    }

    /// Get a specific edge in the snapshot (if it exists)
    pub fn get_edge(&self, src: u32, dst: VertexId) -> Option<Nbr> {
        self.out_csr.get_edge(src, dst)
            .map(|edge| Nbr::new(edge.neighbor, edge.edge_id, edge.prop_offset, edge.timestamp))
    }

    /// Check if an edge exists in this snapshot
    pub fn has_edge(&self, src: u32, dst: VertexId) -> bool {
        self.get_edge(src, dst).is_some()
    }

    /// Get edge count for a vertex
    pub fn degree(&self, src: u32) -> usize {
        self.out_csr.edges_of(src).len()
    }
}

/// Snapshot builder supporting MVCC filtering
pub struct SnapshotBuilder {
    /// Dedup map: (src_vid, edge_id) -> (src_vid, nbr)
    edge_map: HashMap<(u32, EdgeId), (u32, Nbr)>,
}

impl SnapshotBuilder {
    /// Create a new snapshot builder
    pub fn new() -> Self {
        Self {
            edge_map: HashMap::new(),
        }
    }

    /// Add edges from a segment
    pub fn add_segment_edges(
        &mut self,
        segment: &CsrSegment,
        ts: Timestamp,
        tombstones: &HashMap<EdgeId, Timestamp>,
    ) {
        if segment.create_ts_min > ts {
            return;
        }

        if segment.deletion_info.all_deleted_before(ts)
            && segment.deletion_info.all_edges_deleted(segment.csr.edge_count()) {
            return;
        }

        let mut edge_position = 0usize;
        for (src, immutable_nbr) in segment.csr.iter() {
            let edge_id = segment.recover_edge_id(immutable_nbr, edge_position);
            edge_position += 1;

            if immutable_nbr.timestamp > ts {
                continue;
            }

            if let Some(&delete_ts) = tombstones.get(&edge_id) {
                if delete_ts <= ts {
                    continue;
                }
            }

            let src_u32 = src.as_int64().unwrap_or(0) as u32;
            let nbr = Nbr::new(
                immutable_nbr.neighbor,
                edge_id,
                immutable_nbr.prop_offset,
                immutable_nbr.timestamp,
            );
            self.edge_map.insert((src_u32, edge_id), (src_u32, nbr));
        }
    }

    /// Add edges from mutable CSR delta
    pub fn add_delta_edges(
        &mut self,
        delta_edges: Vec<(u32, Nbr)>,
        ts: Timestamp,
        tombstones: &HashMap<EdgeId, Timestamp>,
    ) {
        for (src_u32, nbr) in delta_edges {
            if nbr.create_ts > ts {
                continue;
            }

            if let Some(&delete_ts) = tombstones.get(&nbr.edge_id) {
                if delete_ts <= ts {
                    continue;
                }
            }

            self.edge_map.insert((src_u32, nbr.edge_id), (src_u32, nbr));
        }
    }

    /// Build CSR from collected edges
    pub fn build_csr(
        edges: Vec<(u32, Nbr)>,
        vertex_capacity: usize,
    ) -> StorageResult<Csr> {
        Ok(Csr::from_nbr_entries(&edges, vertex_capacity))
    }

    /// Get collected edges as sorted vector
    pub fn edges(&self) -> Vec<(u32, Nbr)> {
        let mut edges: Vec<_> = self.edge_map.values().cloned().collect();
        edges.sort_by_key(|(src, _)| *src);
        edges
    }
}
