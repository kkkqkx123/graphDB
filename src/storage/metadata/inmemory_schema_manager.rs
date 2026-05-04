use crate::core::types::{EdgeTypeInfo, Index, PropertyDef, SpaceInfo, TagInfo};
use crate::core::StorageError;
use crate::storage::{FieldDef, Schema};
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use super::SchemaManager;

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

const SCHEMA_FORMAT_VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct SchemaSnapshot {
    version: u32,
    spaces: Vec<SpaceInfo>,
    tags: Vec<(u64, TagInfo)>,
    edge_types: Vec<(u64, EdgeTypeInfo)>,
    tag_indexes: Vec<(u64, Index)>,
    edge_indexes: Vec<(u64, Index)>,
    space_id_counter: u64,
    tag_id_counters: Vec<(u64, u32)>,
    edge_type_id_counters: Vec<(u64, u32)>,
}

#[derive(Debug, Clone)]
struct SpaceData {
    info: SpaceInfo,
}

#[derive(Debug, Clone)]
struct TagData {
    info: TagInfo,
}

#[derive(Debug, Clone)]
struct EdgeTypeData {
    info: EdgeTypeInfo,
}

#[derive(Debug, Clone)]
struct IndexData {
    info: Index,
}

pub struct InMemorySchemaManager {
    spaces: Arc<RwLock<HashMap<u64, SpaceData>>>,
    space_name_index: Arc<RwLock<HashMap<String, u64>>>,
    tags: Arc<RwLock<HashMap<(u64, i32), TagData>>>,
    edge_types: Arc<RwLock<HashMap<(u64, i32), EdgeTypeData>>>,
    tag_indexes: Arc<RwLock<HashMap<(u64, String), IndexData>>>,
    edge_indexes: Arc<RwLock<HashMap<(u64, String), IndexData>>>,
    space_id_counter: Arc<AtomicU64>,
    tag_id_counter: Arc<RwLock<HashMap<u64, AtomicU32>>>,
    edge_type_id_counter: Arc<RwLock<HashMap<u64, AtomicU32>>>,
}

impl std::fmt::Debug for InMemorySchemaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySchemaManager")
            .field("spaces_count", &self.spaces.read().len())
            .finish()
    }
}

