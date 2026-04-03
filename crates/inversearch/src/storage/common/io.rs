//! 存储 I/O 操作
//!
//! 提供文件读写、序列化等存储实现共享的操作

use crate::error::Result;
use crate::storage::common::types::FileStorageData;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 保存数据到文件
///
/// 使用 bincode 序列化数据并写入文件
pub async fn save_to_file(path: &Path, data: &FileStorageData) -> Result<()> {
    let serialized = bincode::serialize(data)
        .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

    let mut file = File::create(path).await?;
    file.write_all(&serialized).await?;
    file.sync_all().await?;

    Ok(())
}

/// 从文件加载数据
///
/// 从文件读取并使用 bincode 反序列化
pub async fn load_from_file(path: &Path) -> Result<FileStorageData> {
    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(_) => {
            // 文件不存在，返回空数据
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

    let data: FileStorageData = bincode::deserialize(&contents)
        .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

    Ok(data)
}

/// 原子写入文件
///
/// 先写入临时文件，然后原子重命名到目标文件
pub async fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // 创建临时文件路径
    let temp_path = path.with_extension("tmp");

    // 写入临时文件
    let mut file = File::create(&temp_path).await?;
    file.write_all(data).await?;
    file.sync_all().await?;
    drop(file);

    // 原子重命名
    tokio::fs::rename(&temp_path, path).await?;

    Ok(())
}

/// 安全删除文件
///
/// 忽略文件不存在的错误
pub async fn remove_file_safe(path: &Path) -> Result<()> {
    let _ = tokio::fs::remove_file(path).await;
    Ok(())
}

/// 获取文件大小
///
/// 如果文件不存在返回 0
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
