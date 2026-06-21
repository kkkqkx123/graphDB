//! Edge Table
//!
//! Combines out/in CSRs and property storage for edge management.
//! Uses EdgeOffset (CSR-native offset) instead of global EdgeId for edge identification.

use super::{
    Csr, CsrBase, EdgeRecord, EdgeSchema, EdgeStrategy, LabelId, MutableCsrTrait,
    CsrVariant, ImmutableCsr, Nbr, PropertyTable, Timestamp, VertexId,
};
use crate::core::types::EdgeId;
use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::core::types::CompactConfig;
use crate::storage::persistence::{read_header, section, write_header_to, HEADER_SIZE};
use crate::storage::types::{PropertyId, StoragePropertyDef};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;
use std::sync::Arc;

// Version of the edge table metadata format for compatibility management
const EDGE_META_VERSION: u32 = 2;

/// Statistics about tombstones for observability and debugging.
#[derive(Debug, Clone)]
pub struct TombstoneStats {
    /// Number of active tombstones
    pub count: usize,
    /// Approximate memory used by tombstones (bytes)
    pub memory_bytes: usize,
    /// Oldest deletion timestamp in tombstones
    pub oldest_delete_ts: Option<Timestamp>,
    /// Newest deletion timestamp in tombstones
    pub newest_delete_ts: Option<Timestamp>,
    /// Current minimum active snapshot timestamp
    pub min_active_snapshot_ts: Timestamp,
}

impl TombstoneStats {
    /// Estimate memory usage: EdgeId(u64) + Timestamp(u32) = 12 bytes per entry
    fn estimate_memory(count: usize) -> usize {
        count * std::mem::size_of::<(EdgeId, Timestamp)>()
    }
}

/// Statistics about deletions across all segments for observability.
///
/// Tracks deletion patterns to help identify when segments have significant
/// deletion activity, useful for deciding when to merge or gc segments.
#[derive(Debug, Clone, Default)]
pub struct DeletionStats {
    /// Total edges deleted across all segments
    pub total_deleted_edges: u64,
    /// Total edges frozen (for percentage calculation)
    pub total_frozen_edges: u64,
    /// Number of segments with some deletions
    pub segments_with_deletions: usize,
    /// Number of segments where all edges are deleted (complete deletion)
    pub completely_deleted_segments: usize,
    /// Oldest deletion timestamp across all segments
    pub oldest_deletion_ts: Option<Timestamp>,
    /// Newest deletion timestamp across all segments
    pub newest_deletion_ts: Option<Timestamp>,
}

impl DeletionStats {
    /// Get deletion percentage as a ratio (0.0 to 1.0)
    pub fn deletion_ratio(&self) -> f64 {
        if self.total_frozen_edges == 0 {
            0.0
        } else {
            self.total_deleted_edges as f64 / self.total_frozen_edges as f64
        }
    }

    /// Get deletion percentage (0-100)
    pub fn deletion_percentage(&self) -> f64 {
        self.deletion_ratio() * 100.0
    }

    /// Check if deletions are significant (>10%)
    pub fn is_significant(&self) -> bool {
        self.deletion_ratio() > 0.1
    }
}

/// Statistics about segment merge operations for observability and monitoring.
///
/// Tracks merge activity to understand segment consolidation patterns and
/// evaluate merge strategy effectiveness.
#[derive(Debug, Clone, Default)]
pub struct MergeStats {
    /// Total number of merge operations performed
    pub total_merge_operations: u64,
    /// Total number of segments merged (sum of all merge operations)
    pub total_segments_merged: u64,
    /// Total number of edges involved in merges
    pub total_edges_merged: u64,
    /// Total time spent on merge operations (milliseconds)
    pub total_merge_time_ms: u64,
    /// Current number of segments
    pub current_segment_count: usize,
    /// Maximum segment count reached
    pub max_segment_count: usize,
}

impl MergeStats {
    /// Get average merge time per operation (milliseconds)
    pub fn avg_merge_time_ms(&self) -> f64 {
        if self.total_merge_operations == 0 {
            0.0
        } else {
            self.total_merge_time_ms as f64 / self.total_merge_operations as f64
        }
    }

    /// Get average segments merged per operation
    pub fn avg_segments_per_merge(&self) -> f64 {
        if self.total_merge_operations == 0 {
            0.0
        } else {
            self.total_segments_merged as f64 / self.total_merge_operations as f64
        }
    }

    /// Get average edges merged per operation
    pub fn avg_edges_per_merge(&self) -> f64 {
        if self.total_merge_operations == 0 {
            0.0
        } else {
            self.total_edges_merged as f64 / self.total_merge_operations as f64
        }
    }

    /// Check if segment count is growing too fast (>80% of max)
    pub fn segment_count_pressure(&self) -> bool {
        if self.max_segment_count == 0 {
            false
        } else {
            (self.current_segment_count as f64 / self.max_segment_count as f64) > 0.8
        }
    }
}

#[derive(Debug, Clone)]
pub struct MergeMetrics {
    /// Number of segments before merge
    pub segments_before: usize,
    /// Number of segments after merge
    pub segments_after: usize,
    /// Total number of edges processed in merge
    pub edges_merged: u64,
    /// Time taken for merge operation (milliseconds)
    pub duration_ms: u64,
}

impl MergeMetrics {
    /// Log merge metrics with reduction ratio
    pub fn log(&self) {
        let reduction = if self.segments_before > 0 {
            ((self.segments_before - self.segments_after) as f64 / self.segments_before as f64) * 100.0
        } else {
            0.0
        };
        println!("[MergeMetrics] segments: {} → {} (-{:.1}%), edges: {}, duration: {}ms",
                 self.segments_before, self.segments_after, reduction, self.edges_merged, self.duration_ms);
    }
}

/// Helper structure for merge operation metrics
struct DirectionMergeMetrics {
    edges_processed: u64,
}

/// Result wrapper containing merge metrics and reduced count
pub struct MergeMetricsResult {
    pub metrics: MergeMetrics,
    pub segments_reduced: usize,
}

#[derive(Debug, Clone)]
pub struct EdgeTableConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub max_segments_per_direction: usize,
}

impl Default for EdgeTableConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            max_segments_per_direction: 100,
        }
    }
}

/// Deletion information for a CSR segment.
///
/// Tracks the deletion timestamp range and count for edges in the segment.
/// This enables time-travel query optimizations and accurate MVCC semantics.
#[derive(Debug, Clone, Copy)]
enum DeletionInfo {
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
    fn new(min: Timestamp, max: Timestamp) -> Self {
        if min == u32::MAX || max == 0 {
            DeletionInfo::NoDeletes
        } else {
            DeletionInfo::HasDeletes { min_ts: min, max_ts: max, deleted_count: 0 }
        }
    }

    /// Create with known deleted count (used during freeze/segment creation)
    fn with_count(min: Timestamp, max: Timestamp, deleted_count: u32) -> Self {
        if min == u32::MAX || max == 0 || deleted_count == 0 {
            DeletionInfo::NoDeletes
        } else {
            DeletionInfo::HasDeletes { min_ts: min, max_ts: max, deleted_count }
        }
    }

    /// Check if all deletions happened before or at query_ts
    fn all_deleted_before(&self, query_ts: Timestamp) -> bool {
        match self {
            DeletionInfo::NoDeletes => false,
            DeletionInfo::HasDeletes { max_ts, .. } => *max_ts <= query_ts,
        }
    }

    /// Check if all edges in segment are deleted (fast path for complete deletion)
    fn all_edges_deleted(&self, total_edge_count: u64) -> bool {
        match self {
            DeletionInfo::NoDeletes => false,
            DeletionInfo::HasDeletes { deleted_count, .. } => {
                // If deleted_count == total_edge_count, all edges are deleted
                *deleted_count as u64 == total_edge_count
            }
        }
    }

