# 嵌入式全文检索集成设计方案

## 概述

本方案采用 **嵌入式架构** 集成全文检索功能。将 BM25 和 Inversearch 作为库直接嵌入 GraphDB 进程，通过 Trait 抽象实现统一的全文搜索接口。

## 为什么采用嵌入式架构

### 1. 符合 GraphDB 设计原则

GraphDB 的核心设计目标是提供**轻量级单节点图数据库**，强调：
- **单可执行文件**：简化部署，无需管理多个进程
- **最小依赖**：减少外部依赖，降低运维复杂度
- **高性能**：本地调用优于网络通信

### 2. 架构对比

| 维度 | gRPC 方案 | 嵌入式方案 |
|------|-----------|------------|
| **部署方式** | 多进程管理 | 单进程 |
| **通信开销** | 5-15ms (序列化+网络) | 0.1-1ms (内存调用) |
| **运维复杂度** | 高（多服务监控） | 低（单服务） |
| **资源占用** | 多份内存 | 共享内存 |
| **单文件部署** | ❌ 破坏目标 | ✅ 保持目标 |
| **事务协调** | 困难 | 可行 |

### 3. 嵌入式方案的优势

- **零网络开销**：内存直接调用，无序列化成本
- **简化运维**：单进程监控，单日志流
- **事务协调**：可在同一逻辑中管理图数据和全文索引
- **资源效率**：共享内存，减少冗余缓存
- **保持设计目标**：维持"单可执行文件"的核心优势

## 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      GraphDB 单进程                             │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    查询执行引擎                             │  │
│  │              (nGQL Parser + Planner + Executor)            │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │                           ▼                               │  │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │  │
│  │  │   Redb 存储  │    │  BM25 引擎  │    │Inversearch  │   │  │
│  │  │  (图数据)    │    │  (Tantivy)  │    │  (内存索引)  │   │  │
│  │  │             │    │             │    │             │   │  │
│  │  │ - 顶点数据   │    │ - 倒排索引   │    │ - 倒排索引   │   │  │
│  │  │ - 边数据     │    │ - BM25评分   │    │ - 关键词匹配 │   │  │
│  │  │ - 属性索引   │    │ - 文本分析   │    │ - 多语言支持 │   │  │
│  │  └─────────────┘    └─────────────┘    └─────────────┘   │  │
│  │                                                           │  │
│  │  统一存储目录: data/                                       │  │
│  │  - graph.redb      (图数据)                                │  │
│  │  - fulltext/bm25/  (Tantivy 索引)                          │  │
│  │  - fulltext/inv/   (Inversearch 索引)                      │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │              SearchEngine Trait (统一接口)                  │  │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │  │
│  │  │   index()   │    │  search()   │    │  delete()   │   │  │
│  │  └─────────────┘    └─────────────┘    └─────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件设计

### 1. 搜索引擎 Trait 定义

```rust
// src/search/engine.rs

use async_trait::async_trait;
use crate::core::Value;

/// 全文搜索引擎 Trait
/// 所有搜索引擎实现必须实现此接口
#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    /// 获取引擎名称
    fn name(&self) -> &str;
    
    /// 获取引擎版本
    fn version(&self) -> &str;
    
    /// 索引文档
    /// 
    /// # Arguments
    /// * `doc_id` - 文档唯一标识（通常是顶点ID）
    /// * `content` - 文档内容
    async fn index(&self, doc_id: &str, content: &str) -> Result<()>;
    
    /// 批量索引文档
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<()>;
    
    /// 执行搜索
    /// 
    /// # Arguments
    /// * `query` - 搜索查询字符串
    /// * `limit` - 返回结果数量限制
    /// 
    /// # Returns
    /// 搜索结果列表，按相关性排序
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// 删除文档
    async fn delete(&self, doc_id: &str) -> Result<()>;
    
    /// 提交变更（持久化）
    async fn commit(&self) -> Result<()>;
    
    /// 回滚未提交的变更
    async fn rollback(&self) -> Result<()>;
    
    /// 获取索引统计信息
    async fn stats(&self) -> Result<IndexStats>;
    
    /// 关闭引擎，释放资源
    async fn close(&self) -> Result<()>;
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文档ID
    pub doc_id: Value,
    /// 相关性评分
    pub score: f32,
    /// 高亮片段（可选）
    pub highlights: Option<Vec<String>>,
    /// 匹配字段
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
}

/// 搜索引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    /// BM25 引擎（基于 Tantivy）
    Bm25,
    /// Inversearch 引擎（自定义实现）
    Inversearch,
}
```

