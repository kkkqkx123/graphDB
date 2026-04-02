# Phase 2: FulltextCoordinator 协调器实现

## 阶段目标

实现程序层面的全文检索协调器，管理索引生命周期，处理索引映射关系，提供统一的索引操作接口。

**预计工期**: 4-6 天  
**前置依赖**: Phase 1 (SearchEngine Trait 和适配器)

---

## 新增文件清单

### 1. 协调器核心

| 文件路径 | 说明 |
|---------|------|
| `src/coordinator/mod.rs` | 协调器模块入口 |
| `src/coordinator/fulltext.rs` | `FulltextCoordinator` 实现 |
| `src/coordinator/types.rs` | 协调器类型定义 |

### 2. 索引管理

| 文件路径 | 说明 |
|---------|------|
| `src/search/manager.rs` | `FulltextIndexManager` 索引管理器 |
| `src/search/factory.rs` | 搜索引擎工厂 |
| `src/search/config.rs` | 全文检索配置 |

### 3. 元数据存储

| 文件路径 | 说明 |
|---------|------|
| `src/search/metadata.rs` | 索引元数据结构 |

---

## 详细实现方案

### 1. 索引元数据结构

**文件**: `src/search/metadata.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::search::engine::EngineType;

/// 全文索引元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// 索引唯一标识
    pub index_id: String,
    /// 索引名称
    pub index_name: String,
    /// 所属图空间ID
    pub space_id: u64,
    /// Tag 名称
    pub tag_name: String,
    /// 字段名称
    pub field_name: String,
    /// 搜索引擎类型
    pub engine_type: EngineType,
    /// 索引存储路径
    pub storage_path: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 文档数量
    pub doc_count: usize,
    /// 索引状态
    pub status: IndexStatus,
    /// 引擎特定配置
    pub engine_config: Option<serde_json::Value>,
}

/// 索引状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    /// 创建中
    Creating,
    /// 可用
    Active,
    /// 重建中
    Rebuilding,
    /// 已禁用
    Disabled,
    /// 错误状态
    Error,
}

impl std::fmt::Display for IndexStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexStatus::Creating => write!(f, "CREATING"),
            IndexStatus::Active => write!(f, "ACTIVE"),
            IndexStatus::Rebuilding => write!(f, "REBUILDING"),
            IndexStatus::Disabled => write!(f, "DISABLED"),
            IndexStatus::Error => write!(f, "ERROR"),
        }
    }
}

/// 索引键（用于内存映射）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexKey {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl IndexKey {
    pub fn new(space_id: u64, tag_name: &str, field_name: &str) -> Self {
        Self {
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    
    /// 生成索引ID
    pub fn to_index_id(&self) -> String {
        format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }
}
```

### 2. 全文检索配置

**文件**: `src/search/config.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::search::engine::EngineType;
use crate::search::adapters::InversearchConfig;

/// 全文检索模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextConfig {
    /// 是否启用全文检索
    pub enabled: bool,
    /// 默认搜索引擎
    pub default_engine: EngineType,
    /// 索引存储基础路径
    pub index_path: PathBuf,
    /// 同步策略配置
    pub sync: SyncConfig,
    /// BM25 引擎配置
    pub bm25: Bm25Config,
    /// Inversearch 引擎配置
    pub inversearch: InversearchConfig,
}

impl Default for FulltextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_engine: EngineType::Bm25,
            index_path: PathBuf::from("data/fulltext"),
            sync: SyncConfig::default(),
            bm25: Bm25Config::default(),
            inversearch: InversearchConfig::default(),
        }
    }
}

/// 同步策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 同步模式: "sync" | "async" | "off"
    pub mode: SyncMode,
    /// 异步队列大小
    pub queue_size: usize,
    /// 批量提交间隔（毫秒）
    pub commit_interval_ms: u64,
    /// 批量提交文档数
    pub batch_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Async,
            queue_size: 10000,
            commit_interval_ms: 1000,
            batch_size: 100,
        }
    }
}

/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    /// 同步模式（阻塞）
    Sync,
    /// 异步模式（推荐）
    Async,
    /// 关闭同步
    Off,
}

/// BM25 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    /// 索引内存限制（MB）
    pub memory_limit_mb: usize,
    /// 是否自动提交
    pub auto_commit: bool,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Self {
            memory_limit_mb: 50,
            auto_commit: true,
        }
    }
}
```

### 3. 搜索引擎工厂

**文件**: `src/search/factory.rs`

