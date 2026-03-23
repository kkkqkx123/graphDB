//! 边索引管理模块
//!
//! 提供边索引的更新、删除和查询功能

use crate::core::types::Index;
use crate::core::{StorageError, Value};
use crate::storage::index::index_key_codec::IndexKeyCodec;
use crate::storage::redb_types::{ByteKey, INDEX_DATA_TABLE};
use redb::{Database, ReadableTable};
use std::sync::Arc;

/// 边索引管理器
#[derive(Clone)]
pub struct EdgeIndexManager {
    db: Arc<Database>,
}

impl EdgeIndexManager {
    /// 创建新的边索引管理器
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 更新边索引
    pub fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
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
                let index_key = IndexKeyCodec::build_edge_index_key(
                    space_id, index_name, prop_value, src, dst,
                )?;

                table
                    .insert(index_key, ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入边索引数据失败: {}", e)))?;

                let reverse_key = IndexKeyCodec::build_edge_reverse_key(space_id, index_name, src)?;
                let prop_value_bytes = IndexKeyCodec::serialize_value(prop_value)?;
                let value_key = format!("{}:{}", prop_name, prop_value_bytes.len());
                table
                    .insert(reverse_key, ByteKey(value_key.into_bytes()))
                    .map_err(|e| StorageError::DbError(format!("插入边反向索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// 删除边所有索引
    pub fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let src_bytes = IndexKeyCodec::serialize_value(src)?;
            let reverse_prefix = IndexKeyCodec::build_edge_reverse_prefix(space_id);

            let mut forward_keys_to_delete: Vec<ByteKey> = Vec::new();
            let mut reverse_keys_to_delete: Vec<ByteKey> = Vec::new();

            for (key, value) in table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?.flatten()
            {
                let key_bytes: Vec<u8> = key.value().0.clone();

                if key_bytes.starts_with(&reverse_prefix.0) {
                    if let Ok((index_name, key_src_bytes)) =
                        IndexKeyCodec::parse_edge_reverse_key(&key_bytes)
                    {
                        if key_src_bytes == src_bytes && index_names.contains(&index_name) {
                            reverse_keys_to_delete.push(ByteKey(key_bytes.clone()));

                            let value_bytes: Vec<u8> = value.value().0.clone();
                            let value_str = String::from_utf8_lossy(&value_bytes);
                            let value_parts: Vec<&str> = value_str.split(':').collect();

                            if value_parts.len() >= 2 {
                                let _prop_name = value_parts[0];
                                if let Ok(prop_value_len) = value_parts[1].parse::<usize>() {
                                    let forward_key_start =
                                        IndexKeyCodec::build_edge_index_prefix(
                                            space_id,
                                            &index_name,
                                        );
                                    let forward_key_end =
                                        IndexKeyCodec::build_range_end(&forward_key_start);

                                    for (fwd_key, _) in table
                                        .range::<ByteKey>(&forward_key_start..&forward_key_end)
                                        .map_err(|e| {
                                            StorageError::DbError(format!(
                                                "范围查询失败: {}",
                                                e
                                            ))
                                        })?.flatten()
                                    {
                                        let fwd_key_bytes: Vec<u8> =
                                            fwd_key.value().0.clone();
                                        if fwd_key_bytes.len()
                                            >= forward_key_start.0.len()
                                                + 4
                                                + prop_value_len
                                                + 4
                                        {
                                            let src_start = forward_key_start.0.len()
                                                + 4
                                                + prop_value_len
                                                + 4;
                                            if fwd_key_bytes.len() >= src_start + 4 {
                                                let src_len = u32::from_le_bytes(
                                                    fwd_key_bytes[src_start - 4..src_start]
                                                        .try_into()
                                                        .unwrap_or([0; 4]),
                                                )
                                                    as usize;
                                                if fwd_key_bytes.len()
                                                    >= src_start + src_len + 4
                                                {
                                                    let dst_len_start = src_start + src_len;
                                                    let dst_len = u32::from_le_bytes(
                                                        fwd_key_bytes[dst_len_start
                                                            ..dst_len_start + 4]
                                                            .try_into()
                                                            .unwrap_or([0; 4]),
                                                    )
                                                        as usize;
                                                    let dst_start = dst_len_start + 4;
                                                    if fwd_key_bytes.len()
                                                        >= dst_start + dst_len
                                                    {
                                                        let stored_src = &fwd_key_bytes
                                                            [src_start
                                                                ..src_start + src_len];
                                                        let stored_dst = &fwd_key_bytes
                                                            [dst_start
                                                                ..dst_start + dst_len];
                                                        if stored_src == src_bytes && stored_dst == IndexKeyCodec::serialize_value(dst)? {
                                                            forward_keys_to_delete.push(ByteKey(fwd_key_bytes));
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

    /// 查找边索引
    pub fn lookup_edge_index(
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

        let prefix = IndexKeyCodec::build_edge_index_prefix(space_id, &index.name);
        let end = IndexKeyCodec::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = IndexKeyCodec::serialize_value(value)?;

        for (key, _) in table
            .range::<ByteKey>(&prefix..&end)
            .map_err(|e| StorageError::DbError(format!("范围查询失败: {}", e)))?.flatten()
        {
            let key_bytes: Vec<u8> = key.value().0.clone();

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

    /// 清空边索引
    pub fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let prefix = IndexKeyCodec::build_edge_index_prefix(space_id, index_name);
            let end = IndexKeyCodec::build_range_end(&prefix);

            let mut keys_to_delete: Vec<ByteKey> = Vec::new();

            for (key, _) in table
                .range::<ByteKey>(&prefix..&end)
                .map_err(|e| StorageError::DbError(format!("范围查询失败: {}", e)))?.flatten()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Index, IndexField, IndexType};
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
        Index::new(
            1,
            name.to_string(),
            1,
            schema_name.to_string(),
            vec![IndexField::new(
                "name".to_string(),
                Value::String("".to_string()),
                false,
            )],
            vec![],
            IndexType::EdgeIndex,
            false,
        )
    }

    #[test]
    fn test_update_and_lookup_edge_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = EdgeIndexManager::new(db);

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
        let (db, _temp_dir) = create_test_db();
        let manager = EdgeIndexManager::new(db);

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
        let (db, _temp_dir) = create_test_db();
        let manager = EdgeIndexManager::new(db);

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
