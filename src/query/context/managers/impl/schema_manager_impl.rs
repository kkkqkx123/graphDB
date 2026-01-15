//! Schema管理器实现 - 内存中的Schema管理
//!
use super::super::{
    EdgeTypeDefWithId, FieldDef, Schema, SchemaChange, SchemaChangeType, SchemaExportConfig, SchemaHistory,
    SchemaImportResult, SchemaManager, SchemaVersion, TagDefWithId,
};
use crate::core::error::{ManagerError, ManagerResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 内存中的Schema管理器实现
#[derive(Debug, Clone)]
pub struct MemorySchemaManager {
    schemas: Arc<RwLock<HashMap<String, Schema>>>,
    tags: Arc<RwLock<HashMap<i32, TagDefWithId>>>,
    edge_types: Arc<RwLock<HashMap<i32, EdgeTypeDefWithId>>>,
    space_tags: Arc<RwLock<HashMap<i32, Vec<i32>>>>,
    space_edge_types: Arc<RwLock<HashMap<i32, Vec<i32>>>>,
    next_tag_id: Arc<RwLock<i32>>,
    next_edge_type_id: Arc<RwLock<i32>>,
    storage_path: PathBuf,
    schema_versions: Arc<RwLock<HashMap<i32, SchemaHistory>>>,
    next_version: Arc<RwLock<i32>>,
    schema_changes: Arc<RwLock<HashMap<i32, Vec<SchemaChange>>>>,
    tag_name_to_id: Arc<RwLock<HashMap<i32, HashMap<String, i32>>>>,
    edge_type_name_to_id: Arc<RwLock<HashMap<i32, HashMap<String, i32>>>>,
}

impl MemorySchemaManager {
    /// 创建新的内存Schema管理器
    pub fn new() -> Self {
        Self::with_storage_path("./data/schema")
    }

    /// 使用指定存储路径创建内存Schema管理器
    pub fn with_storage_path<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            edge_types: Arc::new(RwLock::new(HashMap::new())),
            space_tags: Arc::new(RwLock::new(HashMap::new())),
            space_edge_types: Arc::new(RwLock::new(HashMap::new())),
            next_tag_id: Arc::new(RwLock::new(1)),
            next_edge_type_id: Arc::new(RwLock::new(1)),
            storage_path: storage_path.as_ref().to_path_buf(),
            schema_versions: Arc::new(RwLock::new(HashMap::new())),
            next_version: Arc::new(RwLock::new(1)),
            schema_changes: Arc::new(RwLock::new(HashMap::new())),
            tag_name_to_id: Arc::new(RwLock::new(HashMap::new())),
            edge_type_name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加Schema
    pub fn add_schema(&self, schema: Schema) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.insert(schema.name.clone(), schema);
        Ok(())
    }

    /// 删除Schema
    pub fn remove_schema(&self, name: &str) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.remove(name);
        Ok(())
    }

    /// 更新Schema
    pub fn update_schema(&self, name: &str, schema: Schema) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.insert(name.to_string(), schema);
        Ok(())
    }
}

impl Default for MemorySchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaManager for MemorySchemaManager {
    fn get_schema(&self, name: &str) -> Option<Schema> {
        let schemas = self.schemas.read().ok()?;
        schemas.get(name).cloned()
    }

