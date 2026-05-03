//! Vertex Index Management Module
//!
//! Provide functions for updating, deleting, and querying vertex indices.
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
pub struct VertexIndexManager {
    forward_index: Arc<RwLock<BTreeMap<IndexKey, IndexValue>>>,
    reverse_index: Arc<RwLock<BTreeMap<IndexKey, IndexValue>>>,
}

impl VertexIndexManager {
    pub fn new() -> Self {
        Self {
            forward_index: Arc::new(RwLock::new(BTreeMap::new())),
            reverse_index: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        for (prop_name, prop_value) in props {
            let index_key = IndexKeyCodec::build_vertex_index_key(
                space_id, index_name, prop_value, vertex_id,
            )?;

            let reverse_key =
                IndexKeyCodec::build_vertex_reverse_key(space_id, index_name, vertex_id)?;
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

    pub fn delete_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let vertex_bytes = IndexKeyCodec::serialize_value(vertex_id)?;
        let reverse_prefix = IndexKeyCodec::build_vertex_reverse_prefix(space_id);

        let mut forward_keys_to_delete: Vec<IndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (key_bytes, value_bytes) in reverse_index.iter() {
                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_vid_bytes)) =
                        IndexKeyCodec::parse_vertex_reverse_key(key_bytes)
                    {
                        if key_vid_bytes == vertex_bytes {
                            reverse_keys_to_delete.push(key_bytes.clone());

                            let value_str = String::from_utf8_lossy(value_bytes);
                            let value_parts: Vec<&str> = value_str.split(':').collect();

                            if value_parts.len() >= 2 {
                                if let Ok(prop_value_len) = value_parts[1].parse::<usize>() {
                                    let forward_key_start =
                                        IndexKeyCodec::build_vertex_index_prefix(
                                            space_id,
                                            &index_name,
                                        );
                                    let forward_key_end =
                                        IndexKeyCodec::build_range_end(&forward_key_start);

                                    let forward_index = self.forward_index.read();
                                    for (fwd_key_bytes, _) in forward_index
                                        .range(forward_key_start.0.clone()..forward_key_end.0)
                                    {
                                        if let Ok(vid) =
                                            IndexKeyCodec::parse_vertex_id_from_key(fwd_key_bytes)
                                        {
                                            if vid == *vertex_id
                                                && fwd_key_bytes.len()
                                                    >= forward_key_start.0.len()
                                                        + 4
                                                        + prop_value_len
                                                        + 4
                                            {
                                                let vid_start =
                                                    fwd_key_bytes.len() - vertex_bytes.len();
                                                if fwd_key_bytes[vid_start..] == vertex_bytes {
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

    pub fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        let vertex_bytes = IndexKeyCodec::serialize_value(vertex_id)?;
        let reverse_prefix = IndexKeyCodec::build_vertex_reverse_prefix(space_id);

        let mut keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (key_bytes, _) in reverse_index.iter() {
                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_vid_bytes)) =
                        IndexKeyCodec::parse_vertex_reverse_key(key_bytes)
                    {
                        if key_vid_bytes == vertex_bytes && index_name.starts_with(tag_name) {
                            keys_to_delete.push(key_bytes.clone());
                        }
                    }
                }
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            for key in &keys_to_delete {
                reverse_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = IndexKeyCodec::build_vertex_index_prefix(space_id, index_name);
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

    pub fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = IndexKeyCodec::build_vertex_index_prefix(space_id, &index.name);
        let end = IndexKeyCodec::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = IndexKeyCodec::serialize_value(value)?;

        let forward_index = self.forward_index.read();
        for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
            if let Ok(vertex_id) = IndexKeyCodec::parse_vertex_id_from_key(key_bytes) {
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
                            results.push(vertex_id);
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

impl Default for VertexIndexManager {
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
            index_type: IndexType::TagIndex,
            is_unique: false,
        })
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vertex_id);

        let empty_results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert!(empty_results.is_empty());
    }

    #[test]
    fn test_delete_vertex_indexes() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id1 = Value::Int(1);
        let vertex_id2 = Value::Int(2);
        let index_name = "idx_person_name";

        let props1 = vec![("name".to_string(), Value::String("Alice".to_string()))];
        let props2 = vec![("name".to_string(), Value::String("Bob".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id1, index_name, &props1)
            .expect("Failed to update vertex indexes");
        manager
            .update_vertex_indexes(space_id, &vertex_id2, index_name, &props2)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results1 = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results1.len(), 1);

        let results2 = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2.len(), 1);

        manager
            .delete_vertex_indexes(space_id, &vertex_id1)
            .expect("Failed to delete vertex indexes");

        let results1_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert!(results1_after.is_empty());

        let results2_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2_after.len(), 1);
    }

    #[test]
    fn test_multiple_properties_index() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person";

        let props = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results_name = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results_name.len(), 1);
        assert_eq!(results_name[0], vertex_id);

        let results_age = manager
            .lookup_tag_index(space_id, &index, &Value::Int(30))
            .expect("Failed to lookup tag index");
        assert_eq!(results_age.len(), 1);
        assert_eq!(results_age[0], vertex_id);
    }
}