    /// Get deletion percentage (0-100) for observability
    fn deletion_percentage(&self, total_edge_count: u64) -> u32 {
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
    fn merge(&self, other: &DeletionInfo) -> DeletionInfo {
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

/// Parameters for update_edge_property_by_offset operation
pub struct UpdateEdgePropertyByOffsetParams {
    pub src: u32,
    pub dst: u32,
    pub rank: i64,
    pub prop_id: u16,
    pub value: Value,
    pub ts: Timestamp,
}

#[derive(Debug)]
struct CsrSegment {
    csr: Csr,
    // Edge creation time range: [create_ts_min, create_ts_max]
    // All edges were created within this range
    create_ts_min: Timestamp,
    create_ts_max: Timestamp,
    // Deletion information for time-travel queries and GC
    deletion_info: DeletionInfo,
    // Version tracking for recovery
    version: SegmentVersion,
    // Timestamp when this segment was created (for adaptive merge decisions)
    created_at_ts: Timestamp,
}

/// Version tracking for CSR segment recovery
#[derive(Debug, Clone, Copy)]
struct SegmentVersion {
    /// Version number: incremented on each update
    version: u32,
    /// CRC32 checksum for integrity validation
    checksum: u32,
}

impl SegmentVersion {
    /// Create a new segment version
    fn new() -> Self {
        Self {
            version: 1,
            checksum: 0,
        }
    }

    /// Increment version on segment modification
    fn increment(&mut self) {
        self.version = self.version.saturating_add(1);
    }

    /// Compute CRC32 checksum for segment
    fn compute_checksum(segment: &CsrSegment) -> u32 {
        // Simple checksum: XOR of edge count, create_ts_min/max, and edge count mod
        let mut crc = 0u32;
        crc = crc.wrapping_mul(31).wrapping_add(segment.csr.edge_count() as u32);
        crc = crc.wrapping_mul(31).wrapping_add(segment.create_ts_min);
        crc = crc.wrapping_mul(31).wrapping_add(segment.create_ts_max);
        crc
    }

    /// Validate segment integrity
    fn validate(&self, segment: &CsrSegment) -> bool {
        let computed = Self::compute_checksum(segment);
        self.checksum == computed || self.checksum == 0 // 0 = first initialization
    }
}

impl CsrSegment {
    fn new(csr: Csr, create_ts_min: Timestamp, create_ts_max: Timestamp,
           deletion_info: DeletionInfo) -> Self {
        Self::with_creation_ts(csr, create_ts_min, create_ts_max, deletion_info, u32::MAX)
    }

    fn with_creation_ts(csr: Csr, create_ts_min: Timestamp, create_ts_max: Timestamp,
                        deletion_info: DeletionInfo, created_at_ts: Timestamp) -> Self {
        let mut seg = Self {
            csr,
            create_ts_min,
            create_ts_max,
            deletion_info,
            version: SegmentVersion::new(),
            created_at_ts,
        };
        // Compute initial checksum
        seg.version.checksum = SegmentVersion::compute_checksum(&seg);
        seg
    }

    /// Calculate age of this segment in timestamp units
    fn age(&self, current_ts: Timestamp) -> u32 {
        if self.created_at_ts == u32::MAX {
            0  // Unknown age
        } else {
            current_ts.saturating_sub(self.created_at_ts)
        }
    }

    /// Get deletion percentage (0.0-1.0) of this segment
    fn deletion_ratio(&self) -> f64 {
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
    /// Note: deleted_count is not serialized here, only time range
    fn deletion_range(&self) -> (Timestamp, Timestamp) {
        match self.deletion_info {
            DeletionInfo::NoDeletes => (u32::MAX, 0),
            DeletionInfo::HasDeletes { min_ts, max_ts, .. } => (min_ts, max_ts),
        }
    }

    /// Estimate memory usage of this segment in bytes
    ///
    /// Considers: CSR structure (offset + edges arrays), metadata, deletion info.
    /// Used for merge decision heuristics and observability.
    fn estimated_bytes(&self) -> usize {
        let csr_bytes = self.csr.used_memory_size();
        let metadata_bytes = std::mem::size_of::<Timestamp>() * 2  // create_ts_min, create_ts_max
            + std::mem::size_of::<DeletionInfo>();
        csr_bytes + metadata_bytes
    }
}

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
    pub out_csr: ImmutableCsr,
    /// Read-only incoming edges
    pub in_csr: ImmutableCsr,
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
    }

    /// Get incoming edges to a destination vertex (snapshot isolation)
    ///
    /// Returns edges as they existed at snapshot_ts.
    pub fn get_in_edges(&self, dst: u32) -> Vec<Nbr> {
        self.in_csr.edges_of(dst)
    }

    /// Get a specific edge in the snapshot (if it exists)
    pub fn get_edge(&self, src: u32, dst: VertexId) -> Option<Nbr> {
        self.out_csr.get_edge(src, dst, 0)  // timestamp=0 since already filtered
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

#[derive(Debug)]
pub struct EdgeTable {
    label: LabelId,
    label_name: String,
    src_label: LabelId,
    dst_label: LabelId,
    schema: EdgeSchema,
    out_csr: CsrVariant,
    in_csr: CsrVariant,
    out_segments: Vec<CsrSegment>,
    in_segments: Vec<CsrSegment>,
    tombstones: HashMap<EdgeId, Timestamp>,
    properties: PropertyTable,
    is_open: bool,
    next_edge_id: EdgeId,
    /// Minimum timestamp of all active snapshots. Used for tombstone GC.
    /// Tombstones with delete_ts < min_active_snapshot_ts can be safely removed.
    /// Initial value: u32::MAX (no snapshots are active)
    min_active_snapshot_ts: Timestamp,
    config: EdgeTableConfig,
    /// Active snapshot timestamps and their reference count.
    /// Used for automatic garbage collection of tombstones.
    /// When count reaches 0, the timestamp is removed and GC is triggered.
    active_snapshots: HashMap<Timestamp, usize>,
    /// Optional statistics manager for MVCC metrics reporting
    stats_manager: Option<std::sync::Arc<crate::core::stats::StatsManager>>,
    /// Merge statistics for monitoring and observability
    merge_stats: Arc<parking_lot::Mutex<MergeStats>>,
}

impl EdgeTable {
    pub fn new(schema: EdgeSchema) -> StorageResult<Self> {
        Self::with_config(schema, EdgeTableConfig::default())
    }

    pub fn with_config(schema: EdgeSchema, config: EdgeTableConfig) -> StorageResult<Self> {
        schema.validate()?;

        let out_csr = CsrVariant::from_strategy(
            schema.oe_strategy,
            config.initial_vertex_capacity,
            config.initial_edge_capacity,
        )?;
        let in_csr = CsrVariant::from_strategy(
            schema.ie_strategy,
            config.initial_vertex_capacity,
            config.initial_edge_capacity,
        )?;

        let mut properties = PropertyTable::with_capacity(config.initial_edge_capacity);
        for prop in &schema.properties {
            properties.add_property(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        Ok(Self {
            label: schema.label_id,
            label_name: schema.label_name.clone(),
            src_label: schema.src_label,
            dst_label: schema.dst_label,
            schema,
            out_csr,
            in_csr,
            out_segments: Vec::new(),
            in_segments: Vec::new(),
            tombstones: HashMap::new(),
            properties,
            is_open: true,
            next_edge_id: EdgeId(0),
            min_active_snapshot_ts: u32::MAX,
            config,
            active_snapshots: HashMap::new(),
            stats_manager: None,
            merge_stats: Arc::new(parking_lot::Mutex::new(MergeStats::default())),
        })
    }

    fn edge_endpoint_key(endpoint: u32, rank: i64) -> VertexId {
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&(endpoint as i64).to_be_bytes());
        data.extend_from_slice(&rank.to_be_bytes());
        VertexId::from_bytes(data)
    }

    fn decode_edge_endpoint(key: VertexId) -> (VertexId, i64) {
        let bytes = key.as_bytes();
        if bytes.len() != 16 {
            return (key, 0);
        }

        let mut endpoint_bytes = [0u8; 8];
        endpoint_bytes.copy_from_slice(&bytes[..8]);
        let mut rank_bytes = [0u8; 8];
        rank_bytes.copy_from_slice(&bytes[8..16]);

        (
            VertexId::from_int64(i64::from_be_bytes(endpoint_bytes)),
            i64::from_be_bytes(rank_bytes),
        )
    }

    /// Set the statistics manager for recording MVCC metrics
    ///
    /// This enables automatic reporting of tombstone statistics, GC operations,
    /// and active snapshot counts to the monitoring system.
    pub fn set_stats_manager(&mut self, stats: std::sync::Arc<crate::core::stats::StatsManager>) {
        self.stats_manager = Some(stats);
    }

    fn is_tombstoned(&self, edge_id: EdgeId, ts: Timestamp) -> bool {
        self.tombstones
            .get(&edge_id)
            .is_some_and(|delete_ts| *delete_ts <= ts)
    }

    /// Garbage collect tombstones that are no longer needed for snapshot isolation.
    ///
    /// Removes tombstones with delete_ts < min_active_snapshot_ts.
    /// These tombstones cannot affect any active snapshot since all snapshots
    /// have ts >= min_active_snapshot_ts.
    ///
    /// # Arguments
    /// - `min_active_snapshot_ts`: Minimum timestamp of all active snapshots
    ///
    /// # Returns
    /// Number of tombstones that were garbage collected
    ///
    /// # Example
    /// ```ignore
    /// let collected = table.gc_tombstones(200);
    /// println!("Cleaned up {} tombstones", collected);
    /// ```
    pub fn gc_tombstones(&mut self, min_active_snapshot_ts: Timestamp) -> usize {
        let before = self.tombstones.len();
        self.tombstones.retain(|_edge_id, delete_ts| {
            *delete_ts >= min_active_snapshot_ts
        });
        let after = self.tombstones.len();
        self.min_active_snapshot_ts = min_active_snapshot_ts;

        // Record GC operation to metrics
        if let Some(stats) = &self.stats_manager {
            stats.record_tombstone_gc();
        }

        before - after
    }

    /// Register a new active snapshot at the given timestamp.
    ///
    /// This increments the reference count for the snapshot timestamp.
    /// Must be called when a new snapshot is created.
    pub fn register_active_snapshot(&mut self, ts: Timestamp) {
        *self.active_snapshots.entry(ts).or_insert(0) += 1;

        // Update active snapshots count in metrics
        if let Some(stats) = &self.stats_manager {
            stats.set_value(
                crate::core::stats::MetricType::TombstoneActiveSnapshots,
                self.active_snapshots.len() as u64,
            );
        }
    }

    /// Unregister an active snapshot at the given timestamp.
    ///
    /// This decrements the reference count. When count reaches 0,
    /// the timestamp is removed and tombstone GC is automatically triggered.
    ///
    /// # Returns
    /// The new reference count for this timestamp (0 if removed)
    pub fn unregister_active_snapshot(&mut self, ts: Timestamp) -> usize {
        let mut should_gc = false;
        let new_count = if let Some(count) = self.active_snapshots.get_mut(&ts) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                self.active_snapshots.remove(&ts);
                should_gc = true;
                0
            } else {
                *count
            }
        } else {
            0
        };

        if should_gc {
            let new_min_ts = self.active_snapshots
                .keys()
                .copied()
                .min()
                .unwrap_or(u32::MAX);
            self.gc_tombstones(new_min_ts);
        }

        // Update active snapshots count in metrics
        if let Some(stats) = &self.stats_manager {
            stats.set_value(
                crate::core::stats::MetricType::TombstoneActiveSnapshots,
                self.active_snapshots.len() as u64,
            );
        }

        new_count
    }

    /// Get current tombstone statistics for observability.
    pub fn tombstone_stats(&self) -> TombstoneStats {
        let oldest = self.tombstones.values().copied().min();
        let newest = self.tombstones.values().copied().max();

        TombstoneStats {
            count: self.tombstones.len(),
            memory_bytes: TombstoneStats::estimate_memory(self.tombstones.len()),
            oldest_delete_ts: oldest,
            newest_delete_ts: newest,
            min_active_snapshot_ts: self.min_active_snapshot_ts,
        }
    }

    /// Calculate deletion statistics across all segments.
    ///
    /// Analyzes segments to provide insights into deletion patterns:
    /// - How many edges have been deleted vs. frozen
    /// - Which segments have complete deletion (all edges deleted)
    /// - Deletion timestamp range
    ///
    /// Used for observability and merge decisions.
    pub fn deletion_stats(&self) -> DeletionStats {
        let mut stats = DeletionStats::default();

        let mut total_edge_count = 0u64;
        let mut total_deleted_count = 0u64;

        for segment in self.out_segments.iter().chain(self.in_segments.iter()) {
            let edge_count = segment.csr.edge_count();
            total_edge_count += edge_count;

            match segment.deletion_info {
                DeletionInfo::NoDeletes => {
                    // No deletions in this segment
                }
                DeletionInfo::HasDeletes { min_ts, max_ts, deleted_count } => {
                    total_deleted_count += deleted_count as u64;
                    stats.segments_with_deletions += 1;

                    // Track complete deletion segments
                    if (deleted_count as u64) == edge_count {
                        stats.completely_deleted_segments += 1;
                    }

                    // Track oldest and newest deletion timestamps
                    if let Some(ref mut oldest) = stats.oldest_deletion_ts {
                        *oldest = (*oldest).min(min_ts);
                    } else {
                        stats.oldest_deletion_ts = Some(min_ts);
                    }

                    if let Some(ref mut newest) = stats.newest_deletion_ts {
                        *newest = (*newest).max(max_ts);
                    } else {
                        stats.newest_deletion_ts = Some(max_ts);
                    }
                }
            }
        }

        stats.total_frozen_edges = total_edge_count;
        stats.total_deleted_edges = total_deleted_count;

        stats
    }

    /// Calculate total memory used by all segments in both directions
    ///
    /// Used for merge heuristics and observability to understand
    /// memory footprint and segment consolidation impact.
    pub fn segments_total_bytes(&self) -> usize {
        self.out_segments.iter().map(|s| s.estimated_bytes()).sum::<usize>()
            + self.in_segments.iter().map(|s| s.estimated_bytes()).sum::<usize>()
    }

    /// Get number of active snapshots (for testing and debugging)
    #[cfg(test)]
    pub fn active_snapshot_count(&self) -> usize {
        self.active_snapshots.values().sum()
    }

    /// Get the minimum active snapshot timestamp.
    ///
    /// This is the earliest timestamp at which any snapshot is currently active.
    /// All tombstones with delete_ts < this value can be safely garbage collected.
    ///
    /// Returns u32::MAX if no active snapshots exist.
    pub fn get_min_active_snapshot_ts(&self) -> Timestamp {
        self.active_snapshots
            .keys()
            .copied()
            .min()
            .unwrap_or(u32::MAX)
    }

    /// Get current merge statistics
    pub fn merge_stats(&self) -> MergeStats {
        let mut stats = self.merge_stats.lock().clone();
        // Update current segment count
        stats.current_segment_count = self.out_segments.len() + self.in_segments.len();
        stats
    }

    /// Record a merge operation (internal use by merge algorithms)
    pub(crate) fn record_merge(&self, segments_merged: usize, edges_merged: u64, duration_ms: u64) {
        let mut stats = self.merge_stats.lock();
        stats.total_merge_operations += 1;
        stats.total_segments_merged += segments_merged as u64;
        stats.total_edges_merged += edges_merged;
        stats.total_merge_time_ms += duration_ms;

        let current_count = self.out_segments.len() + self.in_segments.len();
        stats.current_segment_count = current_count;
        if current_count > stats.max_segment_count {
            stats.max_segment_count = current_count;
        }
    }

    fn base_get_edge(
        &self,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        for segment in segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            // Time-travel optimization: skip segment if all edges were deleted before query_ts
            if segment.deletion_info.all_deleted_before(ts) {
                continue;
            }

            let Some(edge) = segment.csr.get_edge(src, dst) else {
                continue;
            };

            if edge.timestamp <= ts && !self.is_tombstoned(edge.edge_id, ts) {
                return Some(Nbr::new(
                    edge.neighbor,
                    edge.edge_id,
                    edge.prop_offset,
                    edge.timestamp,
                ));
            }
        }

        None
    }

    fn base_edges_of(&self, segments: &[CsrSegment], src: u32, ts: Timestamp) -> Vec<Nbr> {
        let mut edges = Vec::new();
        for segment in segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            // Time-travel optimization: skip segment if all edges were deleted before query_ts
            if segment.deletion_info.all_deleted_before(ts) {
                continue;
            }

            for edge in segment.csr.edges_of(src) {
                if edge.timestamp <= ts && !self.is_tombstoned(edge.edge_id, ts) {
                    edges.push(Nbr::new(
                        edge.neighbor,
                        edge.edge_id,
                        edge.prop_offset,
                        edge.timestamp,
                    ));
                }
            }
        }

        edges
    }

    fn merged_edges_of(
        &self,
        delta: &CsrVariant,
        segments: &[CsrSegment],
        src: u32,
        ts: Timestamp,
    ) -> Vec<Nbr> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for nbr in delta.edges_of(src, ts) {
            if !self.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
                result.push(nbr);
            }
        }

