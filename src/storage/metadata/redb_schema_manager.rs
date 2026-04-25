use crate::core::types::{EdgeTypeInfo, Index, PropertyDef, SpaceInfo, TagInfo};
use crate::core::value::Value;
use crate::core::StorageError;
use crate::storage::engine::{
    ByteKey, EDGE_INDEXES_TABLE, EDGE_TYPES_TABLE, EDGE_TYPE_ID_COUNTER_TABLE, SPACES_TABLE,
    SPACE_NAME_INDEX_TABLE, TAGS_TABLE, TAG_ID_COUNTER_TABLE, TAG_INDEXES_TABLE,
};
use crate::storage::{FieldDef, Schema};
use oxicode::{decode_from_slice, encode_to_vec};
use redb::{Database, ReadableTable};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Converting TagInfo to Schema
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

/// Converting EdgeTypeInfo to Schema
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

/// Converting a PropertyDef to a FieldDef Map
fn _property_defs_to_fields(properties: &[PropertyDef]) -> BTreeMap<String, FieldDef> {
    properties
        .iter()
        .map(|prop| {
            let field_def: FieldDef = prop.clone().into();
            (field_def.name.clone(), field_def)
        })
        .collect()
}

/// Converting a PropertyDef to a Value HashMap
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
        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        {
            // Check name index
            let mut name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACE_NAME_INDEX_TABLE: {}", e))
            })?;

            let name_key = ByteKey(space.space_name.as_bytes().to_vec());

            if name_index
                .get(&name_key)
                .map_err(|e| StorageError::DbError(format!("Failed to query name index: {}", e)))?
                .is_some()
            {
                return Ok(false);
            }

            // Insertion into the master table
            let mut spaces_table = write_txn.open_table(SPACES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e))
            })?;

            let space_key = ByteKey(space.space_id.to_be_bytes().to_vec());
            let space_value = ByteKey(encode_to_vec(space)?);

            spaces_table
                .insert(space_key, space_value)
                .map_err(|e| StorageError::DbError(format!("Failed to insert space: {}", e)))?;

            // Insert name index
            let id_value = ByteKey(space.space_id.to_be_bytes().to_vec());
            name_index.insert(name_key, id_value).map_err(|e| {
                StorageError::DbError(format!("Failed to insert name index: {}", e))
            })?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(true)
    }

    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        // Find IDs by Name Index
        let name_key = ByteKey(space_name.as_bytes().to_vec());
        let space_id_option = {
            let name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACE_NAME_INDEX_TABLE: {}", e))
            })?;

            let result = name_index
                .get(&name_key)
                .map_err(|e| StorageError::DbError(format!("Failed to query name index: {}", e)))?;

            match result {
                Some(id_value) => {
                    let bytes = id_value.value().0;
                    let array: [u8; 8] = bytes[0..8].try_into().map_err(|_| {
                        StorageError::DbError("ID byte length is less than 8 bytes".to_string())
                    })?;
                    Some(u64::from_be_bytes(array))
                }
                None => None,
            }
        };

        if let Some(space_id) = space_id_option {
            // Deleting Main Table Records
            {
                let mut spaces_table = write_txn.open_table(SPACES_TABLE).map_err(|e| {
                    StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e))
                })?;

                let space_key = ByteKey(space_id.to_be_bytes().to_vec());
                spaces_table
                    .remove(&space_key)
                    .map_err(|e| StorageError::DbError(format!("Failed to delete space: {}", e)))?;
            }

            // Delete Name Index
            {
                let mut name_index = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                    StorageError::DbError(format!("Failed to open SPACE_NAME_INDEX_TABLE: {}", e))
                })?;

                name_index.remove(&name_key).map_err(|e| {
                    StorageError::DbError(format!("Failed to delete name index: {}", e))
                })?;
            }

            write_txn.commit().map_err(|e| {
                StorageError::DbError(format!("Failed to commit transaction: {}", e))
            })?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        // Search by Name Index
        let name_index = read_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
            StorageError::DbError(format!("Failed to open SPACE_NAME_INDEX_TABLE: {}", e))
        })?;

        let name_key = ByteKey(space_name.as_bytes().to_vec());

        if let Some(id_value) = name_index
            .get(&name_key)
            .map_err(|e| StorageError::DbError(format!("Failed to query name index: {}", e)))?
        {
            let id_bytes = id_value.value().0;
            let space_id = u64::from_be_bytes(id_bytes[0..8].try_into().map_err(|_| {
                StorageError::DbError("ID byte length is less than 8 bytes".to_string())
            })?);

            // Get full information by ID
            let spaces_table = read_txn.open_table(SPACES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e))
            })?;

            let space_key = ByteKey(space_id.to_be_bytes().to_vec());

            if let Some(space_value) = spaces_table
                .get(&space_key)
                .map_err(|e| StorageError::DbError(format!("Failed to query space: {}", e)))?
            {
                let space: SpaceInfo = decode_from_slice(&space_value.value().0)?.0;
                return Ok(Some(space));
            }
        }

        Ok(None)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let spaces_table = read_txn
            .open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e)))?;

        let key = ByteKey(space_id.to_be_bytes().to_vec());

        match spaces_table
            .get(&key)
            .map_err(|e| StorageError::DbError(format!("Failed to query space: {}", e)))?
        {
            Some(value) => {
                let space: SpaceInfo = decode_from_slice(&value.value().0)?.0;
                Ok(Some(space))
            }
            None => Ok(None),
        }
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let spaces_table = read_txn
            .open_table(SPACES_TABLE)
            .map_err(|e| StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e)))?;

        let mut spaces = Vec::new();

        let iter = spaces_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("Failed to iterate space: {}", e)))?;

        for result in iter {
            let (_key, value) = result
                .map_err(|e| StorageError::DbError(format!("Failed to iterate space: {}", e)))?;
            let space: SpaceInfo = decode_from_slice(&value.value().0)?.0;
            spaces.push(space);
        }

        Ok(spaces)
    }

    fn update_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        {
            let mut spaces_table = write_txn.open_table(SPACES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACES_TABLE: {}", e))
            })?;

            let space_key = ByteKey(space.space_id.to_be_bytes().to_vec());
            let space_value = ByteKey(encode_to_vec(space)?);

            spaces_table
                .insert(space_key, space_value)
                .map_err(|e| StorageError::DbError(format!("Failed to update space: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(true)
    }

    fn create_tag(&self, space_name: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        // Check if a tag with the same name already exists (before write transaction)
        let existing_tags = self.list_tags(space_name)?;
        if existing_tags.iter().any(|t| t.tag_name == tag.tag_name) {
            return Ok(false);
        }

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        // First, get the new tag_id from counter
        let new_tag_id = {
            let mut id_counter_table = write_txn.open_table(TAG_ID_COUNTER_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open TAG_ID_COUNTER_TABLE: {}", e))
            })?;

            let key = ByteKey(space_info.space_id.to_be_bytes().to_vec());
            let current_id = id_counter_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("Failed to query ID counter: {}", e)))?
                .map(|v| {
                    let bytes = v.value().0;
                    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                })
                .unwrap_or(0);

            let new_id = current_id + 1;
            let value = ByteKey(new_id.to_be_bytes().to_vec());

            id_counter_table.insert(key, value).map_err(|e| {
                StorageError::DbError(format!("Failed to update ID counter: {}", e))
            })?;

            new_id
        };

        // Create tag with the new tag_id
        let mut tag_with_id = tag.clone();
        tag_with_id.tag_id = new_tag_id as i32;

        {
            let mut tags_table = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;

            let key = ByteKey(
                [
                    space_info.space_id.to_be_bytes().to_vec(),
                    tag_with_id.tag_id.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            let value = ByteKey(encode_to_vec(&tag_with_id)?);

            tags_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("Failed to insert tag: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(true)
    }

    fn drop_tag(&self, space_name: &str, tag_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        let mut tag_id: Option<i32> = None;

        {
            let tags_table = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;

            let iter = tags_table
                .iter()
                .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;

            for result in iter {
                let (key, value) = result
                    .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let tag: TagInfo = decode_from_slice(&value.value().0)?.0;
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
                let mut tags_table = write_txn.open_table(TAGS_TABLE).map_err(|e| {
                    StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e))
                })?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                tags_table
                    .remove(&key)
                    .map_err(|e| StorageError::DbError(format!("Failed to delete tag: {}", e)))?;
            }

            write_txn.commit().map_err(|e| {
                StorageError::DbError(format!("Failed to commit transaction: {}", e))
            })?;

            return Ok(true);
        }

        Ok(false)
    }

    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let tags_table = read_txn
            .open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;

        let iter = tags_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;

        for result in iter {
            let (key, value) = result
                .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let tag: TagInfo = decode_from_slice(&value.value().0)?.0;
                if tag.tag_name == tag_name {
                    return Ok(Some(tag));
                }
            }
        }

        Ok(None)
    }

    fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let tags_table = read_txn
            .open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;

        let mut tags = Vec::new();

        let iter = tags_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;

        for result in iter {
            let (key, value) = result
                .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let tag: TagInfo = decode_from_slice(&value.value().0)?.0;
                tags.push(tag);
            }
        }

        Ok(tags)
    }

    fn update_tag(&self, space_name: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        let mut tag_id: Option<i32> = None;

        // Find the tag ID by name
        {
            let tags_table = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;

            let iter = tags_table
                .iter()
                .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;

            for result in iter {
                let (key, value) = result
                    .map_err(|e| StorageError::DbError(format!("Failed to iterate tag: {}", e)))?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let existing_tag: TagInfo = decode_from_slice(&value.value().0)?.0;
                    if existing_tag.tag_name == tag.tag_name {
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
                let mut tags_table = write_txn.open_table(TAGS_TABLE).map_err(|e| {
                    StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e))
                })?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                let value = ByteKey(encode_to_vec(tag)?);

                tags_table
                    .insert(key, value)
                    .map_err(|e| StorageError::DbError(format!("Failed to update tag: {}", e)))?;
            }

            write_txn.commit().map_err(|e| {
                StorageError::DbError(format!("Failed to commit transaction: {}", e))
            })?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_edge_type(
        &self,
        space_name: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let existing_edge_types = self.list_edge_types(space_name)?;
        if existing_edge_types
            .iter()
            .any(|e| e.edge_type_name == edge_type.edge_type_name)
        {
            return Ok(false);
        }

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        let new_edge_type_id = {
            let mut id_counter_table =
                write_txn
                    .open_table(EDGE_TYPE_ID_COUNTER_TABLE)
                    .map_err(|e| {
                        StorageError::DbError(format!(
                            "Failed to open EDGE_TYPE_ID_COUNTER_TABLE: {}",
                            e
                        ))
                    })?;

            let key = ByteKey(space_info.space_id.to_be_bytes().to_vec());
            let current_id = id_counter_table
                .get(&key)
                .map_err(|e| StorageError::DbError(format!("Failed to query ID counter: {}", e)))?
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

            id_counter_table.insert(key, value).map_err(|e| {
                StorageError::DbError(format!("Failed to update ID counter: {}", e))
            })?;

            new_id as i32
        };

        {
            let mut edge_types_table = write_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
            })?;

            let mut edge_type_with_id = edge_type.clone();
            edge_type_with_id.edge_type_id = new_edge_type_id;

            let key = ByteKey(
                [
                    space_info.space_id.to_be_bytes().to_vec(),
                    new_edge_type_id.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            let value = ByteKey(encode_to_vec(&edge_type_with_id)?);

            edge_types_table
                .insert(key, value)
                .map_err(|e| StorageError::DbError(format!("Failed to insert edge type: {}", e)))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(true)
    }

    fn drop_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        let mut edge_type_id: Option<i32> = None;

        {
            let edge_types_table = write_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
            })?;

            let iter = edge_types_table.iter().map_err(|e| {
                StorageError::DbError(format!("Failed to iterate edge type: {}", e))
            })?;

            for result in iter {
                let (key, value) = result.map_err(|e| {
                    StorageError::DbError(format!("Failed to iterate edge type: {}", e))
                })?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let edge_type: EdgeTypeInfo = decode_from_slice(&value.value().0)?.0;
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
                    StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
                })?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                edge_types_table.remove(&key).map_err(|e| {
                    StorageError::DbError(format!("Failed to delete edge type: {}", e))
                })?;
            }

            write_txn.commit().map_err(|e| {
                StorageError::DbError(format!("Failed to commit transaction: {}", e))
            })?;

            return Ok(true);
        }

        Ok(false)
    }

    fn get_edge_type(
        &self,
        space_name: &str,
        edge_type_name: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let edge_types_table = read_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
            StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
        })?;

        let iter = edge_types_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("Failed to iterate edge type: {}", e)))?;

        for result in iter {
            let (key, value) = result.map_err(|e| {
                StorageError::DbError(format!("Failed to iterate edge type: {}", e))
            })?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let edge_type: EdgeTypeInfo = decode_from_slice(&value.value().0)?.0;
                if edge_type.edge_type_name == edge_type_name {
                    return Ok(Some(edge_type));
                }
            }
        }

        Ok(None)
    }

    fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;

        let edge_types_table = read_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
            StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
        })?;

        let mut edge_types = Vec::new();

        let iter = edge_types_table
            .iter()
            .map_err(|e| StorageError::DbError(format!("Failed to iterate edge type: {}", e)))?;

        for result in iter {
            let (key, value) = result.map_err(|e| {
                StorageError::DbError(format!("Failed to iterate edge type: {}", e))
            })?;
            let key_bytes = &key.value().0;
            if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                let edge_type: EdgeTypeInfo = decode_from_slice(&value.value().0)?.0;
                edge_types.push(edge_type);
            }
        }

        Ok(edge_types)
    }

    fn update_edge_type(
        &self,
        space_name: &str,
        edge: &EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let write_txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        let mut edge_type_id: Option<i32> = None;

        // Find the edge type ID by name
        {
            let edge_types_table = write_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
            })?;

            let iter = edge_types_table.iter().map_err(|e| {
                StorageError::DbError(format!("Failed to iterate edge type: {}", e))
            })?;

            for result in iter {
                let (key, value) = result.map_err(|e| {
                    StorageError::DbError(format!("Failed to iterate edge type: {}", e))
                })?;
                let key_bytes = &key.value().0;
                if key_bytes.starts_with(space_info.space_id.to_be_bytes().as_ref()) {
                    let existing_edge: EdgeTypeInfo = decode_from_slice(&value.value().0)?.0;
                    if existing_edge.edge_type_name == edge.edge_type_name {
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
                    StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
                })?;

                let key = ByteKey(
                    [
                        space_info.space_id.to_be_bytes().to_vec(),
                        id.to_be_bytes().to_vec(),
                    ]
                    .concat(),
                );
                let value = ByteKey(encode_to_vec(edge)?);

                edge_types_table.insert(key, value).map_err(|e| {
                    StorageError::DbError(format!("Failed to update edge type: {}", e))
                })?;
            }

            write_txn.commit().map_err(|e| {
                StorageError::DbError(format!("Failed to commit transaction: {}", e))
            })?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_tag_schema(&self, space_name: &str, tag: &str) -> Result<Schema, StorageError> {
        let tag_info = self
            .get_tag(space_name, tag)?
            .ok_or_else(|| StorageError::DbError(format!("Tag \"{}\" does not exist", tag)))?;

        Ok(tag_info_to_schema(tag, &tag_info))
    }

    fn get_edge_type_schema(&self, space_name: &str, edge: &str) -> Result<Schema, StorageError> {
        let edge_type_info = self.get_edge_type(space_name, edge)?.ok_or_else(|| {
            StorageError::DbError(format!("Edge type \"{}\" does not exist", edge))
        })?;

        Ok(edge_type_info_to_schema(edge, &edge_type_info))
    }

    fn list_tag_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let space = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;
        let space_id = space.space_id;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE).map_err(|e| {
            StorageError::DbError(format!("Failed to open TAG_INDEXES_TABLE: {}", e))
        })?;

        let mut indexes = Vec::new();
        let space_prefix = format!("{}:", space_id);
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (key, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&space_prefix) {
                let index_bytes = value.value().0;
                let index: Index = decode_from_slice(&index_bytes)?.0;
                indexes.push(index);
            }
        }
        Ok(indexes)
    }

    fn list_edge_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let space = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;
        let space_id = space.space_id;

        let read_txn = self.db.begin_read().map_err(|e| {
            StorageError::DbError(format!("Failed to start read transaction: {}", e))
        })?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE).map_err(|e| {
            StorageError::DbError(format!("Failed to open EDGE_INDEXES_TABLE: {}", e))
        })?;

        let mut indexes = Vec::new();
        let space_prefix = format!("{}:", space_id);
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (key, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&space_prefix) {
                let index_bytes = value.value().0;
                let index: Index = decode_from_slice(&index_bytes)?.0;
                indexes.push(index);
            }
        }
        Ok(indexes)
    }
}
