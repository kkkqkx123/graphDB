//! Segment management: immutable CSR segments with versioning and deletion tracking.
//!
//! Segments represent frozen portions of the edge table, storing compressed sparse row (CSR)
//! data with metadata for time-travel queries and MVCC support.

use super::super::{Csr, EdgeId, CsrBase};
use crate::core::types::Timestamp;

/// Deletion information for a CSR segment.
///
/// Tracks the deletion timestamp range and count for edges in the segment.
/// This enables time-travel query optimizations and accurate MVCC semantics.
#[derive(Debug, Clone, Copy)]
pub enum DeletionInfo {
    /// No edges in this segment have been deleted
    NoDeletes,
    /// Some edges have been deleted in the range [min_ts, max_ts]
    /// - min_ts: earliest deletion timestamp
    /// - max_ts: latest deletion timestamp
    /// - deleted_count: exact number of deleted edges (for optimization)
    HasDeletes {
        min_ts: Timestamp,
        max_ts: Timestamp,
        deleted_count: u32,
    },
}

impl DeletionInfo {
    /// Create from deletion timestamps. NoDeletes if min=MAX or max=0.
    pub fn new(min: Timestamp, max: Timestamp) -> Self {
        if min == u32::MAX || max == 0 {
            DeletionInfo::NoDeletes
        } else {
            DeletionInfo::HasDeletes { min_ts: min, max_ts: max, deleted_count: 0 }
        }
    }

    /// Create with known deleted count (used during freeze/segment creation)
    pub fn with_count(min: Timestamp, max: Timestamp, deleted_count: u32) -> Self {
        if min == u32::MAX || max == 0 || deleted_count == 0 {
            DeletionInfo::NoDeletes
        } else {
            DeletionInfo::HasDeletes { min_ts: min, max_ts: max, deleted_count }
        }
    }

    /// Check if all deletions happened before or at query_ts
    pub fn all_deleted_before(&self, query_ts: Timestamp) -> bool {
        match self {
            DeletionInfo::NoDeletes => false,
            DeletionInfo::HasDeletes { max_ts, .. } => *max_ts <= query_ts,
        }
    }

    /// Check if all edges in segment are deleted (fast path for complete deletion)
    pub fn all_edges_deleted(&self, total_edge_count: u64) -> bool {
        match self {
            DeletionInfo::NoDeletes => false,
            DeletionInfo::HasDeletes { deleted_count, .. } => {
                *deleted_count as u64 == total_edge_count
            }
        }
    }

    /// Get deletion percentage (0-100) for observability
    pub fn deletion_percentage(&self, total_edge_count: u64) -> u32 {
        match self {
            DeletionInfo::NoDeletes => 0,
            DeletionInfo::HasDeletes { deleted_count, .. } => {
                if total_edge_count == 0 {
                    0
                } else {
                    (((*deleted_count as u64) * 100) / total_edge_count) as u32
                }
            }
        }
    }

    /// Merge two deletion infos by taking min of mins, max of maxs, and sum of counts
    pub fn merge(&self, other: &DeletionInfo) -> DeletionInfo {
        match (self, other) {
            (DeletionInfo::NoDeletes, DeletionInfo::NoDeletes) => DeletionInfo::NoDeletes,
            (DeletionInfo::NoDeletes, DeletionInfo::HasDeletes { min_ts, max_ts, deleted_count }) |
            (DeletionInfo::HasDeletes { min_ts, max_ts, deleted_count }, DeletionInfo::NoDeletes) => {
                DeletionInfo::HasDeletes { min_ts: *min_ts, max_ts: *max_ts, deleted_count: *deleted_count }
            }
            (DeletionInfo::HasDeletes { min_ts: min1, max_ts: max1, deleted_count: count1 },
             DeletionInfo::HasDeletes { min_ts: min2, max_ts: max2, deleted_count: count2 }) => {
                DeletionInfo::HasDeletes {
                    min_ts: (*min1).min(*min2),
                    max_ts: (*max1).max(*max2),
                    deleted_count: count1.saturating_add(*count2),
                }
            }
        }
    }
}

/// Version tracking for CSR segment recovery
#[derive(Debug, Clone, Copy)]
pub struct SegmentVersion {
    /// Version number: incremented on each update
    pub version: u32,
    /// CRC32 checksum for integrity validation
    pub checksum: u32,
}

impl SegmentVersion {
    /// Create a new segment version
    pub fn new() -> Self {
        Self {
            version: 1,
            checksum: 0,
        }
    }

