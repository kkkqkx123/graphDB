//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;

pub mod redis;

/// 存储接口 - 类似JavaScript版本的StorageInterface
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// 挂载索引到存储
    async fn mount(&mut self, index: &Index) -> Result<()>;
    
    /// 打开连接
    async fn open(&mut self) -> Result<()>;
    
    /// 关闭连接
    async fn close(&mut self) -> Result<()>;
    
    /// 销毁数据库
    async fn destroy(&mut self) -> Result<()>;
    
    /// 提交索引变更
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;
    
    /// 获取术语结果
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, resolve: bool, enrich: bool) -> Result<SearchResults>;
    
    /// 富化结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    
    /// 检查ID是否存在
    async fn has(&self, id: DocId) -> Result<bool>;
    
    /// 删除ID
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    /// 清空数据
    async fn clear(&mut self) -> Result<()>;
    
    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// 内存存储实现 - 用于测试和开发
pub struct MemoryStorage {
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    is_open: bool,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            is_open: false,
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInterface for MemoryStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }
    
    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        Ok(())
    }
    
    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }
    
    async fn destroy(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.is_open = false;
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 从索引导出数据到存储
        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_insert_with(HashMap::new)
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        Ok(())
    }
    
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = if let Some(ctx_key) = ctx {
            // 上下文搜索
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            // 普通搜索
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };
        
        Ok(results)
    }
    
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = Vec::new();
        
        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }
        
        Ok(results)
    }
    
    async fn has(&self, id: DocId) -> Result<bool> {
        // 检查文档ID是否存在于索引数据中
        for (_term, doc_ids) in &self.data {
            if doc_ids.contains(&id) {
                return Ok(true);
            }
        }
        
        // 检查上下文数据
        for (_ctx, ctx_map) in &self.context_data {
            for (_term, doc_ids) in ctx_map {
                if doc_ids.contains(&id) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            
            // 从索引数据中移除
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }
            
            // 从上下文数据中移除
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        Ok(())
    }
    
    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (self.data.len() + self.context_data.len() + self.documents.len()) as u64,
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

/// 文件存储实现 - 简单的文件持久化
pub struct FileStorage {
    file_path: String,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
}

impl FileStorage {
    /// 创建新的文件存储
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
        }
    }
    
    /// 保存到文件
    pub async fn save_to_file(&self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        let data = serde_json::json!({
            "data": self.data,
            "context_data": self.context_data,
            "documents": self.documents,
        });
        
        let json_str = serde_json::to_string_pretty(&data)?;
        let mut file = File::create(&self.file_path).await?;
        file.write_all(json_str.as_bytes()).await?;
        
        Ok(())
    }
    
    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let mut file = match File::open(&self.file_path).await {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        if contents.trim().is_empty() {
            return Ok(());
        }

        let data: serde_json::Value = serde_json::from_str(&contents)?;

        self.data = serde_json::from_value(data["data"].clone())?;
        self.context_data = serde_json::from_value(data["context_data"].clone())?;
        self.documents = serde_json::from_value(data["documents"].clone())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        // 尝试从文件加载现有数据
        if let Err(_) = self.load_from_file().await {
            // 文件不存在或加载失败，创建新的空存储
        }
        Ok(())
    }
    
    async fn open(&mut self) -> Result<()> {
        self.load_from_file().await
    }
    
    async fn close(&mut self) -> Result<()> {
        self.save_to_file().await
    }
    
    async fn destroy(&mut self) -> Result<()> {
        use tokio::fs;
        
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        
        // 删除文件
        if let Err(_) = fs::remove_file(&self.file_path).await {
            // 文件可能不存在，忽略错误
        }
        
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 从索引导出数据
        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_insert_with(HashMap::new)
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        // 保存到文件
        self.save_to_file().await
    }
    
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = if let Some(ctx_key) = ctx {
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };
        
        Ok(results)
    }
    
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = Vec::new();
        
        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }
        
        Ok(results)
    }
    
    async fn has(&self, id: DocId) -> Result<bool> {
        Ok(self.documents.contains_key(&id))
    }
    
    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            
            // 从索引数据中移除
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }
            
            // 从上下文数据中移除
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.save_to_file().await
    }
    
    async fn info(&self) -> Result<StorageInfo> {
        let file_size = if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            metadata.len()
        } else {
            0
        };
        
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "0.1.0".to_string(),
            size: file_size,
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: true,
        })
    }
}

/// 应用限制和偏移的辅助函数
fn apply_limit_offset(results: &[DocId], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = if limit > 0 {
        (start + limit).min(results.len())
    } else {
        results.len()
    };

    results[start..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_memory_storage() {
        let mut storage = MemoryStorage::new();
        storage.open().await.unwrap();
        
        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();
        
        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();
        
        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.unwrap();
        println!("Get results: {:?}", results);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));
        
        // 测试存在检查
        println!("Checking has(1)");
        let has_result = storage.has(1).await.unwrap();
        println!("has(1) result: {}", has_result);
        assert!(has_result);
        assert!(!storage.has(3).await.unwrap());
        
        // 测试删除
        storage.remove(&[1]).await.unwrap();
        assert!(!storage.has(1).await.unwrap());
        
        storage.close().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_file_storage() {
        use tempfile::NamedTempFile;
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();
        
        let mut storage = FileStorage::new(file_path);
        storage.open().await.unwrap();
        
        let mut index = Index::default();
        index.add(1, "test document", false).unwrap();
        index.add(2, "another test", false).unwrap();
        
        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();
        
        // 测试获取
        let results = storage.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        
        // 关闭存储（会保存到文件）
        storage.close().await.unwrap();
        
        // 重新打开并验证数据还在
        let mut storage2 = FileStorage::new(temp_file.path().to_str().unwrap().to_string());
        storage2.open().await.unwrap();
        
        let results2 = storage2.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results2.len(), 2);
        
        storage2.destroy().await.unwrap();
    }
}