### 2. BM25 引擎实现

```rust
// src/search/engines/bm25_engine.rs

use tantivy::{
    schema::*,
    Index, IndexWriter, IndexReader, ReloadPolicy,
    query::QueryParser, collector::TopDocs,
};

/// BM25 搜索引擎实现
pub struct Bm25Engine {
    index: Index,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
    query_parser: QueryParser,
    schema: Schema,
    index_path: PathBuf,
}

impl Bm25Engine {
    /// 创建或打开 BM25 引擎
    pub fn open_or_create(path: &Path) -> Result<Self> {
        let schema = Self::build_schema();
        
        let (index, is_new) = if path.exists() {
            (Index::open_in_dir(path)?, false)
        } else {
            std::fs::create_dir_all(path)?;
            (Index::create_in_dir(path, schema.clone())?, true)
        };
        
        let writer = index.writer(50_000_000)?;
        let reader = index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        
        let query_parser = QueryParser::for_index(&index, vec![
            schema.get_field("content").unwrap(),
            schema.get_field("title").unwrap(),
        ]);
        
        Ok(Self {
            index,
            writer: Mutex::new(writer),
            reader,
            query_parser,
            schema,
            index_path: path.to_path_buf(),
        })
    }
    
    fn build_schema() -> Schema {
        let mut builder = Schema::builder();
        builder.add_text_field("doc_id", STRING | STORED);
        builder.add_text_field("title", TEXT | STORED);
        builder.add_text_field("content", TEXT | STORED);
        builder.build()
    }
}

#[async_trait]
impl SearchEngine for Bm25Engine {
    fn name(&self) -> &str { "bm25" }
    fn version(&self) -> &str { "0.1.0" }
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<()> {
        let mut writer = self.writer.lock().await;
        
        let doc_id_field = self.schema.get_field("doc_id").unwrap();
        let content_field = self.schema.get_field("content").unwrap();
        
        let mut doc = Document::default();
        doc.add_text(doc_id_field, doc_id);
        doc.add_text(content_field, content);
        
        writer.add_document(doc)?;
        Ok(())
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let query = self.query_parser.parse_query(query)?;
        
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let doc_id = doc.get_first(self.schema.get_field("doc_id").unwrap())
                .and_then(|v| v.as_text())
                .unwrap_or_default();
            
            results.push(SearchResult {
                doc_id: Value::from(doc_id),
                score,
                highlights: None,
                matched_fields: vec!["content".to_string()],
            });
        }
        
        Ok(results)
    }
    
    async fn commit(&self) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.commit()?;
        Ok(())
    }
    
    // ... 其他方法实现
}
```

### 3. Inversearch 引擎实现