```rust
use std::path::Path;
use std::sync::Arc;

use crate::search::engine::{SearchEngine, EngineType};
use crate::search::adapters::{Bm25SearchEngine, InversearchEngine, InversearchConfig};
use crate::search::error::SearchError;

/// 搜索引擎工厂
pub struct SearchEngineFactory;

impl SearchEngineFactory {
    /// 创建搜索引擎实例
    /// 
    /// # Arguments
    /// * `engine_type` - 引擎类型
    /// * `index_name` - 索引名称
    /// * `base_path` - 基础存储路径
    pub fn create(
        engine_type: EngineType,
        index_name: &str,
        base_path: &Path,
    ) -> Result<Arc<dyn SearchEngine>, SearchError> {
        let engine_path = base_path.join(index_name);
        
        match engine_type {
            EngineType::Bm25 => {
                let engine = Bm25SearchEngine::open_or_create(&engine_path)?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let config = InversearchConfig {
                    persistence_path: Some(engine_path.with_extension("bin")),
                    ..Default::default()
                };
                let engine = if config.persistence_path.as_ref().unwrap().exists() {
                    InversearchEngine::load(&engine_path.with_extension("bin"), config)?
                } else {
                    InversearchEngine::new(config)?
                };
                Ok(Arc::new(engine))
            }
        }
    }
    
    /// 根据配置创建引擎
    pub fn from_config(
        engine_type: EngineType,
        index_name: &str,
        base_path: &Path,
        config: &crate::search::config::FulltextConfig,
    ) -> Result<Arc<dyn SearchEngine>, SearchError> {
        let engine_path = base_path.join(index_name);
        
        match engine_type {
            EngineType::Bm25 => {
                let engine = Bm25SearchEngine::open_or_create(&engine_path)?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let mut inv_config = config.inversearch.clone();
                inv_config.persistence_path = Some(engine_path.with_extension("bin"));
                
                let engine = if inv_config.persistence_path.as_ref().unwrap().exists() {
                    InversearchEngine::load(&engine_path.with_extension("bin"), inv_config)?
                } else {
                    InversearchEngine::new(inv_config)?
                };
                Ok(Arc::new(engine))
            }
        }
    }
}
```

### 4. 索引管理器

**文件**: `src/search/manager.rs`

```rust
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::search::engine::{SearchEngine, EngineType};
use crate::search::factory::SearchEngineFactory;
use crate::search::metadata::{IndexMetadata, IndexKey, IndexStatus};
use crate::search::config::FulltextConfig;
use crate::search::error::SearchError;
use crate::search::result::{SearchResult, IndexStats};

/// 全文索引管理器
/// 
/// 负责管理所有全文索引的生命周期，包括创建、删除、搜索等操作
#[derive(Debug)]
pub struct FulltextIndexManager {
    /// 索引集合: IndexKey -> SearchEngine
    engines: DashMap<IndexKey, Arc<dyn SearchEngine>>,
    /// 索引元数据: IndexKey -> IndexMetadata
    metadata: DashMap<IndexKey, IndexMetadata>,
    /// 基础存储路径
    base_path: PathBuf,
    /// 默认引擎类型
    default_engine: EngineType,
    /// 配置
    config: FulltextConfig,
}

impl FulltextIndexManager {
    /// 创建新的索引管理器
    pub fn new(config: FulltextConfig) -> Result<Self, SearchError> {
        let base_path = config.index_path.clone();
        
        // 确保目录存在
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }
        
        Ok(Self {
            engines: DashMap::new(),
            metadata: DashMap::new(),
            base_path,
            default_engine: config.default_engine,
            config,
        })
    }
    
    /// 创建全文索引
    /// 
    /// # Arguments
    /// * `space_id` - 图空间ID
    /// * `tag_name` - Tag 名称
    /// * `field_name` - 字段名称
    /// * `engine_type` - 引擎类型（None 使用默认）
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, SearchError> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        let index_id = key.to_index_id();
        
        // 检查是否已存在
        if self.engines.contains_key(&key) {
            return Err(SearchError::IndexAlreadyExists(index_id));
        }
        
        let engine_type = engine_type.unwrap_or(self.default_engine);
        
        // 创建引擎实例
        let engine = SearchEngineFactory::from_config(
            engine_type,
            &index_id,
            &self.base_path,
            &self.config,
        )?;
        
        // 创建元数据
        let metadata = IndexMetadata {
            index_id: index_id.clone(),
            index_name: format!("idx_{}_{}_{}", space_id, tag_name, field_name),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            engine_type,
            storage_path: self.base_path.join(&index_id).to_string_lossy().to_string(),
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            doc_count: 0,
            status: IndexStatus::Active,
            engine_config: None,
        };
        
        // 保存到内存映射
        self.engines.insert(key.clone(), engine);
        self.metadata.insert(key, metadata);
        
        Ok(index_id)
    }
    
    /// 获取索引引擎
    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn SearchEngine>> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.engines.get(&key).map(|e| Arc::clone(&*e))
    }
    
    /// 获取索引元数据
    pub fn get_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<IndexMetadata> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.metadata.get(&key).map(|m| m.clone())
    }
    
    /// 检查索引是否存在
    pub fn has_index(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.engines.contains_key(&key)
    }
    
    /// 获取指定空间的所有索引
    pub fn get_space_indexes(&self, space_id: u64) -> Vec<IndexMetadata> {
        self.metadata
            .iter()
            .filter(|entry| entry.value().space_id == space_id)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// 删除索引
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        
        // 关闭引擎
        if let Some((_, engine)) = self.engines.remove(&key) {
            engine.close().await?;
        }
        
        // 删除元数据
        self.metadata.remove(&key);
        
        // 删除索引文件
        let index_id = key.to_index_id();
        let index_path = self.base_path.join(&index_id);
        if index_path.exists() {
            tokio::fs::remove_dir_all(&index_path).await?;
        }
        
        // 删除 Inversearch 的 bin 文件
        let bin_path = index_path.with_extension("bin");
        if bin_path.exists() {
            tokio::fs::remove_file(&bin_path).await?;
        }
        
        Ok(())
    }
    
    /// 执行全文搜索
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let engine = self.get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| SearchError::IndexNotFound(
                format!("{}.{}.{}", space_id, tag_name, field_name)
            ))?;
        
        engine.search(query, limit).await
    }
    
    /// 获取索引统计信息
    pub async fn get_stats(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<IndexStats, SearchError> {
        let engine = self.get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| SearchError::IndexNotFound(
                format!("{}.{}.{}", space_id, tag_name, field_name)
            ))?;
        
        engine.stats().await
    }
    
    /// 提交所有索引的变更
    pub async fn commit_all(&self) -> Result<(), SearchError> {
        for entry in self.engines.iter() {
            entry.value().commit().await?;
        }
        Ok(())
    }
    
    /// 关闭所有索引
    pub async fn close_all(&self) -> Result<(), SearchError> {
        for entry in self.engines.iter() {
            entry.value().close().await?;
        }
        self.engines.clear();
        self.metadata.clear();
        Ok(())
    }
    
    /// 列出所有索引
    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.metadata
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
```

