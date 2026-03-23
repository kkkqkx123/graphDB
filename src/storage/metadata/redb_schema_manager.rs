use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::value::Value;
use crate::core::StorageError;
use crate::storage::redb_types::{
    ByteKey, EDGE_TYPES_TABLE, EDGE_TYPE_ID_COUNTER_TABLE, SPACES_TABLE, SPACE_NAME_INDEX_TABLE,
    TAGS_TABLE, TAG_ID_COUNTER_TABLE,
};
use crate::storage::{FieldDef, Schema};
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use redb::{Database, ReadableTable};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// 将 TagInfo 转换为 Schema
fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
    let fields: BTreeMap<String, FieldDef> = tag_info
        .properties
        .iter()
        .map(|prop| {
            let field_def: FieldDef = prop.clone().into();
            (field_def.name.clone(), field_def)
        })
        .collect();

    Schema {
        name: tag_name.to_string(),
        version: 1,
        fields,
    }
}

/// 将 EdgeTypeInfo 转换为 Schema
fn edge_type_info_to_schema(edge_type_name: &str, edge_info: &EdgeTypeInfo) -> Schema {
    let fields: BTreeMap<String, FieldDef> = edge_info
        .properties
        .iter()
        .map(|prop| {
            let field_def: FieldDef = prop.clone().into();
            (field_def.name.clone(), field_def)
        })
        .collect();

    Schema {
        name: edge_type_name.to_string(),
        version: 1,
        fields,
    }
}

/// 将 PropertyDef 转换为 FieldDef Map
fn _property_defs_to_fields(properties: &[PropertyDef]) -> BTreeMap<String, FieldDef> {
    properties
        .iter()
        .map(|prop| {
            let field_def: FieldDef = prop.clone().into();
            (field_def.name.clone(), field_def)
        })
        .collect()
}

/// 将 PropertyDef 转换为 Value HashMap
fn _property_defs_to_hashmap(properties: &[PropertyDef]) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for prop in properties {
        if let Some(default_value) = &prop.default {
            map.insert(prop.name.clone(), default_value.clone());
        }
    }
    map
}

pub struct RedbSchemaManager {
    db: Arc<Database>,
}

impl std::fmt::Debug for RedbSchemaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbSchemaManager")
            .field("db", &"<Database>")
            .finish()
    }
}

