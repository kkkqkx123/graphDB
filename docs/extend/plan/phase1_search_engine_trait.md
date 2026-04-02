# Phase 1: SearchEngine Trait 和适配器实现

## 阶段目标

建立全文检索的基础抽象层，实现 `SearchEngine` Trait 以及 BM25 和 Inversearch 两个搜索引擎的适配器。

**预计工期**: 3-5 天  
**依赖条件**: 无（本阶段为后续阶段的基础）

---

## 新增文件清单

### 1. 核心 Trait 定义

| 文件路径 | 说明 |
|---------|------|
| `src/search/mod.rs` | Search 模块入口 |
| `src/search/engine.rs` | `SearchEngine` Trait 定义 |
| `src/search/result.rs` | 搜索结果和统计信息结构体 |
| `src/search/error.rs` | 全文检索错误类型定义 |

### 2. BM25 适配器

| 文件路径 | 说明 |
|---------|------|
| `src/search/adapters/mod.rs` | 适配器模块入口 |
| `src/search/adapters/bm25_adapter.rs` | BM25 引擎适配实现 |

### 3. Inversearch 适配器

| 文件路径 | 说明 |
|---------|------|
| `src/search/adapters/inversearch_adapter.rs` | Inversearch 引擎适配实现 |

---

## 详细实现方案

### 1. Cargo.toml 依赖配置

```toml
[dependencies]
# 全文检索引擎
bm25-service = { path = "../crates/bm25", default-features = false }
inversearch-service = { path = "../crates/inversearch", default-features = false, features = ["cache", "store"] }

# 异步支持
async-trait = "0.1"
tokio = { version = "1", features = ["sync", "time"] }

# 并发控制
dashmap = "5"
parking_lot = "0.12"

# 序列化
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }

# 错误处理
thiserror = "1"
anyhow = "1"
```

### 2. SearchEngine Trait 定义

**文件**: `src/search/engine.rs`

```rust
use async_trait::async_trait;
use crate::core::Value;
use crate::search::result::{SearchResult, IndexStats};
use crate::search::error::SearchError;

/// 全文搜索引擎 Trait
/// 
/// 本 Trait 用于抽象 BM25 和 Inversearch 两个搜索引擎，
/// 提供统一的全文检索接口。
#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    /// 获取引擎名称
    fn name(&self) -> &str;
    
    /// 获取引擎版本
    fn version(&self) -> &str;
    
    /// 索引单个文档
    /// 
    /// # Arguments
    /// * `doc_id` - 文档唯一标识（顶点ID的字符串表示）
    /// * `content` - 文档内容
    /// 
    /// # Note
    /// 文档不会立即持久化，需要调用 `commit()` 或等待自动提交
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError>;
    
    /// 批量索引文档
    /// 
    /// # Arguments
    /// * `docs` - 文档列表，每个元素为 (doc_id, content) 元组
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError>;
    
    /// 执行搜索
    /// 
    /// # Arguments
    /// * `query` - 搜索查询字符串
    /// * `limit` - 返回结果数量限制
    /// 
    /// # Returns
    /// 搜索结果列表，按相关性降序排列
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;
    
    /// 删除文档
    /// 
    /// # Arguments
    /// * `doc_id` - 要删除的文档ID
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError>;
    
    /// 批量删除文档
    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError>;
    
    /// 提交所有未保存的变更
    /// 
    /// # Note
    /// - BM25: 调用 Tantivy 的 commit
    /// - Inversearch: 执行显式序列化
    async fn commit(&self) -> Result<(), SearchError>;
    
    /// 回滚未提交的变更
    async fn rollback(&self) -> Result<(), SearchError>;
    
    /// 获取索引统计信息
    async fn stats(&self) -> Result<IndexStats, SearchError>;
    
    /// 关闭引擎，释放资源
    async fn close(&self) -> Result<(), SearchError>;
}

/// 搜索引擎类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    /// BM25 引擎（基于 Tantivy）
    Bm25,
    /// Inversearch 引擎（自定义实现）
    Inversearch,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Bm25 => write!(f, "bm25"),
            EngineType::Inversearch => write!(f, "inversearch"),
        }
    }
}
```

### 3. 搜索结果结构

**文件**: `src/search/result.rs`

