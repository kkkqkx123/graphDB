//! Edge table module: split into focused sub-modules for maintainability.
//!
//! Organization:
//! - `core`: Core operations (CRUD, properties, queries)
//! - `segment`: Segment management (CsrSegment, versioning, deletion tracking)
//! - `merge`: Merge strategies (LSM, adaptive, in-place, aggressive)
//! - `mvcc`: MVCC and snapshot management
//! - `snapshot`: Snapshot export and time-travel queries
//! - `persistence`: Serialization (flush/load)
//! - `stats`: Statistics structures (metrics, observability)

pub mod core;
pub mod segment;
pub mod merge;
pub mod mvcc;
pub mod snapshot;
pub mod persistence;
pub mod stats;

// Re-export public types for backward compatibility
pub use core::{
    EdgeTableCore as EdgeTable, EdgeTableConfig, EdgeTableScanIterator,
    UpdateEdgePropertyByOffsetParams,
};
pub use segment::{CsrSegment, DeletionInfo, SegmentVersion, SEPARATE_EDGE_ID_STORAGE_THRESHOLD};
pub use mvcc::MVCCManager;
pub use snapshot::{ExportedEdgeSnapshot, SnapshotBuilder};
pub use stats::{TombstoneStats, DeletionStats, MergeStats, MergeMetrics, MergeMetricsResult};

// Re-export from parent
pub use super::{
    Csr, CsrVariant, Nbr, ImmutableNbr, EdgeSchema, EdgeRecord, EdgeStrategy, CsrBase, MutableCsrTrait,
};

use crate::core::types::{Timestamp, EdgeId, CompactConfig, LabelId, VertexId};
use crate::core::{StorageResult, StorageError};
use std::time::Instant;
use std::sync::Arc;
use crate::storage::persistence::write_header_to;

impl EdgeTable {
    /// Freeze CSR only (convert mutable delta to immutable segment).
    ///
    /// Converts visible edges (ts <= query_ts) to immutable CSR and records
    /// timestamp ranges for time-travel queries and MVCC support.
    /// Clears mutable delta after freezing.
    /// Does NOT perform physical compaction.
    pub fn freeze_csr_only(&mut self, ts: Timestamp) -> usize {
        let out_result = Self::freeze_delta(
            &mut self.out_csr,
            &mut self.out_segments,
            ts,
            &self.mvcc.pending_segment_deletions,
            &self.mvcc.segment_tombstones,
        );
        let in_result = Self::freeze_delta(
            &mut self.in_csr,
            &mut self.in_segments,
            ts,
            &self.mvcc.pending_segment_deletions,
            &self.mvcc.segment_tombstones,
        );

        self.mvcc.segment_tombstones.extend(self.mvcc.pending_segment_deletions.drain());

        // Rebuild indices after segments are modified
        self.rebuild_segment_indices();

        let total_frozen = out_result.frozen_count + in_result.frozen_count;

        if self.out_segments.len() >= self.config.max_segments_per_direction {
            let _ = merge::merge_aggressive(&mut self.out_segments, 8 * 1024 * 1024);
            self.rebuild_segment_indices();
        }
        if self.in_segments.len() >= self.config.max_segments_per_direction {
            let _ = merge::merge_aggressive(&mut self.in_segments, 8 * 1024 * 1024);
            self.rebuild_segment_indices();
        }

        total_frozen
    }