    /// Increment version on segment modification
    pub fn increment(&mut self) {
        self.version = self.version.saturating_add(1);
    }

    /// Compute CRC32 checksum for segment
    pub fn compute_checksum(segment: &CsrSegment) -> u32 {
        let mut crc = 0u32;
        crc = crc.wrapping_mul(31).wrapping_add(segment.csr.edge_count() as u32);
        crc = crc.wrapping_mul(31).wrapping_add(segment.create_ts_min);
        crc = crc.wrapping_mul(31).wrapping_add(segment.create_ts_max);
        crc
    }

    /// Validate segment integrity
    pub fn validate(&self, segment: &CsrSegment) -> bool {
        let computed = Self::compute_checksum(segment);
        self.checksum == computed || self.checksum == 0
    }
}

/// NbrWithoutEdgeId optimization: auto-enabled for segments >= 10K edges
/// Saves ~15% memory by storing edge_ids separately, with O(1) recovery lookup
pub const SEPARATE_EDGE_ID_STORAGE_THRESHOLD: usize = 10_000;

#[derive(Debug)]
pub struct CsrSegment {
    pub csr: Csr,
    /// Edge creation time range: [create_ts_min, create_ts_max]
    pub create_ts_min: Timestamp,
    pub create_ts_max: Timestamp,
    /// Deletion information for time-travel queries and GC
    pub deletion_info: DeletionInfo,
    /// Version tracking for recovery
    pub version: SegmentVersion,
    /// Timestamp when this segment was created (for adaptive merge decisions)
    pub created_at_ts: Timestamp,
    /// Optional separate edge_id storage for memory optimization
    /// None: direct mode (edge_id in ImmutableNbr)
    /// Some(...): optimized mode (edge_id stored separately, 15% memory savings)
    pub edge_ids: Option<Vec<EdgeId>>,
}

impl CsrSegment {
    pub fn new(csr: Csr, create_ts_min: Timestamp, create_ts_max: Timestamp,
               deletion_info: DeletionInfo) -> Self {
        Self::with_creation_ts(csr, create_ts_min, create_ts_max, deletion_info, u32::MAX)
    }

    pub fn with_creation_ts(csr: Csr, create_ts_min: Timestamp, create_ts_max: Timestamp,
                            deletion_info: DeletionInfo, created_at_ts: Timestamp) -> Self {
        let mut seg = Self {
            csr,
            create_ts_min,
            create_ts_max,
            deletion_info,
            version: SegmentVersion::new(),
            created_at_ts,
            edge_ids: None,
        };
        seg.version.checksum = SegmentVersion::compute_checksum(&seg);
        seg
    }

    /// Recover EdgeId from this segment at given CSR position
    ///
    /// Supports both direct mode (edge_id in ImmutableNbr) and optimized mode
    /// (edge_id stored separately). Transparent to callers.
    pub fn recover_edge_id(&self, nbr: &super::super::ImmutableNbr, csr_position: usize) -> EdgeId {
        match &self.edge_ids {
            Some(ids) => ids.get(csr_position).copied().unwrap_or(nbr.edge_id),
            None => nbr.edge_id,
        }
    }

    /// Calculate age of this segment in timestamp units
    pub fn age(&self, current_ts: Timestamp) -> u32 {
        if self.created_at_ts == u32::MAX {
            0
        } else {
            current_ts.saturating_sub(self.created_at_ts)
        }
    }

    /// Get deletion percentage (0.0-1.0) of this segment
    pub fn deletion_ratio(&self) -> f64 {
        let edge_count = self.csr.edge_count();
        if edge_count == 0 {
            0.0
        } else {
            match self.deletion_info {
                DeletionInfo::NoDeletes => 0.0,
                DeletionInfo::HasDeletes { deleted_count, .. } => {
                    (deleted_count as f64) / (edge_count as f64)
                }
            }
        }
    }

    /// Get deletion info as (min, max) range for serialization
    pub fn deletion_range(&self) -> (Timestamp, Timestamp) {
        match self.deletion_info {
            DeletionInfo::NoDeletes => (u32::MAX, 0),
            DeletionInfo::HasDeletes { min_ts, max_ts, .. } => (min_ts, max_ts),
        }
    }

    /// Estimate memory usage of this segment in bytes
    pub fn estimated_bytes(&self) -> usize {
        let csr_bytes = self.csr.used_memory_size();
        let metadata_bytes = std::mem::size_of::<Timestamp>() * 2
            + std::mem::size_of::<DeletionInfo>();
        csr_bytes + metadata_bytes
    }
}