impl RedbSchemaManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl super::SchemaManager for RedbSchemaManager {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        {
            // 检查名称索引
            let mut name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("打开SPACE_NAME_INDEX_TABLE失败: {}", e))
            })?;

            let name_key = ByteKey(space.space_name.as_bytes().to_vec());

            if name_index
                .get(&name_key)
                .map_err(|e| StorageError::DbError(format!("查询名称索引失败: {}", e)))?
                .is_some()
            {
                return Ok(false);
            }

            // 插入主表
            let mut spaces_table = write_txn
                .open_table(SPACES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开SPACES_TABLE失败: {}", e)))?;

            let space_key = ByteKey(space.space_id.to_be_bytes().to_vec());
            let space_value = ByteKey(encode_to_vec(space, standard())?);

            spaces_table
                .insert(space_key, space_value)
                .map_err(|e| StorageError::DbError(format!("插入空间失败: {}", e)))?;

            // 插入名称索引
            let id_value = ByteKey(space.space_id.to_be_bytes().to_vec());
            name_index
                .insert(name_key, id_value)
                .map_err(|e| StorageError::DbError(format!("插入名称索引失败: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(true)
    }

    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        // 通过名称索引查找ID
        let name_key = ByteKey(space_name.as_bytes().to_vec());
        let space_id_option = {
            let name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("打开SPACE_NAME_INDEX_TABLE失败: {}", e))
            })?;

            let result = name_index
                .get(&name_key)
                .map_err(|e| StorageError::DbError(format!("查询名称索引失败: {}", e)))?;

            match result {
                Some(id_value) => {
                    let bytes = id_value.value().0;
                    let array: [u8; 8] = bytes[0..8]
                        .try_into()
                        .map_err(|_| StorageError::DbError("ID字节长度不足8字节".to_string()))?;
                    Some(u64::from_be_bytes(array))
                }
                None => None,
            }
        };

        if let Some(space_id) = space_id_option {
            // 删除主表记录
            {
                let mut spaces_table = write_txn
                    .open_table(SPACES_TABLE)
                    .map_err(|e| StorageError::DbError(format!("打开SPACES_TABLE失败: {}", e)))?;

                let space_key = ByteKey(space_id.to_be_bytes().to_vec());
                spaces_table
                    .remove(&space_key)
                    .map_err(|e| StorageError::DbError(format!("删除空间失败: {}", e)))?;
            }

            // 删除名称索引
            {
                let mut name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                    StorageError::DbError(format!("打开SPACE_NAME_INDEX_TABLE失败: {}", e))
                })?;

                name_index
                    .remove(&name_key)
                    .map_err(|e| StorageError::DbError(format!("删除名称索引失败: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        // 通过名称索引查找
        let name_index = read_txn
            .open_table(SPACE_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开SPACE_NAME_INDEX_TABLE失败: {}", e)))?;

        let name_key = ByteKey(space_name.as_bytes().to_vec());

        if let Some(id_value) = name_index
            .get(&name_key)
            .map_err(|e| StorageError::DbError(format!("查询名称索引失败: {}", e)))?
        {
            let id_bytes = id_value.value().0;
            let space_id = u64::from_be_bytes(
                id_bytes[0..8]
                    .try_into()
                    .map_err(|_| StorageError::DbError("ID字节长度不足8字节".to_string()))?,
            );

            // 通过ID获取完整信息
            let spaces_table = read_txn
                .open_table(SPACES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开SPACES_TABLE失败: {}", e)))?;

            let space_key = ByteKey(space_id.to_be_bytes().to_vec());

            if let Some(space_value) = spaces_table
                .get(&space_key)
                .map_err(|e| StorageError::DbError(format!("查询空间失败: {}", e)))?
            {
                let space: SpaceInfo = decode_from_slice(&space_value.value().0, standard())?.0;
                return Ok(Some(space));
            }
        }

        Ok(None)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let spaces_table = read_txn
            .open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开SPACES_TABLE失败: {}", e)))?;

        let key = ByteKey(space_id.to_be_bytes().to_vec());

        match spaces_table
            .get(&key)
            .map_err(|e| StorageError::DbError(format!("查询空间失败: {}", e)))?
        {
            Some(value) => {
                let space: SpaceInfo = decode_from_slice(&value.value().0, standard())?.0;
                Ok(Some(space))
            }
            None => Ok(None),
        }
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let spaces_table = read_txn
            .open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开SPACES_TABLE失败: {}", e)))?;

        let mut spaces = Vec::new();

        let iter = spaces_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历空间失败: {}", e)))?;

        for result in iter {
            let (_key, value) =
                result.map_err(|e| StorageError::DbError(format!("迭代空间失败: {}", e)))?;
            let space: SpaceInfo = decode_from_slice(&value.value().0, standard())?.0;
            spaces.push(space);
        }

        Ok(spaces)
    }

    fn create_tag(&self, space_name: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        {
            let mut tags_table = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开TAGS_TABLE失败: {}", e)))?;

            let key = ByteKey(
                [
                    space_info.space_id.to_be_bytes().to_vec(),
                    tag.tag_id.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            let value = ByteKey(encode_to_vec(tag, standard())?);

            if tags_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("查询标签失败: {}", e)))?
                .is_some()
            {
                return Ok(false);
            }

            tags_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("插入标签失败: {}", e)))?;
        }

        {
            let mut id_counter_table = write_txn.open_table(TAG_ID_COUNTER_TABLE).map_err(|e| {
                StorageError::DbError(format!("打开TAG_ID_COUNTER_TABLE失败: {}", e))
            })?;

            let key = ByteKey(space_info.space_id.to_be_bytes().to_vec());
            let current_id = id_counter_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("查询ID计数器失败: {}", e)))?
                .map(|v| {
                    let bytes = v.value().0;
                    u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])
                })
                .unwrap_or(0);

            let new_id = current_id + 1;
            let value = ByteKey(new_id.to_be_bytes().to_vec());

            id_counter_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("更新ID计数器失败: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(true)
    }

    fn drop_tag(&self, space_name: &str, tag_name: &str) -> Result<bool, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        let mut tag_id: Option<i32> = None;

        {
            let tags_table = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开TAGS_TABLE失败: {}", e)))?;

            let iter = tags_table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历标签失败: {}", e)))?;

            for result in iter {
                let (key, value) =
                    result.map_err(|e| StorageError::DbError(format!("迭代标签失败: {}", e)))?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let tag: TagInfo = decode_from_slice(&value.value().0, standard())?.0;
                    if tag.tag_name == tag_name {
                        let id_bytes = &key_bytes[8..12];
                        tag_id = Some(i32::from_be_bytes([
                            id_bytes[0],
                            id_bytes[1],
                            id_bytes[2],
                            id_bytes[3],
                        ]));
                        break;
                    }
                }
            }
        }

        if let Some(id) = tag_id {
            {
                let mut tags_table = write_txn
                    .open_table(TAGS_TABLE)
                    .map_err(|e| StorageError::DbError(format!("打开TAGS_TABLE失败: {}", e)))?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                tags_table
                    .remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除标签失败: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

            return Ok(true);
        }

        Ok(false)
    }

    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let tags_table = read_txn
            .open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开TAGS_TABLE失败: {}", e)))?;

        let iter = tags_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历标签失败: {}", e)))?;

        for result in iter {
            let (key, value) =
                result.map_err(|e| StorageError::DbError(format!("迭代标签失败: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let tag: TagInfo = decode_from_slice(&value.value().0, standard())?.0;
                if tag.tag_name == tag_name {
                    return Ok(Some(tag));
                }
            }
        }

        Ok(None)
    }

    fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let tags_table = read_txn
            .open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开TAGS_TABLE失败: {}", e)))?;

        let mut tags = Vec::new();

        let iter = tags_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历标签失败: {}", e)))?;

        for result in iter {
            let (key, value) =
                result.map_err(|e| StorageError::DbError(format!("迭代标签失败: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let tag: TagInfo = decode_from_slice(&value.value().0, standard())?.0;
                tags.push(tag);
            }
        }

        Ok(tags)
    }

    fn create_edge_type(
        &self,
        space_name: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        {
            let mut edge_types_table = write_txn
                .open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开EDGE_TYPES_TABLE失败: {}", e)))?;

            let key = ByteKey(
                [
                    space_info.space_id.to_be_bytes().to_vec(),
                    edge_type.edge_type_id.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            let value = ByteKey(encode_to_vec(edge_type, standard())?);

            if edge_types_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("查询边类型失败: {}", e)))?
                .is_some()
            {
                return Ok(false);
            }

            edge_types_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("插入边类型失败: {}", e)))?;
        }

        {
            let mut id_counter_table =
                write_txn
                    .open_table(EDGE_TYPE_ID_COUNTER_TABLE)
                    .map_err(|e| {
                        StorageError::DbError(format!("打开EDGE_TYPE_ID_COUNTER_TABLE失败: {}", e))
                    })?;

            let key = ByteKey(space_info.space_id.to_be_bytes().to_vec());
            let current_id = id_counter_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("查询ID计数器失败: {}", e)))?
                .map(|v| {
                    let bytes = v.value().0;
                    u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])
                })
                .unwrap_or(0);

            let new_id = current_id + 1;
            let value = ByteKey(new_id.to_be_bytes().to_vec());

            id_counter_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("更新ID计数器失败: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

        Ok(true)
    }

    fn drop_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(format!("开始写事务失败: {}", e)))?;

        let mut edge_type_id: Option<i32> = None;

        {
            let edge_types_table = write_txn
                .open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开EDGE_TYPES_TABLE失败: {}", e)))?;

            let iter = edge_types_table
                .iter()
                .map_err(|e| StorageError::DbError(format!("遍历边类型失败: {}", e)))?;

            for result in iter {
                let (key, value) =
                    result.map_err(|e| StorageError::DbError(format!("迭代边类型失败: {}", e)))?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let edge_type: EdgeTypeInfo =
                        decode_from_slice(&value.value().0, standard())?.0;
                    if edge_type.edge_type_name == edge_type_name {
                        let id_bytes = &key_bytes[8..12];
                        edge_type_id = Some(i32::from_be_bytes([
                            id_bytes[0],
                            id_bytes[1],
                            id_bytes[2],
                            id_bytes[3],
                        ]));
                        break;
                    }
                }
            }
        }

        if let Some(id) = edge_type_id {
            {
                let mut edge_types_table = write_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
                    StorageError::DbError(format!("打开EDGE_TYPES_TABLE失败: {}", e))
                })?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                edge_types_table
                    .remove(&key)
                    .map_err(|e| StorageError::DbError(format!("删除边类型失败: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;

            return Ok(true);
        }

        Ok(false)
    }

    fn get_edge_type(
        &self,
        space_name: &str,
        edge_type_name: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let edge_types_table = read_txn
            .open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开EDGE_TYPES_TABLE失败: {}", e)))?;

        let iter = edge_types_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历边类型失败: {}", e)))?;

        for result in iter {
            let (key, value) =
                result.map_err(|e| StorageError::DbError(format!("迭代边类型失败: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let edge_type: EdgeTypeInfo = decode_from_slice(&value.value().0, standard())?.0;
                if edge_type.edge_type_name == edge_type_name {
                    return Ok(Some(edge_type));
                }
            }
        }

        Ok(None)
    }

    fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let space_info = self
            .get_space(space_name)?
            .ok_or_else(|| StorageError::DbError(format!("空间 '{}' 不存在", space_name)))?;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(format!("开始读事务失败: {}", e)))?;

        let edge_types_table = read_txn
            .open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(format!("打开EDGE_TYPES_TABLE失败: {}", e)))?;

        let mut edge_types = Vec::new();

        let iter = edge_types_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("遍历边类型失败: {}", e)))?;

        for result in iter {
            let (key, value) =
                result.map_err(|e| StorageError::DbError(format!("迭代边类型失败: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let edge_type: EdgeTypeInfo = decode_from_slice(&value.value().0, standard())?.0;
                edge_types.push(edge_type);
            }
        }

        Ok(edge_types)
    }

    fn get_tag_schema(&self, space_name: &str, tag: &str) -> Result<Schema, StorageError> {
        let tag_info = self
            .get_tag(space_name, tag)?
            .ok_or_else(|| StorageError::DbError(format!("标签 '{}' 不存在", tag)))?;

        Ok(tag_info_to_schema(tag, &tag_info))
    }

    fn get_edge_type_schema(&self, space_name: &str, edge: &str) -> Result<Schema, StorageError> {
        let edge_type_info = self
            .get_edge_type(space_name, edge)?
            .ok_or_else(|| StorageError::DbError(format!("边类型 '{}' 不存在", edge)))?;

        Ok(edge_type_info_to_schema(edge, &edge_type_info))
    }
}
