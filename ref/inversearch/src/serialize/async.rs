//! 异步序列化模块
//!
//! 提供异步的索引导入导出功能

use crate::serialize::{SerializeConfig, IndexExportData};
use crate::async_::AsyncIndex;
use crate::error::Result;
use std::sync::Arc;

/// 异步序列化器
pub struct AsyncSerializer {
    config: SerializeConfig,
}

impl AsyncSerializer {
    /// 创建新的异步序列化器
    pub fn new(config: SerializeConfig) -> Self {
        Self { config }
    }

    /// 异步导出为 JSON
    pub async fn to_json_async(&self, index: &AsyncIndex) -> Result<String> {
        let config = self.config.clone();
        let index_clone = index.clone();
        
        tokio::task::spawn_blocking(move || {
            let index_guard = index_clone.index.blocking_read();
            let data = index_guard.export(&config)?;
            serde_json::to_string_pretty(&data).map_err(Into::into)
        }).await?
    }

    /// 异步导出为二进制
    pub async fn to_binary_async(&self, index: &AsyncIndex) -> Result<Vec<u8>> {
        let config = self.config.clone();
        let index_clone = index.clone();
        
        tokio::task::spawn_blocking(move || {
            let index_guard = index_clone.index.blocking_read();
            index_guard.to_binary(&config)
        }).await?
    }

    /// 异步从 JSON 导入
    pub async fn from_json_async(&self, index: &AsyncIndex, json_str: &str) -> Result<()> {
        let config = self.config.clone();
        let json_str = json_str.to_string();
        let index_clone = index.clone();
        
        tokio::task::spawn_blocking(move || {
            let data: IndexExportData = serde_json::from_str(&json_str)?;
            
            let mut index_guard = index_clone.index.blocking_write();
            index_guard.import(data, &config)
        }).await?
    }

    /// 异步从二进制导入
    pub async fn from_binary_async(&self, index: &AsyncIndex, binary_data: Vec<u8>) -> Result<()> {
        let config = self.config.clone();
        let index_clone = index.clone();
        
        tokio::task::spawn_blocking(move || {
            let data: IndexExportData = if config.compression {
                bincode::deserialize(&binary_data)?
            } else {
                bincode::deserialize(&binary_data)?
            };
            
            let mut index_guard = index_clone.index.blocking_write();
            index_guard.import(data, &config)
        }).await?
    }

    /// 异步导出到文件
    pub async fn export_to_file_async(&self, index: &AsyncIndex, path: &str) -> Result<()> {
        let json_str = self.to_json_async(index).await?;
        
        tokio::fs::write(path, json_str).await?;
        
        Ok(())
    }

    /// 异步从文件导入
    pub async fn import_from_file_async(&self, index: &mut AsyncIndex, path: &str) -> Result<()> {
        let contents = tokio::fs::read_to_string(path).await?;
        self.from_json_async(index, &contents).await
    }
}

impl Default for AsyncSerializer {
    fn default() -> Self {
        Self::new(SerializeConfig::default())
    }
}

/// 异步 Document 序列化器
pub struct AsyncDocumentSerializer {
    config: SerializeConfig,
}

impl AsyncDocumentSerializer {
    /// 创建新的异步 Document 序列化器
    pub fn new(config: SerializeConfig) -> Self {
        Self { config }
    }

    /// 异步导出 Document 为 JSON
    pub async fn to_json_async(&self, document: &crate::document::Document) -> Result<String> {
        let config = self.config.clone();
        let document_data = document.export(&config)?;
        
        tokio::task::spawn_blocking(move || {
            serde_json::to_string_pretty(&document_data).map_err(Into::into)
        }).await?
    }

    /// 异步从 JSON 导入 Document
    pub async fn from_json_async(&self, json_str: &str) -> Result<crate::document::Document> {
        let config = self.config.clone();
        let json_str = json_str.to_string();
        
        tokio::task::spawn_blocking(move || {
            crate::document::Document::from_json(&json_str, &config)
        }).await?
    }

    /// 异步导出到文件
    pub async fn export_to_file_async(&self, document: &crate::document::Document, path: &str) -> Result<()> {
        let json_str = self.to_json_async(document).await?;
        
        tokio::fs::write(path, json_str).await?;
        
        Ok(())
    }

    /// 异步从文件导入
    pub async fn import_from_file_async(&self, path: &str) -> Result<crate::document::Document> {
        let contents = tokio::fs::read_to_string(path).await?;
        self.from_json_async(&contents).await
    }
}

impl Default for AsyncDocumentSerializer {
    fn default() -> Self {
        Self::new(SerializeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;
    use crate::document::{Document, DocumentConfig, FieldConfig};
    use serde_json::json;

    #[tokio::test]
    async fn test_async_serializer_json() {
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        // 异步添加文档
        async_index.add_async(1, "hello world", false).await.unwrap();
        async_index.add_async(2, "rust programming", false).await.unwrap();
        
        // 异步导出为 JSON
        let serializer = AsyncSerializer::default();
        let json_str = serializer.to_json_async(&async_index).await.unwrap();
        
        // 验证 JSON 格式
        let data: IndexExportData = serde_json::from_str(&json_str).unwrap();
        assert_eq!(data.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_async_serializer_file() {
        use tempfile::NamedTempFile;
        
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        async_index.add_async(1, "test document", false).await.unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();
        
        // 异步导出到文件
        let serializer = AsyncSerializer::default();
        serializer.export_to_file_async(&async_index, &file_path).await.unwrap();
        
        // 验证文件存在
        assert!(std::path::Path::new(&file_path).exists());
    }

    #[tokio::test]
    async fn test_async_document_serializer() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();

        let mut document = Document::new(config).unwrap();
        document.add(1, &json!({"title": "Hello World"})).unwrap();
        
        // 异步导出为 JSON
        let serializer = AsyncDocumentSerializer::default();
        let json_str = serializer.to_json_async(&document).await.unwrap();
        
        // 验证 JSON 格式
        let data: crate::document::DocumentExportData = serde_json::from_str(&json_str).unwrap();
        assert_eq!(data.version, "0.1.0");
        assert_eq!(data.document_info.field_count, 1);
    }
}
