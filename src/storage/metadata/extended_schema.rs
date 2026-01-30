use crate::core::error::ManagerError;
use crate::core::types::{
    ClusterInfo, EdgeTypeInfo, PropertyDef, SchemaChange,
    SchemaExportConfig, SchemaHistory, SchemaImportResult,
    SchemaVersion, SpaceInfo, TagInfo,
};
use crate::storage::metadata::SchemaManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub trait ExtendedSchemaManager: Send + Sync {
    fn create_schema_version(&self, space_id: i32) -> Result<i32, ManagerError>;
    fn get_schema_version(&self, space_id: i32) -> Result<i32, ManagerError>;
    fn rollback_schema(&self, space_id: i32, version: i32) -> Result<(), ManagerError>;
    fn record_schema_change(
        &self,
        space_id: i32,
        change: SchemaChange,
    ) -> Result<(), ManagerError>;
    fn get_schema_changes(
        &self,
        space_id: i32,
    ) -> Result<Vec<SchemaChange>, ManagerError>;
    fn clear_schema_changes(&self, space_id: i32) -> Result<(), ManagerError>;
    fn export_schema(&self, config: &SchemaExportConfig)
        -> Result<String, ManagerError>;
    fn import_schema(&self, data: &str) -> Result<SchemaImportResult, ManagerError>;
}

pub struct MemoryExtendedSchemaManager<S: SchemaManager> {
    base: Arc<S>,
    schema_versions: Arc<Mutex<HashMap<i32, Vec<SchemaVersion>>>>,
    schema_changes: Arc<Mutex<HashMap<i32, Vec<SchemaChange>>>>,
    current_versions: Arc<Mutex<HashMap<i32, i32>>>,
    cluster_info: Arc<RwLock<Option<ClusterInfo>>>,
}

impl<S: SchemaManager> MemoryExtendedSchemaManager<S> {
    pub fn new(base: Arc<S>) -> Self {
        Self {
            base,
            schema_versions: Arc::new(Mutex::new(HashMap::new())),
            schema_changes: Arc::new(Mutex::new(HashMap::new())),
            current_versions: Arc::new(Mutex::new(HashMap::new())),
            cluster_info: Arc::new(RwLock::new(None)),
        }
    }

    fn save_schema_snapshot(
        &self,
        space_id: i32,
        tags: Vec<TagInfo>,
        edge_types: Vec<EdgeTypeInfo>,
        comment: Option<String>,
    ) -> Result<SchemaVersion, ManagerError> {
        let mut versions = self
            .schema_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let mut current_versions = self
            .current_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        let current_version = current_versions.get(&space_id).copied().unwrap_or(0);
        let new_version = current_version + 1;

        let snapshot = SchemaVersion {
            version: new_version,
            space_id,
            tags: tags.clone(),
            edge_types,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            comment,
        };

        versions.entry(space_id).or_insert_with(Vec::new).push(snapshot.clone());
        current_versions.insert(space_id, new_version);

        Ok(snapshot)
    }
}

impl<S: SchemaManager> ExtendedSchemaManager for MemoryExtendedSchemaManager<S> {
    fn create_schema_version(&self, space_id: i32) -> Result<i32, ManagerError> {
        let mut versions = self
            .current_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let new_version = versions.get(&space_id).copied().unwrap_or(0) + 1;
        versions.insert(space_id, new_version);
        Ok(new_version)
    }

    fn get_schema_version(&self, space_id: i32) -> Result<i32, ManagerError> {
        let versions = self
            .current_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        Ok(versions.get(&space_id).copied().unwrap_or(0))
    }

    fn rollback_schema(&self, space_id: i32, version: i32) -> Result<(), ManagerError> {
        let versions = self
            .schema_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let space_versions = versions
            .get(&space_id)
            .ok_or_else(|| ManagerError::not_found("Schema version history not found"))?;

        if version < 1 || version > space_versions.len() as i32 {
            return Err(ManagerError::invalid_input("Invalid version number"));
        }

        let mut current_versions = self
            .current_versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        current_versions.insert(space_id, version);

        Ok(())
    }

    fn record_schema_change(
        &self,
        space_id: i32,
        change: SchemaChange,
    ) -> Result<(), ManagerError> {
        let mut changes = self
            .schema_changes
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        changes
            .entry(space_id)
            .or_insert_with(Vec::new)
            .push(change);
        Ok(())
    }

    fn get_schema_changes(&self, space_id: i32) -> Result<Vec<SchemaChange>, ManagerError> {
        let changes = self
            .schema_changes
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        Ok(changes
            .get(&space_id)
            .cloned()
            .unwrap_or_default())
    }

    fn clear_schema_changes(&self, space_id: i32) -> Result<(), ManagerError> {
        let mut changes = self
            .schema_changes
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        changes.remove(&space_id);
        Ok(())
    }

    fn export_schema(
        &self,
        config: &SchemaExportConfig,
    ) -> Result<String, ManagerError> {
        let mut spaces: Vec<SpaceInfo> = self.base.list_spaces().map_err(|e| {
            ManagerError::storage_error(format!("Failed to list spaces: {}", e))
        })?;

        if let Some(space_id) = config.space_id {
            spaces.retain(|s| s.space_id == space_id);
        }

        #[derive(Serialize)]
        #[serde(bound = "")]
        struct ExportData<'a> {
            spaces: Vec<&'a SpaceInfo>,
            exported_at: i64,
            format_version: i32,
        }

