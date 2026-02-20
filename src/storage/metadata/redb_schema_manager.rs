use crate::core::StorageError;
use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::value::Value;
use crate::storage::{FieldDef, Schema};
use crate::storage::redb_types::{
    ByteKey, SPACES_TABLE, TAGS_TABLE, EDGE_TYPES_TABLE,
    TAG_ID_COUNTER_TABLE, EDGE_TYPE_ID_COUNTER_TABLE,
    TAG_NAME_INDEX_TABLE, EDGE_TYPE_NAME_INDEX_TABLE
};
use crate::storage::serializer::{space_to_bytes, space_from_bytes, tag_to_bytes, tag_from_bytes, edge_type_to_bytes, edge_type_from_bytes};
use redb::{Database, ReadableTable};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// 将 TagInfo 转换为 Schema
fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
    let fields: Vec<FieldDef> = tag_info.properties.iter().map(|prop| {
        FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        }
    }).collect();

    Schema {
        name: tag_name.to_string(),
        version: 1,
        fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
    }
}

/// 将 EdgeTypeInfo 转换为 Schema
fn edge_type_info_to_schema(edge_type_name: &str, edge_info: &EdgeTypeInfo) -> Schema {
    let fields: Vec<FieldDef> = edge_info.properties.iter().map(|prop| {
        FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        }
    }).collect();

    Schema {
        name: edge_type_name.to_string(),
        version: 1,
        fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
    }
}

/// 将 PropertyDef 转换为 FieldDef Map
fn _property_defs_to_fields(properties: &[PropertyDef]) -> BTreeMap<String, FieldDef> {
    let mut fields = BTreeMap::new();
    for prop in properties {
        let field = FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        };
        fields.insert(prop.name.clone(), field);
    }
    fields
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
        f.debug_struct("RedbSchemaManager").finish()
    }
}

