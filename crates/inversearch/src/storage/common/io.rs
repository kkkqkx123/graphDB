//! Memory I/O Operations
//!
//! Provide file read/write, serialization, and other storage-enabling shared operations

use crate::error::Result;
use crate::storage::common::types::FileStorageData;
use oxicode::config::standard;
use oxicode::serde::{decode_from_slice, encode_to_vec};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Save data to file
///
/// Serialize data and write to file using bincode
pub async fn save_to_file(path: &Path, data: &FileStorageData) -> Result<()> {
    let serialized = encode_to_vec(data, standard())
        .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

    let mut file = File::create(path).await?;
    file.write_all(&serialized).await?;
    file.sync_all().await?;

    Ok(())
}

/// Load data from file
///
/// Read from file and deserialize using bincode
pub async fn load_from_file(path: &Path) -> Result<FileStorageData> {
    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(_) => {
            // File does not exist, return empty data
            return Ok(FileStorageData {
                version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                data: std::collections::HashMap::new(),
                context_data: std::collections::HashMap::new(),
                documents: std::collections::HashMap::new(),
            });
        }
    };

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;

    if contents.is_empty() {
        return Ok(FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: std::collections::HashMap::new(),
            context_data: std::collections::HashMap::new(),
            documents: std::collections::HashMap::new(),
        });
    }

    let (data, _): (FileStorageData, usize) = decode_from_slice(&contents, standard())
        .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

    Ok(data)
}

/// Atomic Write Files
///
/// Write to temporary file first, then atomically rename to target file
pub async fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    // Make sure the parent directory exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Creating a temporary file path
    let temp_path = path.with_extension("tmp");

    // Write to temporary files
    let mut file = File::create(&temp_path).await?;
    file.write_all(data).await?;
    file.sync_all().await?;
    drop(file);

    // rename an atom
    tokio::fs::rename(&temp_path, path).await?;

    Ok(())
}

/// Secure file deletion
///
/// Ignore file not existing error
pub async fn remove_file_safe(path: &Path) -> Result<()> {
    let _ = tokio::fs::remove_file(path).await;
    Ok(())
}

/// Get file size
///
/// Returns 0 if the file does not exist
pub fn get_file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let data = b"hello world";
        atomic_write(&file_path, data).await.unwrap();

        let content = tokio::fs::read(&file_path).await.unwrap();
        assert_eq!(content, data);
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("data.bin");

        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: std::collections::HashMap::new(),
            context_data: std::collections::HashMap::new(),
            documents: std::collections::HashMap::new(),
        };

        save_to_file(&file_path, &data).await.unwrap();
        let loaded = load_from_file(&file_path).await.unwrap();

        assert_eq!(loaded.version, data.version);
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.bin");

        let loaded = load_from_file(&file_path).await.unwrap();
        assert!(loaded.data.is_empty());
    }
}