### 5. FulltextCoordinator 实现

**文件**: `src/coordinator/fulltext.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Value, Vertex};
use crate::search::manager::FulltextIndexManager;
use crate::search::result::SearchResult;
use crate::search::error::SearchError;
use crate::search::engine::EngineType;
use crate::search::metadata::IndexMetadata;

/// 数据变更类型
#[derive(Debug, Clone, Copy)]
pub enum ChangeType {
    Insert,
    Update,
    Delete,
}

/// 全文检索协调器
/// 
/// 位于程序层面，负责协调图数据变更与全文索引的同步。
/// 不直接操作存储层，由上层业务逻辑调用。
#[derive(Debug)]
pub struct FulltextCoordinator {
    /// 索引管理器
    manager: Arc<FulltextIndexManager>,
}

impl FulltextCoordinator {
    /// 创建新的协调器
    pub fn new(manager: Arc<FulltextIndexManager>) -> Self {
        Self { manager }
    }
    
    /// 创建全文索引
    /// 
    /// # Arguments
    /// * `space_id` - 图空间ID
    /// * `tag_name` - Tag 名称
    /// * `field_name` - 字段名称
    /// * `engine_type` - 引擎类型（可选）
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, SearchError> {
        self.manager.create_index(space_id, tag_name, field_name, engine_type).await
    }
    
    /// 删除全文索引
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        self.manager.drop_index(space_id, tag_name, field_name).await
    }
    
    /// 顶点插入后调用（由上层业务逻辑调用）
    /// 
    /// # Arguments
    /// * `space_id` - 图空间ID
    /// * `vertex` - 插入的顶点
    /// 
    /// # Note
    /// 此方法为异步非阻塞，不等待索引完成
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> Result<(), SearchError> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                    if let Value::String(text) = value {
                        let doc_id = vertex.id.to_string();
                        engine.index(&doc_id, text).await?;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 顶点更新后调用
    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> Result<(), SearchError> {
        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                        if let Value::String(text) = value {
                            let doc_id = vertex.id.to_string();
                            // 先删除旧索引，再添加新索引
                            engine.delete(&doc_id).await?;
                            engine.index(&doc_id, text).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 顶点删除后调用
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<(), SearchError> {
        let doc_id = vertex_id.to_string();
        
        // 获取该 Tag 的所有索引字段
        let indexes = self.manager.get_space_indexes(space_id);
        for metadata in indexes {
            if metadata.tag_name == tag_name {
                if let Some(engine) = self.manager.get_engine(
                    space_id, 
                    &metadata.tag_name, 
                    &metadata.field_name
                ) {
                    engine.delete(&doc_id).await?;
                }
            }
        }
        Ok(())
    }
    
    /// 通用数据变更处理
    /// 
    /// # Arguments
    /// * `space_id` - 图空间ID
    /// * `tag_name` - Tag 名称
    /// * `vertex_id` - 顶点ID
    /// * `properties` - 属性映射
    /// * `change_type` - 变更类型
    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
        change_type: ChangeType,
    ) -> Result<(), SearchError> {
        let doc_id = vertex_id.to_string();
        
        for (field_name, value) in properties {
            if let Some(engine) = self.manager.get_engine(space_id, tag_name, field_name) {
                match change_type {
                    ChangeType::Insert | ChangeType::Update => {
                        if let Value::String(text) = value {
                            engine.index(&doc_id, text).await?;
                        }
                    }
                    ChangeType::Delete => {
                        engine.delete(&doc_id).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 执行全文搜索
    /// 
    /// # Arguments
    /// * `space_id` - 图空间ID
    /// * `tag_name` - Tag 名称
    /// * `field_name` - 字段名称
    /// * `query` - 搜索查询
    /// * `limit` - 结果数量限制
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        self.manager.search(space_id, tag_name, field_name, query, limit).await
    }
    
    /// 批量搜索（多索引）
    /// 
    /// 在多个索引上执行搜索并合并结果
    pub async fn search_multi(
        &self,
        indexes: &[(u64, String, String)], // (space_id, tag_name, field_name)
        query: &str,
        limit: usize,
    ) -> Result<HashMap<String, Vec<SearchResult>>, SearchError> {
        let mut results = HashMap::new();
        
        for (space_id, tag_name, field_name) in indexes {
            let key = format!("{}_{}_{}", space_id, tag_name, field_name);
            match self.search(*space_id, tag_name, field_name, query, limit).await {
                Ok(search_results) => {
                    results.insert(key, search_results);
                }
                Err(SearchError::IndexNotFound(_)) => {
                    // 索引不存在，跳过
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// 获取索引列表
    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.manager.list_indexes()
    }
    
    /// 获取索引信息
    pub fn get_index_info(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<IndexMetadata> {
        self.manager.get_metadata(space_id, tag_name, field_name)
    }
    
    /// 检查字段是否有全文索引
    pub fn has_index(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        self.manager.has_index(space_id, tag_name, field_name)
    }
    
    /// 重建索引
    /// 
    /// # Note
    /// 此操作会删除现有索引并重新创建，需要配合数据扫描重新索引
    pub async fn rebuild_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        // 1. 获取现有索引信息
        let metadata = self.manager.get_metadata(space_id, tag_name, field_name)
            .ok_or_else(|| SearchError::IndexNotFound(
                format!("{}.{}.{}", space_id, tag_name, field_name)
            ))?;
        
        let engine_type = metadata.engine_type;
        
        // 2. 删除旧索引
        self.manager.drop_index(space_id, tag_name, field_name).await?;
        
        // 3. 创建新索引
        self.manager.create_index(space_id, tag_name, field_name, Some(engine_type)).await?;
        
        // 4. 返回成功（数据重新索引由调用方处理）
        Ok(())
    }
    
    /// 提交所有索引变更
    pub async fn commit_all(&self) -> Result<(), SearchError> {
        self.manager.commit_all().await
    }
}
```

