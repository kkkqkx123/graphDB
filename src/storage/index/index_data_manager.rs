//! 索引数据管理器
//!
//! 提供索引数据的更新、删除和查询功能
//! 注意：索引元数据管理由 IndexMetadataManager 负责
//! 所有操作都通过 space_id 来标识空间，实现多空间数据隔离

use crate::core::{StorageError, Value};
use crate::core::Edge;
use crate::index::Index;
use crate::storage::redb_types::{ByteKey, INDEX_DATA_TABLE};
use crate::storage::serializer::{value_to_bytes, value_from_bytes};
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
    /// 使用标准序列化函数，确保与存储层其他部分一致
    fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
        value_to_bytes(value)
    }

    /// 反序列化值
    /// 使用标准反序列化函数，确保与存储层其他部分一致
    fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        value_from_bytes(data)
    }

    /// 构建索引键
    /// 格式: space_id:index_name:prop_value_len:prop_value:vertex_id_len:vertex_id
    /// 使用长度前缀来正确解析二进制数据
    fn build_index_key(space_id: i32, index_name: &str, prop_value: &Value, vertex_id: &Value) -> Result<ByteKey, StorageError> {
        let space_prefix = format!("{}:", space_id);
        let index_part = format!("{}:", index_name);
        let prop_value_bytes = Self::serialize_value(prop_value)?;
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;
        let value_len_part = format!("{}:", prop_value_bytes.len());
        let vertex_len_part = format!("{}:", vertex_id_bytes.len());

        Ok(ByteKey(
            space_prefix.as_bytes()
                .iter()
                .chain(index_part.as_bytes().iter())
                .chain(value_len_part.as_bytes().iter())
                .chain(prop_value_bytes.iter())
                .chain(b":")
                .chain(vertex_len_part.as_bytes().iter())
                .chain(vertex_id_bytes.iter())
                .copied()
                .collect()
        ))
    }

    /// 从索引键中解析 vertex_id
    /// 键格式: space_id:index_name:prop_value_len:prop_value:vertex_id_len:vertex_id
    fn parse_vertex_id_from_key(key_bytes: &[u8]) -> Result<Value, StorageError> {
        // 找到最后一个 ':' 分隔符（vertex_id_len 之前的那个）
        let mut last_colon_pos = None;
        let mut second_last_colon_pos = None;

        for (i, &b) in key_bytes.iter().enumerate().rev() {
            if b == b':' {
                if last_colon_pos.is_none() {
                    last_colon_pos = Some(i);
                } else {
                    second_last_colon_pos = Some(i);
                    break;
                }
            }
        }

        let last_colon = last_colon_pos.ok_or_else(|| StorageError::DbError("Invalid key format: missing colons".to_string()))?;
        let second_last_colon = second_last_colon_pos.ok_or_else(|| StorageError::DbError("Invalid key format: not enough colons".to_string()))?;

        // 解析 vertex_id 长度
        let len_str = std::str::from_utf8(&key_bytes[second_last_colon + 1..last_colon])
            .map_err(|e| StorageError::DbError(format!("Invalid length encoding: {}", e)))?;
        let vertex_id_len: usize = len_str.parse()
            .map_err(|e| StorageError::DbError(format!("Invalid length value: {}", e)))?;

        // 提取 vertex_id 字节
        if last_colon + 1 + vertex_id_len <= key_bytes.len() {
            let vertex_id_bytes = &key_bytes[last_colon + 1..last_colon + 1 + vertex_id_len];
            Self::deserialize_value(vertex_id_bytes)
        } else {
            Err(StorageError::DbError("Invalid key: vertex_id bytes exceed key length".to_string()))
        }
    }

    /// 构建索引键前缀（用于范围查询）
    fn build_index_prefix(space_id: i32, index_name: &str) -> ByteKey {
        ByteKey(format!("{}:{}:", space_id, index_name).into_bytes())
    }

    /// 构建反向索引键
    /// 格式: space_id:reverse:index_name:vertex_id_len:vertex_id
    fn build_reverse_key(space_id: i32, index_name: &str, vertex_id: &Value) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;
        let vertex_len_part = format!("{}:", vertex_id_bytes.len());

        Ok(ByteKey(
            format!("{}:reverse:{}:", space_id, index_name)
                .as_bytes()
                .iter()
                .chain(vertex_len_part.as_bytes().iter())
                .chain(vertex_id_bytes.iter())
                .copied()
                .collect()
        ))
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
                let index_key = Self::build_index_key(space_id, index_name, prop_value, vertex_id)?;

                // 存储索引条目 - 使用 ByteKey 存储属性名
                table.insert(&index_key, ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入索引数据失败: {}", e)))?;

                // 构建反向索引以便删除时查找
                let reverse_key = Self::build_reverse_key(space_id, index_name, vertex_id)?;
                let prop_value_bytes = Self::serialize_value(prop_value)?;
                let value_key = format!("{}:{}", prop_name, prop_value_bytes.len());
                table.insert(&reverse_key, ByteKey(value_key.into_bytes()))
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
                let prop_value_bytes = Self::serialize_value(prop_value)?;
                let src_bytes = Self::serialize_value(src)?;
                let dst_bytes = Self::serialize_value(dst)?;
                let key = format!("{}:{}:{}:{}:{}",
                    space_id,
                    index_name,
                    prop_value_bytes.len(),
                    src_bytes.len(),
                    dst_bytes.len()
                );

                table.insert(ByteKey(key.into_bytes()), ByteKey(prop_name.as_bytes().to_vec()))
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
            let vertex_bytes = Self::serialize_value(vertex_id)?;

            // 查找并删除所有相关的索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        // 检查键是否以反向索引前缀开头，并且包含vertex_id
                        if key_bytes.starts_with(reverse_prefix.as_bytes()) {
                            // 解析键中的vertex_id部分进行比较
                            // 键格式: space_id:reverse:index_name:vertex_id_len:vertex_id
                            let parts: Vec<&[u8]> = key_bytes.split(|&b| b == b':').collect();
                            if parts.len() >= 5 {
                                // 最后一部分是vertex_id
                                if key_bytes.ends_with(&vertex_bytes) {
                                    return Some(ByteKey(key_bytes));
                                }
                            }
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(key)
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
            let src_bytes = Self::serialize_value(src)?;
            let dst_bytes = Self::serialize_value(dst)?;

            // 查找并删除所有相关的边索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        if key_bytes.starts_with(prefix.as_bytes()) &&
                           src_bytes.iter().all(|b| key_bytes.contains(b)) &&
                           key_bytes.ends_with(&dst_bytes) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(key)
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
        let value_bytes = Self::serialize_value(value)?;

        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    if key_bytes.starts_with(&prefix.0) && value_bytes.iter().all(|b| key_bytes.contains(b)) {
                        // 使用正确的反序列化方法从键中提取 vertex_id
                        return match Self::parse_vertex_id_from_key(&key_bytes) {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        };
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
        let value_bytes = Self::serialize_value(value)?;

        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    if key_bytes.starts_with(prefix.as_bytes()) && value_bytes.iter().all(|b| key_bytes.contains(b)) {
                        // 使用正确的反序列化方法从键中提取边信息
                        return match Self::parse_vertex_id_from_key(&key_bytes) {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        };
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
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        if key_bytes.starts_with(prefix.as_bytes()) {
                            return Some(ByteKey(key_bytes));
                        }
                    }
                    None
                })
                .collect();
            
            for key in keys_to_delete {
                table.remove(key)
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
            let vertex_bytes = Self::serialize_value(vertex_id)?;

            // 查找并删除所有相关的索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, value)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        let value_bytes: Vec<u8> = value.value().0.clone();
                        let value_str = String::from_utf8_lossy(&value_bytes);
                        // 检查是否匹配该标签的索引
                        // 键格式: space_id:reverse:index_name:vertex_id_len:vertex_id
                        if key_bytes.starts_with(reverse_prefix.as_bytes()) {
                            // 检查 value（存储的是 "prop_name:prop_value_len"）是否以 tag_name 开头
                            if value_str.starts_with(tag_name) {
                                // 解析键中的 vertex_id 部分进行比较
                                let parts: Vec<&[u8]> = key_bytes.split(|&b| b == b':').collect();
                                if parts.len() >= 5 {
                                    // 最后一部分应该是 vertex_id
                                    if key_bytes.ends_with(&vertex_bytes) {
                                        return Some(ByteKey(key_bytes));
                                    }
                                }
                            }
                        }
                    }
                    None
                })
                .collect();

            for key in keys_to_delete {
                table.remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除标签索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }
}
