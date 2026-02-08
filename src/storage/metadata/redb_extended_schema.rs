use crate::core::error::ManagerError;
use crate::core::types::{SchemaChange, SchemaExportConfig, SchemaImportResult, SchemaVersion, TagInfo, EdgeTypeInfo};
use crate::storage::metadata::ExtendedSchemaManager;
use crate::storage::redb_types::{ByteKey, SCHEMA_VERSIONS_TABLE, SCHEMA_CHANGES_TABLE, CURRENT_VERSIONS_TABLE};
use bincode::{encode_to_vec, decode_from_slice};
use redb::{Database, ReadableTable};
use std::sync::Arc;

pub struct RedbExtendedSchemaManager {
    db: Arc<Database>,
}

impl std::fmt::Debug for RedbExtendedSchemaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbExtendedSchemaManager").finish()
    }
}

impl RedbExtendedSchemaManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn make_version_key(space_id: i32, version: i32) -> ByteKey {
        ByteKey(format!("schema_version:{}:{}", space_id, version).into_bytes())
    }

    fn make_change_key(space_id: i32, timestamp: i64) -> ByteKey {
        ByteKey(format!("schema_change:{}:{}", space_id, timestamp).into_bytes())
    }

    fn make_current_version_key(space_id: i32) -> ByteKey {
        ByteKey(format!("current_version:{}", space_id).into_bytes())
    }

    pub fn save_schema_snapshot(
        &self,
        space_id: i32,
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

        let key = Self::make_version_key(space_id, new_version);
        let value = encode_to_vec(&snapshot, bincode::config::standard())
            .map_err(|e| ManagerError::storage_error(format!("序列化失败: {}", e)))?;

        let write_txn = self.db.begin_write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        {
            let mut table = write_txn.open_table(SCHEMA_VERSIONS_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
            table.insert(key, ByteKey(value))
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        }

        {
            let mut table = write_txn.open_table(CURRENT_VERSIONS_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
            let current_key = Self::make_current_version_key(space_id);
            let current_value = new_version.to_be_bytes().to_vec();
            table.insert(current_key, ByteKey(current_value))
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        }

        write_txn.commit()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        Ok(snapshot)
    }
}

impl ExtendedSchemaManager for RedbExtendedSchemaManager {
    fn create_schema_version(&self, space_id: i32) -> Result<i32, ManagerError> {
        let current_version = self.get_schema_version(space_id)?;
        let new_version = current_version + 1;

        let write_txn = self.db.begin_write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        {
            let mut table = write_txn.open_table(CURRENT_VERSIONS_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
            let key = Self::make_current_version_key(space_id);
            let value = new_version.to_be_bytes().to_vec();
            table.insert(key, ByteKey(value))
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        Ok(new_version)
    }

    fn get_schema_version(&self, space_id: i32) -> Result<i32, ManagerError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let table = read_txn.open_table(CURRENT_VERSIONS_TABLE)
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        let key = Self::make_current_version_key(space_id);
        match table.get(key)
            .map_err(|e| ManagerError::storage_error(e.to_string()))? {
            Some(value) => {
                let bytes = value.value().0;
                if bytes.len() == 4 {
                    let version = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    Ok(version)
                } else {
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }

    fn rollback_schema(&self, space_id: i32, version: i32) -> Result<(), ManagerError> {
        if version < 1 {
            return Err(ManagerError::invalid_input("版本号必须 >= 1"));
        }

        let write_txn = self.db.begin_write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        {
            let mut table = write_txn.open_table(CURRENT_VERSIONS_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
            let key = Self::make_current_version_key(space_id);
            let value = version.to_be_bytes().to_vec();
            table.insert(key, ByteKey(value))
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        Ok(())
    }

    fn record_schema_change(
        &self,
        space_id: i32,
        change: SchemaChange,
    ) -> Result<(), ManagerError> {
        let key = Self::make_change_key(space_id, change.timestamp);
        let value = encode_to_vec(&change, bincode::config::standard())
            .map_err(|e| ManagerError::storage_error(format!("序列化失败: {}", e)))?;

        let write_txn = self.db.begin_write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        {
            let mut table = write_txn.open_table(SCHEMA_CHANGES_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
            table.insert(key, ByteKey(value))
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        Ok(())
    }

    fn get_schema_changes(
        &self,
        space_id: i32,
    ) -> Result<Vec<SchemaChange>, ManagerError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        let table = read_txn.open_table(SCHEMA_CHANGES_TABLE)
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        let prefix = format!("schema_change:{}:", space_id);
        let mut changes = Vec::new();

        for result in table.iter()
            .map_err(|e| ManagerError::storage_error(e.to_string()))? {
            let (key, value) = result.map_err(|e| ManagerError::storage_error(e.to_string()))?;
            let key_bytes = key.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_bytes);
            if key_str.starts_with(&prefix) {
                let change: SchemaChange = decode_from_slice(&value.value().0, bincode::config::standard())
                    .map_err(|e| ManagerError::storage_error(format!("反序列化失败: {}", e)))?
                    .0;
                changes.push(change);
            }
        }

        Ok(changes)
    }

    fn clear_schema_changes(&self, space_id: i32) -> Result<(), ManagerError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;
        {
            let mut table = write_txn.open_table(SCHEMA_CHANGES_TABLE)
                .map_err(|e| ManagerError::storage_error(e.to_string()))?;

            let prefix = format!("schema_change:{}:", space_id);
            let keys_to_remove: Vec<ByteKey> = table.iter()
                .map_err(|e| ManagerError::storage_error(e.to_string()))?
                .filter_map(|result| {
                    result.ok().and_then(|(key, _)| {
                        let key_bytes = key.value().0.clone();
                        let key_str = String::from_utf8_lossy(&key_bytes);
                        if key_str.starts_with(&prefix) {
                            Some(key.value().clone())
                        } else {
                            None
                        }
                    })
                })
                .collect();

            for key in keys_to_remove {
                let _ = table.remove(key);
            }
        }
        write_txn.commit()
            .map_err(|e| ManagerError::storage_error(e.to_string()))?;

        Ok(())
    }

    fn export_schema(&self, _config: &SchemaExportConfig) -> Result<String, ManagerError> {
        Err(ManagerError::storage_error("导出功能暂未实现".to_string()))
    }

    fn import_schema(&self, _data: &str) -> Result<SchemaImportResult, ManagerError> {
        Err(ManagerError::storage_error("导入功能暂未实现".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{SchemaChangeType, TagInfo, PropertyDef, DataType};
    use tempfile::TempDir;

    fn create_test_manager() -> (RedbExtendedSchemaManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Arc::new(Database::create(db_path).unwrap());
        
        // 初始化所需的表
        let write_txn = db.begin_write().unwrap();
        {
            let _ = write_txn.open_table(SCHEMA_VERSIONS_TABLE).unwrap();
        }
        {
            let _ = write_txn.open_table(SCHEMA_CHANGES_TABLE).unwrap();
        }
        {
            let _ = write_txn.open_table(CURRENT_VERSIONS_TABLE).unwrap();
        }
        write_txn.commit().unwrap();
        
        (RedbExtendedSchemaManager::new(db), temp_dir)
    }

    #[test]
    fn test_schema_version() {
        let (manager, _temp_dir) = create_test_manager();
        let space_id = 1;

        // 初始版本为 0
        let version = manager.get_schema_version(space_id).unwrap();
        assert_eq!(version, 0);

        // 创建新版本
        let new_version = manager.create_schema_version(space_id).unwrap();
        assert_eq!(new_version, 1);

        let version = manager.get_schema_version(space_id).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_save_schema_snapshot() {
        let (manager, _temp_dir) = create_test_manager();
        let space_id = 1;

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
        }];

        let edge_types = vec![];

        let snapshot = manager.save_schema_snapshot(
            space_id,
            tags.clone(),
            edge_types,
            Some("创建 Person 标签".to_string()),
        ).unwrap();

        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.space_id, space_id);
        assert_eq!(snapshot.tags.len(), 1);

        // 验证版本已更新
        let version = manager.get_schema_version(space_id).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_record_schema_change() {
        let (manager, _temp_dir) = create_test_manager();
        let space_id = 1;

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

        manager.record_schema_change(space_id, change.clone()).unwrap();

        let changes = manager.get_schema_changes(space_id).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].target, "Person.name");
    }
}