```rust
use crate::core::Value;
use chrono::{DateTime, Utc};

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文档ID
    pub doc_id: Value,
    /// 相关性评分
    pub score: f32,
    /// 高亮片段（可选）
    pub highlights: Option<Vec<String>>,
    /// 匹配字段列表
    pub matched_fields: Vec<String>,
}

/// 索引统计信息
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// 文档数量
    pub doc_count: usize,
    /// 索引大小（字节）
    pub index_size: usize,
    /// 最后更新时间
    pub last_updated: Option<DateTime<Utc>>,
    /// 引擎特定信息
    pub engine_info: Option<serde_json::Value>,
}
```

### 4. 错误类型定义

**文件**: `src/search/error.rs`

```rust
use thiserror::Error;

/// 全文检索错误类型
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("引擎未找到: {0}")]
    EngineNotFound(String),
    
    #[error("索引未找到: {0}")]
    IndexNotFound(String),
    
    #[error("索引已存在: {0}")]
    IndexAlreadyExists(String),
    
    #[error("引擎不可用")]
    EngineUnavailable,
    
    #[error("索引损坏: {0}")]
    IndexCorrupted(String),
    
    #[error("BM25 引擎错误: {0}")]
    Bm25Error(String),
    
    #[error("Inversearch 引擎错误: {0}")]
    InversearchError(String),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("查询解析错误: {0}")]
    QueryParseError(String),
    
    #[error("文档ID格式错误: {0}")]
    InvalidDocId(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SearchError>;
```

### 5. BM25 适配器实现

**文件**: `src/search/adapters/bm25_adapter.rs`

```rust
use async_trait::async_trait;
use bm25_service::index::{IndexManager, IndexSchema, SearchOptions};
use bm25_service::error::Bm25Error;
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::search::engine::{SearchEngine, EngineType};
use crate::search::result::{SearchResult, IndexStats};
use crate::search::error::SearchError;
use crate::core::Value;

/// BM25 搜索引擎适配器
#[derive(Debug)]
pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: std::path::PathBuf,
}

impl Bm25SearchEngine {
    /// 创建或打开 BM25 索引
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let schema = Self::build_schema();
        
        let manager = if path.exists() {
            IndexManager::open(path)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?
        } else {
            std::fs::create_dir_all(path)?;
            IndexManager::create(path)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?
        };
        
        Ok(Self {
            manager: Arc::new(manager),
            schema,
            index_path: path.to_path_buf(),
        })
    }
    
    fn build_schema() -> IndexSchema {
        use bm25_service::index::FieldConfig;
        
        let mut schema = IndexSchema::new();
        schema.add_field("doc_id", FieldConfig::string().set_stored(true));
        schema.add_field("content", FieldConfig::text().set_stored(true));
        schema
    }
    
    /// 获取索引大小
    fn get_index_size(&self) -> Result<usize, SearchError> {
        let mut total_size = 0;
        for entry in walkdir::WalkDir::new(&self.index_path) {
            let entry = entry.map_err(|e| SearchError::IoError(e.into()))?;
            if entry.file_type().is_file() {
                total_size += entry.metadata()
                    .map_err(|e| SearchError::IoError(e))?
                    .len() as usize;
            }
        }
        Ok(total_size)
    }
}

#[async_trait]
impl SearchEngine for Bm25SearchEngine {
    fn name(&self) -> &str {
        "bm25"
    }
    
    fn version(&self) -> &str {
        "0.1.0"
    }
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let doc_id = doc_id.to_string();
        let content = content.to_string();
        
        tokio::task::spawn_blocking(move || {
            manager.add_document(&doc_id, &content)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        
        tokio::task::spawn_blocking(move || {
            for (doc_id, content) in docs {
                manager.add_document(&doc_id, &content)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let manager = self.manager.clone();
        let query = query.to_string();
        let options = SearchOptions::default().with_limit(limit);
        
        tokio::task::spawn_blocking(move || {
            let results = bm25_service::index::search(&manager, &query, &options)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            
            Ok(results.into_iter().map(|r| SearchResult {
                doc_id: Value::from(r.doc_id),
                score: r.score,
                highlights: None,
                matched_fields: vec!["content".to_string()],
            }).collect())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let doc_id = doc_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            manager.delete_document(&doc_id)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let doc_ids: Vec<String> = doc_ids.into_iter().map(|s| s.to_string()).collect();
        
        tokio::task::spawn_blocking(move || {
            for doc_id in doc_ids {
                manager.delete_document(&doc_id)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn commit(&self) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        
        tokio::task::spawn_blocking(move || {
            manager.commit()
                .map_err(|e| SearchError::Bm25Error(e.to_string()))
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn rollback(&self) -> Result<(), SearchError> {
        // BM25/Tantivy 不支持显式回滚
        // 未提交的变更会在下次 commit 前被丢弃
        Ok(())
    }
    
    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let manager = self.manager.clone();
        
        tokio::task::spawn_blocking(move || {
            let doc_count = manager.doc_count()
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            
            Ok(IndexStats {
                doc_count,
                index_size: 0, // 将在外部计算
                last_updated: None,
                engine_info: None,
            })
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }
    
    async fn close(&self) -> Result<(), SearchError> {
        // BM25 的 IndexManager 在 Drop 时自动清理
        self.commit().await?;
        Ok(())
    }
}
```