    /// Compact and freeze in sequence with adaptive configuration.
    pub fn compact_and_freeze_with_config(&mut self, ts: Timestamp, config: &CompactConfig) -> usize {
        let edge_count = self.edge_count() as usize;
        let reserve_ratio = config.compute_reserve_ratio(edge_count, 0);

        let removed = self.compact_csr_only(ts, reserve_ratio);
        self.freeze_csr_only(ts);

        if config.segment_merge_enabled {
            let stats = self.mvcc.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);

        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.mvcc.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.mvcc.active_snapshots.len() as u64,
            );
        }

        removed
    }

    /// Compact, freeze, and perform automatic tombstone GC in sequence.
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
            let stats = self.mvcc.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);

        let min_active_snapshot_ts = self.mvcc.get_min_active_snapshot_ts();
        self.mvcc.gc_tombstones(min_active_snapshot_ts);

        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.mvcc.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.mvcc.active_snapshots.len() as u64,
            );
        }

        removed
    }

    /// Compact, freeze, and perform tombstone GC in sequence (deprecated).
    #[deprecated(since = "0.2.0", note = "use compact_and_freeze_with_auto_gc instead")]
    pub fn compact_and_freeze_with_gc(
        &mut self,
        ts: Timestamp,
        config: &CompactConfig,
        min_active_snapshot_ts: Timestamp,
    ) -> usize {
        debug_assert_eq!(
            min_active_snapshot_ts, self.mvcc.get_min_active_snapshot_ts(),
            "Provided min_active_snapshot_ts doesn't match actual. Use compact_and_freeze_with_auto_gc instead."
        );

        let edge_count = self.edge_count() as usize;
        let reserve_ratio = config.compute_reserve_ratio(edge_count, 0);

        let removed = self.compact_csr_only(ts, reserve_ratio);
        self.freeze_csr_only(ts);

        if config.segment_merge_enabled {
            let stats = self.mvcc.tombstone_stats();
            let merge_threshold = config.compute_merge_size_threshold(stats.memory_bytes);
            self.merge_segments_with_config(config.segment_merge_threshold, merge_threshold);
        }

        self.compact_properties(ts);
        self.mvcc.gc_tombstones(min_active_snapshot_ts);

        if let Some(stats) = &self.stats_manager {
            let tom_stats = self.mvcc.tombstone_stats();
            stats.record_tombstone_stats(
                tom_stats.count as u64,
                tom_stats.memory_bytes as u64,
                tom_stats.oldest_delete_ts,
                tom_stats.newest_delete_ts,
                self.mvcc.active_snapshots.len() as u64,
            );
        }

        removed
    }

    /// Export a read-only snapshot of this edge table at the given timestamp.
    pub fn export_snapshot(&self, ts: Timestamp) -> StorageResult<ExportedEdgeSnapshot> {
        let out_edges = self.collect_edges_for_snapshot_mvcc(&self.out_csr, &self.out_segments, ts)?;
        let in_edges = self.collect_edges_for_snapshot_mvcc(&self.in_csr, &self.in_segments, ts)?;

        let out_csr = Self::build_csr_from_edges(out_edges, self.out_csr.vertex_capacity())?;
        let in_csr = Self::build_csr_from_edges(in_edges, self.in_csr.vertex_capacity())?;

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
    fn collect_edges_for_snapshot_mvcc(
        &self,
        delta: &CsrVariant,
        segments: &[CsrSegment],
        ts: Timestamp,
    ) -> StorageResult<Vec<(u32, Nbr)>> {
        use std::collections::HashMap;

        let mut edge_map: HashMap<(u32, EdgeId), (u32, Nbr)> = HashMap::new();

        for segment in segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            if segment.deletion_info.all_deleted_before(ts)
                && segment.deletion_info.all_edges_deleted(segment.csr.edge_count()) {
                continue;
            }

            let mut edge_position = 0usize;
            for (src, immutable_nbr) in segment.csr.iter() {
                let edge_id = segment.recover_edge_id(immutable_nbr, edge_position);
                edge_position += 1;

                if immutable_nbr.timestamp > ts {
                    continue;
                }

                if let Some(&delete_ts) = self.mvcc.tombstones.get(&edge_id) {
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
                edge_map.insert((src_u32, edge_id), (src_u32, nbr));
            }
        }

        for (src, nbr) in delta.iter(ts) {
            let src_u32 = src.as_int64().unwrap_or(0) as u32;

            if let Some(&delete_ts) = self.mvcc.tombstones.get(&nbr.edge_id) {
                if delete_ts <= ts {
                    continue;
                }
            }

            edge_map.insert((src_u32, nbr.edge_id), (src_u32, nbr));
        }

        let mut edges: Vec<_> = edge_map.into_values().collect();
        edges.sort_by_key(|(src, _)| *src);

        Ok(edges)
    }

    /// Build a CSR from a list of edges.
    fn build_csr_from_edges(
        edges: Vec<(u32, Nbr)>,
        vertex_capacity: usize,
    ) -> StorageResult<Csr> {
        Ok(Csr::from_nbr_entries(&edges, vertex_capacity))
    }

    /// Freeze delta CSR to immutable segment
    fn freeze_delta(
        delta: &mut CsrVariant,
        segments: &mut Vec<CsrSegment>,
        ts: Timestamp,
        pending_deletions: &std::collections::HashMap<EdgeId, Timestamp>,
        segment_tombstones: &std::collections::HashMap<EdgeId, Timestamp>,
    ) -> merge::FreezeDeltaResult {
        let entries: Vec<_> = delta
            .iter(ts)
            .map(|(src, nbr)| {
                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                (src_u32, nbr)
            })
            .collect();

        if entries.is_empty() {
            delta.clear();
            return merge::FreezeDeltaResult {
                frozen_count: 0,
                edge_ids: Vec::new(),
                csr_position_to_edge_ids_index: Vec::new(),
            };
        }

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
            "Vertex ID {} exceeds capacity {}",
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

        let mut deleted_count = 0u32;
        let (delete_ts_min, delete_ts_max) = entries
            .iter()
            .filter_map(|(_, nbr)| {
                if let Some(&ts) = pending_deletions.get(&nbr.edge_id) {
                    deleted_count += 1;
                    return Some(ts);
                }
                if let Some(&ts) = segment_tombstones.get(&nbr.edge_id) {
                    deleted_count += 1;
                    return Some(ts);
                }
                None
            })
            .fold((u32::MAX, 0), |(min, max), ts| {
                (std::cmp::min(min, ts), std::cmp::max(max, ts))
            });

        let csr = Csr::from_nbr_entries(&entries, vertex_capacity);
        let frozen = entries.len();

        let deletion_info = DeletionInfo::with_count(delete_ts_min, delete_ts_max, deleted_count);
        let mut segment = CsrSegment::new(
            csr,
            create_ts_min,
            create_ts_max,
            deletion_info,
        );

        if frozen >= SEPARATE_EDGE_ID_STORAGE_THRESHOLD {
            segment.edge_ids = Some(entries.iter().map(|(_, nbr)| nbr.edge_id).collect());
        }

        segments.push(segment);
        delta.clear();

        merge::FreezeDeltaResult {
            frozen_count: frozen,
            edge_ids: Vec::new(),
            csr_position_to_edge_ids_index: Vec::new(),
        }
    }

    /// Get current merge statistics
    pub fn merge_stats(&self) -> MergeStats {
        MergeStats {
            total_merge_operations: 0,
            total_segments_merged: 0,
            total_edges_merged: 0,
            total_merge_time_ms: 0,
            current_segment_count: self.out_segments.len() + self.in_segments.len(),
            max_segment_count: self.config.max_segments_per_direction * 2,
        }
    }

    /// Merge CSR segments with LSM tiered strategy
    pub fn merge_segments_lsm_tiered(&mut self, current_ts: Timestamp) -> usize {
        let start = Instant::now();
        let out_reduced = merge::merge_lsm_tiered(&mut self.out_segments, current_ts);
        let in_reduced = merge::merge_lsm_tiered(&mut self.in_segments, current_ts);

        let total_reduced = out_reduced + in_reduced;
        if total_reduced > 0 {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.rebuild_segment_indices();
            // Record metrics if needed
        }

        total_reduced
    }

    /// Merge CSR segments with adaptive strategy
    pub fn merge_segments_adaptive(&mut self, current_ts: Timestamp, max_segment_age: Timestamp) -> usize {
        let start = Instant::now();
        let out_reduced = merge::merge_adaptive(&mut self.out_segments, current_ts, max_segment_age);
        let in_reduced = merge::merge_adaptive(&mut self.in_segments, current_ts, max_segment_age);

        let total_reduced = out_reduced + in_reduced;
        if total_reduced > 0 {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.rebuild_segment_indices();
        }

        total_reduced
    }

    /// Merge segments with time and size thresholds
    pub fn merge_segments_with_config(&mut self, time_threshold: Timestamp, size_threshold_bytes: usize) -> MergeMetricsResult {
        let start = Instant::now();
        let segments_before = self.out_segments.len() + self.in_segments.len();

        let out_metrics = merge::merge_in_place(&mut self.out_segments, time_threshold, size_threshold_bytes);
        let in_metrics = merge::merge_in_place(&mut self.in_segments, time_threshold, size_threshold_bytes);

        let segments_after = self.out_segments.len() + self.in_segments.len();
        let total_edges = out_metrics.edges_processed + in_metrics.edges_processed;
        let duration_ms = start.elapsed().as_millis() as u64;

        if segments_before != segments_after {
            self.rebuild_segment_indices();
        }

        MergeMetricsResult {
            metrics: MergeMetrics {
                segments_before,
                segments_after,
                edges_merged: total_edges,
                duration_ms,
            },
            segments_reduced: segments_before.saturating_sub(segments_after),
        }
    }

    /// Persistence: flush to disk
    pub fn flush<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        compression: crate::storage::compression::CompressionType,
    ) -> StorageResult<()> {
        use std::fs;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = std::fs::File::create(&meta_path)?;
        write_header_to(&mut meta_file, crate::storage::persistence::section::EDGE_META)
            .map_err(|e| StorageError::io_error(format!("Failed to write edge meta header: {}", e)))?;

        persistence::flush_metadata(
            &mut meta_file,
            self.label,
            self.src_label,
            self.dst_label,
            &self.label_name,
            self.is_open,
            &self.schema,
            self.next_edge_id,
            &self.mvcc.tombstones,
            self.mvcc.min_active_snapshot_ts,
        )?;

        drop(meta_file);
        crate::storage::compression::compress_file_inplace(&meta_path, compression)?;

        let out_csr_path = path.join("out_csr.bin");
        persistence::flush_csr(
            &self.out_csr,
            &self.out_segments,
            &out_csr_path,
            crate::storage::persistence::section::EDGE_OUT_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&out_csr_path, compression)?;

        let in_csr_path = path.join("in_csr.bin");
        persistence::flush_csr(
            &self.in_csr,
            &self.in_segments,
            &in_csr_path,
            crate::storage::persistence::section::EDGE_IN_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&in_csr_path, compression)?;

        let props_path = path.join("properties.bin");
        persistence::flush_properties(&self.properties, &props_path)?;
        crate::storage::compression::compress_file_inplace(&props_path, compression)?;

        Ok(())
    }

    /// Persistence: load from disk
    pub fn load<P: AsRef<std::path::Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let meta_data = crate::storage::compression::read_decompressed(&meta_path)?;
        let mut meta_cursor = &meta_data[..];
        let mut header_buf = [0u8; crate::storage::persistence::HEADER_SIZE];
        meta_cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = crate::storage::persistence::read_header(&mut slice)?;
            if sid != crate::storage::persistence::section::EDGE_META {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge meta: expected {:#06x}, got {:#06x}",
                    crate::storage::persistence::section::EDGE_META,
                    sid
                )));
            }
        }

        let mut version_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 2 {
            return Err(StorageError::deserialize_error(format!(
                "unsupported edge meta version: {}",
                version
            )));
        }

        let (label, src_label, dst_label, label_name, is_open, schema, next_edge_id, tombstones, min_snapshot_ts) =
            persistence::load_metadata(&mut meta_cursor)?;

        self.label = label;
        self.src_label = src_label;
        self.dst_label = dst_label;
        self.label_name = label_name;
        self.is_open = is_open;
        self.schema = schema;
        self.next_edge_id = next_edge_id;
        self.mvcc.tombstones = tombstones;
        self.mvcc.min_active_snapshot_ts = min_snapshot_ts;

        let out_csr_path = path.join("out_csr.bin");
        persistence::load_csr(&out_csr_path, &mut self.out_csr, &mut self.out_segments)?;

        let in_csr_path = path.join("in_csr.bin");
        persistence::load_csr(&in_csr_path, &mut self.in_csr, &mut self.in_segments)?;

        let props_path = path.join("properties.bin");
        self.properties = persistence::load_properties(&props_path)?;

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
}

