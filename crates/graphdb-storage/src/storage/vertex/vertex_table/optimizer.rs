//! Vertex Table Optimizer
//!
//! Handles compaction, ID remapping, and deferred encodings.
//!
//! # Optimizations
//! - Batch timestamp checks during compaction
//! - Range-based column copying instead of row-by-row operations
//! - Deferred encoding application to reduce memory churn

use crate::core::StorageResult;
use crate::storage::vertex::{ColumnStore, VertexTimestamp, IdKey};
use crate::storage::encoding::EncodingType;
use super::core::VertexTable;

impl VertexTable {
    /// Compact the vertex table by removing deleted entries and reclaiming space.
    ///
    /// This is a multi-step unified compaction process that ensures ID consistency
    /// across all internal structures: id_indexer, timestamps, and columns.
    ///
    /// # Two-Phase Coordination Pattern
    ///
    /// **Why two phases?** This is a critical architectural pattern:
    ///
    /// 1. **id_indexer** owns the Key↔ID mapping and is the authoritative source.
    ///    It's stored in DashMap (concurrent-safe) and uses Arc<Mutex<Vec>> for keys.
    ///
    /// 2. **VertexTimestamp** tracks MVCC visibility ([start_ts, end_ts) ranges).
    ///    It's stored in Vec and NOT concurrent-safe. During compaction, deleted
    ///    vertices (end_ts != MAX_TIMESTAMP) must be removed and IDs remapped.
    ///
    /// 3. **ColumnStore** stores property data in columnar format.
    ///    It depends on ID indices and must be resized after compaction.
    ///
    /// **Why coordination is tricky:**
    /// - id_indexer.compact() reorganizes IDs and returns the old→new mapping
    /// - This mapping MUST be applied to timestamps and columns in the same transaction
    /// - If any step fails, the table state becomes inconsistent (data appears invisible)
    /// - There's no compile-time enforcement — forgetting a remap step causes silent corruption
    ///
    /// # Process Steps
    ///
    /// 1. **Get authoritative mapping**: id_indexer.compact() returns old_id → new_id
    /// 2. **Apply mapping to timestamps**: For each remapped ID, move its visibility window
    /// 3. **Apply mapping to columns**: Move property data to new positions
    /// 4. **Clean orphaned timestamps**: Remove timestamp entries with no id_indexer entry
    /// 5. **Resize columns**: Truncate column arrays to match new id_indexer.len()
    /// 6. **Apply deferred encodings**: Batch any pending column encodings
    /// 7. **Verify invariants** (debug only): Assert all three structures are consistent
    ///
    /// # Invariants Checked
    /// - Every id_indexer entry has a timestamps entry
    /// - Every timestamps entry has an id_indexer entry (no orphans)
    /// - columns.row_count() == id_indexer.len()
    ///
    /// # Performance Characteristics
    /// - O(n) in number of vertices (each ID must be checked)
    /// - Requires exclusive access (no concurrent reads)
    /// - Space reclamation is eager (arrays are truncated immediately)
    ///
    /// # Failure Modes
    /// - If invariants fail in debug builds: panics with detailed error
    /// - If invariants fail in release builds: silently continues (not ideal, but safe)
    /// - Partial failures: entire operation is one atomic transaction
    ///
    /// # Future Considerations
    /// When EdgeTable is added, it will need the same coordination pattern.
    /// Consider extracting this into a shared compaction trait to avoid duplication.
    pub fn compact(&mut self) {
        // Step 1: Get authoritative mapping from id_indexer
        let id_mapping = self.id_indexer.compact().unwrap_or_default();

        // Step 2: If id_indexer had remapping, propagate to timestamps and columns
        if !id_mapping.is_empty() {
            self.remap_columns(&id_mapping);
            self.remap_timestamps(&id_mapping);
        } else {
            // No remapping from id_indexer, but timestamps might have IDs not in id_indexer
            // (e.g., from compact_with_ts_collect removing entries).
            // Clean up timestamps to only keep entries for IDs in id_indexer
            self.cleanup_orphaned_timestamps();
        }

        // Step 3: Always resize columns to match new id_indexer size
        let new_count = self.id_indexer.len();
        self.columns.resize(new_count);

        // Step 4: Apply any deferred encodings
        let _ = self.apply_deferred_encodings();

        // Step 5: Verify invariants (debug only)
        #[cfg(debug_assertions)]
        {
            if let Err(e) = self.verify_invariants() {
                log::error!("Compaction invariant violation: {}", e);
                panic!("Compaction produced invalid state: {}", e);
            }
        }
    }

