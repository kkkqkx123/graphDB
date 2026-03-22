use crate::index::{IndexManager, IndexSchema};
use crate::error::Result;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use tantivy::schema::TantivyDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub path: String,
    pub document_count: u64,
    pub schema_version: u32,
}

impl Default for IndexMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            document_count: 0,
            schema_version: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub index_name: String,
    pub backup_path: PathBuf,
    pub created_at: String,
    pub size_bytes: u64,
    pub document_count: u64,
}

pub struct PersistenceManager {
    base_path: PathBuf,
}

impl PersistenceManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn create_backup(&self, _manager: &IndexManager, index_name: &str) -> Result<BackupInfo> {
        let backup_dir = self.base_path.join("backups").join(index_name);
        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_file = backup_dir.join(format!("backup_{}.json", timestamp));

        let index_path = self.base_path.join(index_name);

        if index_path.exists() {
            fs::copy(&index_path, &backup_file)?;
        }

        let metadata = fs::metadata(&backup_file)?;
        let size_bytes = metadata.len();

        let backup_info = BackupInfo {
            index_name: index_name.to_string(),
            backup_path: backup_file,
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes,
            document_count: 0,
        };

        let backup_info_dir = self.base_path.join("backups").join(&backup_info.index_name);
        fs::create_dir_all(&backup_info_dir)?;

        let info_file = backup_info_dir.join("backup_info.json");
        let json = serde_json::to_string_pretty(&backup_info)?;

        let mut file = File::create(info_file)?;
        file.write_all(json.as_bytes())?;

        Ok(backup_info)
    }

    pub fn restore_backup(&self, index_name: &str, backup_file: &Path) -> Result<()> {
        let index_path = self.base_path.join(index_name);

        if index_path.exists() {
            fs::remove_dir_all(&index_path)?;
        }

        fs::copy(backup_file, &index_path)?;

        Ok(())
    }

    pub fn list_backups(&self, index_name: &str) -> Result<Vec<BackupInfo>> {
        let backup_dir = self.base_path.join("backups").join(index_name);

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e.to_string_lossy()) == Some("json".into()) {
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

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub fn delete_old_backups(&self, index_name: &str, keep_count: u32) -> Result<u32> {
        let backups = self.list_backups(index_name)?;

        if backups.len() <= keep_count as usize {
            return Ok(0);
        }

        let to_delete = backups.len() - keep_count as usize;
        let mut deleted = 0u32;

        for backup in backups.into_iter().take(to_delete) {
            if fs::remove_file(&backup.backup_path).is_ok() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    pub fn export_index(&self, manager: &IndexManager, _index_name: &str, output_file: &Path) -> Result<()> {
        let reader = manager.reader()?;
        let searcher = reader.searcher();

        let mut file = File::create(output_file)?;

        writeln!(file, "{{\"total_docs\": {}}}", searcher.num_docs())?;

        Ok(())
    }

    pub fn import_index(&self, manager: &IndexManager, schema: &IndexSchema, input_file: &Path) -> Result<u64> {
        let mut file = File::open(input_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let imported_count = contents.len();

        let mut doc = TantivyDocument::new();
        doc.add_text(schema.document_id, "imported_0");
        doc.add_text(schema.title, "");
        doc.add_text(schema.content, "");

        let mut writer = manager.writer()?;
        writer.add_document(doc)?;
        writer.commit()?;

        Ok(imported_count as u64)
    }

    pub fn get_index_metadata(&self, index_name: &str) -> Result<IndexMetadata> {
        let metadata_file = self.base_path.join("metadata").join(format!("{}.json", index_name));

        if !metadata_file.exists() {
            return Ok(IndexMetadata::default());
        }

        let mut file = File::open(&metadata_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let metadata: IndexMetadata = serde_json::from_str(&contents)?;
        Ok(metadata)
    }

    pub fn compact_index(&self, manager: &IndexManager) -> Result<()> {
        let writer = manager.writer()?;
        writer.wait_merging_threads()?;
        Ok(())
    }

    pub fn get_index_size(&self, index_name: &str) -> Result<u64> {
        let index_path = self.base_path.join(index_name);

        if !index_path.exists() {
            return Ok(0);
        }

        self.get_dir_size(&index_path)
    }

    fn get_dir_size(&self, dir: &Path) -> Result<u64> {
        let mut total = 0u64;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                total += fs::metadata(&path)?.len();
            } else if path.is_dir() {
                total += self.get_dir_size(&path)?;
            }
        }

        Ok(total)
    }
}
