//! Vertex Index Management Module
//!
//! Provide functions for updating, deleting, and querying vertex indices.

use crate::core::types::Index;
use crate::core::{StorageError, Value};
use crate::storage::index::index_key_codec::IndexKeyCodec;
use crate::storage::redb_types::{ByteKey, INDEX_DATA_TABLE};
use redb::{Database, ReadableTable};
use std::sync::Arc;

/// Vertex Index Manager
#[derive(Clone)]
pub struct VertexIndexManager {
    db: Arc<Database>,
}

impl VertexIndexManager {
    /// Create a new vertex index manager.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Update the vertex index
    pub fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            for (prop_name, prop_value) in props {
                let index_key = IndexKeyCodec::build_vertex_index_key(
                    space_id, index_name, prop_value, vertex_id,
                )?;

                table
                    .insert(&index_key, ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入索引数据失败: {}", e)))?;

                let reverse_key =
                    IndexKeyCodec::build_vertex_reverse_key(space_id, index_name, vertex_id)?;
                let prop_value_bytes = IndexKeyCodec::serialize_value(prop_value)?;
                let value_key = format!("{}:{}", prop_name, prop_value_bytes.len());
                table
                    .insert(&reverse_key, ByteKey(value_key.into_bytes()))
                    .map_err(|e| StorageError::DbError(format!("插入反向索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// Remove all indexes from the vertex.
    pub fn delete_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let vertex_bytes = IndexKeyCodec::serialize_value(vertex_id)?;
            let reverse_prefix = IndexKeyCodec::build_vertex_reverse_prefix(space_id);

            let mut forward_keys_to_delete: Vec<ByteKey> = Vec::new();
            let mut reverse_keys_to_delete: Vec<ByteKey> = Vec::new();

            for (key, value) in table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .flatten()
            {
                let key_bytes: Vec<u8> = key.value().0.clone();

                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_vid_bytes)) =
                        IndexKeyCodec::parse_vertex_reverse_key(&key_bytes)
                    {
                        if key_vid_bytes == vertex_bytes {
                            reverse_keys_to_delete.push(ByteKey(key_bytes.clone()));

                            let value_bytes: Vec<u8> = value.value().0.clone();
                            let value_str = String::from_utf8_lossy(&value_bytes);
                            let value_parts: Vec<&str> = value_str.split(':').collect();

                            if value_parts.len() >= 2 {
                                let _prop_name = value_parts[0];
                                if let Ok(prop_value_len) = value_parts[1].parse::<usize>() {
                                    let forward_key_start =
                                        IndexKeyCodec::build_vertex_index_prefix(
                                            space_id,
                                            &index_name,
                                        );
                                    let forward_key_end =
                                        IndexKeyCodec::build_range_end(&forward_key_start);

                                    for (fwd_key, _) in table
                                        .range::<ByteKey>(&forward_key_start..&forward_key_end)
                                        .map_err(|e| {
                                            StorageError::DbError(format!("范围查询失败: {}", e))
                                        })?
                                        .flatten()
                                    {
                                        let fwd_key_bytes: Vec<u8> = fwd_key.value().0.clone();
                                        if let Ok(vid) =
                                            IndexKeyCodec::parse_vertex_id_from_key(&fwd_key_bytes)
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
                                                        .push(ByteKey(fwd_key_bytes));
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

            for key in &reverse_keys_to_delete {
                table
                    .remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除反向索引失败: {}", e)))?;
            }

            for key in &forward_keys_to_delete {
                table
                    .remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除正向索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// Delete the index of the specified tag.
    pub fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let vertex_bytes = IndexKeyCodec::serialize_value(vertex_id)?;
            let reverse_prefix = IndexKeyCodec::build_vertex_reverse_prefix(space_id);

            let mut keys_to_delete: Vec<ByteKey> = Vec::new();

            for (key, _) in table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .flatten()
            {
                let key_bytes: Vec<u8> = key.value().0.clone();

                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_vid_bytes)) =
                        IndexKeyCodec::parse_vertex_reverse_key(&key_bytes)
                    {
                        if key_vid_bytes == vertex_bytes && index_name.starts_with(tag_name) {
                            keys_to_delete.push(ByteKey(key_bytes));
                        }
                    }
                }
            }

            for key in &keys_to_delete {
                table
                    .remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// Clear the tag index.
    pub fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let prefix = IndexKeyCodec::build_vertex_index_prefix(space_id, index_name);
            let end = IndexKeyCodec::build_range_end(&prefix);

            let mut keys_to_delete: Vec<ByteKey> = Vec::new();

            for (key, _) in table
                .range::<ByteKey>(&prefix..&end)
                .map_err(|e| StorageError::DbError(format!("范围查询失败: {}", e)))?
                .flatten()
            {
                let key_bytes: Vec<u8> = key.value().0.clone();
                keys_to_delete.push(ByteKey(key_bytes));
            }

            for key in &keys_to_delete {
                table
                    .remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// Search for the tag index.
    pub fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读取事务失败: {}", e)))?;

        let table = txn
            .open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

        let prefix = IndexKeyCodec::build_vertex_index_prefix(space_id, &index.name);
        let end = IndexKeyCodec::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = IndexKeyCodec::serialize_value(value)?;

        for (key, _) in table
            .range::<ByteKey>(&prefix..&end)
            .map_err(|e| StorageError::DbError(format!("范围查询失败: {}", e)))?
            .flatten()
        {
            let key_bytes: Vec<u8> = key.value().0.clone();

            if let Ok(vertex_id) = IndexKeyCodec::parse_vertex_id_from_key(&key_bytes) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;
    use tempfile::TempDir;

    fn create_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test.db");
        let db = Arc::new(Database::create(&db_path).expect("Failed to create test database"));

        let txn = db.begin_write().expect("Failed to begin write transaction");
        {
            let _ = txn
                .open_table(INDEX_DATA_TABLE)
                .expect("Failed to open table");
        }
        txn.commit().expect("Failed to commit transaction");

        (db, temp_dir)
    }

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
        let (db, _temp_dir) = create_test_db();
        let manager = VertexIndexManager::new(db);

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
        let (db, _temp_dir) = create_test_db();
        let manager = VertexIndexManager::new(db);

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
        let (db, _temp_dir) = create_test_db();
        let manager = VertexIndexManager::new(db);

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