        for nbr in self.base_edges_of(segments, src, ts) {
            if seen.insert(nbr.edge_id) {
                result.push(nbr);
            }
        }

        result
    }

    fn merged_get_edge(
        &self,
        delta: &CsrVariant,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        if let Some(nbr) = delta.get_edge(src, dst, ts) {
            if !self.is_tombstoned(nbr.edge_id, ts) {
                return Some(nbr);
            }
        }

        self.base_get_edge(segments, src, dst, ts)
    }

    fn edge_record_from_nbr(&self, src: u32, nbr: Nbr) -> EdgeRecord {
        let (dst_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
        EdgeRecord {
            src_vid: VertexId::from_int64(src as i64),
            dst_vid,
            rank,
            properties: self.properties_for_offset(nbr.prop_offset),
        }
    }

    fn properties_for_offset(&self, prop_offset: u32) -> Vec<(String, Value)> {
        if prop_offset == 0 {
            return Vec::new();
        }

        self.properties
            .get(prop_offset)
            .map(|props| {
                props
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|v| (k, v)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Validate segment integrity by checking stored checksums
    ///
    /// Returns number of segments with valid checksums.
    /// Corrupted segments are identified but not repaired (see recover_segment).
    pub fn validate_segment_integrity(&self) -> usize {
        let mut valid_count = 0;

        for segment in &self.out_segments {
            if segment.version.validate(segment) {
                valid_count += 1;
            }
        }

        for segment in &self.in_segments {
            if segment.version.validate(segment) {
                valid_count += 1;
            }
        }

        valid_count
    }

    /// Get segment version information for recovery purposes
    ///
    /// Returns a list of (index, version) tuples for all segments.
    /// Useful for understanding which segments may be corrupted.
    pub fn segment_versions(&self) -> Vec<(usize, u32, u32)> {
        let mut versions = Vec::new();

        for (idx, seg) in self.out_segments.iter().enumerate() {
            versions.push((idx, seg.version.version, seg.version.checksum));
        }

        for (idx, seg) in self.in_segments.iter().enumerate() {
            versions.push((idx + 1000, seg.version.version, seg.version.checksum));
        }

        versions
    }

    /// Recompute checksums for all segments
    ///
    /// Call this after recovery or validation operations to update checksums.
    pub fn update_segment_checksums(&mut self) {
        for segment in &mut self.out_segments {
            segment.version.checksum = SegmentVersion::compute_checksum(segment);
            segment.version.increment();
        }

        for segment in &mut self.in_segments {
            segment.version.checksum = SegmentVersion::compute_checksum(segment);
            segment.version.increment();
        }
    }
}

impl EdgeTable {
    pub fn insert_edge(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        property_values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.schema.oe_strategy == EdgeStrategy::None {
            return Err(StorageError::invalid_operation(
                "Cannot insert edge: out-edge strategy is None".to_string(),
            ));
        }

        let mut converted_values: Vec<(String, Value)> = Vec::with_capacity(property_values.len());
        for (name, value) in property_values {
            let prop_def = self
                .schema
                .properties
                .iter()
                .find(|p| p.name == *name)
                .ok_or_else(|| StorageError::column_not_found(name.clone()))?;

            if value.data_type() != prop_def.data_type {
                let converted = value.try_cast_to(&prop_def.data_type)?;
                converted_values.push((name.clone(), converted));
            } else {
                converted_values.push((name.clone(), value.clone()));
            }
        }

        let prop_offset = if !converted_values.is_empty() {
            self.properties.insert(&converted_values)?
        } else {
            0
        };

        if self.has_edge(src, dst, rank, ts) {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                src, dst, rank
            )));
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let src_key = Self::edge_endpoint_key(src, rank);

        let edge_id = self.next_edge_id.fetch_add();
        if !self
            .out_csr
            .insert_edge(src, dst_key, edge_id, prop_offset, ts)
        {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                src, dst, rank
            )));
        }

        // in_csr.insert_edge safely returns false if strategy is None
        if !self
            .in_csr
            .insert_edge(dst, src_key, edge_id, prop_offset, ts)
        {
            self.out_csr.delete_edge(src, edge_id, ts);
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                dst, src, rank
            )));
        }

        Ok(())
    }

    pub fn delete_edge(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let src_key = Self::edge_endpoint_key(src, rank);

        if let Some(nbr) = self.out_csr.get_edge(src, dst_key, ts) {
            let edge_id = nbr.edge_id;

            self.out_csr.delete_edge(src, edge_id, ts);
            self.in_csr.delete_edge_by_dst(dst, src_key, ts);

            return Ok(true);
        }

        if let Some(nbr) = self.base_get_edge(&self.out_segments, src, dst_key, ts) {
            let edge_id = nbr.edge_id;
            self.tombstones.insert(edge_id, ts);
            // Sync: also mark the reverse edge in in_segments as deleted
            self.sync_delete_in_segment(dst, src_key, edge_id, ts);
            return Ok(true);
        }

        Ok(false)
    }

    /// Synchronize deletion in in_segments when deleting from out_segments.
    /// Marks all edges with the same edge_id in in_segments as deleted via tombstone.
    fn sync_delete_in_segment(
        &self,
        dst: u32,
        src_key: VertexId,
        _edge_id: EdgeId,
        ts: Timestamp,
    ) {
        // Verify the edge exists in in_segments (for debug/logging)
        let _found = self.base_get_edge(&self.in_segments, dst, src_key, ts).is_some();
        // The tombstone we just added will cause both out_segments and in_segments
        // to return None for this edge_id, maintaining consistency.
        // This is safe because is_tombstoned checks the tombstones map.
    }

    pub fn delete_edge_by_offset(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        if self.out_csr.get_edge(src, dst_key, ts).is_some() {
            self.out_csr.delete_edge_by_offset(src, oe_offset, ts);
            self.in_csr.delete_edge_by_offset(dst, ie_offset, ts);

            return Ok(true);
        }

        Ok(false)
    }

    pub fn revert_delete_edge_by_offset(
        &mut self,
        src: u32,
        dst: u32,
        _rank: i64,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let reverted = self.out_csr.revert_delete_by_offset(src, oe_offset, ts);

        if reverted {
            self.in_csr.revert_delete_by_offset(dst, ie_offset, ts);
        }

        Ok(reverted)
    }

    pub fn get_edge(&self, src: u32, dst: u32, rank: i64, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let nbr = self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)?;
        let properties = self.properties_for_offset(nbr.prop_offset);

        Some(EdgeRecord {
            src_vid: VertexId::from_int64(src as i64),
            dst_vid: VertexId::from_int64(dst as i64),
            rank,
            properties,
        })
    }

    pub fn out_edges(&self, src: u32, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.merged_edges_of(&self.out_csr, &self.out_segments, src, ts)
            .into_iter()
            .map(|nbr| {
                let (dst_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
                let properties = self.properties_for_offset(nbr.prop_offset);

                EdgeRecord {
                    src_vid: VertexId::from_int64(src as i64),
                    dst_vid,
                    rank,
                    properties,
                }
            })
            .collect()
    }

    pub fn in_edges(&self, dst: u32, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.merged_edges_of(&self.in_csr, &self.in_segments, dst, ts)
            .into_iter()
            .map(|nbr| {
                let (src_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
                let properties = self.properties_for_offset(nbr.prop_offset);

                EdgeRecord {
                    src_vid,
                    dst_vid: VertexId::from_int64(dst as i64),
                    rank,
                    properties,
                }
            })
            .collect()
    }

    pub fn has_edge(&self, src: u32, dst: u32, rank: i64, ts: Timestamp) -> bool {
        if !self.is_open {
            return false;
        }
        let dst_key = Self::edge_endpoint_key(dst, rank);
        self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)
            .is_some()
    }

    pub fn edge_count(&self) -> u64 {
        self.out_csr.edge_count()
            + self
                .out_segments
                .iter()
                .map(|segment| {
                    segment
                        .csr
                        .iter()
                        .filter(|(_, edge)| !self.is_tombstoned(edge.edge_id, u32::MAX))
                        .count() as u64
                })
                .sum::<u64>()
    }

    /// Get count of edges in mutable delta only (excluding frozen segments).
    /// Used by background freeze to decide when to trigger freezing.
    pub fn delta_edge_count(&self) -> u64 {
        self.out_csr.edge_count() + self.in_csr.edge_count()
    }

    pub fn scan(&self, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.iter(ts).collect()
    }

    pub fn add_property(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.properties.has_property(&name) {
            return Err(StorageError::column_already_exists(name));
        }

        self.properties
            .add_property(name.clone(), data_type.clone(), nullable);
        self.schema
            .properties
            .push(StoragePropertyDef::new(name, data_type));
        Ok(())
    }

    pub fn remove_property(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;

        self.schema.properties.remove(index);
        self.properties.remove_property(name)?;
        Ok(())
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self
            .schema
            .properties
            .iter()
            .any(|prop| prop.name == new_name)
        {
            return Err(StorageError::column_already_exists(new_name.to_string()));
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == old_name)
            .ok_or_else(|| StorageError::column_not_found(old_name.to_string()))?;

        self.schema.properties[index].name = new_name.to_string();
        self.properties.rename_property(old_name, new_name)?;
        Ok(())
    }

    pub fn update_edge_property(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        if let Some(nbr) = self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)
        {
            self.properties
                .set_property(nbr.prop_offset, prop_name, Some(value.clone()))?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn update_edge_property_by_offset(
        &mut self,
        params: UpdateEdgePropertyByOffsetParams,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(params.dst, params.rank);
        if let Some(nbr) = self.merged_get_edge(
            &self.out_csr,
            &self.out_segments,
            params.src,
            dst_key,
            params.ts,
        ) {
            self.properties.set_property_by_id(
                nbr.prop_offset,
                PropertyId(params.prop_id),
                Some(params.value.clone()),
            )?;

            let src_key = Self::edge_endpoint_key(params.src, params.rank);
            if let Some(ie_nbr) = self.merged_get_edge(
                &self.in_csr,
                &self.in_segments,
                params.dst,
                src_key,
                params.ts,
            ) {
                if nbr.prop_offset != ie_nbr.prop_offset {
                    return Err(StorageError::data_corruption(
                        format!(
                            "property offset mismatch: out_csr={}, in_csr={} at edge ({}, {})",
                            nbr.prop_offset, ie_nbr.prop_offset, params.src, params.dst
                        ),
                    ));
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    pub fn label(&self) -> LabelId {
        self.label
    }

    pub fn src_label(&self) -> LabelId {
        self.src_label
    }

    pub fn dst_label(&self) -> LabelId {
        self.dst_label
    }

    pub fn label_name(&self) -> &str {
        &self.label_name
    }

    pub fn schema(&self) -> &EdgeSchema {
        &self.schema
    }

    pub fn set_schema(&mut self, schema: EdgeSchema) {
        self.schema = schema;
    }

    pub fn iter(&self, ts: Timestamp) -> EdgeTableScanIterator<'_> {
        EdgeTableScanIterator::new(self, ts)
    }

    /// Conditionally compact CSR before serialization if fragmentation exceeds threshold.
    /// Call this before `flush()` to reduce serialization size.
    ///
    /// # Arguments
    /// - `ts`: Timestamp for visibility filtering during compaction
    /// - `threshold`: Fragmentation ratio limit (default recommendation: 2.0)
    ///
    /// # Example
    /// ```ignore
    /// table.maybe_compact_for_flush(current_ts, 2.0);
    /// table.flush(&path, compression)?;
    /// ```
    pub fn maybe_compact_for_flush(&mut self, ts: Timestamp, threshold: f32) {
        const RESERVE_RATIO: f32 = 0.25;
        if self.out_csr.fragmentation_ratio() > threshold {
            self.out_csr.compact_with_ts(ts, RESERVE_RATIO);
        }
        if self.in_csr.fragmentation_ratio() > threshold {
            self.in_csr.compact_with_ts(ts, RESERVE_RATIO);
        }
    }

    pub fn flush<P: AsRef<Path>>(
        &self,
        path: P,
        compression: crate::storage::compression::CompressionType,
    ) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;
        write_header_to(&mut meta_file, section::EDGE_META).map_err(|e| {
            StorageError::io_error(format!("Failed to write edge meta header: {}", e))
        })?;

        // Write version for forward/backward compatibility
        meta_file.write_all(&EDGE_META_VERSION.to_le_bytes())?;

        meta_file.write_all(&self.label.to_le_bytes())?;
        meta_file.write_all(&self.src_label.to_le_bytes())?;
        meta_file.write_all(&self.dst_label.to_le_bytes())?;

        let label_name_bytes = self.label_name.as_bytes();
        meta_file.write_all(&(label_name_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(label_name_bytes)?;

        let is_open_flag: u8 = if self.is_open { 1 } else { 0 };
        meta_file.write_all(&is_open_flag.to_le_bytes())?;

        let schema_json = serde_json::to_string(&self.schema)
            .map_err(|e| StorageError::serialize_error(e.to_string()))?;
        let schema_bytes = schema_json.as_bytes();
        meta_file.write_all(&(schema_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(schema_bytes)?;

        meta_file.write_all(&self.next_edge_id.0.to_le_bytes())?;
        meta_file.write_all(&(self.tombstones.len() as u64).to_le_bytes())?;
        for (edge_id, delete_ts) in &self.tombstones {
            meta_file.write_all(&edge_id.0.to_le_bytes())?;
            meta_file.write_all(&delete_ts.to_le_bytes())?;
        }
        meta_file.write_all(&self.min_active_snapshot_ts.to_le_bytes())?;


        drop(meta_file);
        crate::storage::compression::compress_file_inplace(&meta_path, compression)?;

        let out_csr_path = path.join("out_csr.bin");
        self.flush_csr(
            &self.out_csr,
            &self.out_segments,
            &out_csr_path,
            section::EDGE_OUT_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&out_csr_path, compression)?;

        let in_csr_path = path.join("in_csr.bin");
        self.flush_csr(
            &self.in_csr,
            &self.in_segments,
            &in_csr_path,
            section::EDGE_IN_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&in_csr_path, compression)?;

        let props_path = path.join("properties.bin");
        self.flush_properties(&props_path)?;
        crate::storage::compression::compress_file_inplace(&props_path, compression)?;

        Ok(())
    }

    fn flush_csr(
        &self,
        csr: &CsrVariant,
        segments: &[CsrSegment],
        path: &Path,
        section_id: u32,
    ) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section_id)
            .map_err(|e| StorageError::io_error(format!("Failed to write CSR header: {}", e)))?;

        let data = csr.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;
        file.write_all(&(segments.len() as u64).to_le_bytes())?;
        for segment in segments {
            file.write_all(&segment.create_ts_min.to_le_bytes())?;
            file.write_all(&segment.create_ts_max.to_le_bytes())?;
            let (delete_ts_min, delete_ts_max) = segment.deletion_range();
            file.write_all(&delete_ts_min.to_le_bytes())?;
            file.write_all(&delete_ts_max.to_le_bytes())?;
            let data = segment.csr.dump();
            file.write_all(&(data.len() as u64).to_le_bytes())?;
            file.write_all(&data)?;
        }

        Ok(())
    }

    fn flush_properties(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::EDGE_PROPERTIES).map_err(|e| {
            StorageError::io_error(format!("Failed to write properties header: {}", e))
        })?;

        let data = self.properties.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let meta_data = crate::storage::compression::read_decompressed(&meta_path)?;
        let mut meta_cursor = &meta_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        meta_cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_META {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge meta: expected {:#06x}, got {:#06x}",
                    section::EDGE_META,
                    sid
                )));
            }
        }

        // Read and validate version (must be 2)
        let mut version_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 2 {
            return Err(StorageError::deserialize_error(format!(
                "unsupported edge meta version: {} (only v2 is supported)",
                version
            )));
        }

        Self::load_metadata(&mut meta_cursor, self)?;

        let out_csr_path = path.join("out_csr.bin");
        Self::load_csr_static(&mut self.out_csr, &mut self.out_segments, &out_csr_path)?;

        let in_csr_path = path.join("in_csr.bin");
        Self::load_csr_static(&mut self.in_csr, &mut self.in_segments, &in_csr_path)?;

        let props_path = path.join("properties.bin");
        self.load_properties(&props_path)?;

        if self.next_edge_id.0 == 0 {
            let ts = u32::MAX;
            let max_id = self
                .out_csr
                .iter(ts)
                .map(|(_, nbr)| nbr.edge_id.0 + 1)
                .chain(
                    self.out_segments
                        .iter()
                        .flat_map(|segment| segment.csr.iter().map(|(_, nbr)| nbr.edge_id.0 + 1)),
                )
                .max()
                .unwrap_or(0);
            self.next_edge_id = EdgeId(max_id);
        }
        self.is_open = true;
        Ok(())
    }

    /// Load edge table metadata from serialized bytes
    fn load_metadata(cursor: &mut &[u8], table: &mut Self) -> StorageResult<()> {
        use std::io::Read;

        let mut label_bytes = [0u8; 4];
        cursor.read_exact(&mut label_bytes)?;
        table.label = u32::from_le_bytes(label_bytes);

        let mut src_label_bytes = [0u8; 4];
        cursor.read_exact(&mut src_label_bytes)?;
        table.src_label = u32::from_le_bytes(src_label_bytes);

        let mut dst_label_bytes = [0u8; 4];
        cursor.read_exact(&mut dst_label_bytes)?;
        table.dst_label = u32::from_le_bytes(dst_label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        cursor.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        cursor.read_exact(&mut label_name_bytes)?;
        table.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut is_open_bytes = [0u8; 1];
        cursor.read_exact(&mut is_open_bytes)?;
        table.is_open = is_open_bytes[0] != 0;

        let mut schema_len_bytes = [0u8; 4];
        cursor.read_exact(&mut schema_len_bytes)?;
        let schema_len = u32::from_le_bytes(schema_len_bytes) as usize;
        let mut schema_bytes = vec![0u8; schema_len];
        cursor.read_exact(&mut schema_bytes)?;
        let schema_json = String::from_utf8(schema_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
        table.schema = serde_json::from_str(&schema_json)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut next_edge_id_bytes = [0u8; 8];
        cursor.read_exact(&mut next_edge_id_bytes)?;
        table.next_edge_id = EdgeId(u64::from_le_bytes(next_edge_id_bytes));

        let mut tombstone_count_bytes = [0u8; 8];
        cursor.read_exact(&mut tombstone_count_bytes)?;
        let tombstone_count = u64::from_le_bytes(tombstone_count_bytes) as usize;
        table.tombstones.clear();
        for _ in 0..tombstone_count {
            let mut edge_id_bytes = [0u8; 8];
            cursor.read_exact(&mut edge_id_bytes)?;
            let mut delete_ts_bytes = [0u8; 4];
            cursor.read_exact(&mut delete_ts_bytes)?;
            table.tombstones.insert(
                EdgeId(u64::from_le_bytes(edge_id_bytes)),
                u32::from_le_bytes(delete_ts_bytes),
            );
        }

        // Load min_active_snapshot_ts (new in version 2)
        let mut min_snapshot_ts_bytes = [0u8; 4];
        cursor.read_exact(&mut min_snapshot_ts_bytes)?;
        table.min_active_snapshot_ts = u32::from_le_bytes(min_snapshot_ts_bytes);

        Ok(())
    }

    /// Load metadata in version 1 format (without version field and min_active_snapshot_ts)
    /// Load edge table CSR segments from disk
    fn load_csr_static(
        csr: &mut CsrVariant,
        segments: &mut Vec<CsrSegment>,
        path: &Path,
    ) -> StorageResult<()> {
        use std::io::Read;

        let raw_data = crate::storage::compression::read_decompressed(path)?;
        let mut cursor = &raw_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_OUT_CSR && sid != section::EDGE_IN_CSR {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge CSR: expected {:#06x} or {:#06x}, got {:#06x}",
                    section::EDGE_OUT_CSR,
                    section::EDGE_IN_CSR,
                    sid
                )));
            }
        }

        let mut len_bytes = [0u8; 8];
        cursor.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        cursor.read_exact(&mut data)?;

        csr.load(&data)?;
        segments.clear();

        let mut segment_count_bytes = [0u8; 8];
        cursor.read_exact(&mut segment_count_bytes)?;
        let segment_count = u64::from_le_bytes(segment_count_bytes) as usize;
        for _ in 0..segment_count {
            let mut create_ts_min_bytes = [0u8; 4];
            cursor.read_exact(&mut create_ts_min_bytes)?;
            let create_ts_min = u32::from_le_bytes(create_ts_min_bytes);

            let mut create_ts_max_bytes = [0u8; 4];
            cursor.read_exact(&mut create_ts_max_bytes)?;
            let create_ts_max = u32::from_le_bytes(create_ts_max_bytes);

            let mut delete_ts_min_bytes = [0u8; 4];
            cursor.read_exact(&mut delete_ts_min_bytes)?;
            let delete_ts_min = u32::from_le_bytes(delete_ts_min_bytes);

            let mut delete_ts_max_bytes = [0u8; 4];
            cursor.read_exact(&mut delete_ts_max_bytes)?;
            let delete_ts_max = u32::from_le_bytes(delete_ts_max_bytes);

            let mut segment_len_bytes = [0u8; 8];
            cursor.read_exact(&mut segment_len_bytes)?;
            let segment_len = u64::from_le_bytes(segment_len_bytes) as usize;

            let mut segment_data = vec![0u8; segment_len];
            cursor.read_exact(&mut segment_data)?;

            let mut segment_csr = Csr::new();
            segment_csr.load(&segment_data)?;
            let deletion_info = DeletionInfo::new(delete_ts_min, delete_ts_max);
            segments.push(CsrSegment::new(
                segment_csr,
                create_ts_min,
                create_ts_max,
                deletion_info,
            ));
        }

        Ok(())
    }

    fn load_properties(&mut self, path: &Path) -> StorageResult<()> {
        use std::io::Read;

        let raw_data = crate::storage::compression::read_decompressed(path)?;
        let mut cursor = &raw_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_PROPERTIES {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge properties: expected {:#06x}, got {:#06x}",
                    section::EDGE_PROPERTIES,
                    sid
                )));
            }
        }

        let mut len_bytes = [0u8; 8];
        cursor.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        cursor.read_exact(&mut data)?;

        self.properties.load(&data)?;

        Ok(())
    }

    /// Compact CSR only (physical space reclamation + timestamp filtering).
    ///
    /// Removes overflow block fragmentation and edges with timestamps > ts.
    /// Does NOT freeze delta to immutable segments.
    /// See `freeze_csr_only()` and `compact_and_freeze_with_config()` for related operations.
    pub fn compact_csr_only(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        self.out_csr.compact_with_ts(ts, reserve_ratio)
            + self.in_csr.compact_with_ts(ts, reserve_ratio)
    }

    /// Freeze CSR only (convert mutable delta to immutable segment).
    ///
    /// Converts visible edges (ts <= query_ts) to immutable CSR and records
    /// timestamp ranges [create_ts_min, create_ts_max] and [delete_ts_min, delete_ts_max]
    /// for time-travel queries and MVCC support.
    /// Clears mutable delta after freezing.
    /// Does NOT perform physical compaction.
    /// See `compact_csr_only()` and `compact_and_freeze_with_config()` for related operations.
    pub fn freeze_csr_only(&mut self, ts: Timestamp) -> usize {
        let out_frozen = Self::freeze_delta(&mut self.out_csr, &mut self.out_segments, ts, &self.tombstones);
        let in_frozen = Self::freeze_delta(&mut self.in_csr, &mut self.in_segments, ts, &self.tombstones);

        // Check segment counts and trigger aggressive merge if needed
        if self.out_segments.len() >= self.config.max_segments_per_direction {
            let _ = Self::merge_segments_aggressive(&mut self.out_segments, 8 * 1024 * 1024);
        }
        if self.in_segments.len() >= self.config.max_segments_per_direction {
            let _ = Self::merge_segments_aggressive(&mut self.in_segments, 8 * 1024 * 1024);
        }

        out_frozen + in_frozen
    }

    /// Compact and freeze in sequence with adaptive configuration.
    ///
    /// Combines physical compaction (space reclamation) with logical versioning:
    /// 1. Compute reserve_ratio dynamically based on table metrics and strategy
    /// 2. Compact: eliminate overflow fragmentation and remove old edges
    /// 3. Freeze: convert to immutable segment with timestamp range
    /// 4. Merge: optionally combine nearby segments to reduce lookup overhead
    /// 5. Cleanup: remove orphaned properties
    ///
    /// Supports both fixed and adaptive reserve_ratio strategies.
    /// Optionally enables segment merging if configured.
    /// Supports adaptive merge triggered by high tombstone memory pressure.
    /// This is the preferred method for production checkpoint maintenance.
    pub fn compact_and_freeze_with_config(&mut self, ts: Timestamp, config: &CompactConfig) -> usize {
        let edge_count = self.edge_count() as usize;
        let reserve_ratio = config.compute_reserve_ratio(edge_count, 0);

        let removed = self.compact_csr_only(ts, reserve_ratio);
        self.freeze_csr_only(ts);

        if config.segment_merge_enabled {
            let stats = self.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);

        // Record tombstone statistics to metrics
        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.active_snapshots.len() as u64,
            );
        }

        removed
    }

    /// Compact, freeze, and perform automatic tombstone GC in sequence.
    ///
    /// This is the preferred method over `compact_and_freeze_with_gc()` because it
    /// internally computes the correct min_active_snapshot_ts instead of requiring
    /// the caller to pass it (which is error-prone).
    ///
    /// The sequence is:
    /// 1. Compact: eliminate overflow fragmentation and remove old edges
    /// 2. Freeze: convert mutable delta to immutable segment
    /// 3. Merge: optionally combine nearby segments
    /// 4. GC: remove tombstones older than min_active_snapshot_ts (computed internally)
    /// 5. Update metrics
    ///
    /// # Arguments
    /// - `ts`: Current timestamp (for visibility filtering)
    /// - `config`: Configuration for compaction and merging
    ///
    /// # Returns
    /// Number of edges removed during compaction
    pub fn compact_and_freeze_with_auto_gc(
        &mut self,
        ts: Timestamp,
        config: &CompactConfig,
    ) -> usize {
        let edge_count = self.edge_count() as usize;
        let reserve_ratio = config.compute_reserve_ratio(edge_count, 0);

        let removed = self.compact_csr_only(ts, reserve_ratio);
        self.freeze_csr_only(ts);

        if config.segment_merge_enabled {
            let stats = self.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);

        // Automatically compute and apply GC
        let min_active_snapshot_ts = self.get_min_active_snapshot_ts();
        self.gc_tombstones(min_active_snapshot_ts);

        // Record tombstone statistics to metrics
        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.active_snapshots.len() as u64,
            );
        }

        removed
    }

    /// Compact, freeze, and perform tombstone GC in sequence.
    ///
    /// # Deprecated
    /// Prefer `compact_and_freeze_with_auto_gc()` which computes min_ts automatically.
    /// This method is kept for compatibility but is error-prone when the caller
    /// passes incorrect min_active_snapshot_ts.
    pub fn compact_and_freeze_with_gc(
        &mut self,
        ts: Timestamp,
        config: &CompactConfig,
        min_active_snapshot_ts: Timestamp,
    ) -> usize {
        // Verify the provided min_ts matches reality (safety check)
        let actual_min_ts = self.get_min_active_snapshot_ts();
        debug_assert_eq!(
            min_active_snapshot_ts, actual_min_ts,
            "Provided min_active_snapshot_ts ({}) doesn't match actual ({}). \
             This indicates the caller is not properly tracking active snapshots.",
            min_active_snapshot_ts, actual_min_ts
        );

        let edge_count = self.edge_count() as usize;
        let reserve_ratio = config.compute_reserve_ratio(edge_count, 0);

        let removed = self.compact_csr_only(ts, reserve_ratio);
        self.freeze_csr_only(ts);

        if config.segment_merge_enabled {
            let stats = self.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);
        self.gc_tombstones(min_active_snapshot_ts);

        // Record tombstone statistics to metrics
        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.active_snapshots.len() as u64,
            );
        }

        removed
    }


    /// Export a read-only snapshot of this edge table at the given timestamp.
    ///
    /// Creates an immutable copy of all edges visible at `ts`, combining:
    /// - Mutable delta (out_csr, in_csr) - most recent edges
    /// - Historical segments (out_segments, in_segments) - older edges
    ///
    /// # MVCC Semantics
    /// An edge is visible at timestamp `ts` if and only if:
    /// - create_ts <= ts  (edge has been created by timestamp)
    /// - delete_ts > ts   (edge has NOT been deleted by timestamp, or never deleted)
    ///
    /// This provides consistent snapshot isolation: each snapshot is a frozen point-in-time
    /// view where all edges are fully consistent with respect to the transaction at timestamp ts.
    ///
    /// # Algorithm
    /// 1. Collect edges from segments in reverse order (oldest to newest)
    /// 2. Apply MVCC visibility filter: keep only edges where create_ts <= ts < delete_ts
    /// 3. Merge with mutable CSR edges (delta overwrites segment versions)
    /// 4. Build immutable CSR for both out and in edges
    /// 5. Return snapshot with properties and schema
    ///
    /// # Performance
    /// - Time: O(V + E) - linear scan of vertices and edges
    /// - Space: O(E) - temporary buffer for edge collection
    ///
    /// # Arguments
    /// - `ts`: Timestamp to snapshot (snapshot isolation point)
    ///
    /// # Returns
    /// A frozen snapshot suitable for backup, analysis, or time-travel queries
    pub fn export_snapshot(&self, ts: Timestamp) -> StorageResult<ExportedEdgeSnapshot> {
        // Collect edges for both directions with MVCC visibility filtering
        let out_edges = self.collect_edges_for_snapshot_mvcc(&self.out_csr, &self.out_segments, ts)?;
        let in_edges = self.collect_edges_for_snapshot_mvcc(&self.in_csr, &self.in_segments, ts)?;

        // Build immutable CSRs from collected edges
        let out_csr = Self::build_immutable_csr_from_edges(out_edges, self.out_csr.vertex_capacity())?;
        let in_csr = Self::build_immutable_csr_from_edges(in_edges, self.in_csr.vertex_capacity())?;

        Ok(ExportedEdgeSnapshot {
            snapshot_ts: ts,
            label: self.label,
            out_csr,
            in_csr,
            properties: self.properties.clone(),
            schema: self.schema.clone(),
        })
    }

    /// Collect edges visible at timestamp from delta and segments with MVCC filtering.
    ///
    /// MVCC visibility rule: An edge is included if:
    /// - create_ts <= ts  (edge has been created by timestamp)
    /// - delete_ts > ts   (edge has not been deleted by timestamp, or u32::MAX if never deleted)
    ///
    /// Merges edges from:
    /// 1. Historical segments (in reverse order for proper time ordering)
    /// 2. Mutable delta CSR (overrides segment versions with MVCC filtering)
    ///
    /// Uses HashMap deduplication to handle edges updated in both segments and delta.
    /// Also optimizes using DeletionInfo fast paths for completely deleted segments.
    fn collect_edges_for_snapshot_mvcc(
        &self,
        delta: &CsrVariant,
        segments: &[CsrSegment],
        ts: Timestamp,
    ) -> StorageResult<Vec<(u32, Nbr)>> {
        use std::collections::HashMap;

        // Dedup map: (src_vid, edge_id) -> (src_vid, nbr)
        // This ensures latest version of each edge is used
        let mut edge_map: HashMap<(u32, EdgeId), (u32, Nbr)> = HashMap::new();
        let mut _segments_skipped_by_deletion = 0usize;

        // Step 1: Collect from segments in reverse order (older first, newer last)
        for segment in segments.iter().rev() {
            // Skip segment if created in the future
            if segment.create_ts_min > ts {
                continue;
            }

            // Fast path: if all edges in segment are deleted before query_ts, skip entire segment
            // This optimization uses DeletionInfo.deleted_count to avoid scanning edges
            if segment.deletion_info.all_deleted_before(ts)
                && segment.deletion_info.all_edges_deleted(segment.csr.edge_count()) {
                _segments_skipped_by_deletion += 1;
                continue;
            }

            for (src, immutable_nbr) in segment.csr.iter() {
                // Skip edge if created after ts
                if immutable_nbr.timestamp > ts {
                    continue;
                }

                // MVCC filter: check if edge was deleted by timestamp ts
                // If edge is in tombstones, it was logically deleted
                if let Some(&delete_ts) = self.tombstones.get(&immutable_nbr.edge_id) {
                    // Edge is deleted at delete_ts; only include if delete_ts > ts
                    if delete_ts <= ts {
                        continue;  // Edge was already deleted by ts
                    }
                } else {
                    // Edge not in tombstones; check segment's deletion info as hint
                    // If segment.deletion_info suggests all deletions happened before ts
                    // (This is advisory; actual delete status is in tombstones map)
                    if segment.deletion_info.all_deleted_before(ts) {
                        // Segment hint suggests edges might be deleted, but verify with tombstones
                        // Since we didn't find it above, it's NOT deleted -> include it
                    }
                }

                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                let nbr = Nbr::new(
                    immutable_nbr.neighbor,
                    immutable_nbr.edge_id,
                    immutable_nbr.prop_offset,
                    immutable_nbr.timestamp,
                );
                edge_map.insert((src_u32, immutable_nbr.edge_id), (src_u32, nbr));
            }
        }

        // Step 2: Collect from mutable CSR delta (overrides segment versions)
        for (src, nbr) in delta.iter(ts) {
            let src_u32 = src.as_int64().unwrap_or(0) as u32;

            // MVCC filter: check if edge was deleted
            if let Some(&delete_ts) = self.tombstones.get(&nbr.edge_id) {
                // Edge is marked as deleted at delete_ts
                if delete_ts <= ts {
                    continue;  // Skip deleted edge
                }
            }

            edge_map.insert((src_u32, nbr.edge_id), (src_u32, nbr));
        }

        // Step 3: Convert to sorted vector
        let mut edges: Vec<_> = edge_map.into_values().collect();
        edges.sort_by_key(|(src, _)| *src);

        Ok(edges)
    }

    /// Build an immutable CSR from a list of edges.
    ///
    /// Edges must be sorted by source vertex for correct offset computation.
    fn build_immutable_csr_from_edges(
        edges: Vec<(u32, Nbr)>,
        vertex_capacity: usize,
    ) -> StorageResult<ImmutableCsr> {
        if edges.is_empty() {
            return Ok(ImmutableCsr::builder(vertex_capacity).build());
        }

        let mut builder = ImmutableCsr::builder(vertex_capacity);

        for (src, nbr) in edges {
            builder.batch_put_edge(src, nbr.neighbor, nbr.edge_id, nbr.prop_offset);
        }

        Ok(builder.build())
    }

    fn freeze_delta(
        delta: &mut CsrVariant,
        segments: &mut Vec<CsrSegment>,
        ts: Timestamp,
        tombstones: &HashMap<EdgeId, Timestamp>,
    ) -> usize {
        let entries: Vec<_> = delta
            .iter(ts)
            .map(|(src, nbr)| {
                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                (src_u32, nbr)
            })
            .collect();
        if entries.is_empty() {
            delta.clear();
            return 0;
        }

        // Validate that all vertex IDs fit within capacity.
        // This prevents off-by-one errors in CSR offset/degree array indexing.
        let max_vid = entries
            .iter()
            .map(|(src, nbr)| {
                let nbr_id = nbr.neighbor.as_int64().unwrap_or(0) as usize;
                std::cmp::max(*src as usize, nbr_id)
            })
            .max()
            .unwrap_or(0);
        let vertex_capacity = delta.vertex_capacity();
        assert!(
            max_vid < vertex_capacity,
            "Vertex ID {} exceeds capacity {} during freeze",
            max_vid,
            vertex_capacity
        );

        let create_ts_min = entries
            .iter()
            .map(|(_, nbr)| nbr.create_ts)
            .min()
            .unwrap_or(0);
        let create_ts_max = entries
            .iter()
            .map(|(_, nbr)| nbr.create_ts)
            .max()
            .unwrap_or(0);

        // Compute deletion timestamp range from tombstones of frozen edges.
        // For MVCC support, track when edges in this segment were deleted.
        let mut deleted_count = 0u32;
        let (delete_ts_min, delete_ts_max) = entries
            .iter()
            .filter_map(|(_, nbr)| {
                tombstones.get(&nbr.edge_id).map(|&ts| {
                    deleted_count += 1;
                    ts
                })
            })
            .fold((u32::MAX, 0), |(min, max), ts| {
                (std::cmp::min(min, ts), std::cmp::max(max, ts))
            });

        let csr = Csr::from_nbr_entries(&entries, vertex_capacity);
        let frozen = entries.len();

        let deletion_info = DeletionInfo::with_count(delete_ts_min, delete_ts_max, deleted_count);
        segments.push(CsrSegment::new(
            csr,
            create_ts_min,
            create_ts_max,
            deletion_info,
        ));
        delta.clear();
        frozen
    }

    /// LSM-style tiered merge strategy
    ///
    /// Organizes segments into levels based on size and merges within/across levels:
    /// - L0: < 1MB (fresh from freeze)
    /// - L1: 1-8MB
    /// - L2: 8-32MB
    /// - L3+: > 32MB
    ///
    /// When a level has too many segments, merges them up to the next level.
    /// This prevents unbounded segment count while keeping query performance good.
    pub fn merge_segments_lsm_tiered(&mut self, current_ts: Timestamp) -> usize {
        use crate::storage::engine::config::LSMSegmentLevel;

        let mut total_merged = 0usize;
        let start = std::time::Instant::now();

        // Process both directions
        for segments in [&mut self.out_segments, &mut self.in_segments].iter_mut() {
            // Group segments by level
            let mut levels: std::collections::BTreeMap<LSMSegmentLevel, Vec<usize>> =
                std::collections::BTreeMap::new();

            for (idx, segment) in segments.iter().enumerate() {
                let size = segment.estimated_bytes();
                let level = LSMSegmentLevel::for_size(size);
                levels.entry(level).or_insert_with(Vec::new).push(idx);
            }

            // Check each level for merge triggers
            for (level, indices) in &levels {
                if indices.len() >= level.merge_trigger_count() {
                    // Merge segments in this level
                    let merged = Self::merge_selected_segments(
                        segments,
                        indices.clone(),
                        current_ts,
                        level.merge_target_size(),
                    );
                    total_merged += merged;
                }
            }
        }

        if total_merged > 0 {
            let duration_ms = start.elapsed().as_millis() as u64;
            let edges_merged = (total_merged as u64 * 1000).min(self.edge_count() as u64);
            self.record_merge(total_merged, edges_merged, duration_ms);
        }

        total_merged
    }

    /// Helper: merge selected segments by indices
    fn merge_selected_segments(
        segments: &mut Vec<CsrSegment>,
        indices: Vec<usize>,
        current_ts: Timestamp,
        _target_size: usize,
    ) -> usize {
        if indices.len() <= 1 {
            return 0;
        }

        // Sort indices in reverse to avoid index shifting on removal
        let mut sorted_indices = indices;
        sorted_indices.sort_by(|a, b| b.cmp(a));
        let merge_count = sorted_indices.len();

        let mut merged_entries = Vec::new();
        let mut min_create_ts = u32::MAX;
        let mut max_create_ts = 0u32;
        let mut merged_deletion_info = DeletionInfo::NoDeletes;

        // Collect edges from all selected segments
        for idx in &sorted_indices {
            let seg = &segments[*idx];
            min_create_ts = min_create_ts.min(seg.create_ts_min);
            max_create_ts = max_create_ts.max(seg.create_ts_max);
            merged_deletion_info = merged_deletion_info.merge(&seg.deletion_info);

            for (src, immutable_nbr) in seg.csr.iter() {
                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                let nbr = Nbr::new(
                    immutable_nbr.neighbor,
                    immutable_nbr.edge_id,
                    immutable_nbr.prop_offset,
                    immutable_nbr.timestamp,
                );
                merged_entries.push((src_u32, nbr));
            }
        }

        // Create merged segment if there are entries
        if !merged_entries.is_empty() {
            let vertex_capacity = merged_entries
                .iter()
                .map(|(src, _)| *src as usize + 1)
                .max()
                .unwrap_or(1024)
                .max(1024);

            let merged_csr = Csr::from_nbr_entries(&merged_entries, vertex_capacity);
            let merged_segment = CsrSegment::with_creation_ts(
                merged_csr,
                min_create_ts,
                max_create_ts,
                merged_deletion_info,
                current_ts,
            );

            // Remove old segments
            for idx in sorted_indices {
                segments.remove(idx);
            }

            // Add merged segment
            segments.push(merged_segment);

            merge_count  // Return number of merged segments
        } else {
            0
        }
    }


    ///
    /// Prioritizes merging:
    /// 1. Older segments (age > threshold)
    /// 2. Segments with high deletion ratio (>30%)
    /// 3. Segments that can fit together
    ///
    /// This prevents accumulation of small segments while respecting
    /// size constraints and deletion patterns.
    pub fn merge_segments_adaptive(&mut self, current_ts: Timestamp, max_segment_age: Timestamp) -> usize {
        let size_threshold = 8 * 1024 * 1024;  // 8MB per direction
        let deletion_threshold = 0.3;  // 30% deletion ratio
        let start = std::time::Instant::now();

        let out_reduced = Self::merge_segments_adaptive_impl(
            &mut self.out_segments,
            current_ts,
            max_segment_age,
            deletion_threshold,
            size_threshold,
        );

        let in_reduced = Self::merge_segments_adaptive_impl(
            &mut self.in_segments,
            current_ts,
            max_segment_age,
            deletion_threshold,
            size_threshold,
        );

        let total_reduced = out_reduced + in_reduced;
        if total_reduced > 0 {
            let duration_ms = start.elapsed().as_millis() as u64;
            // Estimate edges merged (rough estimate: reduced segments × avg edges per segment)
            let avg_edges_per_segment = if total_reduced > 0 {
                let total_edges = self.edge_count();
                (total_edges as u64 / (total_reduced as u64).max(1)).max(100)
            } else {
                100
            };
            self.record_merge(total_reduced, total_reduced as u64 * avg_edges_per_segment, duration_ms);
        }

        total_reduced
    }

    /// Implementation of adaptive merge for a single direction
    fn merge_segments_adaptive_impl(
        segments: &mut Vec<CsrSegment>,
        current_ts: Timestamp,
        max_segment_age: Timestamp,
        deletion_threshold: f64,
        size_threshold: usize,
    ) -> usize {
        if segments.len() <= 1 {
            return 0;
        }

        // Compute merge priority score for each segment
        let mut scored_segments: Vec<_> = segments
            .iter()
            .enumerate()
            .map(|(idx, seg)| {
                let age = seg.age(current_ts);
                let deletion_ratio = seg.deletion_ratio();

                // Priority score: higher = should merge sooner
                // Older segments and high-deletion segments get higher scores
                let age_score = if age > max_segment_age {
                    100.0
                } else {
                    (age as f64 / max_segment_age as f64) * 100.0
                };

                let deletion_score = if deletion_ratio > deletion_threshold {
                    (deletion_ratio / 0.5) * 100.0  // Scale to 100+ for high deletion
                } else {
                    deletion_ratio * 100.0
                };

                let score = (age_score + deletion_score) / 2.0;
                (idx, score, seg.csr.edge_count())
            })
            .collect();

        // Sort by priority score (highest first)
        scored_segments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Greedily merge high-priority segments
        let mut to_merge = Vec::new();
        let mut current_size = 0usize;

        for (idx, _score, edge_count) in scored_segments {
            let estimated_size = (current_size / 30) + (edge_count as usize);

            if !to_merge.is_empty() && estimated_size > size_threshold {
                break;  // Can't fit more segments
            }

            to_merge.push(idx);
            current_size += edge_count as usize * 30;
        }

        if to_merge.len() <= 1 {
            return 0;  // Nothing to merge
        }

        // Sort indices to merge them in order
        to_merge.sort();

        // Perform the merge: collect edges from all marked segments
        let mut merged_entries = Vec::new();
        let mut min_create_ts = u32::MAX;
        let mut max_create_ts = 0u32;
        let mut merged_deletion_info = DeletionInfo::NoDeletes;

        let mut to_remove = Vec::new();

        for idx in &to_merge {
            let seg = &segments[*idx];
            min_create_ts = min_create_ts.min(seg.create_ts_min);
            max_create_ts = max_create_ts.max(seg.create_ts_max);
            merged_deletion_info = merged_deletion_info.merge(&seg.deletion_info);

            for (src, immutable_nbr) in seg.csr.iter() {
                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                let nbr = Nbr::new(
                    immutable_nbr.neighbor,
                    immutable_nbr.edge_id,
                    immutable_nbr.prop_offset,
                    immutable_nbr.timestamp,
                );
                merged_entries.push((src_u32, nbr));
            }

            to_remove.push(*idx);
        }

        // Create merged segment
        if !merged_entries.is_empty() {
            let vertex_capacity = merged_entries
                .iter()
                .map(|(src, _)| *src as usize + 1)
                .max()
                .unwrap_or(1024)
                .max(1024);

            let merged_csr = Csr::from_nbr_entries(&merged_entries, vertex_capacity);
            let merged_segment = CsrSegment::with_creation_ts(
                merged_csr,
                min_create_ts,
                max_create_ts,
                merged_deletion_info,
                current_ts,
            );

            // Remove old segments (in reverse order to avoid index shifting)
            to_remove.sort_by(|a, b| b.cmp(a));
            for idx in to_remove {
                segments.remove(idx);
            }

            // Add merged segment
            segments.push(merged_segment);

            to_merge.len()
        } else {
            0
        }
    }



    /// Merge CSR segments with time and size thresholds, returning merge metrics.
    ///
    /// Merges segments when:
    /// - Time gap between segments <= time_threshold, AND
    /// - Combined merged size <= size_threshold_bytes
    ///
    /// This two-dimensional strategy prevents unbounded segment size growth while
    /// still reducing lookup overhead by combining nearby segments.
    ///
    /// Returns a MergeMetricsResult containing both the merge metrics and the number
    /// of segments reduced.
    pub fn merge_segments_with_config(&mut self, time_threshold: Timestamp, size_threshold_bytes: usize) -> MergeMetricsResult {
        let start = Instant::now();
        let segments_before = self.out_segments.len() + self.in_segments.len();

        let out_metrics = Self::merge_segments_in_place(&mut self.out_segments, time_threshold, size_threshold_bytes);
        let in_metrics = Self::merge_segments_in_place(&mut self.in_segments, time_threshold, size_threshold_bytes);

        let segments_after = self.out_segments.len() + self.in_segments.len();
        let total_edges = out_metrics.edges_processed + in_metrics.edges_processed;
        let duration_ms = start.elapsed().as_millis() as u64;

        let metrics = MergeMetrics {
            segments_before,
            segments_after,
            edges_merged: total_edges,
            duration_ms,
        };

        MergeMetricsResult {
            metrics,
            segments_reduced: segments_before.saturating_sub(segments_after),
        }
    }

    fn merge_segments_in_place(segments: &mut Vec<CsrSegment>, time_threshold: Timestamp, size_threshold: usize) -> DirectionMergeMetrics {
        if segments.len() <= 1 {
            return DirectionMergeMetrics { edges_processed: 0 };
        }

        let mut merged = Vec::new();
        let mut current_entries = Vec::new();
        let mut total_edges = 0u64;
        let mut current_create_ts_min = segments[0].create_ts_min;
        let mut current_create_ts_max = segments[0].create_ts_max;
        let mut current_deletion_info = segments[0].deletion_info;

        for segment in segments.drain(..) {
            // Check if this segment should merge with current accumulation
            // based on BOTH time gap AND estimated size
            let time_gap = if segment.create_ts_min > current_create_ts_max {
                segment.create_ts_min - current_create_ts_max
            } else if current_create_ts_max > segment.create_ts_min {
                0
            } else {
                segment.create_ts_min - current_create_ts_max
            };

            // Estimate size using strategy-specific bytes per edge
            let total_edge_count = current_entries.len() + segment.csr.edge_count() as usize;
            let bytes_per_edge = segment.csr.bytes_per_edge();
            let estimated_size = total_edge_count * bytes_per_edge;
            let size_ok = estimated_size <= size_threshold;

            if time_gap <= time_threshold && size_ok && !current_entries.is_empty() {
                // Merge: accumulate this segment's edges
                for (src, immutable_nbr) in segment.csr.iter() {
                    let src_u32 = src.as_int64().unwrap_or(0) as u32;
                    let nbr = Nbr::new(
                        immutable_nbr.neighbor,
                        immutable_nbr.edge_id,
                        immutable_nbr.prop_offset,
                        immutable_nbr.timestamp,
                    );
                    current_entries.push((src_u32, nbr));
                }
                current_create_ts_min = current_create_ts_min.min(segment.create_ts_min);
                current_create_ts_max = current_create_ts_max.max(segment.create_ts_max);
                current_deletion_info = current_deletion_info.merge(&segment.deletion_info);
            } else {
                // No merge: flush current accumulation and start new one
                if !current_entries.is_empty() {
                    let vertex_capacity = current_entries
                        .iter()
                        .map(|(src, _)| *src as usize + 1)
                        .max()
                        .unwrap_or(1024)
                        .max(1024);

                    let merged_csr = Csr::from_nbr_entries(&current_entries, vertex_capacity);
                    total_edges += merged_csr.edge_count() as u64;
                    merged.push(CsrSegment::new(
                        merged_csr,
                        current_create_ts_min,
                        current_create_ts_max,
                        current_deletion_info,
                    ));
                    current_entries.clear();
                }

                // Start new accumulation with current segment
                for (src, immutable_nbr) in segment.csr.iter() {
                    let src_u32 = src.as_int64().unwrap_or(0) as u32;
                    let nbr = Nbr::new(
                        immutable_nbr.neighbor,
                        immutable_nbr.edge_id,
                        immutable_nbr.prop_offset,
                        immutable_nbr.timestamp,
                    );
                    current_entries.push((src_u32, nbr));
                }
                current_create_ts_min = segment.create_ts_min;
                current_create_ts_max = segment.create_ts_max;
                current_deletion_info = segment.deletion_info;
            }
        }

        // Flush remaining accumulation
        if !current_entries.is_empty() {
            let vertex_capacity = current_entries
                .iter()
                .map(|(src, _)| *src as usize + 1)
                .max()
                .unwrap_or(1024)
                .max(1024);

            let merged_csr = Csr::from_nbr_entries(&current_entries, vertex_capacity);
            total_edges += merged_csr.edge_count() as u64;
            merged.push(CsrSegment::new(
                merged_csr,
                current_create_ts_min,
                current_create_ts_max,
                current_deletion_info,
            ));
        }

        *segments = merged;
        DirectionMergeMetrics {
            edges_processed: total_edges,
        }
    }

    /// Aggressively merge segments when count exceeds limit.
    ///
    /// Unlike `merge_segments_in_place`, this method ignores time gaps and merges
    /// solely based on size constraints, making it suitable for reducing segment
    /// count when the limit is reached.
    ///
    /// Strategy: greedily merge segments from the beginning while keeping the
    /// accumulated size within the threshold.
    fn merge_segments_aggressive(segments: &mut Vec<CsrSegment>, size_threshold_bytes: usize) -> DirectionMergeMetrics {
        if segments.len() <= 1 {
            return DirectionMergeMetrics { edges_processed: 0 };
        }

        let mut merged = Vec::new();
        let mut current_entries = Vec::new();
        let mut total_edges = 0u64;
        let mut current_create_ts_min = segments[0].create_ts_min;
        let mut current_create_ts_max = segments[0].create_ts_max;
        let mut current_deletion_info = segments[0].deletion_info;

        for segment in segments.drain(..) {
            let estimated_size = (current_entries.len() + segment.csr.edge_count() as usize) * 30;
            let size_ok = estimated_size <= size_threshold_bytes;

            if size_ok && !current_entries.is_empty() {
                // Merge: accumulate this segment's edges
                for (src, immutable_nbr) in segment.csr.iter() {
                    let src_u32 = src.as_int64().unwrap_or(0) as u32;
                    let nbr = Nbr::new(
                        immutable_nbr.neighbor,
                        immutable_nbr.edge_id,
                        immutable_nbr.prop_offset,
                        immutable_nbr.timestamp,
                    );
                    current_entries.push((src_u32, nbr));
                }
                current_create_ts_min = current_create_ts_min.min(segment.create_ts_min);
                current_create_ts_max = current_create_ts_max.max(segment.create_ts_max);
                current_deletion_info = current_deletion_info.merge(&segment.deletion_info);
            } else {
                // No merge: flush current accumulation and start new one
                if !current_entries.is_empty() {
                    let vertex_capacity = current_entries
                        .iter()
                        .map(|(src, _)| *src as usize + 1)
                        .max()
                        .unwrap_or(1024)
                        .max(1024);

                    let merged_csr = Csr::from_nbr_entries(&current_entries, vertex_capacity);
                    total_edges += merged_csr.edge_count() as u64;
                    merged.push(CsrSegment::new(
                        merged_csr,
                        current_create_ts_min,
                        current_create_ts_max,
                        current_deletion_info,
                    ));
                    current_entries.clear();
                }

                // Start new accumulation with current segment
                for (src, immutable_nbr) in segment.csr.iter() {
                    let src_u32 = src.as_int64().unwrap_or(0) as u32;
                    let nbr = Nbr::new(
                        immutable_nbr.neighbor,
                        immutable_nbr.edge_id,
                        immutable_nbr.prop_offset,
                        immutable_nbr.timestamp,
                    );
                    current_entries.push((src_u32, nbr));
                }
                current_create_ts_min = segment.create_ts_min;
                current_create_ts_max = segment.create_ts_max;
                current_deletion_info = segment.deletion_info;
            }
        }

        // Flush remaining accumulation
        if !current_entries.is_empty() {
            let vertex_capacity = current_entries
                .iter()
                .map(|(src, _)| *src as usize + 1)
                .max()
                .unwrap_or(1024)
                .max(1024);

            let merged_csr = Csr::from_nbr_entries(&current_entries, vertex_capacity);
            total_edges += merged_csr.edge_count() as u64;
            merged.push(CsrSegment::new(
                merged_csr,
                current_create_ts_min,
                current_create_ts_max,
                current_deletion_info,
            ));
        }

        *segments = merged;
        DirectionMergeMetrics {
            edges_processed: total_edges,
        }
    }

    pub fn compact_properties(&mut self, ts: Timestamp) {
        let mut valid_offsets = std::collections::HashSet::new();

        for (_, nbr) in self.out_csr.iter(ts) {
            if nbr.prop_offset > 0 {
                valid_offsets.insert(nbr.prop_offset);
            }
        }

        for segment in &self.out_segments {
            for (_, nbr) in segment.csr.iter() {
                if nbr.timestamp <= ts
                    && !self.is_tombstoned(nbr.edge_id, ts)
                    && nbr.prop_offset > 0
                {
                    valid_offsets.insert(nbr.prop_offset);
                }
            }
        }

        for (_, nbr) in self.in_csr.iter(ts) {
            if nbr.prop_offset > 0 {
                valid_offsets.insert(nbr.prop_offset);
            }
        }

        for segment in &self.in_segments {
            for (_, nbr) in segment.csr.iter() {
                if nbr.timestamp <= ts
                    && !self.is_tombstoned(nbr.edge_id, ts)
                    && nbr.prop_offset > 0
                {
                    valid_offsets.insert(nbr.prop_offset);
                }
            }
        }

        self.properties.compact(&valid_offsets);
    }

    pub fn memory_size(&self) -> usize {
        self.used_memory_size()
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0;

        total += self.out_csr.used_memory_size();
        total += self.in_csr.used_memory_size();
        total += self
            .out_segments
            .iter()
            .map(|segment| segment.csr.used_memory_size())
            .sum::<usize>();
        total += self
            .in_segments
            .iter()
            .map(|segment| segment.csr.used_memory_size())
            .sum::<usize>();
        total += self.tombstones.len() * std::mem::size_of::<(EdgeId, Timestamp)>();
        total += self.properties.used_memory_size();

        total
    }
}

