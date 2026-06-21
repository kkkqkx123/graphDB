//! MVCC and tombstone management: snapshot isolation and garbage collection.
//!
//! Provides multi-version concurrency control through active snapshot tracking,
//! tombstone lifecycle management, and automatic garbage collection.

use std::collections::HashMap;
use super::stats::TombstoneStats;
use crate::core::types::{Timestamp, EdgeId};

/// MVCC and snapshot management for EdgeTable
pub struct MVCCManager {
    /// Deletions of edges still in mutable CSR or recently deleted
    pub pending_segment_deletions: HashMap<EdgeId, Timestamp>,
    /// Deletions of edges already in frozen segments
    pub segment_tombstones: HashMap<EdgeId, Timestamp>,
    /// Legacy tombstones field for backward compatibility during transition
    pub tombstones: HashMap<EdgeId, Timestamp>,
    /// Minimum timestamp of all active snapshots
    pub min_active_snapshot_ts: Timestamp,
    /// Active snapshot timestamps and their reference count
    pub active_snapshots: HashMap<Timestamp, usize>,
}

impl Default for MVCCManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MVCCManager {
    /// Create a new MVCC manager
    pub fn new() -> Self {
        Self {
            pending_segment_deletions: HashMap::new(),
            segment_tombstones: HashMap::new(),
            tombstones: HashMap::new(),
            min_active_snapshot_ts: u32::MAX,
            active_snapshots: HashMap::new(),
        }
    }

    /// Check if an edge is tombstoned at a given timestamp
    pub fn is_tombstoned(&self, edge_id: EdgeId, ts: Timestamp) -> bool {
        let pending_deleted = self.pending_segment_deletions
            .get(&edge_id)
            .is_some_and(|delete_ts| *delete_ts <= ts);

        let segment_deleted = self.segment_tombstones
            .get(&edge_id)
            .is_some_and(|delete_ts| *delete_ts <= ts);

        let legacy_deleted = self.tombstones
            .get(&edge_id)
            .is_some_and(|delete_ts| *delete_ts <= ts);

        pending_deleted || segment_deleted || legacy_deleted
    }

    /// Garbage collect tombstones that are no longer needed for snapshot isolation.
    ///
    /// Removes tombstones with delete_ts < min_active_snapshot_ts.
    /// These tombstones cannot affect any active snapshot since all snapshots
    /// have ts >= min_active_snapshot_ts.
    pub fn gc_tombstones(&mut self, min_active_snapshot_ts: Timestamp) -> usize {
        let before = self.tombstones.len();
        self.tombstones.retain(|_edge_id, delete_ts| {
            *delete_ts >= min_active_snapshot_ts
        });
        let after = self.tombstones.len();
        self.min_active_snapshot_ts = min_active_snapshot_ts;

        before - after
    }

    /// Register a new active snapshot at the given timestamp.
    ///
    /// This increments the reference count for the snapshot timestamp.
    /// Must be called when a new snapshot is created.
    pub fn register_active_snapshot(&mut self, ts: Timestamp) {
        *self.active_snapshots.entry(ts).or_insert(0) += 1;
    }

    /// Unregister an active snapshot at the given timestamp.
    ///
    /// This decrements the reference count. When count reaches 0,
    /// the timestamp is removed and tombstone GC is automatically triggered.
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

    /// Get the minimum active snapshot timestamp.
    ///
    /// This is the earliest timestamp at which any snapshot is currently active.
    /// All tombstones with delete_ts < this value can be safely garbage collected.
    pub fn get_min_active_snapshot_ts(&self) -> Timestamp {
        self.active_snapshots
            .keys()
            .copied()
            .min()
            .unwrap_or(u32::MAX)
    }

    /// Get number of active snapshots (for testing and debugging)
    #[cfg(test)]
    pub fn active_snapshot_count(&self) -> usize {
        self.active_snapshots.values().sum()
    }
}