impl InMemorySchemaManager {
    pub fn new() -> Self {
        Self {
            spaces: Arc::new(RwLock::new(HashMap::new())),
            space_name_index: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            edge_types: Arc::new(RwLock::new(HashMap::new())),
            tag_indexes: Arc::new(RwLock::new(HashMap::new())),
            edge_indexes: Arc::new(RwLock::new(HashMap::new())),
            space_id_counter: Arc::new(AtomicU64::new(0)),
            tag_id_counter: Arc::new(RwLock::new(HashMap::new())),
            edge_type_id_counter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_next_space_id(&self) -> u64 {
        self.space_id_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn get_next_tag_id(&self, space_id: u64) -> i32 {
        let counters = self.tag_id_counter.read();
        if let Some(counter) = counters.get(&space_id) {
            (counter.fetch_add(1, Ordering::SeqCst) + 1) as i32
        } else {
            drop(counters);
            let mut counters = self.tag_id_counter.write();
            counters.insert(space_id, AtomicU32::new(1));
            1
        }
    }

    fn get_next_edge_type_id(&self, space_id: u64) -> i32 {
        let counters = self.edge_type_id_counter.read();
        if let Some(counter) = counters.get(&space_id) {
            (counter.fetch_add(1, Ordering::SeqCst) + 1) as i32
        } else {
            drop(counters);
            let mut counters = self.edge_type_id_counter.write();
            counters.insert(space_id, AtomicU32::new(1));
            1
        }
    }
}

impl Default for InMemorySchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl super::SchemaManager for InMemorySchemaManager {
    fn create_space(&self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        let mut name_index = self.space_name_index.write();
        if name_index.contains_key(&space.space_name) {
            return Ok(false);
        }

        let space_id = self.get_next_space_id();
        space.space_id = space_id;

        name_index.insert(space.space_name.clone(), space_id);
        drop(name_index);

        let mut spaces = self.spaces.write();
        spaces.insert(space_id, SpaceData { info: space.clone() });

        Ok(true)
    }

    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError> {
        let mut name_index = self.space_name_index.write();
        if let Some(space_id) = name_index.remove(space_name) {
            drop(name_index);

            let mut spaces = self.spaces.write();
            spaces.remove(&space_id);

            let mut tags = self.tags.write();
            tags.retain(|(sid, _), _| *sid != space_id);

            let mut edge_types = self.edge_types.write();
            edge_types.retain(|(sid, _), _| *sid != space_id);

            let mut tag_indexes = self.tag_indexes.write();
            tag_indexes.retain(|(sid, _), _| *sid != space_id);

            let mut edge_indexes = self.edge_indexes.write();
            edge_indexes.retain(|(sid, _), _| *sid != space_id);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let name_index = self.space_name_index.read();
        if let Some(space_id) = name_index.get(space_name) {
            let spaces = self.spaces.read();
            if let Some(data) = spaces.get(space_id) {
                return Ok(Some(data.info.clone()));
            }
        }
        Ok(None)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.spaces.read();
        Ok(spaces.get(&space_id).map(|d| d.info.clone()))
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let spaces = self.spaces.read();
        Ok(spaces.values().map(|d| d.info.clone()).collect())
    }

    fn update_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.write();
        if spaces.contains_key(&space.space_id) {
            spaces.insert(space.space_id, SpaceData { info: space.clone() });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_tag(&self, space_name: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let existing_tags = self.list_tags(space_name)?;
        if existing_tags.iter().any(|t| t.tag_name == tag.tag_name) {
            return Ok(false);
        }

        let tag_id = self.get_next_tag_id(space_info.space_id);
        let mut tag_with_id = tag.clone();
        tag_with_id.tag_id = tag_id;

        let mut tags = self.tags.write();
        tags.insert((space_info.space_id, tag_id), TagData { info: tag_with_id });

        Ok(true)
    }

    fn drop_tag(&self, space_name: &str, tag_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut tags = self.tags.write();
        let tag_key = tags.iter().find(|(_, data)| {
            data.info.tag_name == tag_name
        }).map(|(k, _)| *k);

        if let Some(key) = tag_key {
            tags.remove(&key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let tags = self.tags.read();
        Ok(tags.values().find(|data| {
            data.info.tag_name == tag_name
        }).map(|d| d.info.clone()))
    }

    fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let tags = self.tags.read();
        Ok(tags.iter()
            .filter(|((sid, _), _)| *sid == space_info.space_id)
            .map(|(_, data)| data.info.clone())
            .collect())
    }

    fn update_tag(&self, space_name: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut tags = self.tags.write();
        let tag_key = tags.iter().find(|(_, data)| {
            data.info.tag_name == tag.tag_name
        }).map(|(k, _)| *k);

        if let Some(key) = tag_key {
            if let Some(data) = tags.get_mut(&key) {
                data.info = tag.clone();
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn create_edge_type(&self, space_name: &str, edge_type: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let existing = self.list_edge_types(space_name)?;
        if existing.iter().any(|e| e.edge_type_name == edge_type.edge_type_name) {
            return Ok(false);
        }

        let edge_type_id = self.get_next_edge_type_id(space_info.space_id);
        let mut edge_with_id = edge_type.clone();
        edge_with_id.edge_type_id = edge_type_id;

        let mut edge_types = self.edge_types.write();
        edge_types.insert((space_info.space_id, edge_type_id), EdgeTypeData { info: edge_with_id });

        Ok(true)
    }

    fn drop_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut edge_types = self.edge_types.write();
        let key = edge_types.iter().find(|(_, data)| {
            data.info.edge_type_name == edge_type_name
        }).map(|(k, _)| *k);

        if let Some(k) = key {
            edge_types.remove(&k);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let edge_types = self.edge_types.read();
        Ok(edge_types.values().find(|data| {
            data.info.edge_type_name == edge_type_name
        }).map(|d| d.info.clone()))
    }

    fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let edge_types = self.edge_types.read();
        Ok(edge_types.iter()
            .filter(|((sid, _), _)| *sid == space_info.space_id)
            .map(|(_, data)| data.info.clone())
            .collect())
    }

    fn update_edge_type(&self, space_name: &str, edge_type: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut edge_types = self.edge_types.write();
        let key = edge_types.iter().find(|(_, data)| {
            data.info.edge_type_name == edge_type.edge_type_name
        }).map(|(k, _)| *k);

        if let Some(k) = key {
            if let Some(data) = edge_types.get_mut(&k) {
                data.info = edge_type.clone();
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn get_tag_schema(&self, space_name: &str, tag_name: &str) -> Result<Schema, StorageError> {
        let tag = self.get_tag(space_name, tag_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Tag \"{}\" does not exist in space \"{}\"", tag_name, space_name))
        })?;
        Ok(tag_info_to_schema(tag_name, &tag))
    }

    fn get_edge_type_schema(&self, space_name: &str, edge_type_name: &str) -> Result<Schema, StorageError> {
        let edge_type = self.get_edge_type(space_name, edge_type_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Edge type \"{}\" does not exist in space \"{}\"", edge_type_name, space_name))
        })?;
        Ok(edge_type_info_to_schema(edge_type_name, &edge_type))
    }

    fn list_tag_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let indexes = self.tag_indexes.read();
        Ok(indexes.iter()
            .filter(|((sid, _), _)| *sid == space_info.space_id)
            .map(|(_, data)| data.info.clone())
            .collect())
    }

    fn list_edge_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let indexes = self.edge_indexes.read();
        Ok(indexes.iter()
            .filter(|((sid, _), _)| *sid == space_info.space_id)
            .map(|(_, data)| data.info.clone())
            .collect())
    }

    fn save_schema(&self, path: &std::path::Path) -> Result<(), StorageError> {
        use std::fs::{self, File};
        use std::io::Write;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::IOError(e.to_string()))?;
        }

        let spaces: Vec<SpaceInfo> = self.spaces.read().values().map(|d| d.info.clone()).collect();
        
        let tags: Vec<(u64, TagInfo)> = self.tags.read()
            .iter()
            .map(|((space_id, _), data)| (*space_id, data.info.clone()))
            .collect();

        let edge_types: Vec<(u64, EdgeTypeInfo)> = self.edge_types.read()
            .iter()
            .map(|((space_id, _), data)| (*space_id, data.info.clone()))
            .collect();

        let tag_indexes: Vec<(u64, Index)> = self.tag_indexes.read()
            .iter()
            .map(|((space_id, _), data)| (*space_id, data.info.clone()))
            .collect();

        let edge_indexes: Vec<(u64, Index)> = self.edge_indexes.read()
            .iter()
            .map(|((space_id, _), data)| (*space_id, data.info.clone()))
            .collect();

        let space_id_counter = self.space_id_counter.load(std::sync::atomic::Ordering::SeqCst);

        let tag_id_counters: Vec<(u64, u32)> = self.tag_id_counter.read()
            .iter()
            .map(|(k, v)| (*k, v.load(std::sync::atomic::Ordering::SeqCst)))
            .collect();

        let edge_type_id_counters: Vec<(u64, u32)> = self.edge_type_id_counter.read()
            .iter()
            .map(|(k, v)| (*k, v.load(std::sync::atomic::Ordering::SeqCst)))
            .collect();

        let snapshot = SchemaSnapshot {
            version: SCHEMA_FORMAT_VERSION,
            spaces,
            tags,
            edge_types,
            tag_indexes,
            edge_indexes,
            space_id_counter,
            tag_id_counters,
            edge_type_id_counters,
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;

        let mut file = File::create(path)
            .map_err(|e| StorageError::IOError(e.to_string()))?;
        
        file.write_all(json.as_bytes())
            .map_err(|e| StorageError::IOError(e.to_string()))?;

        Ok(())
    }

    fn load_schema(&mut self, path: &std::path::Path) -> Result<(), StorageError> {
        use std::fs::File;
        use std::io::Read;

        if !path.exists() {
            return Ok(());
        }

        let mut file = File::open(path)
            .map_err(|e| StorageError::IOError(e.to_string()))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|e| StorageError::IOError(e.to_string()))?;

        let snapshot: SchemaSnapshot = serde_json::from_str(&json)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))?;