    fn list_schemas(&self) -> Vec<String> {
        match self.schemas.read() {
            Ok(schemas) => schemas.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn has_schema(&self, name: &str) -> bool {
        match self.schemas.read() {
            Ok(schemas) => schemas.contains_key(name),
            Err(_) => false,
        }
    }

    fn create_tag(
        &self,
        space_id: i32,
        tag_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32> {
        let mut next_id = self
            .next_tag_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let tag_def = TagDefWithId {
            tag_id,
            tag_name: tag_name.to_string(),
            fields,
            comment: None,
        };

        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        tags.insert(tag_id, tag_def.clone());
        drop(tags);

        let mut space_tags = self
            .space_tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        space_tags
            .entry(space_id)
            .or_insert_with(Vec::new)
            .push(tag_id);
        drop(space_tags);

        let mut tag_name_to_id = self
            .tag_name_to_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        tag_name_to_id
            .entry(space_id)
            .or_insert_with(HashMap::new)
            .insert(tag_name.to_string(), tag_id);
        drop(tag_name_to_id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::CreateTag,
            target_name: tag_name.to_string(),
            description: format!("创建标签 {}", tag_name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(tag_id)
    }

    fn drop_tag(&self, space_id: i32, tag_id: i32) -> ManagerResult<()> {
        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_def = tags
            .get(&tag_id)
            .ok_or_else(|| ManagerError::NotFound(format!("Tag {} 不存在", tag_id)))?
            .clone();
        tags.remove(&tag_id);
        drop(tags);

        let mut space_tags = self
            .space_tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(tag_list) = space_tags.get_mut(&space_id) {
            tag_list.retain(|&id| id != tag_id);
        }
        drop(space_tags);

        let mut tag_name_to_id = self
            .tag_name_to_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(name_map) = tag_name_to_id.get_mut(&space_id) {
            name_map.remove(&tag_def.tag_name);
        }
        drop(tag_name_to_id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::DropTag,
            target_name: tag_def.tag_name.clone(),
            description: format!("删除标签 {}", tag_def.tag_name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn get_tag(&self, _space_id: i32, tag_id: i32) -> Option<TagDefWithId> {
        let tags = self.tags.read().ok()?;
        tags.get(&tag_id).cloned()
    }

    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDefWithId>> {
        let space_tags = self
            .space_tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_ids = space_tags.get(&space_id).cloned().unwrap_or_default();
        drop(space_tags);

        let tags = self
            .tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_list: Vec<TagDefWithId> = tag_ids
            .iter()
            .filter_map(|id| tags.get(id).cloned())
            .collect();
        Ok(tag_list)
    }

    fn has_tag(&self, _space_id: i32, tag_id: i32) -> bool {
        match self.tags.read() {
            Ok(tags) => tags.contains_key(&tag_id),
            Err(_) => false,
        }
    }

    fn create_edge_type(
        &self,
        space_id: i32,
        edge_type_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32> {
        let mut next_id = self
            .next_edge_type_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let edge_type_def = EdgeTypeDefWithId {
            edge_type_id,
            edge_type_name: edge_type_name.to_string(),
            fields,
            comment: None,
        };

        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        edge_types.insert(edge_type_id, edge_type_def.clone());
        drop(edge_types);

        let mut space_edge_types = self
            .space_edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        space_edge_types
            .entry(space_id)
            .or_insert_with(Vec::new)
            .push(edge_type_id);
        drop(space_edge_types);

        let mut edge_type_name_to_id = self
            .edge_type_name_to_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        edge_type_name_to_id
            .entry(space_id)
            .or_insert_with(HashMap::new)
            .insert(edge_type_name.to_string(), edge_type_id);
        drop(edge_type_name_to_id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::CreateEdgeType,
            target_name: edge_type_name.to_string(),
            description: format!("创建边类型 {}", edge_type_name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(edge_type_id)
    }

    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> ManagerResult<()> {
        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_def = edge_types
            .get(&edge_type_id)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "EdgeType {} 不存在",
                edge_type_id
            )))?
            .clone();
        edge_types.remove(&edge_type_id);
        drop(edge_types);

        let mut space_edge_types = self
            .space_edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(edge_type_list) = space_edge_types.get_mut(&space_id) {
            edge_type_list.retain(|&id| id != edge_type_id);
        }
        drop(space_edge_types);

        let mut edge_type_name_to_id = self
            .edge_type_name_to_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(name_map) = edge_type_name_to_id.get_mut(&space_id) {
            name_map.remove(&edge_type_def.edge_type_name);
        }
        drop(edge_type_name_to_id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::DropEdgeType,
            target_name: edge_type_def.edge_type_name.clone(),
            description: format!("删除边类型 {}", edge_type_def.edge_type_name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn get_edge_type(&self, _space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDefWithId> {
        let edge_types = self.edge_types.read().ok()?;
        edge_types.get(&edge_type_id).cloned()
    }

    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDefWithId>> {
        let space_edge_types = self
            .space_edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_ids = space_edge_types.get(&space_id).cloned().unwrap_or_default();
        drop(space_edge_types);

        let edge_types = self
            .edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_list: Vec<EdgeTypeDefWithId> = edge_type_ids
            .iter()
            .filter_map(|id| edge_types.get(id).cloned())
            .collect();
        Ok(edge_type_list)
    }

    fn has_edge_type(&self, _space_id: i32, edge_type_id: i32) -> bool {
        match self.edge_types.read() {
            Ok(edge_types) => edge_types.contains_key(&edge_type_id),
            Err(_) => false,
        }
    }

    fn load_from_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            return Ok(());
        }

        let schemas_file = self.storage_path.join("schemas.json");
        let tags_file = self.storage_path.join("tags.json");
        let edge_types_file = self.storage_path.join("edge_types.json");
        let space_tags_file = self.storage_path.join("space_tags.json");
        let space_edge_types_file = self.storage_path.join("space_edge_types.json");

        if schemas_file.exists() {
            let content = fs::read_to_string(&schemas_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let schema_list: Vec<Schema> = serde_json::from_str(&content)
                .map_err(|e| ManagerError::SchemaError(format!("反序列化Schema失败: {}", e)))?;
            let mut schemas = self
                .schemas
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            schemas.clear();
            for schema in schema_list {
                schemas.insert(schema.name.clone(), schema);
            }
        }

        if tags_file.exists() {
            let content = fs::read_to_string(&tags_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let tag_list: Vec<TagDefWithId> = serde_json::from_str(&content)
                .map_err(|e| ManagerError::SchemaError(format!("反序列化Tag失败: {}", e)))?;
            let mut tags = self
                .tags
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            tags.clear();
            for tag_def in tag_list {
                let tag_id = tag_def.tag_id;
                tags.insert(tag_id, tag_def);
                let mut next_id = self
                    .next_tag_id
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if tag_id >= *next_id {
                    *next_id = tag_id + 1;
                }
            }
        }

        if edge_types_file.exists() {
            let content = fs::read_to_string(&edge_types_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let edge_type_list: Vec<EdgeTypeDefWithId> = serde_json::from_str(&content)
                .map_err(|e| ManagerError::SchemaError(format!("反序列化EdgeType失败: {}", e)))?;
            let mut edge_types = self
                .edge_types
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            edge_types.clear();
            for edge_type_def in edge_type_list {
                let edge_type_id = edge_type_def.edge_type_id;
                edge_types.insert(edge_type_id, edge_type_def);
                let mut next_id = self
                    .next_edge_type_id
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if edge_type_id >= *next_id {
                    *next_id = edge_type_id + 1;
                }
            }
        }

        if space_tags_file.exists() {
            let content = fs::read_to_string(&space_tags_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let space_tags_map: HashMap<i32, Vec<i32>> =
                serde_json::from_str(&content).map_err(|e| {
                    ManagerError::SchemaError(format!("反序列化Space Tag映射失败: {}", e))
                })?;
            let mut space_tags = self
                .space_tags
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            *space_tags = space_tags_map;
        }

        if space_edge_types_file.exists() {
            let content = fs::read_to_string(&space_edge_types_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let space_edge_types_map: HashMap<i32, Vec<i32>> = serde_json::from_str(&content)
                .map_err(|e| {
                    ManagerError::SchemaError(format!("反序列化Space EdgeType映射失败: {}", e))
                })?;
            let mut space_edge_types = self
                .space_edge_types
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            *space_edge_types = space_edge_types_map;
        }

        self.load_schema_versions_from_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn save_to_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        let schemas = self
            .schemas
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let schema_list: Vec<Schema> = schemas.values().cloned().collect();
        let schemas_content = serde_json::to_string_pretty(&schema_list)
            .map_err(|e| ManagerError::SchemaError(format!("序列化Schema失败: {}", e)))?;
        let schemas_file = self.storage_path.join("schemas.json");
        fs::write(&schemas_file, schemas_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let tags = self
            .tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_list: Vec<TagDefWithId> = tags.values().cloned().collect();
        let tags_content = serde_json::to_string_pretty(&tag_list)
            .map_err(|e| ManagerError::SchemaError(format!("序列化Tag失败: {}", e)))?;
        let tags_file = self.storage_path.join("tags.json");
        fs::write(&tags_file, tags_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let edge_types = self
            .edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_list: Vec<EdgeTypeDefWithId> = edge_types.values().cloned().collect();
        let edge_types_content = serde_json::to_string_pretty(&edge_type_list)
            .map_err(|e| ManagerError::SchemaError(format!("序列化EdgeType失败: {}", e)))?;
        let edge_types_file = self.storage_path.join("edge_types.json");
        fs::write(&edge_types_file, edge_types_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let space_tags = self
            .space_tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_tags_content = serde_json::to_string_pretty(&*space_tags)
            .map_err(|e| ManagerError::SchemaError(format!("序列化Space Tag映射失败: {}", e)))?;
        let space_tags_file = self.storage_path.join("space_tags.json");
        fs::write(&space_tags_file, space_tags_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let space_edge_types = self
            .space_edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_edge_types_content =
            serde_json::to_string_pretty(&*space_edge_types).map_err(|e| {
                ManagerError::SchemaError(format!("序列化Space EdgeType映射失败: {}", e))
            })?;
        let space_edge_types_file = self.storage_path.join("space_edge_types.json");
        fs::write(&space_edge_types_file, space_edge_types_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn create_schema_version(&self, space_id: i32, comment: Option<String>) -> ManagerResult<i32> {
        let tags = self
            .tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_types = self
            .edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_tags = self
            .space_tags
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_edge_types = self
            .space_edge_types
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let tag_ids = space_tags.get(&space_id).cloned().unwrap_or_default();
        let edge_type_ids = space_edge_types.get(&space_id).cloned().unwrap_or_default();

        let version_tags: Vec<TagDefWithId> = tag_ids
            .iter()
            .filter_map(|id| tags.get(id).cloned())
            .collect();

        let version_edge_types: Vec<EdgeTypeDefWithId> = edge_type_ids
            .iter()
            .filter_map(|id| edge_types.get(id).cloned())
            .collect();

        drop(tags);
        drop(edge_types);
        drop(space_tags);
        drop(space_edge_types);

        let mut next_version = self
            .next_version
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let version = *next_version;
        *next_version += 1;
        drop(next_version);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let schema_version = SchemaVersion {
            version,
            space_id,
            tags: version_tags,
            edge_types: version_edge_types,
            created_at: now,
            comment,
        };

        let mut schema_versions = self
            .schema_versions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let history = schema_versions
            .entry(space_id)
            .or_insert_with(|| SchemaHistory {
                space_id,
                versions: Vec::new(),
                current_version: 0,
            });
        history.versions.push(schema_version.clone());
        history.current_version = version;
        drop(schema_versions);

        self.save_schema_versions_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(version)
    }

    fn get_schema_version(&self, space_id: i32, version: i32) -> Option<SchemaVersion> {
        let schema_versions = self.schema_versions.read().ok()?;
        let history = schema_versions.get(&space_id)?;
        history
            .versions
            .iter()
            .find(|v| v.version == version)
            .cloned()
    }

    fn get_latest_schema_version(&self, space_id: i32) -> Option<i32> {
        let schema_versions = self.schema_versions.read().ok()?;
        let history = schema_versions.get(&space_id)?;
        Some(history.current_version)
    }

    fn get_schema_history(&self, space_id: i32) -> ManagerResult<SchemaHistory> {
        let schema_versions = self
            .schema_versions
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        match schema_versions.get(&space_id) {
            Some(history) => Ok(history.clone()),
            None => Err(ManagerError::NotFound(format!(
                "Space {} 没有Schema历史记录",
                space_id
            ))),
        }
    }

    fn rollback_schema(&self, space_id: i32, version: i32) -> ManagerResult<()> {
        let schema_version = self
            .get_schema_version(space_id, version)
            .ok_or_else(|| ManagerError::NotFound(format!("版本 {} 不存在", version)))?;

        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut space_tags = self
            .space_tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut space_edge_types = self
            .space_edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for tag_def in &schema_version.tags {
            tags.insert(tag_def.tag_id, tag_def.clone());
        }

        for edge_type_def in &schema_version.edge_types {
            edge_types.insert(edge_type_def.edge_type_id, edge_type_def.clone());
        }

        let tag_ids: Vec<i32> = schema_version.tags.iter().map(|t| t.tag_id).collect();
        let edge_type_ids: Vec<i32> = schema_version
            .edge_types
            .iter()
            .map(|e| e.edge_type_id)
            .collect();

        space_tags.insert(space_id, tag_ids);
        space_edge_types.insert(space_id, edge_type_ids);

        drop(tags);
        drop(edge_types);
        drop(space_tags);
        drop(space_edge_types);

        let mut schema_versions = self
            .schema_versions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(history) = schema_versions.get_mut(&space_id) {
            history.current_version = version;
        }
        drop(schema_versions);

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        self.save_schema_versions_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn get_current_version(&self, space_id: i32) -> Option<i32> {
        let schema_versions = self.schema_versions.read().ok()?;
        let history = schema_versions.get(&space_id)?;
        if history.versions.is_empty() {
            None
        } else {
            Some(history.current_version)
        }
    }

    fn add_tag_field(&self, space_id: i32, tag_name: &str, field: FieldDef) -> ManagerResult<()> {
        let tag_name_to_id = self
            .tag_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = tag_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let tag_id = name_map
            .get(tag_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;
        drop(tag_name_to_id);

        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_def = tags
            .get_mut(&tag_id)
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;

        if tag_def.fields.iter().any(|f| f.name == field.name) {
            return Err(ManagerError::InvalidInput(format!(
                "字段 {} 已存在",
                field.name
            )));
        }

        tag_def.fields.push(field.clone());
        drop(tags);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterTag,
            target_name: tag_name.to_string(),
            description: format!("为标签 {} 添加字段 {}", tag_name, field.name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn drop_tag_field(
        &self,
        space_id: i32,
        tag_name: &str,
        field_name: &str,
    ) -> ManagerResult<()> {
        let tag_name_to_id = self
            .tag_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = tag_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let tag_id = name_map
            .get(tag_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;
        drop(tag_name_to_id);

        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_def = tags
            .get_mut(&tag_id)
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;

        let original_len = tag_def.fields.len();
        tag_def.fields.retain(|f| f.name != field_name);

        if tag_def.fields.len() == original_len {
            return Err(ManagerError::NotFound(format!(
                "字段 {} 不存在",
                field_name
            )));
        }
        drop(tags);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterTag,
            target_name: tag_name.to_string(),
            description: format!("从标签 {} 删除字段 {}", tag_name, field_name),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn alter_tag_field(
        &self,
        space_id: i32,
        tag_name: &str,
        field_name: &str,
        new_field: FieldDef,
    ) -> ManagerResult<()> {
        let tag_name_to_id = self
            .tag_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = tag_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let tag_id = name_map
            .get(tag_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;
        drop(tag_name_to_id);

        let mut tags = self
            .tags
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let tag_def = tags
            .get_mut(&tag_id)
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))?;

        let field = tag_def
            .fields
            .iter_mut()
            .find(|f| f.name == field_name)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "字段 {} 不存在",
                field_name
            )))?;

        *field = new_field.clone();
        drop(tags);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterTag,
            target_name: tag_name.to_string(),
            description: format!("修改标签 {} 的字段 {} 为 {:?}", tag_name, field_name, new_field),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn add_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field: FieldDef,
    ) -> ManagerResult<()> {
        let edge_type_name_to_id = self
            .edge_type_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = edge_type_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let edge_type_id = name_map
            .get(edge_type_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;
        drop(edge_type_name_to_id);

        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_def = edge_types
            .get_mut(&edge_type_id)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;

        if edge_type_def.fields.iter().any(|f| f.name == field.name) {
            return Err(ManagerError::InvalidInput(format!(
                "字段 {} 已存在",
                field.name
            )));
        }

        edge_type_def.fields.push(field.clone());
        drop(edge_types);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterEdgeType,
            target_name: edge_type_name.to_string(),
            description: format!(
                "为边类型 {} 添加字段 {}",
                edge_type_name, field.name
            ),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn drop_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field_name: &str,
    ) -> ManagerResult<()> {
        let edge_type_name_to_id = self
            .edge_type_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = edge_type_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let edge_type_id = name_map
            .get(edge_type_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;
        drop(edge_type_name_to_id);

        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_def = edge_types
            .get_mut(&edge_type_id)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;

        let original_len = edge_type_def.fields.len();
        edge_type_def.fields.retain(|f| f.name != field_name);

        if edge_type_def.fields.len() == original_len {
            return Err(ManagerError::NotFound(format!(
                "字段 {} 不存在",
                field_name
            )));
        }
        drop(edge_types);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterEdgeType,
            target_name: edge_type_name.to_string(),
            description: format!(
                "从边类型 {} 删除字段 {}",
                edge_type_name, field_name
            ),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn alter_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field_name: &str,
        new_field: FieldDef,
    ) -> ManagerResult<()> {
        let edge_type_name_to_id = self
            .edge_type_name_to_id
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let name_map = edge_type_name_to_id
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;
        let edge_type_id = name_map
            .get(edge_type_name)
            .copied()
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;
        drop(edge_type_name_to_id);

        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edge_type_def = edge_types
            .get_mut(&edge_type_id)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_type_name
            )))?;

        let field = edge_type_def
            .fields
            .iter_mut()
            .find(|f| f.name == field_name)
            .ok_or_else(|| ManagerError::NotFound(format!(
                "字段 {} 不存在",
                field_name
            )))?;

        *field = new_field.clone();
        drop(edge_types);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?
            .as_secs() as i64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AlterEdgeType,
            target_name: edge_type_name.to_string(),
            description: format!(
                "修改边类型 {} 的字段 {} 为 {:?}",
                edge_type_name, field_name, new_field
            ),
            timestamp: now,
        };

        self.record_schema_change(space_id, change)?;

        self.save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn record_schema_change(&self, space_id: i32, change: SchemaChange) -> ManagerResult<()> {
        let mut schema_changes = self
            .schema_changes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        schema_changes
            .entry(space_id)
            .or_insert_with(Vec::new)
            .push(change);
        drop(schema_changes);

        Ok(())
    }

    fn get_schema_changes(&self, space_id: i32) -> ManagerResult<Vec<SchemaChange>> {
        let schema_changes = self
            .schema_changes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(schema_changes.get(&space_id).cloned().unwrap_or_default())
    }

    fn clear_schema_changes(&self, space_id: i32) -> ManagerResult<()> {
        let mut schema_changes = self
            .schema_changes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        schema_changes.remove(&space_id);
        drop(schema_changes);

        Ok(())
    }

    fn export_schema(&self, space_id: i32, config: SchemaExportConfig) -> ManagerResult<String> {
        let tags = self.list_tags(space_id)?;
        let edge_types = self.list_edge_types(space_id)?;

        let mut export_data = serde_json::json!({
            "space_id": space_id,
            "tags": tags,
            "edge_types": edge_types,
        });

        if config.include_versions {
            if let Some(_current_version) = self.get_current_version(space_id) {
                let history = self.get_schema_history(space_id)?;
                export_data["schema_versions"] = serde_json::to_value(&history)
                    .map_err(|e| ManagerError::SchemaError(format!("序列化Schema版本失败: {}", e)))?;
            }
        }

        if !config.include_comments {
            if let Some(tags_array) = export_data.get_mut("tags").and_then(|v| v.as_array_mut()) {
                for tag in tags_array {
                    if let Some(obj) = tag.as_object_mut() {
                        obj.remove("comment");
                    }
                }
            }
            if let Some(edge_types_array) = export_data.get_mut("edge_types").and_then(|v| v.as_array_mut()) {
                for edge_type in edge_types_array {
                    if let Some(obj) = edge_type.as_object_mut() {
                        obj.remove("comment");
                    }
                }
            }
        }

        match config.format.as_str() {
            "json" => serde_json::to_string_pretty(&export_data)
                .map_err(|e| ManagerError::SchemaError(format!("序列化Schema失败: {}", e))),
            "compact" => serde_json::to_string(&export_data)
                .map_err(|e| ManagerError::SchemaError(format!("序列化Schema失败: {}", e))),
            _ => Err(ManagerError::InvalidInput(format!(
                "不支持的格式: {}",
                config.format
            ))),
        }
    }

    fn import_schema(
        &self,
        space_id: i32,
        schema_data: &str,
    ) -> ManagerResult<SchemaImportResult> {
        let mut result = SchemaImportResult {
            imported_tags: Vec::new(),
            imported_edge_types: Vec::new(),
            skipped_items: Vec::new(),
            errors: Vec::new(),
        };

        let parsed: serde_json::Value = serde_json::from_str(schema_data)
            .map_err(|e| ManagerError::SchemaError(format!("解析Schema数据失败: {}", e)))?;

        if let Some(tags_array) = parsed.get("tags").and_then(|v| v.as_array()) {
            for tag_value in tags_array {
                match serde_json::from_value::<TagDefWithId>(tag_value.clone()) {
                    Ok(tag_def) => {
                        match self.create_tag(
                            space_id,
                            &tag_def.tag_name,
                            tag_def.fields.clone(),
                        ) {
                            Ok(_) => result.imported_tags.push(tag_def.tag_name.clone()),
                            Err(e) => {
                                if e.to_string().contains("已存在") {
                                    result.skipped_items.push(format!("标签 {}", tag_def.tag_name));
                                } else {
                                    result.errors.push(format!(
                                        "导入标签 {} 失败: {}",
                                        tag_def.tag_name, e
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("解析标签定义失败: {}", e));
                    }
                }
            }
        }

        if let Some(edge_types_array) = parsed.get("edge_types").and_then(|v| v.as_array()) {
            for edge_type_value in edge_types_array {
                match serde_json::from_value::<EdgeTypeDefWithId>(edge_type_value.clone()) {
                    Ok(edge_type_def) => {
                        match self.create_edge_type(
                            space_id,
                            &edge_type_def.edge_type_name,
                            edge_type_def.fields.clone(),
                        ) {
                            Ok(_) => result
                                .imported_edge_types
                                .push(edge_type_def.edge_type_name.clone()),
                            Err(e) => {
                                if e.to_string().contains("已存在") {
                                    result
                                        .skipped_items
                                        .push(format!("边类型 {}", edge_type_def.edge_type_name));
                                } else {
                                    result.errors.push(format!(
                                        "导入边类型 {} 失败: {}",
                                        edge_type_def.edge_type_name, e
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("解析边类型定义失败: {}", e));
                    }
                }
            }
        }

        Ok(result)
    }

    fn validate_schema_compatibility(
        &self,
        space_id: i32,
        target_version: i32,
    ) -> ManagerResult<bool> {
        let current_version = self
            .get_current_version(space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 没有当前版本", space_id)))?;

        if current_version == target_version {
            return Ok(true);
        }

        let target_schema = self
            .get_schema_version(space_id, target_version)
            .ok_or_else(|| ManagerError::NotFound(format!("版本 {} 不存在", target_version)))?;

        let current_schema = self
            .get_schema_version(space_id, current_version)
            .ok_or_else(|| ManagerError::NotFound(format!("版本 {} 不存在", current_version)))?;

        let mut is_compatible = true;

        for target_tag in &target_schema.tags {
            if let Some(current_tag) = current_schema.tags.iter().find(|t| t.tag_name == target_tag.tag_name) {
                for target_field in &target_tag.fields {
                    if let Some(current_field) = current_tag.fields.iter().find(|f| f.name == target_field.name) {
                        if target_field.data_type != current_field.data_type {
                            is_compatible = false;
                        }
                        if !target_field.nullable && current_field.nullable {
                            is_compatible = false;
                        }
                    }
                }
            }
        }

        for target_edge_type in &target_schema.edge_types {
            if let Some(current_edge_type) = current_schema
                .edge_types
                .iter()
                .find(|e| e.edge_type_name == target_edge_type.edge_type_name)
            {
                for target_field in &target_edge_type.fields {
                    if let Some(current_field) = current_edge_type
                        .fields
                        .iter()
                        .find(|f| f.name == target_field.name)
                    {
                        if target_field.data_type != current_field.data_type {
                            is_compatible = false;
                        }
                        if !target_field.nullable && current_field.nullable {
                            is_compatible = false;
                        }
                    }
                }
            }
        }

        Ok(is_compatible)
    }
}

impl MemorySchemaManager {
    fn save_schema_versions_to_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        let schema_versions = self
            .schema_versions
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let versions_content = serde_json::to_string_pretty(&*schema_versions)
            .map_err(|e| ManagerError::SchemaError(format!("序列化Schema版本失败: {}", e)))?;
        let versions_file = self.storage_path.join("schema_versions.json");
        fs::write(&versions_file, versions_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn load_schema_versions_from_disk(&self) -> ManagerResult<()> {
        use std::fs;

        let versions_file = self.storage_path.join("schema_versions.json");
        if !versions_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&versions_file)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let versions_map: HashMap<i32, SchemaHistory> = serde_json::from_str(&content)
            .map_err(|e| ManagerError::SchemaError(format!("反序列化Schema版本失败: {}", e)))?;

        let mut schema_versions = self
            .schema_versions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        *schema_versions = versions_map;

        let mut max_version = 0;
        for history in schema_versions.values() {
            for version in &history.versions {
                if version.version > max_version {
                    max_version = version.version;
                }
            }
        }

        let mut next_version = self
            .next_version
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        *next_version = max_version + 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_schema_manager_creation() {
        let manager = MemorySchemaManager::new();
        assert!(manager.list_schemas().is_empty());
    }

    #[test]
    fn test_memory_schema_manager_add_schema() {
        let manager = MemorySchemaManager::new();

        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };

        assert!(manager.add_schema(schema).is_ok());
        assert!(manager.has_schema("users"));
        assert_eq!(manager.list_schemas(), vec!["users".to_string()]);
    }

    #[test]
    fn test_memory_schema_manager_get_schema() {
        let manager = MemorySchemaManager::new();

        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };

        manager
            .add_schema(schema.clone())
            .expect("Expected successful addition of schema");

        let retrieved = manager.get_schema("users");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.expect("Expected schema 'users' to exist").name,
            "users"
        );
    }

    #[test]
    fn test_memory_schema_manager_remove_schema() {
        let manager = MemorySchemaManager::new();

        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::new(),
            is_vertex: true,
        };

        manager
            .add_schema(schema)
            .expect("Expected successful addition of schema for removal test");
        assert!(manager.has_schema("users"));

        manager
            .remove_schema("users")
            .expect("Expected successful removal of schema");
        assert!(!manager.has_schema("users"));
    }

    #[test]
    fn test_memory_schema_manager_update_schema() {
        let manager = MemorySchemaManager::new();

        let schema1 = Schema {
            name: "users".to_string(),
            fields: HashMap::from([("id".to_string(), "int".to_string())]),
            is_vertex: true,
        };

        let schema2 = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };

        manager
            .add_schema(schema1)
            .expect("Expected successful addition of first schema");
        assert_eq!(
            manager
                .get_schema("users")
                .expect("Expected schema 'users' to exist")
                .fields
                .len(),
            1
        );

        manager
            .update_schema("users", schema2)
            .expect("Expected successful update of schema");
        assert_eq!(
            manager
                .get_schema("users")
                .expect("Expected schema 'users' to exist after update")
                .fields
                .len(),
            2
        );
    }

    #[test]
    fn test_create_tag() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields = vec![
            FieldDef {
                name: "id".to_string(),
                data_type: "int".to_string(),
                nullable: false,
                default_value: None,
            },
            FieldDef {
                name: "name".to_string(),
                data_type: "string".to_string(),
                nullable: false,
                default_value: None,
            },
        ];

        let tag_id = manager
            .create_tag(1, "person", fields)
            .expect("Failed to create tag");
        assert_eq!(tag_id, 1);
        assert!(manager.has_tag(1, tag_id));

        let tag_def = manager.get_tag(1, tag_id).expect("Failed to get tag");
        assert_eq!(tag_def.tag_name, "person");
        assert_eq!(tag_def.fields.len(), 2);
    }

    #[test]
    fn test_drop_tag() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        let tag_id = manager
            .create_tag(1, "person", fields)
            .expect("Failed to create tag");
        assert!(manager.has_tag(1, tag_id));

        manager.drop_tag(1, tag_id).expect("Failed to drop tag");
        assert!(!manager.has_tag(1, tag_id));
    }

    #[test]
    fn test_list_tags() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        let fields2 = vec![FieldDef {
            name: "name".to_string(),
            data_type: "string".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag1");
        manager
            .create_tag(1, "company", fields2)
            .expect("Failed to create tag2");

        let tags = manager.list_tags(1).expect("Failed to list tags");
        assert_eq!(tags.len(), 2);
        assert!(tags.iter().any(|t| t.tag_name == "person"));
        assert!(tags.iter().any(|t| t.tag_name == "company"));
    }

    #[test]
    fn test_create_edge_type() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields = vec![
            FieldDef {
                name: "weight".to_string(),
                data_type: "double".to_string(),
                nullable: true,
                default_value: None,
            },
            FieldDef {
                name: "since".to_string(),
                data_type: "timestamp".to_string(),
                nullable: false,
                default_value: None,
            },
        ];

        let edge_type_id = manager
            .create_edge_type(1, "friend", fields)
            .expect("Failed to create edge type");
        assert_eq!(edge_type_id, 1);
        assert!(manager.has_edge_type(1, edge_type_id));

        let edge_type_def = manager
            .get_edge_type(1, edge_type_id)
            .expect("Failed to get edge type");
        assert_eq!(edge_type_def.edge_type_name, "friend");
        assert_eq!(edge_type_def.fields.len(), 2);
    }

    #[test]
    fn test_drop_edge_type() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields = vec![FieldDef {
            name: "weight".to_string(),
            data_type: "double".to_string(),
            nullable: true,
            default_value: None,
        }];

        let edge_type_id = manager
            .create_edge_type(1, "friend", fields)
            .expect("Failed to create edge type");
        assert!(manager.has_edge_type(1, edge_type_id));

        manager
            .drop_edge_type(1, edge_type_id)
            .expect("Failed to drop edge type");
        assert!(!manager.has_edge_type(1, edge_type_id));
    }

    #[test]
    fn test_list_edge_types() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "weight".to_string(),
            data_type: "double".to_string(),
            nullable: true,
            default_value: None,
        }];

        let fields2 = vec![FieldDef {
            name: "since".to_string(),
            data_type: "timestamp".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_edge_type(1, "friend", fields1)
            .expect("Failed to create edge type1");
        manager
            .create_edge_type(1, "follow", fields2)
            .expect("Failed to create edge type2");

        let edge_types = manager
            .list_edge_types(1)
            .expect("Failed to list edge types");
        assert_eq!(edge_types.len(), 2);
        assert!(edge_types.iter().any(|e| e.edge_type_name == "friend"));
        assert!(edge_types.iter().any(|e| e.edge_type_name == "follow"));
    }

    #[test]
    fn test_save_and_load_from_disk() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let manager1 = MemorySchemaManager::with_storage_path(storage_path);

        let fields = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager1
            .create_tag(1, "person", fields.clone())
            .expect("Failed to create tag");
        manager1
            .create_edge_type(1, "friend", fields)
            .expect("Failed to create edge type");

        manager1.save_to_disk().expect("Failed to save to disk");

        assert!(storage_path.join("tags.json").exists());
        assert!(storage_path.join("edge_types.json").exists());

        let manager2 = MemorySchemaManager::with_storage_path(storage_path);
        manager2.load_from_disk().expect("Failed to load from disk");

        let tags = manager2.list_tags(1).expect("Failed to list tags");
        assert_eq!(tags.len(), 1);

        let edge_types = manager2
            .list_edge_types(1)
            .expect("Failed to list edge types");
        assert_eq!(edge_types.len(), 1);
    }

    #[test]
    fn test_load_from_disk_empty() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let result = manager.load_from_disk();
        assert!(result.is_ok());

        let tags = manager.list_tags(1).expect("Failed to list tags");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_auto_save_on_create_and_drop() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let manager1 = MemorySchemaManager::with_storage_path(storage_path);

        let fields = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager1
            .create_tag(1, "person", fields)
            .expect("Failed to create tag");

        assert!(storage_path.join("tags.json").exists());

        let manager2 = MemorySchemaManager::with_storage_path(storage_path);
        manager2.load_from_disk().expect("Failed to load from disk");
        assert!(manager2.has_tag(1, 1));

        manager2.drop_tag(1, 1).expect("Failed to drop tag");

        let manager3 = MemorySchemaManager::with_storage_path(storage_path);
        manager3.load_from_disk().expect("Failed to load from disk");
        assert!(!manager3.has_tag(1, 1));
    }

    #[test]
    fn test_space_isolation() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields.clone())
            .expect("Failed to create tag in space1");
        manager
            .create_tag(2, "person", fields)
            .expect("Failed to create tag in space2");

        let tags_space1 = manager.list_tags(1).expect("Failed to list tags in space1");
        let tags_space2 = manager.list_tags(2).expect("Failed to list tags in space2");

        assert_eq!(tags_space1.len(), 1);
        assert_eq!(tags_space2.len(), 1);
    }

    #[test]
    fn test_create_schema_version() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag");

        let version = manager
            .create_schema_version(1, Some("初始版本".to_string()))
            .expect("Failed to create schema version");
        assert_eq!(version, 1);

        let current_version = manager
            .get_current_version(1)
            .expect("Failed to get current version");
        assert_eq!(current_version, 1);
    }

    #[test]
    fn test_get_schema_version() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag");
        manager
            .create_schema_version(1, Some("版本1".to_string()))
            .expect("Failed to create schema version");

        let schema_version = manager
            .get_schema_version(1, 1)
            .expect("Failed to get schema version");
        assert_eq!(schema_version.version, 1);
        assert_eq!(schema_version.space_id, 1);
        assert_eq!(schema_version.tags.len(), 1);
        assert_eq!(schema_version.tags[0].tag_name, "person");
    }

    #[test]
    fn test_get_schema_history() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag");
        manager
            .create_schema_version(1, Some("版本1".to_string()))
            .expect("Failed to create schema version");

        let fields2 = vec![FieldDef {
            name: "name".to_string(),
            data_type: "string".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "company", fields2)
            .expect("Failed to create tag");
        manager
            .create_schema_version(1, Some("版本2".to_string()))
            .expect("Failed to create schema version");

        let history = manager
            .get_schema_history(1)
            .expect("Failed to get schema history");
        assert_eq!(history.space_id, 1);
        assert_eq!(history.versions.len(), 2);
        assert_eq!(history.current_version, 2);
    }

    #[test]
    fn test_rollback_schema() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = MemorySchemaManager::with_storage_path(temp_dir.path());

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag");
        manager
            .create_schema_version(1, Some("版本1".to_string()))
            .expect("Failed to create schema version");

        let fields2 = vec![FieldDef {
            name: "name".to_string(),
            data_type: "string".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager
            .create_tag(1, "company", fields2)
            .expect("Failed to create tag");
        manager
            .create_schema_version(1, Some("版本2".to_string()))
            .expect("Failed to create schema version");

        let tags = manager.list_tags(1).expect("Failed to list tags");
        assert_eq!(tags.len(), 2);

        manager
            .rollback_schema(1, 1)
            .expect("Failed to rollback schema");

        let tags_after_rollback = manager
            .list_tags(1)
            .expect("Failed to list tags after rollback");
        assert_eq!(tags_after_rollback.len(), 1);
        assert_eq!(tags_after_rollback[0].tag_name, "person");

        let current_version = manager
            .get_current_version(1)
            .expect("Failed to get current version");
        assert_eq!(current_version, 1);
    }

    #[test]
    fn test_schema_version_persistence() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let manager1 = MemorySchemaManager::with_storage_path(storage_path);

        let fields1 = vec![FieldDef {
            name: "id".to_string(),
            data_type: "int".to_string(),
            nullable: false,
            default_value: None,
        }];

        manager1
            .create_tag(1, "person", fields1)
            .expect("Failed to create tag");
        manager1
            .create_schema_version(1, Some("持久化测试".to_string()))
            .expect("Failed to create schema version");

        let manager2 = MemorySchemaManager::with_storage_path(storage_path);
        manager2.load_from_disk().expect("Failed to load from disk");

        let current_version = manager2
            .get_current_version(1)
            .expect("Failed to get current version");
        assert_eq!(current_version, 1);

        let schema_version = manager2
            .get_schema_version(1, 1)
            .expect("Failed to get schema version");
        assert_eq!(schema_version.tags.len(), 1);
        assert_eq!(schema_version.tags[0].tag_name, "person");
    }
}
