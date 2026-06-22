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
    pub fn compact(&mut self) {
        let id_mapping = self.id_indexer.compact().unwrap_or_default();
        if id_mapping.is_empty() {
            // id_indexer has no remapping (concurrent version returns empty)
            // So we need to compute the remapping from timestamps.compact_with_mapping()
            let ts_mapping = self.timestamps.compact_with_mapping();
            if !ts_mapping.is_empty() {
                // Timestamps were remapped, but id_indexer wasn't aware of it.
                // We need to update id_indexer to reflect this change.
                self.remap_id_indexer(&ts_mapping);
            } else {
                // No remapping happened in timestamps
                let old_count = self.id_indexer.len();
                self.columns.resize(old_count);
                let _ = self.apply_deferred_encodings();
            }
            return;
        }
        self.remap_columns(&id_mapping);
        self.remap_timestamps(&id_mapping);
        let _ = self.apply_deferred_encodings();
    }

    fn remap_id_indexer(&mut self, ts_mapping: &std::collections::HashMap<u32, u32>) {
        // Update id_indexer to reflect the ID remapping caused by timestamps.compact()
        // This is critical for maintaining consistency between id_indexer and timestamps/columns
        let mut updated_entries = Vec::new();

        for (old_id, new_id) in ts_mapping {
            if let Some(key) = self.id_indexer.get_key(*old_id) {
                updated_entries.push((key, *new_id));
            }
        }

        // Clear and rebuild id_indexer with correct mappings
        self.id_indexer.clear();
        for (key, new_id) in updated_entries {
            self.id_indexer.set_at(new_id, key);
        }

        // Resize columns to match new size
        let new_count = self.id_indexer.len();
        self.columns.resize(new_count);
        let _ = self.apply_deferred_encodings();
    }

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