### 6. 模块入口文件

**文件**: `src/coordinator/mod.rs`

```rust
//! 协调器模块
//! 
//! 本模块提供程序层面的协调功能，负责协调不同子系统之间的交互。

pub mod fulltext;
pub mod types;

pub use fulltext::{FulltextCoordinator, ChangeType};
```

**文件**: `src/coordinator/types.rs`

```rust
//! 协调器通用类型

/// 协调结果
pub type CoordinatorResult<T> = Result<T, CoordinatorError>;

/// 协调器错误
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("全文检索错误: {0}")]
    SearchError(#[from] crate::search::error::SearchError),
    
    #[error("存储错误: {0}")]
    StorageError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
}
```

---

## 数据流设计

### 创建索引流程

```
用户/查询引擎
    │ CREATE FULLTEXT INDEX
    ▼
FulltextCoordinator::create_index()
    │ 1. 生成索引ID
    │ 2. 调用 SearchEngineFactory
    ▼
FulltextIndexManager::create_index()
    │ 1. 创建引擎实例
    │ 2. 保存元数据
    ▼
SearchEngineFactory::from_config()
    │ 根据引擎类型创建实例
    ▼
Bm25SearchEngine / InversearchEngine
    │ 初始化引擎
    ▼
返回索引ID
```