### 6. Inversearch 适配器实现

**文件**: `src/search/adapters/inversearch_adapter.rs`

```rust
use async_trait::async_trait;
use inversearch_service::{Index, IndexOptions, search, SearchOptions, SearchResult as InverResult};
use inversearch_service::serialize::{export_index, import_index, ExportFormat};
use std::path::{Path, PathBuf};
use parking_lot::Mutex;

use crate::search::engine::{SearchEngine, EngineType};
use crate::search::result::{SearchResult, IndexStats};
use crate::search::error::SearchError;
use crate::core::Value;

/// Inversearch 配置
#[derive(Debug, Clone)]
pub struct InversearchConfig {
    /// 分词模式: "strict" | "forward" | "reverse" | "full"
    pub tokenize_mode: String,
    /// 索引分辨率 (1-15)
    pub resolution: usize,
    /// 缓存大小
    pub cache_size: Option<usize>,
    /// 持久化路径
    pub persistence_path: Option<PathBuf>,
}

impl Default for InversearchConfig {
    fn default() -> Self {
        Self {
            tokenize_mode: "strict".to_string(),
            resolution: 9,
            cache_size: Some(1000),
            persistence_path: None,
        }
    }
}

/// Inversearch 搜索引擎适配器
#[derive(Debug)]
pub struct InversearchEngine {
    index: Mutex<Index>,
    config: InversearchConfig,
}

impl InversearchEngine {
    /// 创建新的 Inversearch 引擎
    pub fn new(config: InversearchConfig) -> Result<Self, SearchError> {
        let options = IndexOptions {
            resolution: Some(config.resolution),
            tokenize_mode: Some(&config.tokenize_mode),
            cache_size: config.cache_size,
            ..Default::default()
        };
        
        let index = Index::new(options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        
        Ok(Self {
            index: Mutex::new(index),
            config,
        })
    }
    
    /// 从持久化文件加载
    pub fn load(path: &Path, config: InversearchConfig) -> Result<Self, SearchError> {
        if path.exists() {
            let index = import_index(path, ExportFormat::Binary)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
            
            Ok(Self {
                index: Mutex::new(index),
                config: InversearchConfig {
                    persistence_path: Some(path.to_path_buf()),
                    ..config
                },
            })
        } else {
            Self::new(config)
        }
    }
    
    /// 持久化索引
    fn persist(&self) -> Result<(), SearchError> {
        if let Some(ref path) = self.config.persistence_path {
            let index = self.index.lock();
            export_index(&*index, path, ExportFormat::Binary)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }
        Ok(())
    }
    
    /// 将字符串ID转换为 u64
    fn parse_doc_id(&self, doc_id: &str) -> Result<u64, SearchError> {
        doc_id.parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))
    }
}

#[async_trait]
impl SearchEngine for InversearchEngine {
    fn name(&self) -> &str {
        "inversearch"
    }
    
    fn version(&self) -> &str {
        "0.1.0"
    }
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let id = self.parse_doc_id(doc_id)?;
        let mut index = self.index.lock();
        
        index.add(id, content, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let mut index = self.index.lock();
        
        for (doc_id, content) in docs {
            let id = self.parse_doc_id(&doc_id)?;
            index.add(id, &content, false)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let index = self.index.lock();
        
        let options = SearchOptions {
            query: Some(query.to_string()),
            limit: Some(limit),
            ..Default::default()
        };
        
        let results = search(&*index, &options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        
        Ok(results.results.into_iter().map(|r| SearchResult {
            doc_id: Value::from(r.doc_id),
            score: r.score as f32,
            highlights: r.highlights,
            matched_fields: vec!["content".to_string()],
        }).collect())
    }
    
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let id = self.parse_doc_id(doc_id)?;
        let mut index = self.index.lock();
        
        index.remove(id, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let mut index = self.index.lock();
        
        for doc_id in doc_ids {
            let id = self.parse_doc_id(doc_id)?;
            index.remove(id, false)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn commit(&self) -> Result<(), SearchError> {
        // Inversearch 内存索引自动管理
        // 但我们需要执行持久化
        self.persist()
    }
    
    async fn rollback(&self) -> Result<(), SearchError> {
        // Inversearch 不支持回滚
        // 需要重新加载持久化文件
        Ok(())
    }
    
    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let index = self.index.lock();
        
        Ok(IndexStats {
            doc_count: index.len(),
            index_size: 0, // Inversearch 不提供此信息
            last_updated: None,
            engine_info: None,
        })
    }
    
    async fn close(&self) -> Result<(), SearchError> {
        self.commit().await
    }
}
```

