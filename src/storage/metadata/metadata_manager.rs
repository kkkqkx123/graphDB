//! 元数据管理器
//!
//! 提供元数据的加载和保存功能

use crate::core::StorageError;
use crate::core::types::{SpaceInfo, TagInfo};
use crate::core::types::metadata::UserInfo;
use crate::storage::redb_types::{ByteKey, SPACES_TABLE, TAGS_TABLE, EDGE_TYPES_TABLE, PASSWORDS_TABLE};
use redb::Database;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use crate::core::types::EdgeTypeInfo as EdgeTypeSchema;

/// 元数据管理器 trait
pub trait MetadataManager {
    fn load_metadata(&self) -> Result<(), StorageError>;
    fn save_metadata(&self) -> Result<(), StorageError>;
}

/// 基于 Redb 的元数据管理器实现
#[derive(Clone)]
pub struct RedbMetadataManager {
    db: Arc<Database>,
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
}

impl RedbMetadataManager {
    pub fn new(
        db: Arc<Database>,
        spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
        tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
        edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
        users: Arc<Mutex<HashMap<String, UserInfo>>>,
    ) -> Self {
        Self {
            db,
            spaces,
            tags,
            edge_type_infos,
            users,
        }
    }

    /// 加载元数据
    pub fn load_metadata(&self) -> Result<(), StorageError> {
        // 加载 spaces
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        if let Ok(table) = read_txn.open_table(SPACES_TABLE) {
            if let Ok(Some(data)) = table.get(&ByteKey(b"__metadata:spaces".to_vec())) {
                let spaces: HashMap<String, SpaceInfo> = bincode::decode_from_slice(&data.value().0, bincode::config::standard())
                    .map_err(|e| StorageError::DbError(format!("解析 spaces 元数据失败: {}", e)))?
                    .0;
                let mut spaces_lock = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                *spaces_lock = spaces;
            }
        }

        // 加载 tags
        if let Ok(table) = read_txn.open_table(TAGS_TABLE) {
            if let Ok(Some(data)) = table.get(&ByteKey(b"__metadata:tags".to_vec())) {
                let tags: HashMap<String, HashMap<String, TagInfo>> = bincode::decode_from_slice(&data.value().0, bincode::config::standard())
                    .map_err(|e| StorageError::DbError(format!("解析 tags 元数据失败: {}", e)))?
                    .0;
                let mut tags_lock = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                *tags_lock = tags;
            }
        }

        // 加载 edge_types
        if let Ok(table) = read_txn.open_table(EDGE_TYPES_TABLE) {
            if let Ok(Some(data)) = table.get(&ByteKey(b"__metadata:edge_types".to_vec())) {
                let edge_types: HashMap<String, HashMap<String, EdgeTypeSchema>> = bincode::decode_from_slice(&data.value().0, bincode::config::standard())
                    .map_err(|e| StorageError::DbError(format!("解析 edge_types 元数据失败: {}", e)))?
                    .0;
                let mut edge_types_lock = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                *edge_types_lock = edge_types;
            }
        }

        // 加载 users
        if let Ok(table) = read_txn.open_table(PASSWORDS_TABLE) {
            if let Ok(Some(data)) = table.get(&ByteKey(b"__metadata:users".to_vec())) {
                let users: HashMap<String, UserInfo> = bincode::decode_from_slice(&data.value().0, bincode::config::standard())
                    .map_err(|e| StorageError::DbError(format!("解析 users 元数据失败: {}", e)))?
                    .0;
                let mut users_lock = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                *users_lock = users;
            }
        }

        Ok(())
    }

    /// 保存元数据
    pub fn save_metadata(&self) -> Result<(), StorageError> {
        // 保存 spaces
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let spaces_data = bincode::encode_to_vec(&*spaces, bincode::config::standard())
                .map_err(|e| StorageError::DbError(format!("序列化 spaces 失败: {}", e)))?;
            let mut table = write_txn.open_table(SPACES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(b"__metadata:spaces".to_vec()), &ByteKey(spaces_data))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        // 保存 tags
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let tags_data = bincode::encode_to_vec(&*tags, bincode::config::standard())
                .map_err(|e| StorageError::DbError(format!("序列化 tags 失败: {}", e)))?;
            let mut table = write_txn.open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(b"__metadata:tags".to_vec()), &ByteKey(tags_data))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        // 保存 edge_types
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let edge_types = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let edge_types_data = bincode::encode_to_vec(&*edge_types, bincode::config::standard())
                .map_err(|e| StorageError::DbError(format!("序列化 edge_types 失败: {}", e)))?;
            let mut table = write_txn.open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(b"__metadata:edge_types".to_vec()), &ByteKey(edge_types_data))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        // 保存 users
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let users_data = bincode::encode_to_vec(&*users, bincode::config::standard())
                .map_err(|e| StorageError::DbError(format!("序列化 users 失败: {}", e)))?;
            let mut table = write_txn.open_table(PASSWORDS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(b"__metadata:users".to_vec()), &ByteKey(users_data))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }
}
