//! 索引数据管理器
//!
//! 提供索引数据的更新、删除和查询功能
//! 注意：索引元数据管理由 IndexMetadataManager 负责

use crate::core::{StorageError, Value};
use crate::core::Edge;
use crate::index::Index;
use crate::storage::redb_types::{ByteKey, INDEX_DATA_TABLE};
use redb::{Database, ReadableTable};
use std::sync::Arc;

/// 索引数据管理器 trait
pub trait IndexDataManager {
    fn update_vertex_indexes(&self, space: &str, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    fn update_edge_indexes(&self, space: &str, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    fn delete_vertex_indexes(&self, space: &str, vertex_id: &Value) -> Result<(), StorageError>;
    fn delete_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError>;
    fn lookup_tag_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    fn lookup_edge_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    fn clear_edge_index(&self, space: &str, index_name: &str) -> Result<(), StorageError>;
    fn build_edge_index_entry(&self, space: &str, index: &Index, edge: &Edge) -> Result<(), StorageError>;
}

/// 基于 Redb 的索引数据管理器实现
#[derive(Clone)]
pub struct RedbIndexDataManager {
    db: Arc<Database>,
}

impl RedbIndexDataManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 序列化值
    fn serialize_value(value: &Value) -> Vec<u8> {
        match value {
            Value::String(s) => s.as_bytes().to_vec(),
            Value::Int(i) => i.to_be_bytes().to_vec(),
            Value::Float(f) => f.to_be_bytes().to_vec(),
            Value::Bool(b) => vec![*b as u8],
            Value::Null(_) => vec![0],
            Value::List(arr) => arr.iter().flat_map(|v| Self::serialize_value(v)).collect(),
            Value::Map(map) => map.iter().flat_map(|(k, v)| {
                [k.as_bytes().to_vec(), Self::serialize_value(v)].concat()
            }).collect(),
            _ => vec![],
        }
    }

    /// 反序列化值
    fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        if data.len() == 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(data);
            Ok(Value::Int(i64::from_be_bytes(bytes)))
        } else {
            Ok(Value::String(String::from_utf8_lossy(data).to_string()))
        }
    }
}