        let export = ExportData {
            spaces: spaces.iter().collect(),
            exported_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            format_version: 1,
        };

        serde_json::to_string_pretty(&export).map_err(|e| {
            ManagerError::storage_error(format!("Failed to serialize schema: {}", e))
        })
    }

    fn import_schema(&self, data: &str) -> Result<SchemaImportResult, ManagerError> {
        #[derive(Deserialize)]
        #[serde(bound = "")]
        struct ImportData {
            spaces: Vec<SpaceInfo>,
        }

        let import: ImportData = serde_json::from_str(data).map_err(|e| {
            ManagerError::invalid_input(format!("Invalid schema format: {}", e))
        })?;

        let mut result = SchemaImportResult::new();

        for space in import.spaces {
            match self.base.create_space(&space) {
                Ok(true) => {
                    for tag in &space.tags {
                        if self.base.create_tag(&space.space_name, tag).is_ok() {
                            result.imported_tags.push(tag.tag_name.clone());
                        } else {
                            result.skipped_items.push(format!("tag:{}", tag.tag_name));
                        }
                    }
                    for edge_type in &space.edge_types {
                        if self.base.create_edge_type(&space.space_name, edge_type).is_ok() {
                            result.imported_edge_types.push(edge_type.edge_type_name.clone());
                        } else {
                            result.skipped_items.push(format!("edge_type:{}", edge_type.edge_type_name));
                        }
                    }
                }
                Ok(false) => {
                    result.skipped_items.push(format!("space:{}", space.space_name));
                }
                Err(e) => {
                    result.errors.push(format!("space:{} - {}", space.space_name, e));
                }
            }
        }

        Ok(result)
    }
}

impl<S: SchemaManager> MemoryExtendedSchemaManager<S> {
    pub fn get_cluster_info(&self) -> Result<ClusterInfo, ManagerError> {
        let info = self
            .cluster_info
            .read()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        info.clone()
            .ok_or_else(|| ManagerError::not_found("Cluster info not initialized"))
    }

    pub fn update_cluster_info(&self, info: ClusterInfo) -> Result<(), ManagerError> {
        let mut cluster_info = self
            .cluster_info
            .write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        *cluster_info = Some(info);
        Ok(())
    }
}

pub struct SchemaVersionManager {
    versions: Arc<Mutex<HashMap<String, i32>>>,
    histories: Arc<Mutex<HashMap<String, Vec<SchemaHistory>>>>,
}

impl SchemaVersionManager {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(Mutex::new(HashMap::new())),
            histories: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_version(&self, space_name: &str, description: String) -> Result<i32, ManagerError> {
        let mut versions = self
            .versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let current = versions.get(space_name).copied().unwrap_or(0);
        let new_version = current + 1;
        versions.insert(space_name.to_string(), new_version);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        let history = SchemaHistory {
            space_id: 0,
            versions: vec![SchemaVersion {
                version: new_version,
                space_id: 0,
                tags: Vec::new(),
                edge_types: Vec::new(),
                created_at: timestamp,
                comment: Some(description),
            }],
            current_version: new_version,
        };

        let mut histories = self
            .histories
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        histories.insert(space_name.to_string(), vec![history]);

        Ok(new_version)
    }

    pub fn get_version(&self, space_name: &str) -> Result<i32, ManagerError> {
        let versions = self
            .versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        Ok(versions.get(space_name).copied().unwrap_or(0))
    }

    pub fn rollback(&self, space_name: &str, version: i32) -> Result<(), ManagerError> {
        let mut versions = self
            .versions
            .lock()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        if version < 1 {
            return Err(ManagerError::invalid_input("Version must be >= 1"));
        }
        versions.insert(space_name.to_string(), version);
        Ok(())
    }
}

impl Default for SchemaVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFieldChange {
    pub field_name: String,
    pub change_type: FieldChangeType,
    pub old_value: Option<PropertyDef>,
    pub new_value: Option<PropertyDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAlterOperation {
    pub space_name: String,
    pub target_type: AlterTargetType,
    pub target_name: String,
    pub field_changes: Vec<SchemaFieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlterTargetType {
    Tag,
    EdgeType,
}

impl SchemaAlterOperation {
    pub fn new_add_tag_field(
        space_name: String,
        tag_name: String,
        field: PropertyDef,
    ) -> Self {
        let field_name = field.name.clone();
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Added,
                old_value: None,
                new_value: Some(field),
            }],
        }
    }

    pub fn new_remove_tag_field(
        space_name: String,
        tag_name: String,
        field_name: String,
    ) -> Self {
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Removed,
                old_value: None,
                new_value: None,
            }],
        }
    }

    pub fn new_modify_tag_field(
        space_name: String,
        tag_name: String,
        old_field: PropertyDef,
        new_field: PropertyDef,
    ) -> Self {
        let field_name = old_field.name.clone();
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Modified,
                old_value: Some(old_field),
                new_value: Some(new_field),
            }],
        }
    }
}