### 数据同步流程

```
存储层 (RedbStorage)
    │ 数据变更成功（事务提交）
    ▼
查询引擎/业务层
    │ 调用协调器方法
    ▼
FulltextCoordinator::on_vertex_inserted()
    │ 遍历顶点的所有Tag和属性
    ▼
检查字段是否有索引
    │ 是 → 调用 engine.index()
    ▼
异步完成（不阻塞主流程）
```

### 搜索流程

```
用户
    │ MATCH ... WHERE field MATCH "query"
    ▼
查询引擎
    │ 解析全文搜索条件
    ▼
FulltextCoordinator::search()
    │ 获取对应引擎
    ▼
SearchEngine::search()
    │ 执行引擎特定搜索
    ▼
返回 doc_ids 和 scores
    ▼
查询引擎
    │ 根据 doc_ids 查询完整数据
    ▼
返回结果给用户
```

---

## 测试方案

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn create_test_coordinator() -> (FulltextCoordinator, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = FulltextConfig {
            index_path: temp_dir.path().join("fulltext"),
            ..Default::default()
        };
        let manager = Arc::new(FulltextIndexManager::new(config).unwrap());
        let coordinator = FulltextCoordinator::new(manager);
        (coordinator, temp_dir)
    }
    
    #[tokio::test]
    async fn test_create_and_search_index() {
        let (coordinator, _temp) = create_test_coordinator().await;
        
        // 创建索引
        let index_id = coordinator
            .create_index(1, "Post", "content", None)
            .await
            .unwrap();
        
        assert!(index_id.contains("Post"));
        assert!(index_id.contains("content"));
        
        // 索引文档
        let mut properties = HashMap::new();
        properties.insert("content".to_string(), Value::from("Hello world"));
        
        coordinator
            .on_vertex_change(1, "Post", &Value::from(1i64), &properties, ChangeType::Insert)
            .await
            .unwrap();
        
        coordinator.commit_all().await.unwrap();
        
        // 搜索
        let results = coordinator.search(1, "Post", "content", "Hello", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, Value::from(1i64));
    }
    
    #[tokio::test]
    async fn test_drop_index() {
        let (coordinator, _temp) = create_test_coordinator().await;
        
        coordinator.create_index(1, "Post", "content", None).await.unwrap();
        assert!(coordinator.has_index(1, "Post", "content"));
        
        coordinator.drop_index(1, "Post", "content").await.unwrap();
        assert!(!coordinator.has_index(1, "Post", "content"));
    }
    
    #[tokio::test]
    async fn test_rebuild_index() {
        let (coordinator, _temp) = create_test_coordinator().await;
        
        // 创建并索引数据
        coordinator.create_index(1, "Post", "content", None).await.unwrap();
        
        let mut properties = HashMap::new();
        properties.insert("content".to_string(), Value::from("Test content"));
        coordinator
            .on_vertex_change(1, "Post", &Value::from(1i64), &properties, ChangeType::Insert)
            .await
            .unwrap();
        
        // 重建索引
        coordinator.rebuild_index(1, "Post", "content").await.unwrap();
        
        // 验证索引存在但数据已清空
        assert!(coordinator.has_index(1, "Post", "content"));
        let results = coordinator.search(1, "Post", "content", "Test", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
```

---

## 验收标准

- [ ] `FulltextIndexManager` 实现所有索引管理功能
- [ ] `FulltextCoordinator` 实现所有协调功能
- [ ] 支持创建、删除、搜索、重建索引
- [ ] 支持 BM25 和 Inversearch 两种引擎
- [ ] 所有单元测试通过
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 索引元数据持久化 | 中 | 后续阶段添加元数据持久化到 Redb |
| 并发索引操作 | 中 | DashMap 提供线程安全，引擎内部使用 Mutex |
| 内存中索引过多 | 中 | 后续添加索引卸载机制 |

---

## 下一阶段依赖

本阶段完成后，以下接口可供 Phase 3 使用：

- `FulltextCoordinator::create_index()` - 创建索引
- `FulltextCoordinator::search()` - 执行搜索
- `FulltextCoordinator::on_vertex_inserted()` - 数据同步
- `FulltextCoordinator::list_indexes()` - 获取索引列表
