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
                // 格式: space_id:index_name:prop_value_len:prop_value:src_len:src:dst_len:dst
                let prop_value_bytes = Self::serialize_value(prop_value)?;
                let src_bytes = Self::serialize_value(src)?;
                let dst_bytes = Self::serialize_value(dst)?;

                let mut key_bytes = Vec::new();
                // space_id:index_name:
                key_bytes.extend_from_slice(format!("{}:{}:", space_id, index_name).as_bytes());
                // prop_value_len:prop_value
                key_bytes.extend_from_slice(format!("{}:", prop_value_bytes.len()).as_bytes());
                key_bytes.extend_from_slice(&prop_value_bytes);
                key_bytes.push(b':');
                // src_len:src
                key_bytes.extend_from_slice(format!("{}:", src_bytes.len()).as_bytes());
                key_bytes.extend_from_slice(&src_bytes);
                key_bytes.push(b':');
                // dst_len:dst
                key_bytes.extend_from_slice(format!("{}:", dst_bytes.len()).as_bytes());
                key_bytes.extend_from_slice(&dst_bytes);

                table.insert(ByteKey(key_bytes), ByteKey(prop_name.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(format!("插入边索引数据失败: {}", e)))?;

                // 构建反向索引以便删除时查找
                // 格式: space_id:reverse:index_name:src_len:src
                let mut reverse_key_bytes = Vec::new();
                reverse_key_bytes.extend_from_slice(format!("{}:reverse:{}:", space_id, index_name).as_bytes());
                reverse_key_bytes.extend_from_slice(format!("{}:", src_bytes.len()).as_bytes());
                reverse_key_bytes.extend_from_slice(&src_bytes);

                let prop_value_key = format!("{}:{}", prop_name, prop_value_bytes.len());
                table.insert(ByteKey(reverse_key_bytes), ByteKey(prop_value_key.into_bytes()))
                    .map_err(|e| StorageError::DbError(format!("插入边反向索引失败: {}", e)))?;
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

            let vertex_bytes = Self::serialize_value(vertex_id)?;

            // 首先通过反向索引查找所有相关的正向索引键
            // 反向索引格式: space_id:reverse:index_name:vertex_id_len:vertex_id
            let reverse_prefix = format!("{}:reverse:", space_id);

            let mut forward_keys_to_delete: Vec<ByteKey> = Vec::new();
            let mut reverse_keys_to_delete: Vec<ByteKey> = Vec::new();

            for entry in table.iter().map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))? {
                if let Ok((key, value)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    let key_str = String::from_utf8_lossy(&key_bytes);

                    // 检查是否是反向索引
                    if key_str.starts_with(&reverse_prefix) {
                        // 解析反向索引键: space_id:reverse:index_name:vertex_id_len:vertex_id
                        let parts: Vec<&str> = key_str.split(':').collect();
                        if parts.len() >= 5 {
                            if let Ok(vid_len) = parts[3].parse::<usize>() {
                                let vid_start = key_bytes.len() - vid_len;
                                if vid_start < key_bytes.len() {
                                    let key_vid_bytes = &key_bytes[vid_start..];
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
                                            // 获取索引名称
                                            let index_name = parts[2];

                                            // 构建正向索引键前缀: space_id:index_name:
                                            let forward_prefix = format!("{}:{}:", space_id, index_name);

                                            // 查找匹配的正向索引
                                            for fwd_entry in table.iter().map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))? {
                                                if let Ok((fwd_key, fwd_value)) = fwd_entry {
                                                    let fwd_key_bytes: Vec<u8> = fwd_key.value().0.clone();
                                                    let fwd_key_str = String::from_utf8_lossy(&fwd_key_bytes);

                                                    // 检查是否是正向索引且匹配前缀
                                                    if fwd_key_str.starts_with(&forward_prefix) && !fwd_key_str.contains("reverse") {
                                                        // 检查 value 是否匹配 prop_name
                                                        let fwd_value_bytes: Vec<u8> = fwd_value.value().0.clone();
                                                        let fwd_value_str = String::from_utf8_lossy(&fwd_value_bytes);

                                                        if fwd_value_str == prop_name {
                                                            // 解析正向索引键检查 vertex_id
                                                            // 格式: space_id:index_name:prop_value_len:prop_value:vertex_id_len:vertex_id
                                                            let fwd_parts: Vec<&str> = fwd_key_str.split(':').collect();
                                                            if fwd_parts.len() >= 6 {
                                                                if let (Ok(prop_val_len), Ok(vid_len2)) = (fwd_parts[2].parse::<usize>(), fwd_parts[4].parse::<usize>()) {
                                                                    // 计算 vertex_id 的位置
                                                                    let prefix_len = forward_prefix.len();
                                                                    let prop_val_start = prefix_len + fwd_parts[2].len() + 1;
                                                                    let prop_val_end = prop_val_start + prop_val_len;
                                                                    let vid_start2 = prop_val_end + 1 + fwd_parts[4].len() + 1;
                                                                    let vid_end2 = vid_start2 + vid_len2;

                                                                    if vid_end2 <= fwd_key_bytes.len() {
                                                                        let fwd_vid_bytes = &fwd_key_bytes[vid_start2..vid_end2];
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

    fn delete_edge_indexes(&self, space_id: i32, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写入事务失败: {}", e)))?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开索引数据表失败: {}", e)))?;

            let src_bytes = Self::serialize_value(src)?;

            // 首先通过反向索引查找所有相关的边索引
            // 反向索引格式: space_id:reverse:index_name:src_len:src
            let reverse_prefix = format!("{}:reverse:", space_id);

            let mut index_names_to_check: Vec<String> = Vec::new();

            // 收集所有与该边类型相关的索引名称
            // TODO: 当前实现无法根据 edge_type 过滤索引，因为数据层无法访问索引元数据
            // 需要在调用方（IndexUpdater）根据 edge_type 过滤索引后，传入索引名称列表
            for entry in table.iter().map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))? {
                if let Ok((key, _)) = entry {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    if key_bytes.starts_with(reverse_prefix.as_bytes()) {
                        // 检查是否匹配 src
                        // 键格式: space_id:reverse:index_name:src_len:src
                        let key_str = String::from_utf8_lossy(&key_bytes);
                        let parts: Vec<&str> = key_str.split(':').collect();
                        if parts.len() >= 5 {
                            if let Ok(src_len) = parts[3].parse::<usize>() {
                                let src_start = key_bytes.len() - src_len;
                                if src_start < key_bytes.len() {
                                    let key_src_bytes = &key_bytes[src_start..];
                                    if key_src_bytes == src_bytes {
                                        // 提取索引名称
                                        if parts.len() >= 4 {
                                            index_names_to_check.push(parts[2].to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 记录边类型信息用于调试
            // 当前实现会删除所有匹配 src 和 dst 的边索引，而不管边类型
            // 这可能在同一 src 和 dst 之间有不同边类型时导致误删
            if index_names_to_check.is_empty() {
                return Ok(());
            }

            // edge_type 用于标识要删除索引的边类型
            // 当前实现遍历所有索引，未来应该根据 edge_type 过滤索引名称
            let _edge_type_info = edge_type;

            // 删除所有相关的边索引条目
            // 正向索引格式: space_id:index_name:prop_value_len:prop_value:src_len:src:dst_len:dst
            let keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        let key_str = String::from_utf8_lossy(&key_bytes);

                        // 检查是否是正向索引且匹配该边
                        for index_name in &index_names_to_check {
                            let prefix = format!("{}:{}:", space_id, index_name);
                            if key_str.starts_with(&prefix) {
                                // 解析键中的 src 和 dst
                                // 格式: space_id:index_name:prop_value_len:prop_value:src_len:src:dst_len:dst
                                let parts: Vec<&str> = key_str.split(':').collect();
                                if parts.len() >= 8 {
                                    // parts[2] = prop_value_len, parts[4] = src_len, parts[6] = dst_len
                                    if let (Ok(prop_len), Ok(src_len), Ok(dst_len)) = (
                                        parts[2].parse::<usize>(),
                                        parts[4].parse::<usize>(),
                                        parts[6].parse::<usize>()
                                    ) {
                                        // 计算 src 和 dst 的位置
                                        // space_id:index_name:prop_value_len: (3 parts + 2 colons)
                                        let mut pos = prefix.len() + parts[2].len() + 1 + prop_len + 1;
                                        // skip src_len:
                                        pos += parts[4].len() + 1;
                                        let src_end = pos + src_len;
                                        // skip :dst_len:
                                        let dst_start = src_end + 1 + parts[6].len() + 1;
                                        let dst_end = dst_start + dst_len;

                                        if src_end <= key_bytes.len() && dst_end <= key_bytes.len() {
                                            let key_src = &key_bytes[pos..src_end];
                                            let key_dst = &key_bytes[dst_start..dst_end];

                                            let dst_bytes = Self::serialize_value(dst).ok()?;
                                            if key_src == src_bytes && key_dst == dst_bytes {
                                                return Some(ByteKey(key_bytes));
                                            }
                                        }
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
                    .map_err(|e| StorageError::DbError(format!("删除边索引数据失败: {}", e)))?;
            }

            // 删除反向索引
            let reverse_keys_to_delete: Vec<ByteKey> = table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历索引数据失败: {}", e)))?
                .filter_map(|entry| {
                    if let Ok((key, _)) = entry {
                        let key_bytes: Vec<u8> = key.value().0.clone();
                        if key_bytes.starts_with(reverse_prefix.as_bytes()) {
                            // 检查是否匹配 src
                            let key_str = String::from_utf8_lossy(&key_bytes);
                            let parts: Vec<&str> = key_str.split(':').collect();
                            if parts.len() >= 5 {
                                if let Ok(src_len) = parts[3].parse::<usize>() {
                                    let src_start = key_bytes.len() - src_len;
                                    if src_start < key_bytes.len() {
                                        let key_src_bytes = &key_bytes[src_start..];
                                        if key_src_bytes == src_bytes {
                                            return Some(ByteKey(key_bytes));
                                        }
                                    }
                                }
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
                entry.ok().and_then(|(key, _)| {
                    let key_bytes: Vec<u8> = key.value().0.clone();
                    let key_str = String::from_utf8_lossy(&key_bytes);

                    // 检查前缀匹配
                    if !key_str.starts_with(&prefix) {
                        return None;
                    }

                    // 解析边索引键
                    // 格式: space_id:index_name:prop_value_len:prop_value:src_len:src:dst_len:dst
                    let parts: Vec<&str> = key_str.split(':').collect();
                    if parts.len() < 8 {
                        return None;
                    }

                    // 解析 prop_value_len
                    let prop_len = parts[2].parse::<usize>().ok()?;

                    // 计算 prop_value 的起始位置
                    // space_id:index_name:prop_value_len: (3 parts + 3 colons)
                    let prop_start = prefix.len() + parts[2].len() + 1;
                    let prop_end = prop_start + prop_len;

                    if prop_end > key_bytes.len() {
                        return None;
                    }

                    let key_prop_bytes = &key_bytes[prop_start..prop_end];

                    // 检查属性值是否匹配
                    if key_prop_bytes != value_bytes {
                        return None;
                    }

                    // 解析 src_len 和 src
                    let src_len = parts[4].parse::<usize>().ok()?;
                    // prop_end + 1 (colon) + parts[4].len() + 1 (colon)
                    let src_start = prop_end + 1 + parts[4].len() + 1;
                    let src_end = src_start + src_len;

                    if src_end > key_bytes.len() {
                        return None;
                    }

                    let src_bytes = &key_bytes[src_start..src_end];

                    // 反序列化 src
                    Self::deserialize_value(src_bytes).ok()
                })
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
