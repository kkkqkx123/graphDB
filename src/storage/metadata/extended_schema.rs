use crate::core::error::ManagerError;
use crate::core::types::{
    SchemaChange,
    SchemaExportConfig, SchemaImportResult,
    SchemaVersion, TagInfo, EdgeTypeInfo,
};

pub trait ExtendedSchemaManager: Send + Sync {
    fn create_schema_version(&self, space_id: i32) -> Result<i32, ManagerError>;
    fn get_schema_version(&self, space_id: i32) -> Result<i32, ManagerError>;
    fn rollback_schema(&self, space_id: i32, version: i32) -> Result<(), ManagerError>;
    fn save_schema_snapshot(
        &self,
        space_id: i32,
        tags: Vec<TagInfo>,
        edge_types: Vec<EdgeTypeInfo>,
        comment: Option<String>,
    ) -> Result<SchemaVersion, ManagerError>;
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