        if snapshot.version > SCHEMA_FORMAT_VERSION {
            return Err(StorageError::DeserializeError(format!(
                "Schema version {} is newer than supported version {}",
                snapshot.version, SCHEMA_FORMAT_VERSION
            )));
        }

        self.spaces.write().clear();
        self.space_name_index.write().clear();
        self.tags.write().clear();
        self.edge_types.write().clear();
        self.tag_indexes.write().clear();
        self.edge_indexes.write().clear();

        for space in snapshot.spaces {
            let space_id = space.space_id;
            self.space_name_index.write().insert(space.space_name.clone(), space_id);
            self.spaces.write().insert(space_id, SpaceData { info: space });
        }

        for (space_id, tag) in snapshot.tags {
            let tag_id = tag.tag_id;
            self.tags.write().insert((space_id, tag_id), TagData { info: tag });
        }

        for (space_id, edge_type) in snapshot.edge_types {
            let edge_type_id = edge_type.edge_type_id;
            self.edge_types.write().insert((space_id, edge_type_id), EdgeTypeData { info: edge_type });
        }

        for (space_id, index) in snapshot.tag_indexes {
            let index_name = index.name.clone();
            self.tag_indexes.write().insert((space_id, index_name), IndexData { info: index });
        }

        for (space_id, index) in snapshot.edge_indexes {
            let index_name = index.name.clone();
            self.edge_indexes.write().insert((space_id, index_name), IndexData { info: index });
        }

