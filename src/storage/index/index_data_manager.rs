//! 索引数据管理器
//!
//! 提供索引数据的更新、删除和查询功能
//! 注意：索引元数据管理由 IndexMetadataManager 负责
//! 所有操作都通过 space_id 来标识空间，实现多空间数据隔离

use crate::core::{StorageError, Value};
use crate::core::Edge;
use crate::index::Index;
use crate::storage::redb_types::{ByteKey, INDEX_DATA_TABLE};
use redb::{Database, ReadableTable};
use std::sync::Arc;

/// 索引数据管理器 trait
///
/// 提供索引数据的增删改查功能
/// 所有操作都通过 space_id 来标识空间，实现多空间数据隔离
pub trait IndexDataManager {
    /// 更新顶点索引
    fn update_vertex_indexes(&self, space_id: i32, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    /// 更新边索引
    fn update_edge_indexes(&self, space_id: i32, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    /// 删除顶点所有索引
    fn delete_vertex_indexes(&self, space_id: i32, vertex_id: &Value) -> Result<(), StorageError>;
    /// 删除边所有索引
    fn delete_edge_indexes(&self, space_id: i32, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError>;
    /// 查找标签索引
    fn lookup_tag_index(&self, space_id: i32, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    /// 查找边索引
    fn lookup_edge_index(&self, space_id: i32, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    /// 清空边索引
    fn clear_edge_index(&self, space_id: i32, index_name: &str) -> Result<(), StorageError>;
    /// 构建边索引条目
    fn build_edge_index_entry(&self, space_id: i32, index: &Index, edge: &Edge) -> Result<(), StorageError>;
    /// 删除指定标签的索引
    fn delete_tag_indexes(&self, space_id: i32, vertex_id: &Value, tag_name: &str) -> Result<(), StorageError>;
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
    fn deserialize_value(data: &[u8], value_type: &Value) -> Value {
        match value_type {
            Value::String(_) => String::from_utf8_lossy(data).to_string().into(),
            Value::Int(_) => i64::from_be_bytes(data.try_into().unwrap_or([0; 8])).into(),
            Value::Float(_) => f64::from_be_bytes(data.try_into().unwrap_or([0; 8])).into(),
            Value::Bool(_) => (data.first().copied().unwrap_or(0) != 0).into(),
            _ => Value::Null(crate::core::DataType::Any),
        }
    }

    /// 构建索引键
    /// 格式: space_id:index_name:prop_value:vertex_id
    fn build_index_key(space_id: i32, index_name: &str, prop_value: &Value, vertex_id: &Value) -> ByteKey {
        let space_prefix = format!("{}:", space_id);
        let index_part = format!("{}:", index_name);
        let value_part = format!("{}:", Self::serialize_value(prop_value).len());
        let vertex_part = format!("{}", Self::serialize_value(vertex_id).len());
        
        ByteKey(
            space_prefix.as_bytes()
                .iter()
                .chain(index_part.as_bytes().iter())
                .chain(Self::serialize_value(prop_value).iter())
                .chain(b":")
                .chain(Self::serialize_value(vertex_id).iter())
                .copied()
                .collect()
        )
    }

    /// 构建索引键前缀（用于范围查询）
    fn build_index_prefix(space_id: i32, index_name: &str) -> ByteKey {
        ByteKey(format!("{}:{}:", space_id, index_name).into_bytes())
    }

    /// 构建反向索引键
    /// 格式: space_id:reverse:index_name:vertex_id
    fn build_reverse_key(space_id: i32, index_name: &str, vertex_id: &Value) -> ByteKey {
        ByteKey(
            format!("{}:reverse:{}:", space_id, index_name)
                .as_bytes()
                .iter()
                .chain(Self::serialize_value(vertex_id).iter())
                .copied()
                .collect()
        )
    }
}

impl IndexDataManager for RedbIndexDataManager {
    fn update_vertex_indexes(&self, space_id: i32, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            for (prop_name, prop_value) in props {
                // 构建索引键
                let index_key = Self::build_index_key(space_id, index_name, prop_value, vertex_id);
                
                // 存储索引条目
                table.insert(&index_key, prop_name.as_str())
                    .map_err(|e| StorageError::DbError(format!("插入索引数据失败: {}", e)))?;
                
                // 构建反向索引以便删除时查找
                let reverse_key = Self::build_reverse_key(space_id, index_name, vertex_id);
                let value_key = format!("{}:{}", prop_name, Self::serialize_value(prop_value).len());
                table.insert(&reverse_key, value_key.as_str())
                    .map_err(|e| StorageError::DbError(format!("插入反向索引失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }

    fn update_edge_indexes(&self, space_id: i32, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            for (prop_name, prop_value) in props {
                // 构建边索引键
                let key = format!("{}:{}:{}:{}:{}", 
                    space_id, 
                    index_name, 
                    Self::serialize_value(prop_value).len(),
                    Self::serialize_value(src).len(),
                    Self::serialize_value(dst).len()
                );
                
                table.insert(ByteKey(key.into_bytes()), prop_name.as_str())
                    .map_err(|e| StorageError::DbError(format!("插入边索引数据失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }

    fn delete_vertex_indexes(&self, space_id: i32, vertex_id: &Value) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            // 构建反向索引键前缀
            let reverse_prefix = format!("{}:reverse:", space_id);
            let vertex_bytes = Self::serialize_value(vertex_id);
            
            // 查找并删除所有相关的索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().to_vec();
                        if key_bytes.starts_with(reverse_prefix.as_bytes()) && 
                           key_bytes.ends_with(&vertex_bytes) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除索引数据失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }

    fn delete_edge_indexes(&self, space_id: i32, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            // 构建边索引键前缀
            let prefix = format!("{}:{}:", space_id, edge_type);
            let src_bytes = Self::serialize_value(src);
            let dst_bytes = Self::serialize_value(dst);
            
            // 查找并删除所有相关的边索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().to_vec();
                        if key_bytes.starts_with(prefix.as_bytes()) &&
                           key_bytes.contains(&src_bytes) &&
                           key_bytes.ends_with(&dst_bytes) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除边索引数据失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }

    fn lookup_tag_index(&self, space_id: i32, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读取事务失败: {}", e)))?;
        
        let table = txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
        
        let prefix = Self::build_index_prefix(space_id, &index.name);
        let value_bytes = Self::serialize_value(value);
        
        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().to_vec();
                    if key_bytes.starts_with(&prefix.0) && key_bytes.contains(&value_bytes) {
                        // 从键中提取 vertex_id
                        let parts: Vec<&[u8]> = key_bytes.split(|&b| b == b':').collect();
                        if parts.len() >= 4 {
                            return Some(Value::String(String::from_utf8_lossy(parts[3]).to_string()));
                        }
                    }
                }
                None
            })
            .collect();
        
        Ok(results)
    }

    fn lookup_edge_index(&self, space_id: i32, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读取事务失败: {}", e)))?;
        
        let table = txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
        
        let prefix = format!("{}:{}:", space_id, index.name);
        let value_bytes = Self::serialize_value(value);
        
        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().to_vec();
                    if key_bytes.starts_with(prefix.as_bytes()) && key_bytes.contains(&value_bytes) {
                        // 从键中提取边信息
                        let parts: Vec<&[u8]> = key_bytes.split(|&b| b == b':').collect();
                        if parts.len() >= 5 {
                            return Some(Value::String(String::from_utf8_lossy(parts[4]).to_string()));
                        }
                    }
                }
                None
            })
            .collect();
        
        Ok(results)
    }

    fn clear_edge_index(&self, space_id: i32, index_name: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            let prefix = format!("{}:{}:", space_id, index_name);
            
            // 查找并删除所有匹配的索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().to_vec();
                        if key_bytes.starts_with(prefix.as_bytes()) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除索引数据失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }

    fn build_edge_index_entry(&self, space_id: i32, index: &Index, edge: &Edge) -> Result<(), StorageError> {
        // 收集索引字段值
        let mut props: Vec<(String, Value)> = Vec::new();
        for field in &index.fields {
            if let Some(value) = edge.props.get(&field.name) {
                props.push((field.name.clone(), value.clone()));
            }
        }
        
        // 更新边索引
        self.update_edge_indexes(space_id, &edge.src, &edge.dst, &index.name, &props)
    }

    fn delete_tag_indexes(&self, space_id: i32, vertex_id: &Value, tag_name: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;
        
        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;
            
            // 构建反向索引键前缀
            let reverse_prefix = format!("{}:reverse:", space_id);
            let vertex_bytes = Self::serialize_value(vertex_id);
            
            // 查找并删除所有相关的索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, value)) = entry {
                        let key_bytes: Vec<u8> = key.value().to_vec();
                        let value_str = String::from_utf8_lossy(&value.value());
                        // 检查是否匹配该标签的索引
                        if key_bytes.starts_with(reverse_prefix.as_bytes()) && 
                           key_bytes.ends_with(&vertex_bytes) &&
                           value_str.starts_with(tag_name) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除标签索引失败: {}", e)))?;
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }
}
