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

/// 索引键类型标记
const KEY_TYPE_VERTEX_REVERSE: u8 = 0x01;
const KEY_TYPE_EDGE_REVERSE: u8 = 0x02;
const KEY_TYPE_VERTEX_FORWARD: u8 = 0x03;
const KEY_TYPE_EDGE_FORWARD: u8 = 0x04;

/// 索引数据管理器 trait
///
/// 提供索引数据的增删改查功能
/// 所有操作都通过 space_id 来标识空间，实现多空间数据隔离
pub trait IndexDataManager {
    /// 更新顶点索引
    fn update_vertex_indexes(&self, space_id: u64, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    /// 更新边索引
    fn update_edge_indexes(&self, space_id: u64, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    /// 删除顶点所有索引
    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError>;
    /// 删除边所有索引
    fn delete_edge_indexes(&self, space_id: u64, src: &Value, dst: &Value, index_names: &[String]) -> Result<(), StorageError>;
    /// 查找标签索引
    fn lookup_tag_index(&self, space_id: u64, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    /// 查找边索引
    fn lookup_edge_index(&self, space_id: u64, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    /// 清空边索引
    fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError>;
    /// 构建边索引条目
    fn build_edge_index_entry(&self, space_id: u64, index: &Index, edge: &Edge) -> Result<(), StorageError>;
    /// 删除指定标签的索引
    fn delete_tag_indexes(&self, space_id: u64, vertex_id: &Value, tag_name: &str) -> Result<(), StorageError>;
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

    /// 构建顶点正向索引键
    /// 格式: [space_id: u64] [type: u8=0x03] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [vertex_id_len: u32] [vertex_id]
    fn build_vertex_index_key(space_id: u64, index_name: &str, prop_value: &Value, vertex_id: &Value) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = Self::serialize_value(prop_value)?;
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;

        let mut key = Vec::new();
        // space_id (8 bytes)
        key.extend_from_slice(&space_id.to_le_bytes());
        // type marker (1 byte)
        key.push(KEY_TYPE_VERTEX_FORWARD);
        // index_name_len (4 bytes) + index_name
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        // prop_value_len (4 bytes) + prop_value
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        // vertex_id_len (4 bytes) + vertex_id
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    /// 构建顶点正向索引键前缀（用于范围查询）
    fn build_vertex_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    /// 从顶点正向索引键中解析 vertex_id
    fn parse_vertex_id_from_key(key_bytes: &[u8]) -> Result<Value, StorageError> {
        // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [vertex_id_len: u32] [vertex_id]
        let mut pos = 9; // skip space_id (8) + type (1)

        // 读取 index_name_len 并跳过 index_name
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid key: too short".to_string()));
        }
        let index_name_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        // 读取 prop_value_len 并跳过 prop_value
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid key: missing prop_value_len".to_string()));
        }
        let prop_value_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + prop_value_len;

        // 读取 vertex_id_len
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid key: missing vertex_id_len".to_string()));
        }
        let vertex_id_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        // 提取 vertex_id
        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError("Invalid key: vertex_id exceeds key length".to_string()));
        }
        let vertex_id_bytes = &key_bytes[pos..pos + vertex_id_len];
        Self::deserialize_value(vertex_id_bytes)
    }

    /// 构建顶点反向索引键
    /// 格式: [space_id: u64] [type: u8=0x01] [index_name_len: u32] [index_name] [vertex_id_len: u32] [vertex_id]
    fn build_vertex_reverse_key(space_id: u64, index_name: &str, vertex_id: &Value) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    /// 构建顶点反向索引键前缀
    fn build_vertex_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        ByteKey(key)
    }

    /// 解析顶点反向索引键
    fn parse_vertex_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [vertex_id_len: u32] [vertex_id]
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError("Invalid reverse key: too short".to_string()));
        }

        let mut pos = 9; // skip space_id (8) + type (1)

        // 读取 index_name
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid reverse key: missing index_name_len".to_string()));
        }
        let index_name_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError("Invalid reverse key: index_name exceeds key length".to_string()));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos+index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;
        pos += index_name_len;

        // 读取 vertex_id
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid reverse key: missing vertex_id_len".to_string()));
        }
        let vertex_id_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError("Invalid reverse key: vertex_id exceeds key length".to_string()));
        }
        let vertex_id_bytes = key_bytes[pos..pos+vertex_id_len].to_vec();

        Ok((index_name, vertex_id_bytes))
    }

    /// 构建边正向索引键
    /// 格式: [space_id: u64] [type: u8=0x04] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [src_len: u32] [src] [dst_len: u32] [dst]
    fn build_edge_index_key(space_id: u64, index_name: &str, prop_value: &Value, src: &Value, dst: &Value) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = Self::serialize_value(prop_value)?;
        let src_bytes = Self::serialize_value(src)?;
        let dst_bytes = Self::serialize_value(dst)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);
        key.extend_from_slice(&(dst_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&dst_bytes);

        Ok(ByteKey(key))
    }

    /// 构建边正向索引键前缀
    fn build_edge_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    /// 构建边反向索引键
    /// 格式: [space_id: u64] [type: u8=0x02] [index_name_len: u32] [index_name] [src_len: u32] [src]
    fn build_edge_reverse_key(space_id: u64, index_name: &str, src: &Value) -> Result<ByteKey, StorageError> {
        let src_bytes = Self::serialize_value(src)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);

        Ok(ByteKey(key))
    }

    /// 构建边反向索引键前缀
    fn build_edge_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        ByteKey(key)
    }

    /// 构建范围查询的结束键（前缀的下一个值）
    /// 用于优化范围查询，避免全表扫描
    fn build_range_end(prefix: &ByteKey) -> ByteKey {
        let mut end = prefix.0.clone();
        // 将最后一个字节加 1，得到前缀范围的上界
        for i in (0..end.len()).rev() {
            if end[i] == 255 {
                end[i] = 0;
            } else {
                end[i] += 1;
                break;
            }
        }
        ByteKey(end)
    }

    /// 解析边反向索引键
    fn parse_edge_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [src_len: u32] [src]
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError("Invalid edge reverse key: too short".to_string()));
        }

        let mut pos = 9; // skip space_id (8) + type (1)

        // 读取 index_name
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid edge reverse key: missing index_name_len".to_string()));
        }
        let index_name_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError("Invalid edge reverse key: index_name exceeds key length".to_string()));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos+index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;
        pos += index_name_len;

        // 读取 src
        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid edge reverse key: missing src_len".to_string()));
        }
        let src_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + src_len {
            return Err(StorageError::DbError("Invalid edge reverse key: src exceeds key length".to_string()));
        }
        let src_bytes = key_bytes[pos..pos+src_len].to_vec();

        Ok((index_name, src_bytes))
    }
}

