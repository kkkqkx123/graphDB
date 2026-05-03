use crate::core::error::ManagerError;
use crate::core::types::{
    EdgeTypeInfo, SchemaChange, SchemaExportConfig, SchemaImportResult, SchemaVersion, TagInfo,
};
use crate::storage::metadata::ExtendedSchemaManager;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub struct InMemoryExtendedSchemaManager {
    current_versions: Arc<RwLock<HashMap<u64, i32>>>,
    schema_versions: Arc<RwLock<HashMap<(u64, i32), SchemaVersion>>>,
    schema_changes: Arc<RwLock<HashMap<u64, Vec<SchemaChange>>>>,
    version_counter: Arc<AtomicI32>,
}

impl std::fmt::Debug for InMemoryExtendedSchemaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryExtendedSchemaManager")
            .field("spaces_count", &self.current_versions.read().len())
            .finish()
    }
}

impl InMemoryExtendedSchemaManager {
    pub fn new() -> Self {
        Self {
            current_versions: Arc::new(RwLock::new(HashMap::new())),
            schema_versions: Arc::new(RwLock::new(HashMap::new())),
            schema_changes: Arc::new(RwLock::new(HashMap::new())),
            version_counter: Arc::new(AtomicI32::new(0)),
        }
    }
}

impl Default for InMemoryExtendedSchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtendedSchemaManager for InMemoryExtendedSchemaManager {
    fn create_schema_version(&self, space_id: u64) -> Result<i32, ManagerError> {
        let new_version = self.version_counter.fetch_add(1, Ordering::SeqCst) + 1;
        let mut versions = self.current_versions.write();
        versions.insert(space_id, new_version);
        Ok(new_version)
    }

    fn get_schema_version(&self, space_id: u64) -> Result<i32, ManagerError> {
        let versions = self.current_versions.read();
        Ok(versions.get(&space_id).copied().unwrap_or(0))
    }

    fn rollback_schema(&self, space_id: u64, version: i32) -> Result<(), ManagerError> {
        if version < 1 {
            return Err(ManagerError::invalid_input("Version number must be >= 1"));
        }

        let mut versions = self.current_versions.write();
        versions.insert(space_id, version);
        Ok(())
    }

    fn save_schema_snapshot(
        &self,
        space_id: u64,
        tags: Vec<TagInfo>,
        edge_types: Vec<EdgeTypeInfo>,
        comment: Option<String>,
    ) -> Result<SchemaVersion, ManagerError> {
        let current_version = self.get_schema_version(space_id)?;
        let new_version = current_version + 1;

        let snapshot = SchemaVersion {
            version: new_version,
            space_id,
            tags,
            edge_types,
            created_at: chrono::Utc::now().timestamp_millis(),
            comment,
        };

        let mut versions = self.schema_versions.write();
        versions.insert((space_id, new_version), snapshot.clone());

        let mut current = self.current_versions.write();
        current.insert(space_id, new_version);

        Ok(snapshot)
    }

    fn record_schema_change(
        &self,
        space_id: u64,
        change: SchemaChange,
    ) -> Result<(), ManagerError> {
        let mut changes = self.schema_changes.write();
        changes.entry(space_id).or_default().push(change);
        Ok(())
    }

    fn get_schema_changes(&self, space_id: u64) -> Result<Vec<SchemaChange>, ManagerError> {
        let changes = self.schema_changes.read();
        Ok(changes.get(&space_id).cloned().unwrap_or_default())
    }

    fn clear_schema_changes(&self, space_id: u64) -> Result<(), ManagerError> {
        let mut changes = self.schema_changes.write();
        changes.remove(&space_id);
        Ok(())
    }

    fn export_schema(&self, _config: &SchemaExportConfig) -> Result<String, ManagerError> {
        Err(ManagerError::storage_error(
            "Export functionality is not yet implemented".to_string(),
        ))
    }

    fn import_schema(&self, _data: &str) -> Result<SchemaImportResult, ManagerError> {
        Err(ManagerError::storage_error(
            "Import functionality is not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{DataType, PropertyDef, SchemaChangeType, TagInfo};

    fn create_test_manager() -> InMemoryExtendedSchemaManager {
        InMemoryExtendedSchemaManager::new()
    }

    #[test]
    fn test_schema_version() {
        let manager = create_test_manager();
        let space_id = 1u64;

        let version = manager
            .get_schema_version(space_id)
            .expect("Failed to get schema version");
        assert_eq!(version, 0);

        let new_version = manager
            .create_schema_version(space_id)
            .expect("Failed to create schema version");
        assert!(new_version >= 1);

        let version = manager
            .get_schema_version(space_id)
            .expect("Failed to get schema version");
        assert_eq!(version, new_version);
    }

    #[test]
    fn test_save_schema_snapshot() {
        let manager = create_test_manager();
        let space_id = 1u64;

        let tags = vec![TagInfo {
            tag_id: 1,
            tag_name: "Person".to_string(),
            properties: vec![PropertyDef {
                name: "name".to_string(),
                data_type: DataType::String,
                nullable: false,
                default: None,
                comment: None,
            }],
            comment: None,
            ttl_duration: None,
            ttl_col: None,
        }];

        let edge_types = vec![];

        let snapshot = manager
            .save_schema_snapshot(
                space_id,
                tags.clone(),
                edge_types,
                Some("Create Person tag".to_string()),
            )
            .expect("Failed to save schema snapshot");

        assert_eq!(snapshot.space_id, space_id);
        assert_eq!(snapshot.tags.len(), 1);
    }

    #[test]
    fn test_record_schema_change() {
        let manager = create_test_manager();
        let space_id = 1u64;

        let change = SchemaChange {
            change_type: SchemaChangeType::AddProperty,
            target: "Person.name".to_string(),
            property: Some(PropertyDef {
                name: "name".to_string(),
                data_type: DataType::String,
                nullable: false,
                default: None,
                comment: None,
            }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        manager
            .record_schema_change(space_id, change.clone())
            .expect("Failed to record schema change");

        let changes = manager
            .get_schema_changes(space_id)
            .expect("Failed to get schema change");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].target, "Person.name");
    }
}