```rust
// src/search/engines/inversearch_engine.rs

use inversearch::{Index, IndexOptions, TokenizeMode};

/// Inversearch 搜索引擎实现
pub struct InversearchEngine {
    index: Mutex<Index>,
    config: InversearchConfig,
}

#[derive(Debug, Clone)]
pub struct InversearchConfig {
    /// 分词模式
    pub tokenize_mode: TokenizeMode,
    /// 索引分辨率
    pub resolution: usize,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存大小
    pub cache_size: usize,
}

impl Default for InversearchConfig {
    fn default() -> Self {
        Self {
            tokenize_mode: TokenizeMode::Strict,
            resolution: 9,
            enable_cache: true,
            cache_size: 1000,
        }
    }
}

impl InversearchEngine {
    /// 创建 Inversearch 引擎
    pub fn new(config: InversearchConfig) -> Result<Self> {
        let options = IndexOptions {
            resolution: Some(config.resolution),
            tokenize_mode: Some(match config.tokenize_mode {
                TokenizeMode::Strict => "strict",
                TokenizeMode::Forward => "forward",
                TokenizeMode::Reverse => "reverse",
                TokenizeMode::Full => "full",
            }),
            cache_size: if config.enable_cache { Some(config.cache_size) } else { None },
            ..Default::default()
        };
        
        let index = Index::new(options)?;
        
        Ok(Self {
            index: Mutex::new(index),
            config,
        })
    }
}

#[async_trait]
impl SearchEngine for InversearchEngine {
    fn name(&self) -> &str { "inversearch" }
    fn version(&self) -> &str { "0.1.0" }
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<()> {
        let mut index = self.index.lock().await;
        let id = doc_id.parse::<u64>()?;
        index.add(id, content, false)?;
        Ok(())
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let index = self.index.lock().await;
        let results = index.search(query, limit)?;
        
        Ok(results.into_iter().map(|r| SearchResult {
            doc_id: Value::from(r.doc_id),
            score: r.score as f32,
            highlights: r.highlights,
            matched_fields: vec!["content".to_string()],
        }).collect())
    }
    
    // ... 其他方法实现
}
```

### 4. 搜索引擎工厂

```rust
// src/search/factory.rs

use std::sync::Arc;

/// 搜索引擎工厂
pub struct SearchEngineFactory;

impl SearchEngineFactory {
    /// 创建搜索引擎实例
    pub fn create(
        engine_type: EngineType,
        index_name: &str,
        base_path: &Path,
    ) -> Result<Arc<dyn SearchEngine>> {
        let engine_path = base_path.join(index_name);
        
        match engine_type {
            EngineType::Bm25 => {
                let engine = Bm25Engine::open_or_create(&engine_path)?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let config = InversearchConfig::default();
                let engine = InversearchEngine::new(config)?;
                Ok(Arc::new(engine))
            }
        }
    }
    
    /// 根据配置创建引擎
    pub fn from_config(config: &SearchEngineConfig) -> Result<Arc<dyn SearchEngine>> {
        Self::create(config.engine_type, &config.index_name, &config.base_path)
    }
}

/// 搜索引擎配置
#[derive(Debug, Clone)]
pub struct SearchEngineConfig {
    pub engine_type: EngineType,
    pub index_name: String,
    pub base_path: PathBuf,
    pub bm25_config: Option<Bm25Config>,
    pub inversearch_config: Option<InversearchConfig>,
}
```

### 5. 全文索引管理器

