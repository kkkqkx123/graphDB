//! Edge Index Management Module
//!
//! Provide functions for updating, deleting, and querying edge indexes.
//! This implementation uses in-memory storage with BTreeMap for efficient range queries.
//! Supports persistence through flush/load operations.
//! Supports MVCC (Multi-Version Concurrency Control) for snapshot isolation.

use crate::core::types::{Timestamp, MAX_TIMESTAMP};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::index::generic_index_manager::GenericIndexManager;
use crate::storage::index::index_data_manager::IndexEntry;
use crate::storage::index::key_codec::{
    serialize_value, EdgeIndexKeyGen, KeyBuilder, KeyParser, SecondaryIndexKey,
};
use std::path::Path;

#[derive(Clone)]
pub struct EdgeIndexManager {
    base: GenericIndexManager<EdgeIndexKeyGen>,
}

impl EdgeIndexManager {
    pub fn new() -> Self {
        Self {
            base: GenericIndexManager::new(),
        }
    }

    fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        Ok(compressed.to_vec())
    }

    pub fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_mvcc(space_id, src, dst, index_name, props, MAX_TIMESTAMP)
    }

    pub fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        for (_prop_name, prop_value) in props {
            let logical_forward_key =
                KeyBuilder::build_edge_index_key(space_id, index_name, prop_value, src, dst)?;
            let logical_reverse_key =
                KeyBuilder::build_edge_reverse_key_v2(space_id, src, dst, index_name)?;

            let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
            let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

            {
                let forward_index = self.base.forward_index().read();
                let forward_end = KeyBuilder::build_range_end(&logical_forward_key);
                for (key, entry) in
                    forward_index.range(logical_forward_key.0.clone()..forward_end.0)
                {
                    if entry.is_visible_at(write_ts) {
                        forward_keys_to_delete.push(key.clone());
                    }
                }
            }

            {
                let reverse_index = self.base.reverse_index().read();
                let reverse_end = KeyBuilder::build_range_end(&logical_reverse_key);
                for (key, entry) in
                    reverse_index.range(logical_reverse_key.0.clone()..reverse_end.0)
                {
                    if entry.is_visible_at(write_ts) {
                        reverse_keys_to_delete.push(key.clone());
                    }
                }
            }

            {
                let mut forward_index = self.base.forward_index().write();
                for key in &forward_keys_to_delete {
                    if let Some(entry) = forward_index.get_mut(key) {
                        entry.mark_deleted(write_ts);
                    }
                }
            }

            {
                let mut reverse_index = self.base.reverse_index().write();
                for key in &reverse_keys_to_delete {
                    if let Some(entry) = reverse_index.get_mut(key) {
                        entry.mark_deleted(write_ts);
                    }
                }
            }

            let index_key = logical_forward_key;
            let reverse_key = logical_reverse_key;
            let entry = IndexEntry::new(write_ts);
            let compressed_forward = self.base.physical_key(&index_key.0);
            let compressed_reverse = self.base.physical_key(&reverse_key.0);
            {
                let mut forward_index = self.base.forward_index().write();
                forward_index.insert(compressed_forward, entry.clone());
            }
            {
                let mut reverse_index = self.base.reverse_index().write();
                reverse_index.insert(compressed_reverse, entry);
            }
        }

        Ok(())
    }

    pub fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_mvcc(space_id, src, dst, index_names, MAX_TIMESTAMP)
    }

    pub fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_edge_reverse_prefix_v2_with_dst(space_id, src, dst)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (compressed_key, entry) in
                reverse_index.range(reverse_prefix.0.clone()..reverse_end.0)
            {
                if !entry.is_visible_at(write_ts) {
                    continue;
                }

                let key_bytes = self.decompress_key(compressed_key)?;
                if let Ok((_src_bytes, _dst_bytes, index_name)) =
                    KeyParser::parse_edge_reverse_key_v2(&key_bytes)
                {
                    if index_names.contains(&index_name) {
                        reverse_keys_to_delete.push(compressed_key.clone());

                        let forward_key_start =
                            KeyBuilder::build_edge_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let src_bytes = serialize_value(src)?;
                        let dst_bytes = serialize_value(dst)?;
                        let forward_index = self.base.forward_index().read();
                        for (fwd_compressed_key, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if !fwd_entry.is_visible_at(write_ts) {
                                continue;
                            }

                            let fwd_key_bytes = self.decompress_key(fwd_compressed_key)?;
                            if fwd_key_bytes.len() >= forward_key_start.0.len() + 4 {
                                let prop_len_start = forward_key_start.0.len();
                                let prop_value_len = u32::from_le_bytes(
                                    fwd_key_bytes[prop_len_start..prop_len_start + 4]
                                        .try_into()
                                        .unwrap_or([0; 4]),
                                ) as usize;

                                let src_start = forward_key_start.0.len() + 4 + prop_value_len + 4;
                                if fwd_key_bytes.len() >= src_start {
                                    let src_len = u32::from_le_bytes(
                                        fwd_key_bytes[src_start - 4..src_start]
                                            .try_into()
                                            .unwrap_or([0; 4]),
                                    ) as usize;
                                    if fwd_key_bytes.len() >= src_start + src_len + 4 {
                                        let dst_len_start = src_start + src_len;
                                        let dst_len = u32::from_le_bytes(
                                            fwd_key_bytes[dst_len_start..dst_len_start + 4]
                                                .try_into()
                                                .unwrap_or([0; 4]),
                                        )
                                            as usize;
                                        let dst_start = dst_len_start + 4;
                                        if fwd_key_bytes.len() >= dst_start + dst_len {
                                            let stored_src =
                                                &fwd_key_bytes[src_start..src_start + src_len];
                                            let stored_dst =
                                                &fwd_key_bytes[dst_start..dst_start + dst_len];
                                            if stored_src == src_bytes && stored_dst == dst_bytes {
                                                forward_keys_to_delete
                                                    .push(fwd_compressed_key.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut reverse_index = self.base.reverse_index().write();
            for key in &reverse_keys_to_delete {
                if let Some(entry) = reverse_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for key in &forward_keys_to_delete {
                if let Some(entry) = forward_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        Ok(())
    }

    pub fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let forward_index = self.base.forward_index().read();
            for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
                forward_keys_to_delete.push(key_bytes.clone());
            }
        }

        {
            let reverse_index = self.base.reverse_index().read();
            for (key_bytes, _) in reverse_index.iter() {
                if key_bytes.len() < 9 || key_bytes[0..8] != space_id.to_le_bytes() {
                    continue;
                }

                if let Ok((_src_bytes, _dst_bytes, parsed_index_name)) =
                    KeyParser::parse_edge_reverse_key_v2(key_bytes)
                {
                    if parsed_index_name == index_name {
                        reverse_keys_to_delete.push(key_bytes.clone());
                    }
                }
            }
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for key in &forward_keys_to_delete {
                forward_index.remove(key);
            }
        }

        {
            let mut reverse_index = self.base.reverse_index().write();
            for key in &reverse_keys_to_delete {
                reverse_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        self.base.flush(path)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        self.base.load(path)
    }

    pub fn gc_tombstones(&self, safe_ts: Timestamp) -> Result<usize, StorageError> {
        self.base.gc_tombstones(safe_ts)
    }

    pub fn gc_tombstones_incremental(
        &self,
        safe_ts: Timestamp,
        batch_size: usize,
    ) -> Result<usize, StorageError> {
        self.base.gc_tombstones_incremental(safe_ts, batch_size)
    }

    pub fn tombstone_count(&self) -> usize {
        self.base.tombstone_count()
    }
}

impl Default for EdgeIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeIndexManager;
    use crate::core::Value;

    #[test]
    fn test_update_and_lookup_edge_index() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";
        let props = vec![("weight".to_string(), Value::Float(10.5))];

        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        assert_eq!(manager.base.entry_count(), (1, 1));
    }

    #[test]
    fn test_delete_edge_indexes() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src1 = Value::Int(1);
        let dst1 = Value::Int(2);
        let src2 = Value::Int(3);
        let dst2 = Value::Int(4);
        let index_name = "idx_edge_weight";

        let props1 = vec![("weight".to_string(), Value::Float(10.5))];
        let props2 = vec![("weight".to_string(), Value::Float(20.5))];

        manager
            .update_edge_indexes(space_id, &src1, &dst1, index_name, &props1)
            .expect("Failed to update edge indexes");
        manager
            .update_edge_indexes(space_id, &src2, &dst2, index_name, &props2)
            .expect("Failed to update edge indexes");

        manager
            .delete_edge_indexes(space_id, &src1, &dst1, &[index_name.to_string()])
            .expect("Failed to delete edge indexes");

        assert_eq!(manager.base.entry_count(), (2, 2));

        manager
            .gc_tombstones(u32::MAX)
            .expect("Failed to gc tombstones");
        assert_eq!(manager.base.entry_count(), (1, 1));
    }

    #[test]
    fn test_clear_edge_index() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";

        let props = vec![("weight".to_string(), Value::Float(10.5))];
        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        manager
            .clear_edge_index(space_id, index_name)
            .expect("Failed to clear edge index");

        assert_eq!(manager.base.entry_count(), (0, 0));
    }

    #[test]
    fn test_update_hides_previous_value_at_new_timestamp() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";

        manager
            .update_edge_indexes_mvcc(
                space_id,
                &src,
                &dst,
                index_name,
                &[("weight".to_string(), Value::Float(10.5))],
                10,
            )
            .expect("Failed to insert initial edge index");

        manager
            .update_edge_indexes_mvcc(
                space_id,
                &src,
                &dst,
                index_name,
                &[("weight".to_string(), Value::Float(20.5))],
                20,
            )
            .expect("Failed to update edge index");

        assert_eq!(manager.base.entry_count(), (2, 2));

        manager
            .delete_edge_indexes_mvcc(space_id, &src, &dst, &[index_name.to_string()], 30)
            .expect("Failed to delete edge index");

        manager
            .gc_tombstones(u32::MAX)
            .expect("Failed to gc tombstones");

        assert_eq!(manager.base.entry_count(), (0, 0));
    }

    #[test]
    fn test_clear_edge_index_is_space_scoped() {
        let manager = EdgeIndexManager::new();

        let src_one = Value::Int(1);
        let dst_one = Value::Int(2);
        let src_two = Value::Int(3);
        let dst_two = Value::Int(4);
        let index_name = "idx_shared";

        manager
            .update_edge_indexes(
                1,
                &src_one,
                &dst_one,
                index_name,
                &[("weight".to_string(), Value::Float(10.5))],
            )
            .expect("Failed to insert space one edge index");
        manager
            .update_edge_indexes(
                2,
                &src_two,
                &dst_two,
                index_name,
                &[("weight".to_string(), Value::Float(20.5))],
            )
            .expect("Failed to insert space two edge index");

        manager
            .clear_edge_index(1, index_name)
            .expect("Failed to clear space one edge index");

        assert_eq!(manager.base.entry_count(), (1, 1));
    }
}
