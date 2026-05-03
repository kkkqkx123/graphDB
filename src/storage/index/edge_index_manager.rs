//! Edge Index Management Module
//!
//! Provide functions for updating, deleting, and querying edge indexes.
//! This implementation uses in-memory storage with BTreeMap for efficient range queries.

use crate::core::types::Index;
use crate::core::{StorageError, Value};
use crate::storage::index::index_key_codec::IndexKeyCodec;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

type IndexKey = Vec<u8>;
type IndexValue = Vec<u8>;

#[derive(Clone)]
pub struct EdgeIndexManager {
    forward_index: Arc<RwLock<BTreeMap<IndexKey, IndexValue>>>,
    reverse_index: Arc<RwLock<BTreeMap<IndexKey, IndexValue>>>,
}

impl EdgeIndexManager {
    pub fn new() -> Self {
        Self {
            forward_index: Arc::new(RwLock::new(BTreeMap::new())),
            reverse_index: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        for (prop_name, prop_value) in props {
            let index_key =
                IndexKeyCodec::build_edge_index_key(space_id, index_name, prop_value, src, dst)?;

            let reverse_key = IndexKeyCodec::build_edge_reverse_key(space_id, index_name, src)?;
            let prop_value_bytes = IndexKeyCodec::serialize_value(prop_value)?;
            let value_key = format!("{}:{}", prop_name, prop_value_bytes.len());

            self.forward_index
                .write()
                .insert(index_key.0, prop_name.as_bytes().to_vec());
            self.reverse_index
                .write()
                .insert(reverse_key.0, value_key.into_bytes());
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
        let src_bytes = IndexKeyCodec::serialize_value(src)?;
        let reverse_prefix = IndexKeyCodec::build_edge_reverse_prefix(space_id);

        let mut forward_keys_to_delete: Vec<IndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (key_bytes, value_bytes) in reverse_index.iter() {
                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_src_bytes)) =
                        IndexKeyCodec::parse_edge_reverse_key(key_bytes)
                    {
                        if key_src_bytes == src_bytes && index_names.contains(&index_name) {
                            reverse_keys_to_delete.push(key_bytes.clone());

                            let value_str = String::from_utf8_lossy(value_bytes);
                            let value_parts: Vec<&str> = value_str.split(':').collect();

                            if value_parts.len() >= 2 {
                                if let Ok(prop_value_len) = value_parts[1].parse::<usize>() {
                                    let forward_key_start =
                                        IndexKeyCodec::build_edge_index_prefix(
                                            space_id,
                                            &index_name,
                                        );
                                    let forward_key_end =
                                        IndexKeyCodec::build_range_end(&forward_key_start);

                                    let forward_index = self.forward_index.read();
                                    for (fwd_key_bytes, _) in forward_index
                                        .range(forward_key_start.0.clone()..forward_key_end.0)
                                    {
                                        if fwd_key_bytes.len()
                                            >= forward_key_start.0.len() + 4 + prop_value_len + 4
                                        {
                                            let src_start =
                                                forward_key_start.0.len() + 4 + prop_value_len + 4;
                                            if fwd_key_bytes.len() >= src_start + 4 {
                                                let src_len = u32::from_le_bytes(
                                                    fwd_key_bytes[src_start - 4..src_start]
                                                        .try_into()
                                                        .unwrap_or([0; 4]),
                                                )
                                                    as usize;
                                                if fwd_key_bytes.len() >= src_start + src_len + 4
                                                {
                                                    let dst_len_start = src_start + src_len;
                                                    let dst_len = u32::from_le_bytes(
                                                        fwd_key_bytes
                                                            [dst_len_start..dst_len_start + 4]
                                                            .try_into()
                                                            .unwrap_or([0; 4]),
                                                    )
                                                        as usize;
                                                    let dst_start = dst_len_start + 4;
                                                    if fwd_key_bytes.len() >= dst_start + dst_len {
                                                        let stored_src = &fwd_key_bytes
                                                            [src_start..src_start + src_len];
                                                        let stored_dst = &fwd_key_bytes
                                                            [dst_start..dst_start + dst_len];
                                                        if stored_src == src_bytes
                                                            && stored_dst
                                                                == IndexKeyCodec::serialize_value(
                                                                    dst,
                                                                )?
                                                        {
                                                            forward_keys_to_delete
                                                                .push(fwd_key_bytes.clone());
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
                }
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            for key in &reverse_keys_to_delete {
                reverse_index.remove(key);
            }
        }

        {
            let mut forward_index = self.forward_index.write();
            for key in &forward_keys_to_delete {
                forward_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = IndexKeyCodec::build_edge_index_prefix(space_id, &index.name);
        let end = IndexKeyCodec::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = IndexKeyCodec::serialize_value(value)?;

        let forward_index = self.forward_index.read();
        for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
            if key_bytes.len() > prefix.0.len() + 4 {
                let prop_len_start = prefix.0.len();
                let prop_value_len = u32::from_le_bytes(
                    key_bytes[prop_len_start..prop_len_start + 4]
                        .try_into()
                        .unwrap_or([0; 4]),
                ) as usize;

                if prop_value_len == value_bytes.len() {
                    let prop_value_start = prop_len_start + 4;
                    let stored_prop_value =
                        &key_bytes[prop_value_start..prop_value_start + prop_value_len];

                    if stored_prop_value == value_bytes.as_slice() {
                        let src_len_start = prop_value_start + prop_value_len;
                        if key_bytes.len() >= src_len_start + 4 {
                            let src_len = u32::from_le_bytes(
                                key_bytes[src_len_start..src_len_start + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            ) as usize;
                            let src_start = src_len_start + 4;
                            if key_bytes.len() >= src_start + src_len {
                                let src_bytes = &key_bytes[src_start..src_start + src_len];
                                if let Ok(src) = IndexKeyCodec::deserialize_value(src_bytes) {
                                    results.push(src);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = IndexKeyCodec::build_edge_index_prefix(space_id, index_name);
        let end = IndexKeyCodec::build_range_end(&prefix);

        let mut keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let forward_index = self.forward_index.read();
            for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
                keys_to_delete.push(key_bytes.clone());
            }
        }

        {
            let mut forward_index = self.forward_index.write();
            for key in &keys_to_delete {
                forward_index.remove(key);
            }
        }

        Ok(())
    }
}

impl Default for EdgeIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;

    fn create_test_index(name: &str, schema_name: &str) -> Index {
        Index::new(IndexConfig {
            id: 1,
            name: name.to_string(),
            space_id: 1,
            schema_name: schema_name.to_string(),
            fields: vec![IndexField::new(
                "name".to_string(),
                Value::String("".to_string()),
                false,
            )],
            properties: vec![],
            index_type: IndexType::EdgeIndex,
            is_unique: false,
        })
    }

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

        let index = create_test_index(index_name, "knows");

        let results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], src);

        let empty_results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(99.9))
            .expect("Failed to lookup edge index");
        assert!(empty_results.is_empty());
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

        let index = create_test_index(index_name, "knows");

        let results1 = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results1.len(), 1);

        let results2 = manager
            .lookup_edge_index(space_id, &index, &Value::Float(20.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results2.len(), 1);

        manager
            .delete_edge_indexes(space_id, &src1, &dst1, &[index_name.to_string()])
            .expect("Failed to delete edge indexes");

        let results1_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert!(results1_after.is_empty());

        let results2_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(20.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results2_after.len(), 1);
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

        let index = create_test_index(index_name, "knows");
        let results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);

        manager
            .clear_edge_index(space_id, index_name)
            .expect("Failed to clear edge index");

        let results_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert!(results_after.is_empty());
    }
}