```rust
// src/search/manager.rs

use dashmap::DashMap;

/// 全文索引管理器
/// 管理所有全文索引的生命周期
pub struct FulltextIndexManager {
    /// 索引集合: (space_id, tag_name, field_name) -> SearchEngine
    indexes: DashMap<IndexKey, Arc<dyn SearchEngine>>,
    /// 基础存储路径
    base_path: PathBuf,
    /// 默认引擎类型
    default_engine: EngineType,
}

type IndexKey = (u64, String, String);

impl FulltextIndexManager {
    pub fn new(base_path: PathBuf, default_engine: EngineType) -> Self {
        Self {
            indexes: DashMap::new(),
            base_path: base_path.join("fulltext"),
            default_engine,
        }
    }
    
    /// 创建全文索引
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String> {
        let index_name = format!("{}_{}_{}", space_id, tag_name, field_name);
        let engine_type = engine_type.unwrap_or(self.default_engine);
        
        // 创建引擎实例
        let engine = SearchEngineFactory::create(
            engine_type,
            &index_name,
            &self.base_path,
        )?;
        
        // 保存索引
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        self.indexes.insert(key, engine);
        
        Ok(index_name)
    }
    
    /// 获取索引
    pub fn get_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn SearchEngine>> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        self.indexes.get(&key).map(|e| Arc::clone(&*e))
    }
    
    /// 删除索引
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<()> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        
        if let Some((_, engine)) = self.indexes.remove(&key) {
            engine.close().await?;
        }
        
        // 删除索引文件
        let index_name = format!("{}_{}_{}", space_id, tag_name, field_name);
        let index_path = self.base_path.join(&index_name);
        if index_path.exists() {
            tokio::fs::remove_dir_all(&index_path).await?;
        }
        
        Ok(())
    }
    
    /// 数据变更时同步到全文索引
    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
        change_type: ChangeType,
    ) -> Result<()> {
        for (field_name, value) in properties {
            if let Some(engine) = self.get_index(space_id, tag_name, field_name) {
                match change_type {
                    ChangeType::Insert | ChangeType::Update => {
                        if let Value::String(text) = value {
                            engine.index(
                                &vertex_id.to_string(),
                                text,
                            ).await?;
                        }
                    }
                    ChangeType::Delete => {
                        engine.delete(&vertex_id.to_string()).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 提交所有索引的变更
    pub async fn commit_all(&self) -> Result<()> {
        for entry in self.indexes.iter() {
            entry.value().commit().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChangeType {
    Insert,
    Update,
    Delete,
}
```

## 数据流设计

### 1. 创建全文索引

```
用户
  │ CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25
  ▼
GraphDB 查询引擎
  │ 1. 解析 SQL
  │ 2. 调用 FulltextIndexManager.create_index()
  ▼
FulltextIndexManager
  │ 1. 创建索引目录 data/fulltext/idx_content/
  │ 2. 调用 SearchEngineFactory.create(Bm25)
  │ 3. 保存索引到内存映射
  ▼
Bm25Engine
  │ 初始化 Tantivy 索引
  ▼
返回成功
```

### 2. 插入数据同步

```
用户
  │ INSERT VERTEX Post(content) VALUES "图数据库文章"
  ▼
GraphDB 存储层
  │ 1. 写入 Redb (事务)
  │ 2. 提交事务成功
  ▼
FulltextIndexManager
  │ on_vertex_change(Insert)
  │ 检查 Post.content 是否有全文索引
  │ 是 → 调用 engine.index()
  ▼
Bm25Engine
  │ 添加文档到 Tantivy IndexWriter
  │ (不立即提交，等待批量提交)
  ▼
异步完成
```

### 3. 全文搜索

```
用户
  │ MATCH (p:Post) WHERE p.content MATCH "图数据库"
  ▼
GraphDB 查询引擎
  │ 1. 解析 MATCH 表达式
  │ 2. 识别全文搜索条件
  │ 3. 调用 FulltextIndexManager.search()
  ▼
FulltextIndexManager
  │ 获取对应索引 Bm25Engine
  │ 调用 engine.search("图数据库", limit)
  ▼
Bm25Engine
  │ Tantivy 搜索，返回 doc_ids
  ▼
GraphDB
  │ 根据 doc_ids 查询完整数据 (Redb)
  ▼
返回结果给用户
```

## 存储层集成

### 1. 扩展 StorageClient

```rust
// src/storage/fulltext_ext.rs

use crate::search::{FulltextIndexManager, SearchResult};

/// 全文检索扩展 Trait
#[async_trait]
pub trait FulltextStorageExt: StorageClient {
    /// 创建全文索引
    async fn create_fulltext_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String>;
    
    /// 删除全文索引
    async fn drop_fulltext_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<()>;
    
    /// 全文搜索
    async fn fulltext_search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    
    /// 重建全文索引
    async fn rebuild_fulltext_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<()>;
}

/// 为 RedbStorage 实现全文检索扩展
#[async_trait]
impl FulltextStorageExt for RedbStorage {
    async fn create_fulltext_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String> {
        self.fulltext_manager
            .create_index(space_id, tag_name, field_name, engine_type)
            .await
    }
    
    async fn fulltext_search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let engine = self.fulltext_manager
            .get_index(space_id, tag_name, field_name)
            .ok_or_else(|| StorageError::IndexNotFound(
                format!("{}.{}.{}", space_id, tag_name, field_name)
            ))?;
        
        engine.search(query, limit).await
    }
    
    // ... 其他实现
}
```