### 7. 模块入口文件

**文件**: `src/search/mod.rs`

```rust
//! 全文检索模块
//! 
//! 本模块提供全文检索功能，支持 BM25 和 Inversearch 两种引擎。

pub mod engine;
pub mod result;
pub mod error;
pub mod adapters;

pub use engine::{SearchEngine, EngineType};
pub use result::{SearchResult, IndexStats};
pub use error::{SearchError, Result};
```

**文件**: `src/search/adapters/mod.rs`

```rust
//! 搜索引擎适配器

pub mod bm25_adapter;
pub mod inversearch_adapter;

pub use bm25_adapter::Bm25SearchEngine;
pub use inversearch_adapter::{InversearchEngine, InversearchConfig};
```

---

## 测试方案

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_bm25_index_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path())
            .expect("Failed to create engine");
        
        // 索引文档
        engine.index("1", "Hello world").await.unwrap();
        engine.index("2", "Hello Rust").await.unwrap();
        engine.commit().await.unwrap();
        
        // 搜索
        let results = engine.search("Hello", 10).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // 验证排序
        assert!(results[0].score >= results[1].score);
    }
    
    #[tokio::test]
    async fn test_inversearch_index_and_search() {
        let engine = InversearchEngine::new(InversearchConfig::default())
            .expect("Failed to create engine");
        
        // 索引文档（使用数字ID）
        engine.index("1", "Hello world").await.unwrap();
        engine.index("2", "Hello Rust").await.unwrap();
        
        // 搜索
        let results = engine.search("Hello", 10).await.unwrap();
        assert_eq!(results.len(), 2);
    }
    
    #[tokio::test]
    async fn test_delete_document() {
        let temp_dir = TempDir::new().unwrap();
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path()).unwrap();
        
        engine.index("1", "Test document").await.unwrap();
        engine.commit().await.unwrap();
        
        // 删除
        engine.delete("1").await.unwrap();
        engine.commit().await.unwrap();
        
        // 验证删除
        let results = engine.search("Test", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
```

---

## 验收标准

- [ ] `SearchEngine` Trait 定义完成并通过编译
- [ ] `Bm25SearchEngine` 实现所有 Trait 方法
- [ ] `InversearchEngine` 实现所有 Trait 方法
- [ ] 所有单元测试通过
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| BM25 API 不匹配 | 高 | 根据实际 API 调整适配器实现 |
| Inversearch ID 类型限制 | 中 | 适配层负责字符串到 u64 的转换 |
| 异步包装性能开销 | 低 | 使用 `spawn_blocking` 处理阻塞操作 |

---

## 下一阶段依赖

本阶段完成后，以下数据结构可供 Phase 2 使用：

- `SearchEngine` Trait
- `Bm25SearchEngine` 和 `InversearchEngine` 实现
- `SearchResult` 和 `IndexStats` 结构体
- `SearchError` 错误类型