impl IndexDataManager for RedbIndexDataManager {
    fn update_vertex_indexes(&self, space: &str, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        // 构建索引键: space:idx:v:index_name:tag_name:prop_value:vertex_id
        let mut index_key = format!("{}:idx:v:{}:", space, index_name).into_bytes();
        for (prop_name, prop_value) in props {
            index_key.extend_from_slice(prop_name.as_bytes());
            index_key.push(b':');
            index_key.extend_from_slice(&Self::serialize_value(prop_value));
            index_key.push(b':');
        }
        index_key.extend_from_slice(&Self::serialize_value(vertex_id));
        
        // 存储索引
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(index_key), &ByteKey(Self::serialize_value(vertex_id)))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(())
    }

    fn update_edge_indexes(&self, space: &str, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        // 构建索引键: space:idx:e:index_name:edge_type:prop_value:src:dst
        let mut index_key = format!("{}:idx:e:{}:", space, index_name).into_bytes();
        for (prop_name, prop_value) in props {
            index_key.extend_from_slice(prop_name.as_bytes());
            index_key.push(b':');
            index_key.extend_from_slice(&Self::serialize_value(prop_value));
            index_key.push(b':');
        }
        index_key.extend_from_slice(&Self::serialize_value(src));
        index_key.push(b':');
        index_key.extend_from_slice(&Self::serialize_value(dst));
        
        // 存储索引
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table.insert(&ByteKey(index_key), &ByteKey(Self::serialize_value(src)))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(())
    }

    fn delete_vertex_indexes(&self, space: &str, vertex_id: &Value) -> Result<(), StorageError> {
        let mut keys_to_delete = Vec::new();
        
        // 扫描所有索引条目，找到匹配的顶点ID
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let prefix = format!("{}:idx:v:", space).into_bytes();
        
        // 遍历所有匹配的键
        for result in table.iter().map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            
            if key_bytes.starts_with(&prefix) {
                // 检查值是否匹配顶点ID
                if let Ok(id) = Self::deserialize_value(&value.value().0) {
                    if id == *vertex_id {
                        keys_to_delete.push(key_bytes.to_vec());
                    }
                }
            }
        }
        
        drop(read_txn);
        
        // 删除匹配的索引条目
        if !keys_to_delete.is_empty() {
            let write_txn = self.db.begin_write()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            {
                let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                for key in keys_to_delete {
                    table.remove(&ByteKey(key))
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                }
            }
            write_txn.commit()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        
        Ok(())
    }

    fn delete_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let mut keys_to_delete = Vec::new();
        
        // 扫描所有索引条目，找到匹配的边
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let prefix = format!("{}:idx:e:", space).into_bytes();
        let src_bytes = Self::serialize_value(src);
        let dst_bytes = Self::serialize_value(dst);
        
        // 遍历所有匹配的键
        for result in table.iter().map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, _) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            
            if key_bytes.starts_with(&prefix) {
                // 检查键是否以 src:dst 结尾
                let key_str = String::from_utf8_lossy(&key_bytes);
                let src_str = String::from_utf8_lossy(&src_bytes);
                let dst_str = String::from_utf8_lossy(&dst_bytes);
                
                if key_str.ends_with(&format!(":{}:{}", src_str, dst_str)) {
                    keys_to_delete.push(key_bytes.to_vec());
                }
            }
        }
        
        drop(read_txn);
        
        // 删除匹配的索引条目
        if !keys_to_delete.is_empty() {
            let write_txn = self.db.begin_write()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            {
                let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                for key in keys_to_delete {
                    table.remove(&ByteKey(key))
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                }
            }
            write_txn.commit()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        
        Ok(())
    }

    fn lookup_tag_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let mut results = Vec::new();
        let prefix = format!("{}:idx:v:{}:", space, index.name).into_bytes();
        let value_bytes = Self::serialize_value(value);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 遍历所有匹配的键
        for result in table.iter().map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, val) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            
            if key_bytes.starts_with(&prefix) {
                // 检查键是否包含值
                if key_bytes.windows(value_bytes.len()).any(|w| w == value_bytes) {
                    // 反序列化顶点ID
                    if let Ok(vertex_id) = Self::deserialize_value(&val.value().0) {
                        results.push(vertex_id);
                    }
                }
            }
        }
        
        Ok(results)
    }

    fn lookup_edge_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let mut results = Vec::new();
        let prefix = format!("{}:idx:e:{}:", space, index.name).into_bytes();
        let value_bytes = Self::serialize_value(value);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 遍历所有匹配的键
        for result in table.iter().map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, val) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            
            if key_bytes.starts_with(&prefix) {
                // 检查键是否包含值
                if key_bytes.windows(value_bytes.len()).any(|w| w == value_bytes) {
                    // 反序列化源顶点ID
                    if let Ok(src_id) = Self::deserialize_value(&val.value().0) {
                        results.push(src_id);
                    }
                }
            }
        }
        
        Ok(results)
    }

    fn clear_edge_index(&self, space: &str, index_name: &str) -> Result<(), StorageError> {
        let prefix = format!("{}:idx:e:{}:", space, index_name).into_bytes();
        let mut keys_to_delete = Vec::new();
        
        // 扫描所有匹配的键
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        for result in table.iter().map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key, _) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            
            if key_bytes.starts_with(&prefix) {
                keys_to_delete.push(key_bytes.to_vec());
            }
        }
        
        drop(read_txn);
        
        // 删除匹配的索引条目
        if !keys_to_delete.is_empty() {
            let write_txn = self.db.begin_write()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            {
                let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                for key in keys_to_delete {
                    table.remove(&ByteKey(key))
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                }
            }
            write_txn.commit()
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }

        Ok(())
    }

    fn build_edge_index_entry(&self, space: &str, index: &Index, edge: &Edge) -> Result<(), StorageError> {
        for field in &index.fields {
            if let Some(value) = edge.props.get(&field.name) {
                // 构建索引键: space:idx:e:index_name:edge_type:field_name:field_value:src:dst
                let mut index_key = format!("{}:idx:e:{}:{}:", space, index.name, edge.edge_type).into_bytes();
                index_key.extend_from_slice(field.name.as_bytes());
                index_key.push(b':');
                index_key.extend_from_slice(&Self::serialize_value(value));
                index_key.push(b':');
                index_key.extend_from_slice(&Self::serialize_value(&edge.src));
                index_key.push(b':');
                index_key.extend_from_slice(&Self::serialize_value(&edge.dst));
                
                // 存储索引
                let write_txn = self.db.begin_write()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                {
                    let mut table = write_txn.open_table(INDEX_DATA_TABLE)
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                    table.insert(&ByteKey(index_key), &ByteKey(Self::serialize_value(&edge.src)))
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                }
                write_txn.commit()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        Ok(())
    }
}

// 公开方法供外部使用
impl RedbIndexDataManager {
    pub fn update_vertex_indexes(&self, space: &str, vertex_id: &Value, tag_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        <Self as IndexDataManager>::update_vertex_indexes(self, space, vertex_id, tag_name, props)
    }

    pub fn update_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        <Self as IndexDataManager>::update_edge_indexes(self, space, src, dst, edge_type, props)
    }

    pub fn delete_vertex_indexes(&self, space: &str, vertex_id: &Value) -> Result<(), StorageError> {
        <Self as IndexDataManager>::delete_vertex_indexes(self, space, vertex_id)
    }

    pub fn delete_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        <Self as IndexDataManager>::delete_edge_indexes(self, space, src, dst, edge_type)
    }

    pub fn lookup_tag_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        <Self as IndexDataManager>::lookup_tag_index(self, space, index, value)
    }

    pub fn lookup_edge_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        <Self as IndexDataManager>::lookup_edge_index(self, space, index, value)
    }

    pub fn clear_edge_index(&self, space: &str, index_name: &str) -> Result<(), StorageError> {
        <Self as IndexDataManager>::clear_edge_index(self, space, index_name)
    }

    pub fn build_edge_index_entry(&self, space: &str, index: &Index, edge: &Edge) -> Result<(), StorageError> {
        <Self as IndexDataManager>::build_edge_index_entry(self, space, index, edge)
    }
}