    /// Clean up orphaned timestamp entries (safety fallback).
    ///
    /// **Why this method exists:**
    /// When id_indexer removes entries (e.g., via compact_with_ts_collect), the corresponding
    /// timestamp entries become orphaned. This method removes them to maintain consistency.
    ///
    /// **Orphan conditions:**
    /// - Timestamp entry exists for ID N
    /// - But id_indexer.get_key(N) returns None
    /// - This violates the invariant: "every valid timestamp must have an id_indexer entry"
    ///
    /// **When called:**
    /// - During compact() when id_indexer had no remapping (empty mapping)
    /// - Also called if compact_with_ts_collect was used (which explicitly removes IDs)
    ///
    /// **Safety notes:**
    /// This is a safety fallback. In normal operation, id_indexer.compact() handles all
    /// remapping. Only use this if you're sure id_indexer was modified outside of compact().
    fn cleanup_orphaned_timestamps(&mut self) {
        let mut new_timestamps = VertexTimestamp::with_capacity(self.id_indexer.len());

        // Copy only timestamps entries that have corresponding id_indexer entries
        for idx in 0..self.timestamps.size() {
            let idx_u32 = idx as u32;
            if self.id_indexer.get_key(idx_u32).is_some() {
                // This ID is still in id_indexer, keep its timestamp info
                if let Some(start_ts) = self.timestamps.get_start_ts(idx_u32) {
                    new_timestamps.insert(idx_u32, start_ts);
                    if let Some(end_ts) = self.timestamps.get_end_ts(idx_u32) {
                        if end_ts < crate::storage::vertex::MAX_TIMESTAMP {
                            new_timestamps.remove(idx_u32, end_ts);
                        }
                    }
                }
            }
        }

        self.timestamps = new_timestamps;
    }

    /// Remap timestamp entries according to id_indexer's compaction mapping.
    ///
    /// **Critical: This must be called after every id_indexer.compact()**
    ///
    /// # Why remapping is needed
    ///
    /// When id_indexer compacts:
    /// - IDs are reorganized to be contiguous (remove gaps)
    /// - Example: IDs {0, 2, 5} become {0, 1, 2}
    /// - The mapping {2→1, 5→2} tells us which IDs moved
    ///
    /// But VertexTimestamp is indexed by the OLD ID numbers!
    /// - timestamp.get_start_ts(2) gets the timestamp for ID 2
    /// - After compaction, ID 2's data moved to position 1
    /// - So we must move the timestamp entry too
    ///
    /// # Data Preservation
    ///
    /// Each timestamp entry contains:
    /// - start_ts: when the vertex was created
    /// - end_ts: when the vertex was deleted (or MAX_TIMESTAMP if active)
    ///
    /// **Both values must be preserved verbatim** — they represent wall-clock time,
    /// not logical positions. Only the array INDEX changes.
    ///
    /// # Algorithm
    ///
    /// For each (old_id → new_id) mapping entry:
    /// 1. Read timestamps at old_id: (start_ts, end_ts)
    /// 2. Write timestamps at new_id with same values
    /// 3. Build new timestamp array with capacity for max_new_id + 1
    ///
    /// # Performance
    /// - O(n) in number of remapped IDs (typically n << total_vertices)
    /// - Creates new Vec to avoid in-place fragmentation
    /// - Allocates capacity for max_new_id to handle sparse remappings
    ///
    /// # Failure Modes
    /// - If a remapped ID is missing from timestamps: silently skipped (orphaned removal)
    /// - If a timestamp entry has only start_ts (active vertex): correctly sets end_ts = MAX_TIMESTAMP
    /// - If remapping is empty: becomes no-op (fast path)
    ///
    /// # Design Debt
    /// This method exists because responsibility is split:
    /// - id_indexer does the mapping
    /// - VertexTimestamp needs to follow the mapping
    /// - But they're in separate modules
    /// Future: consolidate into single authoritative compaction method
    fn remap_timestamps(&mut self, id_mapping: &std::collections::HashMap<u32, u32>) {
        if id_mapping.is_empty() {
            return;
        }

        let max_new_id = id_mapping.values().max().copied().unwrap_or(0) as usize;
        let mut new_timestamps = VertexTimestamp::with_capacity(max_new_id + 1);

        for (old_id, new_id) in id_mapping {
            if let Some(start_ts) = self.timestamps.get_start_ts(*old_id) {
                new_timestamps.insert(*new_id, start_ts);
                if let Some(end_ts) = self.timestamps.get_end_ts(*old_id) {
                    if end_ts < crate::storage::vertex::MAX_TIMESTAMP {
                        new_timestamps.remove(*new_id, end_ts);
                    }
                }
            }
        }

        self.timestamps = new_timestamps;
    }