pub struct EdgeTableScanIterator<'a> {
    _table: &'a EdgeTable,
    records: std::vec::IntoIter<EdgeRecord>,
}

impl<'a> EdgeTableScanIterator<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp) -> Self {
        let mut seen = HashSet::new();
        let mut records = Vec::new();

        for (src_vid, nbr) in table.out_csr.iter(ts) {
            if !table.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
                records
                    .push(table.edge_record_from_nbr(src_vid.as_int64().unwrap_or(0) as u32, nbr));
            }
        }

        for segment in table.out_segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            for (src_vid, edge) in segment.csr.iter() {
                if edge.timestamp <= ts
                    && !table.is_tombstoned(edge.edge_id, ts)
                    && seen.insert(edge.edge_id)
                {
                    records.push(table.edge_record_from_nbr(
                        src_vid.as_int64().unwrap_or(0) as u32,
                        Nbr::new(
                            edge.neighbor,
                            edge.edge_id,
                            edge.prop_offset,
                            edge.timestamp,
                        ),
                    ));
                }
            }
        }

        Self {
            _table: table,
            records: records.into_iter(),
        }
    }
}

impl<'a> Iterator for EdgeTableScanIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.next()
    }
}

#[cfg(test)]
#[path = "edge_table_tests.rs"]
mod tests;