impl IndexDataManager for RedbIndexDataManager {
    fn update_vertex_indexes(&self, space_id: u64, vertex_id: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            for (prop_name, prop_value) in props {
                // 构建正向索引键
                let index_key = Self::build_vertex_index_key(space_id, index_name, prop_value, vertex_id)?;

                // 存储索引条目 - 使用 ByteKey 存储属性名
                table.insert(&index_key, ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入索引数据失败: {}", e)))?;

                // 构建反向索引以便删除时查找
                let reverse_key = Self::build_vertex_reverse_key(space_id, index_name, vertex_id)?;
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

    fn update_edge_indexes(&self, space_id: u64, src: &Value, dst: &Value, index_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            for (prop_name, prop_value) in props {
                // 构建边正向索引键
                let index_key = Self::build_edge_index_key(space_id, index_name, prop_value, src, dst)?;

                table.insert(index_key, ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入边索引数据失败: {}", e)))?;

                // 构建边反向索引以便删除时查找
                let reverse_key = Self::build_edge_reverse_key(space_id, index_name, src)?;
                let prop_value_bytes = Self::serialize_value(prop_value)?;
                let value_key = format!("{}:{}", prop_name, prop_value_bytes.len());
                table.insert(reverse_key, ByteKey(value_key.into_bytes()))
                    .map_err(|e| StorageError::DbError(format!("插入边反向索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let vertex_bytes = Self::serialize_value(vertex_id)?;

            // 首先通过反向索引查找所有相关的正向索引键
            let reverse_prefix = Self::build_vertex_reverse_prefix(space_id);

            let mut forward_keys_to_delete: Vec<ByteKey> = Vec::new();
            let mut reverse_keys_to_delete: Vec<ByteKey> = Vec::new();

            for entry in table.iter().map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))? {
                if let Ok((key, value)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();

                    // 检查是否是顶点反向索引（通过前缀匹配）
                    if key_bytes.starts_with(&reverse_prefix.0) {
                        // 解析反向索引键
                        if let Ok((index_name, key_vid_bytes)) = Self::parse_vertex_reverse_key(&key_bytes) {
                            if key_vid_bytes == vertex_bytes {
                                // 找到匹配的反向索引，记录要删除
                                reverse_keys_to_delete.push(ByteKey(key_bytes.clone()));

                                // 从反向索引的 value 中提取信息构建正向索引键
                                // value 格式: prop_name:prop_value_len
                                let value_bytes: Vec<u8> = value.value().0.clone();
                                let value_str = String::from_utf8_lossy(&value_bytes);
                                let value_parts: Vec<&str> = value_str.split(':').collect();

                                if value_parts.len() >= 2 {
                                    let prop_name = value_parts[0];

                                    // 构建正向索引键前缀
                                    let forward_prefix = Self::build_vertex_index_prefix(space_id, &index_name);

                                    // 查找匹配的正向索引
                                    for fwd_entry in table.iter().map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))? {
                                        if let Ok((fwd_key, fwd_value)) = fwd_entry {
                                            let fwd_key_bytes: Vec<u8> = fwd_key.value().0.clone();

                                            // 检查是否是正向索引且匹配前缀
                                            if fwd_key_bytes.starts_with(&forward_prefix.0) &&
                                               fwd_key_bytes.get(8) != Some(&KEY_TYPE_VERTEX_REVERSE) {
                                                // 检查 value 是否匹配 prop_name
                                                let fwd_value_bytes: Vec<u8> = fwd_value.value().0.clone();
                                                let fwd_value_str = String::from_utf8_lossy(&fwd_value_bytes);

                                                if fwd_value_str == prop_name {
                                                    // 从正向索引键中解析 vertex_id
                                                    if let Ok(fwd_vid) = Self::parse_vertex_id_from_key(&fwd_key_bytes) {
                                                        if let Ok(fwd_vid_bytes) = Self::serialize_value(&fwd_vid) {
                                                            if fwd_vid_bytes == vertex_bytes {
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
            }

            // 删除所有正向索引
            for key in forward_keys_to_delete {
                let _ = table.remove(key);
            }

            // 删除所有反向索引
            for key in reverse_keys_to_delete {
                let _ = table.remove(key);
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    fn delete_edge_indexes(&self, space_id: u64, src: &Value, dst: &Value, index_names: &[String]) -> Result<(), StorageError> {
        if index_names.is_empty() {
            return Ok(());
        }

        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let src_bytes = Self::serialize_value(src)?;
            let dst_bytes = Self::serialize_value(dst)?;

            // 删除所有指定的边索引条目
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();

                        // 检查是否是边正向索引
                        if key_bytes.len() < 9 || key_bytes[8] != KEY_TYPE_EDGE_FORWARD {
                            return None;
                        }

                        // 检查是否匹配索引名称
                        for index_name in index_names {
                            let prefix = Self::build_edge_index_prefix(space_id, index_name);
                            if key_bytes.starts_with(&prefix.0) {
                                // 解析键中的 src 和 dst
                                // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [src_len: u32] [src] [dst_len: u32] [dst]
                                let mut pos = prefix.0.len();

                                // 跳过 prop_value
                                if key_bytes.len() < pos + 4 {
                                    return None;
                                }
                                let prop_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                                pos += 4 + prop_len;

                                // 读取 src
                                if key_bytes.len() < pos + 4 {
                                    return None;
                                }
                                let src_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                                pos += 4;

                                if key_bytes.len() < pos + src_len {
                                    return None;
                                }
                                let key_src = &key_bytes[pos..pos + src_len];
                                pos += src_len;

                                // 读取 dst
                                if key_bytes.len() < pos + 4 {
                                    return None;
                                }
                                let dst_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                                pos += 4;

                                if key_bytes.len() < pos + dst_len {
                                    return None;
                                }
                                let key_dst = &key_bytes[pos..pos + dst_len];

                                if key_src == src_bytes && key_dst == dst_bytes {
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
                    .map_err(|e| StorageError::DbError(format!("删除边索引数据失败: {}", e)))?;
            }

            // 删除反向索引
            let reverse_prefix = Self::build_edge_reverse_prefix(space_id);
            let range_end = Self::build_range_end(&reverse_prefix);
            let range_start = reverse_prefix.clone();
            let reverse_keys_to_delete: Vec<ByteKey> = table
                .range(range_start..range_end)
                .map_err(|e| StorageError::DbError(format!("范围查询索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        // 解析并检查是否匹配 src 和索引名称
                        if let Ok((index_name, key_src_bytes)) = Self::parse_edge_reverse_key(&key_bytes) {
                            if key_src_bytes == src_bytes && index_names.contains(&index_name) {
                                return Some(ByteKey(key_bytes));
                            }
                        }
                    }
                    None
                })
                .collect();

            for key in reverse_keys_to_delete {
                table.remove(key)
                    .map_err(|e| StorageError::DbError(format!("删除边反向索引失败: {}", e)))?;
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    fn lookup_tag_index(&self, space_id: u64, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读取事务失败: {}", e)))?;

        let table = txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

        let prefix = Self::build_vertex_index_prefix(space_id, &index.name);
        let value_bytes = Self::serialize_value(value)?;

        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    // 检查是否是顶点正向索引且匹配前缀
                    if key_bytes.starts_with(&prefix.0) && key_bytes.get(8) == Some(&KEY_TYPE_VERTEX_FORWARD) {
                        // 解析键中的 prop_value
                        // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [vertex_id_len: u32] [vertex_id]
                        let mut pos = prefix.0.len();

                        if key_bytes.len() < pos + 4 {
                            return None;
                        }
                        let prop_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                        pos += 4;

                        if key_bytes.len() < pos + prop_len {
                            return None;
                        }
                        let key_prop_bytes = &key_bytes[pos..pos + prop_len];

                        // 检查属性值是否匹配
                        if key_prop_bytes == value_bytes.as_slice() {
                            // 使用正确的反序列化方法从键中提取 vertex_id
                            return match Self::parse_vertex_id_from_key(&key_bytes) {
                                Ok(v) => Some(v),
                                Err(_) => None,
                            };
                        }
                    }
                }
                None
            })
            .collect();

        Ok(results)
    }

    fn lookup_edge_index(&self, space_id: u64, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
        let txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读取事务失败: {}", e)))?;

        let table = txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

        let prefix = Self::build_edge_index_prefix(space_id, &index.name);
        let value_bytes = Self::serialize_value(value)?;

        let results: Vec<Value> = table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
            .filter_map(|entry| {
                entry.ok().and_then(|(key, _)| {
                    let key_bytes: Vec<u8> = key.value().0.clone();

                    // 检查是否是边正向索引且匹配前缀
                    if !key_bytes.starts_with(&prefix.0) || key_bytes.get(8) != Some(&KEY_TYPE_EDGE_FORWARD) {
                        return None;
                    }

                    // 解析边索引键
                    // 格式: [space_id: u64] [type: u8] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [src_len: u32] [src] [dst_len: u32] [dst]
                    let mut pos = prefix.0.len();

                    // 读取 prop_value
                    if key_bytes.len() < pos + 4 {
                        return None;
                    }
                    let prop_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                    pos += 4;

                    if key_bytes.len() < pos + prop_len {
                        return None;
                    }
                    let key_prop_bytes = &key_bytes[pos..pos + prop_len];

                    // 检查属性值是否匹配
                    if key_prop_bytes != value_bytes.as_slice() {
                        return None;
                    }

                    // 跳过 prop_value，读取 src
                    pos += prop_len;
                    if key_bytes.len() < pos + 4 {
                        return None;
                    }
                    let src_len = u32::from_le_bytes(key_bytes[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
                    pos += 4;

                    if key_bytes.len() < pos + src_len {
                        return None;
                    }
                    let src_bytes = &key_bytes[pos..pos + src_len];

                    // 反序列化 src
                    Self::deserialize_value(src_bytes).ok()
                })
            })
            .collect();

        Ok(results)
    }

    fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let forward_prefix = Self::build_edge_index_prefix(space_id, index_name);
            let reverse_prefix = Self::build_edge_reverse_prefix(space_id);

            // 查找并删除所有匹配的索引条目（正向和反向）
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        // 检查是否匹配正向索引前缀或反向索引前缀
                        if key_bytes.starts_with(&forward_prefix.0) {
                            return Some(ByteKey(key_bytes));
                        }
                        // 对于反向索引，需要解析出 index_name 进行比较
                        if key_bytes.starts_with(&reverse_prefix.0) {
                            if let Ok((idx_name, _)) = Self::parse_edge_reverse_key(&key_bytes) {
                                if idx_name == index_name {
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

    fn build_edge_index_entry(&self, space_id: u64, index: &Index, edge: &Edge) -> Result<(), StorageError> {
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

    fn delete_tag_indexes(&self, space_id: u64, vertex_id: &Value, tag_name: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let reverse_prefix = Self::build_vertex_reverse_prefix(space_id);
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

                        // 检查是否是顶点反向索引
                        if key_bytes.starts_with(&reverse_prefix.0) {
                            // 检查 value（存储的是 "prop_name:prop_value_len"）是否以 tag_name 开头
                            if value_str.starts_with(tag_name) {
                                // 解析键中的 vertex_id 部分进行比较
                                if let Ok((_, key_vid_bytes)) = Self::parse_vertex_reverse_key(&key_bytes) {
                                    if key_vid_bytes == vertex_bytes {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::index::{Index, IndexField, IndexType};
    use std::sync::Arc;
    use tempfile::TempDir;

    /// 创建测试用的临时数据库
    fn create_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test.db");
        let db = Arc::new(Database::create(&db_path).expect("Failed to create test database"));

        // 初始化索引数据表
        let txn = db.begin_write().expect("Failed to begin write transaction");
        {
            let _ = txn.open_table(INDEX_DATA_TABLE).expect("Failed to open table");
        }
        txn.commit().expect("Failed to commit transaction");

        (db, temp_dir)
    }

    /// 创建测试用的索引
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
            IndexType::TagIndex,
            false,
        )
    }

    #[test]
    fn test_build_vertex_index_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key = RedbIndexDataManager::build_vertex_index_key(space_id, index_name, &prop_value, &vertex_id).expect("Failed to build vertex index key");

        // 验证键的基本结构
        assert!(key.0.len() > 9); // 至少包含 space_id (8) + type (1) + 一些数据
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_FORWARD);

        // 验证可以正确解析 vertex_id
        let parsed_vid = RedbIndexDataManager::parse_vertex_id_from_key(&key.0).expect("Failed to parse vertex id from key");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_vertex_reverse_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_test";
        let vertex_id = Value::Int(456);

        let key = RedbIndexDataManager::build_vertex_reverse_key(space_id, index_name, &vertex_id).expect("Failed to build vertex reverse key");

        // 验证键的基本结构
        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_REVERSE);

        // 验证可以正确解析
        let (parsed_name, parsed_vid_bytes) = RedbIndexDataManager::parse_vertex_reverse_key(&key.0).expect("Failed to parse vertex reverse key");
        assert_eq!(parsed_name, index_name);
        let parsed_vid = RedbIndexDataManager::deserialize_value(&parsed_vid_bytes).expect("Failed to deserialize value");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_edge_index_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let prop_value = Value::String("edge_prop".to_string());
        let src = Value::Int(100);
        let dst = Value::Int(200);

        let key = RedbIndexDataManager::build_edge_index_key(space_id, index_name, &prop_value, &src, &dst).expect("Failed to build edge index key");

        // 验证键的基本结构
        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_FORWARD);
    }

    #[test]
    fn test_build_edge_reverse_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let src = Value::Int(300);

        let key = RedbIndexDataManager::build_edge_reverse_key(space_id, index_name, &src).expect("Failed to build edge reverse key");

        // 验证键的基本结构
        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_REVERSE);

        // 验证可以正确解析
        let (parsed_name, parsed_src_bytes) = RedbIndexDataManager::parse_edge_reverse_key(&key.0).expect("Failed to parse edge reverse key");
        assert_eq!(parsed_name, index_name);
        let parsed_src = RedbIndexDataManager::deserialize_value(&parsed_src_bytes).expect("Failed to deserialize value");
        assert_eq!(parsed_src, src);
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        // 更新索引
        manager.update_vertex_indexes(space_id, &vertex_id, index_name, &props).expect("Failed to update vertex indexes");

        // 创建索引对象用于查询
        let index = create_test_index(index_name, "person");

        // 查询索引
        let results = manager.lookup_tag_index(space_id, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vertex_id);

        // 查询不存在的值
        let empty_results = manager.lookup_tag_index(space_id, &index, &Value::String("Bob".to_string())).expect("Failed to lookup tag index");
        assert!(empty_results.is_empty());
    }

    #[test]
    fn test_update_and_lookup_edge_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";
        let props = vec![("weight".to_string(), Value::Float(10.5))];

        // 更新边索引
        manager.update_edge_indexes(space_id, &src, &dst, index_name, &props).expect("Failed to update edge indexes");

        // 创建索引对象用于查询
        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;

        // 查询边索引
        let results = manager.lookup_edge_index(space_id, &index, &Value::Float(10.5)).expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], src);

        // 查询不存在的值
        let empty_results = manager.lookup_edge_index(space_id, &index, &Value::Float(99.9)).expect("Failed to lookup edge index");
        assert!(empty_results.is_empty());
    }

    #[test]
    fn test_delete_vertex_indexes() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let vertex_id1 = Value::Int(1);
        let vertex_id2 = Value::Int(2);
        let index_name = "idx_person_name";

        // 为两个顶点创建索引
        let props1 = vec![("name".to_string(), Value::String("Alice".to_string()))];
        let props2 = vec![("name".to_string(), Value::String("Bob".to_string()))];

        manager.update_vertex_indexes(space_id, &vertex_id1, index_name, &props1).expect("Failed to update vertex indexes");
        manager.update_vertex_indexes(space_id, &vertex_id2, index_name, &props2).expect("Failed to update vertex indexes");

        // 验证两个索引都存在
        let index = create_test_index(index_name, "person");
        let results1 = manager.lookup_tag_index(space_id, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results1.len(), 1);

        let results2 = manager.lookup_tag_index(space_id, &index, &Value::String("Bob".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results2.len(), 1);

        // 删除第一个顶点的索引
        manager.delete_vertex_indexes(space_id, &vertex_id1).expect("Failed to delete vertex indexes");

        // 验证第一个顶点的索引已被删除，第二个仍然存在
        let results1_after = manager.lookup_tag_index(space_id, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert!(results1_after.is_empty());

        let results2_after = manager.lookup_tag_index(space_id, &index, &Value::String("Bob".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results2_after.len(), 1);
    }

    #[test]
    fn test_delete_edge_indexes() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let src1 = Value::Int(1);
        let dst1 = Value::Int(2);
        let src2 = Value::Int(3);
        let dst2 = Value::Int(4);
        let index_name = "idx_edge_weight";

        // 创建两条边的索引
        let props1 = vec![("weight".to_string(), Value::Float(10.5))];
        let props2 = vec![("weight".to_string(), Value::Float(20.5))];

        manager.update_edge_indexes(space_id, &src1, &dst1, index_name, &props1).expect("Failed to update edge indexes");
        manager.update_edge_indexes(space_id, &src2, &dst2, index_name, &props2).expect("Failed to update edge indexes");

        // 验证两条边索引都存在
        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;

        let results1 = manager.lookup_edge_index(space_id, &index, &Value::Float(10.5)).expect("Failed to lookup edge index");
        assert_eq!(results1.len(), 1);

        let results2 = manager.lookup_edge_index(space_id, &index, &Value::Float(20.5)).expect("Failed to lookup edge index");
        assert_eq!(results2.len(), 1);

        // 删除第一条边的索引
        manager.delete_edge_indexes(space_id, &src1, &dst1, &["knows".to_string()]).expect("Failed to delete edge indexes");

        // 验证第一条边的索引已被删除，第二条仍然存在
        let results1_after = manager.lookup_edge_index(space_id, &index, &Value::Float(10.5)).expect("Failed to lookup edge index");
        assert!(results1_after.is_empty());

        let results2_after = manager.lookup_edge_index(space_id, &index, &Value::Float(20.5)).expect("Failed to lookup edge index");
        assert_eq!(results2_after.len(), 1);
    }

    #[test]
    fn test_clear_edge_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";

        // 创建边索引
        let props = vec![("weight".to_string(), Value::Float(10.5))];
        manager.update_edge_indexes(space_id, &src, &dst, index_name, &props).expect("Failed to update edge indexes");

        // 验证索引存在
        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;
        let results = manager.lookup_edge_index(space_id, &index, &Value::Float(10.5)).expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);

        // 清空索引
        manager.clear_edge_index(space_id, index_name).expect("Failed to clear edge index");

        // 验证索引已被清空
        let results_after = manager.lookup_edge_index(space_id, &index, &Value::Float(10.5)).expect("Failed to lookup edge index");
        assert!(results_after.is_empty());
    }

    #[test]
    fn test_multiple_properties_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person";

        // 为同一个顶点创建多个属性索引
        let props = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ];

        manager.update_vertex_indexes(space_id, &vertex_id, index_name, &props).expect("Failed to update vertex indexes");

        // 创建索引对象
        let index = create_test_index(index_name, "person");

        // 验证可以通过不同属性值查询到同一个顶点
        let results_name = manager.lookup_tag_index(space_id, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results_name.len(), 1);
        assert_eq!(results_name[0], vertex_id);

        let results_age = manager.lookup_tag_index(space_id, &index, &Value::Int(30)).expect("Failed to lookup tag index");
        assert_eq!(results_age.len(), 1);
        assert_eq!(results_age[0], vertex_id);
    }

    #[test]
    fn test_binary_value_in_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_test";

        // 使用包含特殊字符的属性值（包含 ':' 字符）
        let props = vec![("data".to_string(), Value::String("hello:world:test".to_string()))];

        // 更新索引
        manager.update_vertex_indexes(space_id, &vertex_id, index_name, &props).expect("Failed to update vertex indexes");

        // 创建索引对象
        let index = create_test_index(index_name, "test");

        // 验证可以正确查询（证明二进制键格式正确处理了 ':' 字符）
        let results = manager.lookup_tag_index(space_id, &index, &Value::String("hello:world:test".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vertex_id);
    }

    #[test]
    fn test_different_space_isolation() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id1 = 1u64;
        let space_id2 = 2u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        // 在两个不同的空间创建相同 vertex_id 的索引
        manager.update_vertex_indexes(space_id1, &vertex_id, index_name, &props.clone()).expect("Failed to update vertex indexes");
        manager.update_vertex_indexes(space_id2, &vertex_id, index_name, &props).expect("Failed to update vertex indexes");

        // 创建索引对象
        let index = create_test_index(index_name, "person");

        // 验证空间隔离
        let results1 = manager.lookup_tag_index(space_id1, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results1.len(), 1);

        let results2 = manager.lookup_tag_index(space_id2, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results2.len(), 1);

        // 删除空间1的索引
        manager.delete_vertex_indexes(space_id1, &vertex_id).expect("Failed to delete vertex indexes");

        // 验证空间1的索引被删除，空间2的索引仍然存在
        let results1_after = manager.lookup_tag_index(space_id1, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert!(results1_after.is_empty());

        let results2_after = manager.lookup_tag_index(space_id2, &index, &Value::String("Alice".to_string())).expect("Failed to lookup tag index");
        assert_eq!(results2_after.len(), 1);
    }
}
