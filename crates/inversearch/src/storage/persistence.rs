//! Persistence Manager
//!
//! Provide index persistence management functions, including backup, restore, export, etc.
//! Refer to the persistence.rs implementation of BM25.

use crate::error::Result;
use crate::storage::manager::StorageManager;
use crate::{DocId, Index};
use oxicode::config::standard;
use oxicode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Indexing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub path: String,
    pub document_count: u64,
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for IndexMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            document_count: 0,
            schema_version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Backup Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub index_name: String,
    pub backup_id: String,
    pub backup_path: PathBuf,
    pub created_at: String,
    pub size_bytes: u64,
    pub document_count: u64,
}

/// Persistence Manager
pub struct PersistenceManager {
    base_path: PathBuf,
    #[allow(dead_code)]
    storage_manager: Option<StorageManager>,
}

impl PersistenceManager {
    /// Creating a New Persistence Manager
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            storage_manager: None,
        }
    }

    /// Creating a Persistence Manager with a Storage Manager
    pub fn with_storage<P: AsRef<Path>>(base_path: P, storage_manager: StorageManager) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            storage_manager: Some(storage_manager),
        }
    }

    /// Getting the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Save index metadata
    pub fn save_metadata(&self, metadata: &IndexMetadata) -> Result<()> {
        let metadata_path = self.base_path.join("metadata.json");
        fs::create_dir_all(&self.base_path)?;

        let json = serde_json::to_string_pretty(metadata)?;
        let mut file = File::create(metadata_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Load index metadata
    pub fn load_metadata(&self) -> Result<IndexMetadata> {
        let metadata_path = self.base_path.join("metadata.json");

        if !metadata_path.exists() {
            return Ok(IndexMetadata::default());
        }

        let mut file = File::open(metadata_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let metadata = serde_json::from_str(&contents)?;
        Ok(metadata)
    }

    /// Creating a Backup
    pub fn create_backup(&self, index: &Index, index_name: &str) -> Result<BackupInfo> {
        let backup_dir = self.base_path.join("backups").join(index_name);
        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let backup_path = backup_dir.join(format!("backup_{}", timestamp));

        // Serialized Index Data
        let index_data = self.serialize_index(index)?;

        // Save to backup directory
        fs::create_dir_all(&backup_path)?;
        let data_file = backup_path.join("index_data.bin");
        let mut file = File::create(&data_file)?;
        file.write_all(&index_data)?;

        // Calculating Backup Size
        let size_bytes = self.get_dir_size(&backup_path)?;
        let document_count = index.documents.len() as u64;

        let backup_info = BackupInfo {
            index_name: index_name.to_string(),
            backup_id: timestamp.clone(),
            backup_path: backup_path.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes,
            document_count,
        };

        // Creating a separate metadata file
        let info_file = backup_dir.join(format!("backup_info_{}.json", timestamp));
        let json = serde_json::to_string_pretty(&backup_info)?;
        let mut file = File::create(info_file)?;
        file.write_all(json.as_bytes())?;

        Ok(backup_info)
    }

    /// Restore from backup
    pub fn restore_backup(&self, index: &mut Index, backup_path: &Path) -> Result<()> {
        let data_file = backup_path.join("index_data.bin");

        if !data_file.exists() {
            return Err(crate::error::InversearchError::Storage(
                crate::error::StorageError::Generic("Backup data file not found".to_string()),
            ));
        }

        let mut file = File::open(&data_file)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        self.deserialize_index(index, &data)?;

        Ok(())
    }

    /// List all backups
    pub fn list_backups(&self, index_name: &str) -> Result<Vec<BackupInfo>> {
        let backup_dir = self.base_path.join("backups").join(index_name);

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Find the backup_info_*.json file
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("backup_info_") && filename_str.ends_with(".json") {
                    let mut contents = String::new();
                    if let Ok(mut file) = File::open(&path) {
                        if file.read_to_string(&mut contents).is_ok() {
                            if let Ok(info) = serde_json::from_str::<BackupInfo>(&contents) {
                                backups.push(info);
                            }
                        }
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    /// Deleting old backups
    pub fn delete_old_backups(&self, index_name: &str, keep_count: u32) -> Result<u32> {
        let backups = self.list_backups(index_name)?;

        if backups.len() <= keep_count as usize {
            return Ok(0);
        }

        let to_delete = backups.len() - keep_count as usize;
        let mut deleted = 0u32;

        for backup in backups.into_iter().take(to_delete) {
            // Delete the backup directory
            if backup.backup_path.is_dir() {
                if fs::remove_dir_all(&backup.backup_path).is_ok() {
                    deleted += 1;
                }
            } else if fs::remove_file(&backup.backup_path).is_ok() {
                deleted += 1;
            }

            // Delete the corresponding metadata file
            let backup_dir = self.base_path.join("backups").join(index_name);
            let info_file = backup_dir.join(format!("backup_info_{}.json", backup.backup_id));

            if info_file.exists() {
                let _ = fs::remove_file(info_file);
            }
        }

        Ok(deleted)
    }

    /// Exporting an Index to a File
    pub fn export_index(&self, index: &Index, output_file: &Path) -> Result<()> {
        let data = self.serialize_index(index)?;

        fs::create_dir_all(output_file.parent().unwrap_or(Path::new(".")))?;
        let mut file = File::create(output_file)?;
        file.write_all(&data)?;

        Ok(())
    }

    /// Importing an index from a file
    pub fn import_index(&self, index: &mut Index, input_file: &Path) -> Result<()> {
        let mut file = File::open(input_file)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        self.deserialize_index(index, &data)?;

        Ok(())
    }

    /// Serialized Indexes
    fn serialize_index(&self, index: &Index) -> Result<Vec<u8>> {
        use crate::serialize::IndexExportData;

        let export_data = IndexExportData::from_index(index)?;
        let data = encode_to_vec(&export_data, standard())
            .map_err(|e| crate::error::InversearchError::Serialization(e.to_string()))?;

        Ok(data)
    }

    /// Deserialized indexes
    fn deserialize_index(&self, index: &mut Index, data: &[u8]) -> Result<()> {
        use crate::serialize::IndexExportData;

        let (export_data, _): (IndexExportData, usize) = decode_from_slice(data, standard())
            .map_err(|e| crate::error::InversearchError::Deserialization(e.to_string()))?;

        export_data.apply_to_index(index)?;

        Ok(())
    }

    /// Recursive Copy Catalog
    #[allow(dead_code)]
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Get directory size
    fn get_dir_size(&self, path: &Path) -> Result<u64> {
        let mut size = 0u64;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                size += self.get_dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }

        Ok(size)
    }

    /// Synchronized index to storage (asynchronous)
    pub async fn sync_to_storage(&self, index: &Index, storage: &StorageManager) -> Result<()> {
        storage.commit(index, false, true).await
    }

    /// Recovering indexes from storage (asynchronous)
    pub async fn restore_from_storage(
        &self,
        index: &mut Index,
        storage: &StorageManager,
    ) -> Result<()> {
        // Mounting indexes with storage manager
        storage.mount_index(index).await
    }
}

/// Index Export Data (simplified version for PersistenceManager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub documents: Vec<(DocId, String)>,
    pub map_entries: Vec<(String, Vec<DocId>)>,
    pub ctx_entries: Vec<(String, Vec<DocId>)>,
    pub timestamp: String,
}

impl IndexSnapshot {
    /// Creating a Snapshot from an Index
    pub fn from_index(index: &Index) -> Self {
        let documents: Vec<(DocId, String)> = index
            .documents
            .iter()
            .map(|(&id, content)| (id, content.clone()))
            .collect();

        // Note: This simplifies the process, and the actual implementation requires traversing the KeystoreMap.
        let map_entries = Vec::new();
        let ctx_entries = Vec::new();

        Self {
            documents,
            map_entries,
            ctx_entries,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Applying Snapshots to Indexes
    pub fn apply_to_index(&self, index: &mut Index) -> Result<()> {
        index.clear();

        for (id, content) in &self.documents {
            index.documents.insert(*id, content.clone());
        }

        // Note: Index mapping needs to be rebuilt
        // Here the processing is simplified, the actual implementation requires re-indexing all documents

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_save_load() {
        let temp_dir = std::env::temp_dir().join("inversearch_test_metadata");
        let _ = fs::remove_dir_all(&temp_dir);

        let manager = PersistenceManager::new(&temp_dir);

        let metadata = IndexMetadata {
            name: "test_index".to_string(),
            path: temp_dir.to_string_lossy().to_string(),
            document_count: 100,
            schema_version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        assert!(manager.save_metadata(&metadata).is_ok());

        let loaded = manager.load_metadata().unwrap();
        assert_eq!(loaded.name, "test_index");
        assert_eq!(loaded.document_count, 100);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_index_snapshot() {
        let index = Index::new(crate::IndexOptions::default()).unwrap();
        let snapshot = IndexSnapshot::from_index(&index);

        assert!(snapshot.documents.is_empty());
        assert!(!snapshot.timestamp.is_empty());
    }
}