impl RedbSchemaManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 为指定Space生成下一个Tag ID
    fn next_tag_id(&self, space: &str) -> Result<i32, StorageError> {
        let key = space.as_bytes();
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let next_id = {
            let mut table = write_txn.open_table(TAG_ID_COUNTER_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            let current = match table.get(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let bytes = value.value().0;
                    i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
                }
                None => 0,
            };
            
            let next = current + 1;
            table.insert(ByteKey(key.to_vec()), ByteKey(next.to_le_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            next
        };
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(next_id)
    }

    /// 为指定Space生成下一个Edge Type ID
    fn next_edge_type_id(&self, space: &str) -> Result<i32, StorageError> {
        let key = space.as_bytes();
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let next_id = {
            let mut table = write_txn.open_table(EDGE_TYPE_ID_COUNTER_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            let current = match table.get(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let bytes = value.value().0;
                    i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
                }
                None => 0,
            };
            
            let next = current + 1;
            table.insert(ByteKey(key.to_vec()), ByteKey(next.to_le_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            next
        };
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(next_id)
    }

    /// 检查Tag名称是否与现有Edge冲突
    fn check_tag_name_conflict(&self, space: &str, tag_name: &str) -> Result<(), StorageError> {
        let key = format!("{}:{}", space, tag_name);
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        if table.get(ByteKey(key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .is_some() {
            return Err(StorageError::DbError(
                format!("Tag '{}' conflicts with existing edge type in space '{}'", tag_name, space)
            ));
        }
        
        Ok(())
    }

    /// 检查Edge名称是否与现有Tag冲突
    fn check_edge_name_conflict(&self, space: &str, edge_name: &str) -> Result<(), StorageError> {
        let key = format!("{}:{}", space, edge_name);
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        if table.get(ByteKey(key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .is_some() {
            return Err(StorageError::DbError(
                format!("Edge type '{}' conflicts with existing tag in space '{}'", edge_name, space)
            ));
        }
        
        Ok(())
    }
}

impl crate::storage::metadata::SchemaManager for RedbSchemaManager {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let key = space.space_name.as_bytes();
        let space_bytes = space_to_bytes(space)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(SPACES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key.to_vec()), ByteKey(space_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError> {
        let key = space_name.as_bytes();

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(SPACES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let key = space_name.as_bytes();

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let space_bytes = value.value().0;
                let space: SpaceInfo = space_from_bytes(&space_bytes)?;
                Ok(Some(space))
            }
            None => Ok(None),
        }
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.list_spaces()?;
        Ok(spaces.into_iter().find(|s| s.space_id == space_id))
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut spaces = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (_, space_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let space: SpaceInfo = space_from_bytes(&space_bytes.value().0)?;
            spaces.push(space);
        }

        Ok(spaces)
    }

    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        // 1. 检查名称冲突 - Tag和Edge不能同名
        self.check_tag_name_conflict(space, &tag.tag_name)?;
        
        // 2. 生成Tag ID（如果未设置）
        let tag_id = if tag.tag_id == 0 {
            self.next_tag_id(space)?
        } else {
            tag.tag_id
        };
        
        // 3. 创建带有ID的TagInfo
        let mut tag_with_id = tag.clone();
        tag_with_id.tag_id = tag_id;
        
        // 4. 存储键：space:tag_id（使用ID作为主键）
        let schema_key = format!("{}:{}", space, tag_id);
        let tag_bytes = tag_to_bytes(&tag_with_id)?;
        
        // 5. 名称索引键：space:tag_name
        let name_index_key = format!("{}:{}", space, tag.tag_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            // 检查名称索引是否已存在
            let mut name_table = write_txn.open_table(TAG_NAME_INDEX_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            if name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false); // Tag已存在
            }
            
            // 写入名称索引: name -> id
            name_table.insert(
                ByteKey(name_index_key.as_bytes().to_vec()),
                ByteKey(tag_id.to_le_bytes().to_vec())
            ).map_err(|e| StorageError::DbError(e.to_string()))?;
            
            // 写入Schema数据: id -> tag_info
            let mut schema_table = write_txn.open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            schema_table.insert(ByteKey(schema_key.as_bytes().to_vec()), ByteKey(tag_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        // 1. 通过名称索引查找Tag ID
        let name_index_key = format!("{}:{}", space, tag_name);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 2. 从名称索引表获取Tag ID
        let name_table = read_txn.open_table(TAG_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let tag_id = match name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let bytes = value.value().0;
                i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
            }
            None => return Ok(None),
        };
        
        // 3. 使用Tag ID从Schema表获取完整信息
        let schema_key = format!("{}:{}", space, tag_id);
        let schema_table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        match schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let tag_bytes = value.value().0;
                let tag: TagInfo = tag_from_bytes(&tag_bytes)?;
                Ok(Some(tag))
            }
            None => Ok(None),
        }
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 从名称索引表获取所有Tag名称，然后通过ID查询完整信息
        let name_table = read_txn.open_table(TAG_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let schema_table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut tags = Vec::new();
        for result in name_table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key_bytes, id_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            
            // 检查是否属于当前Space
            if key_str.starts_with(&format!("{}:", space)) {
                let tag_id = i32::from_le_bytes(id_bytes.value().0.try_into().unwrap_or([0; 4]));
                let schema_key = format!("{}:{}", space, tag_id);
                
                // 获取完整Tag信息
                if let Some(value) = schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(e.to_string()))? {
                    let tag: TagInfo = tag_from_bytes(&value.value().0)?;
                    tags.push(tag);
                }
            }
        }

        Ok(tags)
    }

    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        let name_index_key = format!("{}:{}", space, tag_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            // 1. 从名称索引表获取Tag ID
            let mut name_table = write_txn.open_table(TAG_NAME_INDEX_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            let tag_id = match name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let bytes = value.value().0;
                    i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
                }
                None => return Ok(false),
            };
            
            // 2. 删除名称索引
            name_table.remove(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            // 3. 删除Schema数据
            let schema_key = format!("{}:{}", space, tag_id);
            let mut schema_table = write_txn.open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            schema_table.remove(ByteKey(schema_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        // 1. 检查名称冲突 - Edge和Tag不能同名
        self.check_edge_name_conflict(space, &edge.edge_type_name)?;
        
        // 2. 生成Edge Type ID（如果未设置）
        let edge_type_id = if edge.edge_type_id == 0 {
            self.next_edge_type_id(space)?
        } else {
            edge.edge_type_id
        };
        
        // 3. 创建带有ID的EdgeTypeInfo
        let mut edge_with_id = edge.clone();
        edge_with_id.edge_type_id = edge_type_id;
        
        // 4. 存储键：space:edge_type_id（使用ID作为主键）
        let schema_key = format!("{}:{}", space, edge_type_id);
        let edge_bytes = edge_type_to_bytes(&edge_with_id)?;
        
        // 5. 名称索引键：space:edge_type_name
        let name_index_key = format!("{}:{}", space, edge.edge_type_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            // 检查名称索引是否已存在
            let mut name_table = write_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            if name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false); // Edge type已存在
            }
            
            // 写入名称索引: name -> id
            name_table.insert(
                ByteKey(name_index_key.as_bytes().to_vec()),
                ByteKey(edge_type_id.to_le_bytes().to_vec())
            ).map_err(|e| StorageError::DbError(e.to_string()))?;
            
            // 写入Schema数据: id -> edge_type_info
            let mut schema_table = write_txn.open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            schema_table.insert(ByteKey(schema_key.as_bytes().to_vec()), ByteKey(edge_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        // 1. 通过名称索引查找Edge Type ID
        let name_index_key = format!("{}:{}", space, edge_type_name);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 2. 从名称索引表获取Edge Type ID
        let name_table = read_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let edge_type_id = match name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let bytes = value.value().0;
                i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
            }
            None => return Ok(None),
        };
        
        // 3. 使用Edge Type ID从Schema表获取完整信息
        let schema_key = format!("{}:{}", space, edge_type_id);
        let schema_table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        match schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let edge_bytes = value.value().0;
                let edge: EdgeTypeInfo = edge_type_from_bytes(&edge_bytes)?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 从名称索引表获取所有Edge Type名称，然后通过ID查询完整信息
        let name_table = read_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let schema_table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in name_table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key_bytes, id_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            
            // 检查是否属于当前Space
            if key_str.starts_with(&format!("{}:", space)) {
                let edge_type_id = i32::from_le_bytes(id_bytes.value().0.try_into().unwrap_or([0; 4]));
                let schema_key = format!("{}:{}", space, edge_type_id);
                
                // 获取完整Edge Type信息
                if let Some(value) = schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
                    .map_err(|e| StorageError::DbError(e.to_string()))? {
                    let edge: EdgeTypeInfo = edge_type_from_bytes(&value.value().0)?;
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let name_index_key = format!("{}:{}", space, edge_type_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            // 1. 从名称索引表获取Edge Type ID
            let mut name_table = write_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            let edge_type_id = match name_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let bytes = value.value().0;
                    i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
                }
                None => return Ok(false),
            };
            
            // 2. 删除名称索引
            name_table.remove(ByteKey(name_index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            // 3. 删除Schema数据
            let schema_key = format!("{}:{}", space, edge_type_id);
            let mut schema_table = write_txn.open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            schema_table.remove(ByteKey(schema_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        // 1. 先通过名称索引查找TagID
        let name_index_key = format!("{}:{}", space, tag);
        let name_index_table = read_txn.open_table(TAG_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let tag_id = match name_index_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let id_bytes = value.value().0;
                i32::from_le_bytes([id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]])
            }
            None => return Err(StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))),
        };

        // 2. 再通过TagID查询schema
        let schema_key = format!("{}:{}", space, tag_id);
        let schema_table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let tag_bytes = value.value().0;
                let tag_info: TagInfo = tag_from_bytes(&tag_bytes)?;
                Ok(tag_info_to_schema(tag, &tag_info))
            }
            None => Err(StorageError::DbError(format!("Tag '{}' schema not found in space '{}'", tag, space))),
        }
    }

    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        // 1. 先通过名称索引查找EdgeTypeID
        let name_index_key = format!("{}:{}", space, edge);
        let name_index_table = read_txn.open_table(EDGE_TYPE_NAME_INDEX_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edge_type_id = match name_index_table.get(ByteKey(name_index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let id_bytes = value.value().0;
                i32::from_le_bytes([id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]])
            }
            None => return Err(StorageError::DbError(format!("Edge type '{}' not found in space '{}'", edge, space))),
        };

        // 2. 再通过EdgeTypeID查询schema
        let schema_key = format!("{}:{}", space, edge_type_id);
        let schema_table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match schema_table.get(ByteKey(schema_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let edge_bytes = value.value().0;
                let edge_info: EdgeTypeInfo = edge_type_from_bytes(&edge_bytes)?;
                Ok(edge_type_info_to_schema(edge, &edge_info))
            }
            None => Err(StorageError::DbError(format!("Edge type '{}' schema not found in space '{}'", edge, space))),
        }
    }
}
