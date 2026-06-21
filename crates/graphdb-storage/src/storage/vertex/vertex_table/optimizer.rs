//! Vertex Table Optimizer
//!
//! Handles compaction, ID remapping, and deferred encodings.

use crate::core::StorageResult;
use crate::storage::vertex::{ColumnStore, VertexTimestamp, IdKey};
use crate::storage::encoding::EncodingType;
use super::core::VertexTable;

impl VertexTable {
    pub fn compact(&mut self) {
        let id_mapping = self.id_indexer.compact().unwrap_or_default();
        if id_mapping.is_empty() {
            let old_count = self.timestamps.size();
            self.timestamps.compact();
            let new_count = self.timestamps.size();
            if new_count < old_count && new_count < self.columns.row_count() {
                self.columns.resize(new_count);
            }
            let _ = self.apply_deferred_encodings();
            return;
        }
        self.remap_columns(&id_mapping);
        self.remap_timestamps(&id_mapping);
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
