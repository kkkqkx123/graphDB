use crate::core::error::ManagerError;
use crate::core::types::{
    ClusterInfo, EdgeTypeInfo, PropertyDef, SchemaChange,
    SchemaExportConfig, SchemaHistory, SchemaImportResult,
    SchemaVersion, SpaceInfo, TagInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
            current_version: new_version as i64,
            timestamp,
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
