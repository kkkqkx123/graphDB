use crate::core::StorageError;
use crate::index::Index;
use crate::storage::metadata::IndexMetadataManager;
use crate::storage::redb_types::{ByteKey, TAG_INDEXES_TABLE, EDGE_INDEXES_TABLE};
use crate::storage::serializer::{storage_index_to_bytes, storage_index_from_bytes};
use redb::{Database, ReadableTable};
use std::sync::Arc;

/// Redb 索引元数据管理器
///
/// 使用 space_id 作为空间标识符，实现多空间数据隔离
pub struct RedbIndexMetadataManager {
    db: Arc<Database>,
}

impl std::fmt::Debug for RedbIndexMetadataManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbIndexMetadataManager").finish()
    }
}

impl RedbIndexMetadataManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 构建标签索引键
    /// 格式: space_id:index_name
    fn make_tag_index_key(space_id: u64, index_name: &str) -> Vec<u8> {
        format!("{}:{}", space_id, index_name).as_bytes().to_vec()
    }

    /// 构建边索引键
    /// 格式: space_id:index_name
    fn make_edge_index_key(space_id: u64, index_name: &str) -> Vec<u8> {
        format!("{}:{}", space_id, index_name).as_bytes().to_vec()
    }
}

impl IndexMetadataManager for RedbIndexMetadataManager {
    fn create_tag_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError> {
        let key = Self::make_tag_index_key(space_id, &index.name);
        let index_bytes = storage_index_to_bytes(index)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAG_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key), ByteKey(index_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn drop_tag_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError> {
        let key = Self::make_tag_index_key(space_id, index_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAG_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(key))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag_index(&self, space_id: u64, index_name: &str) -> Result<Option<Index>, StorageError> {
        let key = Self::make_tag_index_key(space_id, index_name);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes)?;
                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    fn list_tag_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut indexes = Vec::new();
        let space_prefix = format!("{}:", space_id);
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&space_prefix) {
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes)?;
                indexes.push(index);
            }
        }
        Ok(indexes)
    }

    fn drop_tag_indexes_by_tag(&self, space_id: u64, tag_name: &str) -> Result<(), StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let space_prefix = format!("{}:", space_id);
        let indexes_to_drop: Vec<Vec<u8>> = table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .filter_map(|result| {
                let (key, value) = result.ok()?;
                let key_data = key.value().0.clone();
                let key_str = String::from_utf8_lossy(&key_data);
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes).ok()?;
                if key_str.starts_with(&space_prefix) && index.schema_name == tag_name {
                    Some(key_data)
                } else {
                    None
                }
            })
            .collect();
        drop(read_txn);

        if indexes_to_drop.is_empty() {
            return Ok(());
        }

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAG_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for key in indexes_to_drop {
                table.remove(ByteKey(key))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn create_edge_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError> {
        let key = Self::make_edge_index_key(space_id, &index.name);
        let index_bytes = storage_index_to_bytes(index)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key), ByteKey(index_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn drop_edge_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError> {
        let key = Self::make_edge_index_key(space_id, index_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(key))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_edge_index(&self, space_id: u64, index_name: &str) -> Result<Option<Index>, StorageError> {
        let key = Self::make_edge_index_key(space_id, index_name);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes)?;
                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    fn list_edge_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut indexes = Vec::new();
        let space_prefix = format!("{}:", space_id);
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&space_prefix) {
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes)?;
                indexes.push(index);
            }
        }
        Ok(indexes)
    }

    fn drop_edge_indexes_by_type(&self, space_id: u64, edge_type: &str) -> Result<(), StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let space_prefix = format!("{}:", space_id);
        let indexes_to_drop: Vec<Vec<u8>> = table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .filter_map(|result| {
                let (key, value) = result.ok()?;
                let key_data = key.value().0.clone();
                let key_str = String::from_utf8_lossy(&key_data);
                let index_bytes = value.value().0;
                let index: Index = storage_index_from_bytes(&index_bytes).ok()?;
                if key_str.starts_with(&space_prefix) && index.schema_name == edge_type {
                    Some(key_data)
                } else {
                    None
                }
            })
            .collect();
        drop(read_txn);

        if indexes_to_drop.is_empty() {
            return Ok(());
        }

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for key in indexes_to_drop {
                table.remove(ByteKey(key))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }
}