        self.space_id_counter.store(snapshot.space_id_counter, std::sync::atomic::Ordering::SeqCst);

        {
            let mut counters = self.tag_id_counter.write();
            counters.clear();
            for (space_id, counter) in snapshot.tag_id_counters {
                counters.insert(space_id, std::sync::atomic::AtomicU32::new(counter));
            }
        }

        {
            let mut counters = self.edge_type_id_counter.write();
            counters.clear();
            for (space_id, counter) in snapshot.edge_type_id_counters {
                counters.insert(space_id, std::sync::atomic::AtomicU32::new(counter));
            }
        }

        Ok(())
    }
}

impl InMemorySchemaManager {
    pub fn alter_tag(
        &self,
        space_name: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut tags = self.tags.write();
        let tag_key = tags.iter().find(|(_, data)| {
            data.info.tag_name == tag_name
        }).map(|(k, _)| *k);

        if let Some(key) = tag_key {
            if let Some(data) = tags.get_mut(&key) {
                for prop in additions {
                    if !data.info.properties.iter().any(|p| p.name == prop.name) {
                        data.info.properties.push(prop);
                    }
                }
                data.info.properties.retain(|p| !deletions.contains(&p.name));
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn alter_edge_type(
        &self,
        space_name: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut edge_types = self.edge_types.write();
        let key = edge_types.iter().find(|(_, data)| {
            data.info.edge_type_name == edge_type_name
        }).map(|(k, _)| *k);

        if let Some(k) = key {
            if let Some(data) = edge_types.get_mut(&k) {
                for prop in additions {
                    if !data.info.properties.iter().any(|p| p.name == prop.name) {
                        data.info.properties.push(prop);
                    }
                }
                data.info.properties.retain(|p| !deletions.contains(&p.name));
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn create_tag_index(&self, space_name: &str, index: &Index) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut indexes = self.tag_indexes.write();
        let key = (space_info.space_id, index.name.clone());
        if indexes.contains_key(&key) {
            return Ok(false);
        }
        indexes.insert(key, IndexData { info: index.clone() });
        Ok(true)
    }

    pub fn drop_tag_index(&self, space_name: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut indexes = self.tag_indexes.write();
        let key = (space_info.space_id, index_name.to_string());
        Ok(indexes.remove(&key).is_some())
    }

    pub fn get_tag_index(&self, space_name: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let indexes = self.tag_indexes.read();
        Ok(indexes.get(&(space_info.space_id, index_name.to_string())).map(|d| d.info.clone()))
    }

    pub fn create_edge_index(&self, space_name: &str, index: &Index) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut indexes = self.edge_indexes.write();
        let key = (space_info.space_id, index.name.clone());
        if indexes.contains_key(&key) {
            return Ok(false);
        }
        indexes.insert(key, IndexData { info: index.clone() });
        Ok(true)
    }

    pub fn drop_edge_index(&self, space_name: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let mut indexes = self.edge_indexes.write();
        let key = (space_info.space_id, index_name.to_string());
        Ok(indexes.remove(&key).is_some())
    }

    pub fn get_edge_index(&self, space_name: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let space_info = self.get_space(space_name)?.ok_or_else(|| {
            StorageError::DbError(format!("Space \"{}\" does not exist", space_name))
        })?;

        let indexes = self.edge_indexes.read();
        Ok(indexes.get(&(space_info.space_id, index_name.to_string())).map(|d| d.info.clone()))
    }

    pub fn get_schema(&self, space_name: &str, schema_name: &str) -> Result<Option<Schema>, StorageError> {
        if let Some(tag) = self.get_tag(space_name, schema_name)? {
            return Ok(Some(tag_info_to_schema(schema_name, &tag)));
        }
        if let Some(edge_type) = self.get_edge_type(space_name, schema_name)? {
            return Ok(Some(edge_type_info_to_schema(schema_name, &edge_type)));
        }
        Ok(None)
    }

    pub fn get_space_id(&self, space_name: &str) -> Result<u64, StorageError> {
        self.get_space(space_name)?
            .map(|s| s.space_id)
            .ok_or_else(|| StorageError::DbError(format!("Space \"{}\" does not exist", space_name)))
    }
}