#[cfg(test)]
mod core_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{VertexId, DataType};
    use crate::core::Value;
    use crate::storage::types::StoragePropertyDef;

    fn create_test_schema() -> EdgeSchema {
        EdgeSchema {
            label_id: 0,
            label_name: "knows".to_string(),
            src_label: 0,
            dst_label: 0,
            properties: vec![StoragePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }

    #[test]
    fn test_freeze_csr_preserves_reads() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        table
            .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
            .unwrap();
        table
            .insert_edge(0, 2, 0, &[("weight".to_string(), Value::Double(2.5))], 110)
            .unwrap();

        let before = table.scan(150);
        let frozen = table.freeze_csr_only(150);
        let after = table.scan(150);

        assert_eq!(frozen, 4);
        assert_eq!(table.out_segments.len(), 1);
        assert_eq!(table.in_segments.len(), 1);
        assert_eq!(before.len(), after.len());
        assert!(table.has_edge(0, 1, 0, 150));
        assert!(table.has_edge(0, 2, 0, 150));
    }

    #[test]
    fn test_delete_base_segment_uses_tombstone() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        table.insert_edge(0, 1, 0, &[], 100).unwrap();
        table.freeze_csr_only(150);

        assert!(table.delete_edge(0, 1, 0, 200).unwrap());
        assert!(table.has_edge(0, 1, 0, 150));
        assert!(!table.has_edge(0, 1, 0, 250));
        assert_eq!(table.scan(250).len(), 0);
    }
}
