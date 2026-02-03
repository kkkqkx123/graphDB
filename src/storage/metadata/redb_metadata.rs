use crate::core::StorageError;
use crate::core::types::{EdgeTypeInfo, SpaceInfo, TagInfo};
use crate::storage::Schema;
use crate::storage::redb_types::{ByteKey, SPACES_TABLE, TAGS_TABLE, EDGE_TYPES_TABLE};
use crate::storage::serializer::{space_to_bytes, space_from_bytes, tag_to_bytes, tag_from_bytes, edge_type_to_bytes, edge_type_from_bytes};
use crate::storage::utils::{tag_info_to_schema, edge_type_info_to_schema};
use redb::{Database, ReadableTable};
use std::sync::Arc;

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
        let key = format!("{}:{}", space, tag.tag_name);
        let tag_bytes = tag_to_bytes(tag)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key.as_bytes().to_vec()), ByteKey(tag_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let key = format!("{}:{}", space, tag_name);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key.as_bytes().to_vec()))
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
        let table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut tags = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key_bytes, tag_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&format!("{}:", space)) {
                let tag: TagInfo = tag_from_bytes(&tag_bytes.value().0)?;
                tags.push(tag);
            }
        }

        Ok(tags)
    }

    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        let key = format!("{}:{}", space, tag_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let key = format!("{}:{}", space, edge.edge_type_name);
        let edge_bytes = edge_type_to_bytes(edge)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key.as_bytes().to_vec()), ByteKey(edge_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let key = format!("{}:{}", space, edge_type_name);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key.as_bytes().to_vec()))
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
        let table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (key_bytes, edge_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&format!("{}:", space)) {
                let edge: EdgeTypeInfo = edge_type_from_bytes(&edge_bytes.value().0)?;
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let key = format!("{}:{}", space, edge_type_name);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_TYPES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError> {
        let key = format!("{}:{}", space, tag);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAGS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let tag_bytes = value.value().0;
                let tag_info: TagInfo = tag_from_bytes(&tag_bytes)?;
                Ok(tag_info_to_schema(tag, &tag_info))
            }
            None => Err(StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))),
        }
    }

    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError> {
        let key = format!("{}:{}", space, edge);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_TYPES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let edge_bytes = value.value().0;
                let edge_info: EdgeTypeInfo = edge_type_from_bytes(&edge_bytes)?;
                Ok(edge_type_info_to_schema(edge, &edge_info))
            }
            None => Err(StorageError::DbError(format!("Edge type '{}' not found in space '{}'", edge, space))),
        }
    }
}