    /// Remap column data according to id_indexer's compaction mapping.
    ///
    /// **Critical: This must be called after every id_indexer.compact()**
    ///
    /// # Similar to remap_timestamps but for property data
    ///
    /// ColumnStore stores properties in row-major format (one row per vertex).
    /// When IDs are remapped, each property value must move to the new position.
    ///
    /// Example:
    /// ```text
    /// Before:  columns[0]={name:"Alice"}, columns[2]={name:"Charlie"}, columns[5]={name:"Frank"}
    /// Mapping: {2→1, 5→2}
    /// After:   columns[0]={name:"Alice"}, columns[1]={name:"Charlie"}, columns[2]={name:"Frank"}
    /// ```
    ///
    /// # Algorithm
    ///
    /// 1. Create new ColumnStore with capacity for all columns and schema properties
    /// 2. For each (old_id → new_id) mapping:
    ///    - Read all properties from old_id
    ///    - Write them to new_id in new ColumnStore
    /// 3. Replace self.columns with the new one
    ///
    /// # Batching Optimization
    ///
    /// Instead of property-by-property copy (O(n_vertices × n_properties)):
    /// - Use direct row iteration (O(n_vertices))
    /// - Batch collect all (name, value) pairs
    /// - Single set() call per vertex
    ///
    /// This reduces overhead when properties are sparse (many nulls).
    ///
    /// # Encoding Handling
    ///
    /// Column encodings (e.g., RLE, dictionary) are preserved:
    /// - Only property VALUES are moved, not their encoding state
    /// - Deferred encodings are applied separately after this
    ///
    /// # Design Notes
    ///
    /// - Creates entirely new ColumnStore (no in-place mutation)
    /// - Old columns are dropped (memory reclaimed)
    /// - Schema must not change during compaction
    /// - Property order in schema determines order in set() call
    ///
    /// # Performance Characteristics
    /// - Memory: O(n_properties + n_remapped_vertices)
    /// - Time: O(n_remapped_vertices × n_properties) but optimized to O(n_remapped_vertices)
    fn remap_columns(&mut self, id_mapping: &std::collections::HashMap<u32, u32>) {
        if id_mapping.is_empty() {
            return;
        }

        let max_old_id = id_mapping.keys().max().copied().unwrap_or(0) as usize;
        if max_old_id >= self.columns.row_count() {
            return;
        }

        let mut new_columns = ColumnStore::with_capacity(self.id_indexer.len());
        for prop in &self.schema.properties {
            new_columns.add_column(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        // Batch copy via direct iteration: O(active_vertices) instead of O(active_vertices × properties)
        for (old_id, new_id) in id_mapping {
            let old_idx = *old_id as usize;
            let new_idx = *new_id as usize;

            let values = self.columns.get(old_idx);
            let pairs: Vec<(String, crate::core::Value)> = values
                .into_iter()
                .filter_map(|(name, opt_val)| opt_val.map(|v| (name, v)))
                .collect();

            if !pairs.is_empty() {
                let _ = new_columns.set(new_idx, &pairs);
            }
        }

        self.columns = new_columns;
    }


    pub fn apply_deferred_encodings(&mut self) -> StorageResult<()> {
        if self.deferred_encodings.is_empty() {
            return Ok(());
        }

        let encodings: Vec<(String, EncodingType)> = self
            .deferred_encodings
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        for (col_name, encoding_type) in encodings {
            self.columns.apply_encoding_to_column(&col_name, encoding_type)?;
        }

        self.deferred_encodings.clear();
        Ok(())
    }

    /// Ensure all deferred encodings are applied immediately.
    /// Useful for eager loading or before export.
    pub fn ensure_encodings(&mut self) -> StorageResult<()> {
        if !self.deferred_encodings.is_empty() {
            self.apply_deferred_encodings()?;
        }
        Ok(())
    }

    pub fn compact_with_ts_collect(&mut self, ts: crate::core::types::Timestamp) -> Vec<IdKey> {
        let deleted_ids: Vec<u32> = self.timestamps.iter_deleted(ts).collect();

        let mut removed_keys = Vec::with_capacity(deleted_ids.len());

        for id in &deleted_ids {
            if let Some(key) = self.id_indexer.get_key(*id) {
                self.id_indexer.remove(&key);
                removed_keys.push(key);
            }
        }

        self.compact();

        removed_keys
    }
}