### 2. 数据变更钩子

```rust
// src/storage/redb_storage.rs

impl RedbStorage {
    /// 插入顶点后触发全文索引同步
    pub async fn insert_vertex(
        &mut self,
        space: &str,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        // 1. 写入 Redb
        let vertex_id = self.vertex_storage.insert(space, &vertex).await?;
        
        // 2. 同步到全文索引
        if let Some(ref manager) = self.fulltext_manager {
            let space_id = self.get_space_id(space)?;
            for tag in vertex.tags() {
                manager.on_vertex_change(
                    space_id,
                    &tag.name,
                    &vertex_id,
                    &tag.properties,
                    ChangeType::Insert,
                ).await?;
            }
        }
        
        Ok(vertex_id)
    }
}
```

## 配置设计

### GraphDB 配置 (config.toml)

```toml
[fulltext]
# 是否启用全文检索
enabled = true

# 默认搜索引擎: "bm25" | "inversearch"
default_engine = "bm25"

# 存储路径（相对于数据目录）
index_path = "fulltext"

# 同步策略
[fulltext.sync]
# 同步模式: "sync" | "async" | "off"
mode = "async"
# 异步队列大小
queue_size = 10000
# 批量提交间隔（毫秒）
commit_interval_ms = 1000
# 批量提交文档数
batch_size = 100

# BM25 引擎配置
[fulltext.bm25]
# 索引内存限制（MB）
memory_limit_mb = 50
# 自动提交策略
auto_commit = true

# Inversearch 引擎配置
[fulltext.inversearch]
# 分词模式: "strict" | "forward" | "reverse" | "full"
tokenize_mode = "strict"
# 索引分辨率
resolution = 9
# 缓存大小
cache_size = 1000
```

## SQL 语法扩展

### 创建全文索引

```sql
-- 使用默认引擎创建全文索引
CREATE FULLTEXT INDEX idx_post_content ON Post(content);

-- 指定 BM25 引擎
CREATE FULLTEXT INDEX idx_post_content ON Post(content) USING bm25;

-- 指定 Inversearch 引擎，使用 CJK 分词
CREATE FULLTEXT INDEX idx_post_content ON Post(content) 
USING inversearch 
WITH TOKENIZER = 'cjk';

-- 多字段全文索引
CREATE FULLTEXT INDEX idx_post ON Post(title, content) USING bm25;
```

### 全文搜索

```sql
-- 基本搜索
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p;

-- 带评分排序
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p, score(p) as relevance
ORDER BY relevance DESC
LIMIT 10;

-- 多字段搜索
MATCH (p:Post)
WHERE p.title MATCH "Rust" OR p.content MATCH "图数据库"
RETURN p;

-- 使用索引直接搜索
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *;

-- 搜索并高亮
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p.id, highlight(p.content) as highlighted;
```

### 索引管理

```sql
-- 查看全文索引列表
SHOW FULLTEXT INDEXES;

-- 查看索引状态
SHOW FULLTEXT INDEX STATUS idx_post_content;

-- 重建全文索引
REBUILD FULLTEXT INDEX idx_post_content;

-- 删除全文索引
DROP FULLTEXT INDEX idx_post_content;
```

## 错误处理与容错

### 1. 引擎故障隔离

```rust
// src/search/error_handler.rs

/// 搜索引擎错误处理器
pub struct SearchErrorHandler;

impl SearchErrorHandler {
    /// 处理搜索引擎错误，防止影响主流程
    pub async fn handle<T>(
        operation: impl Future<Output = Result<T>>,
        context: &str,
    ) -> Result<T> {
        match operation.await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("全文检索操作失败 [{}]: {:?}", context, e);
                
                // 根据错误类型决定处理方式
                match e {
                    SearchError::EngineUnavailable => {
                        // 引擎不可用，降级为禁用搜索
                        log::warn!("搜索引擎不可用，全文检索功能已降级");
                        Err(e)
                    }
                    SearchError::IndexCorrupted => {
                        // 索引损坏，尝试重建
                        log::error!("索引损坏，需要重建");
                        Err(e)
                    }
                    _ => Err(e),
                }
            }
        }
    }
}
```

### 2. 数据一致性策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| **最终一致性** | 异步同步，允许短暂不一致 | 默认推荐 |
| **近实时** | 定时批量提交（默认1秒） | 平衡性能和一致性 |
| **手动同步** | 提供重建索引命令 | 修复数据 |

```rust
// 异步同步实现
pub struct AsyncSyncManager {
    queue: Arc<Mutex<Vec<SyncTask>>>,
    commit_interval: Duration,
}

impl AsyncSyncManager {
    pub async fn start_background_commit(&self, manager: Arc<FulltextIndexManager>) {
        let queue = self.queue.clone();
        let interval = self.commit_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                // 批量处理队列中的任务
                let tasks = {
                    let mut q = queue.lock().await;
                    if q.is_empty() {
                        continue;
                    }
                    q.drain(..).collect::<Vec<_>>()
                };
                
                // 执行批量提交
                if let Err(e) = manager.commit_all().await {
                    log::error!("全文索引批量提交失败: {:?}", e);
                }
            }
        });
    }
}
```

## 性能预期

| 指标 | 预期值 | 说明 |
|------|--------|------|
| 单次搜索延迟 | 0.5-2ms | 内存调用，无网络开销 |
| 批量索引速度 | 5000-10000 doc/s | 异步批量提交 |
| 内存占用 | +100-300MB | 相比纯 GraphDB |
| 启动时间 | < 500ms | 引擎初始化 |
| 索引构建速度 | 1000-3000 doc/s | 初始全量索引 |

## 部署方案

### 单机部署（推荐）

```
┌─────────────────────────────────────┐
│           单机服务器                 │
│                                     │
│  ┌─────────────────────────────┐   │
│  │      GraphDB Server         │   │
│  │      (含全文检索)            │   │
│  │         :8080               │   │
│  └─────────────────────────────┘   │
│                                     │
│  数据目录:                          │
│  - data/graph.redb    (图数据)      │
│  - data/fulltext/     (全文索引)    │
│                                     │
└─────────────────────────────────────┘
```

### 启动方式

```powershell
# 标准启动（启用全文检索）
./graphdb-server.exe --config config.toml

# 禁用全文检索
./graphdb-server.exe --config config.toml --disable-fulltext
```

## 与 gRPC 方案对比

| 维度 | gRPC 方案 | 嵌入式方案（本方案） |
|------|-----------|----------------------|
| **部署复杂度** | 高（多进程管理） | 低（单进程） |
| **运维成本** | 高（多服务监控） | 低（单服务） |
| **性能** | 5-15ms 延迟 | 0.5-2ms 延迟 |
| **资源占用** | 多份内存 | 共享内存 |
| **单文件部署** | ❌ | ✅ |
| **事务协调** | 困难 | 可行 |
| **故障隔离** | ✅ 进程隔离 | ⚠️ 依赖代码隔离 |
| **独立升级** | ✅ | ⚠️ 需重新编译 |

## 总结

本方案的核心思想：

1. **保持轻量**：单进程架构，维持 GraphDB "单可执行文件"的设计目标
2. **高性能**：内存直接调用，避免网络开销
3. **可扩展**：通过 Trait 抽象，支持多种搜索引擎
4. **渐进集成**：先支持嵌入式，未来可扩展支持 gRPC 远程引擎

下一步：参考 `fulltext_embedded_implementation_plan.md` 开始具体实现。
