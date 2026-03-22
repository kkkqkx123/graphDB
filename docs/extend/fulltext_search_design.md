# 全文检索与BM25支持设计方案

## 概述

本文档描述 GraphDB 项目引入全文检索和 BM25 评分支持的详细设计方案。基于对 `ref/bm25` 和 `ref/inversearch` 参考项目的分析，结合当前项目架构，提供完整的集成方案。

---

## 一、参考项目分析

### 1.1 BM25 模块分析

**核心特性：**
- **算法**: 基于 Tantivy 搜索引擎库实现 BM25 (Best Matching 25) 算法
- **适用场景**: 长文本、文档类内容的全文语义搜索
- **主要功能**:
  - 文档索引与批量索引
  - 全文搜索（支持字段权重）
  - 文档删除与更新
  - 索引统计信息
- **依赖**: `tantivy = "0.24"`
- **存储**: 独立索引目录，支持持久化

**架构特点：**
```
BM25 Service
├── Tantivy 搜索引擎（核心）
├── Redis 缓存（可选）
├── gRPC 接口层
└── 配置管理
```

### 1.2 Inversearch 模块分析

**核心特性：**
- **算法**: 自定义倒排索引 + 多种分词策略
- **适用场景**: 短文本、关键词级别的精确匹配
- **主要功能**:
  - 多种分词模式（严格/正向/反向/双向/全匹配）
  - 字符集处理（CJK/Latin/规范化）
  - 上下文相关性搜索
  - 查询建议与自动补全
  - 结果高亮
- **依赖**: 纯 Rust 实现，无外部搜索引擎依赖
- **存储**: 基于自定义 Keystore 的内存/Redis 存储

**架构特点：**
```
Inversearch Service
├── 自定义倒排索引
├── 分词器（多种模式）
├── 字符集处理器
├── 查询解析器
├── 搜索协调器
└── 高亮处理器
```

### 1.3 对比分析

| 特性 | BM25 (Tantivy) | Inversearch (自定义) |
|------|----------------|---------------------|
| 算法成熟度 | 高（成熟搜索引擎） | 中（自定义实现） |
| 功能丰富度 | 高 | 中 |
| 依赖复杂度 | 高（依赖 Tantivy） | 低（纯 Rust） |
| 性能 | 高 | 中 |
| 定制化 | 中 | 高 |
| 适用场景 | 长文本搜索 | 短文本/关键词搜索 |

---

## 二、当前项目架构分析

### 2.1 存储层架构

```
src/storage/
├── mod.rs                    # 存储模块入口
├── redb_storage.rs          # Redb 存储引擎
├── storage_client.rs        # 存储客户端接口
├── index/                   # 索引管理
│   ├── mod.rs
│   ├── index_data_manager.rs    # 索引数据管理
│   ├── index_key_codec.rs       # 索引键编码
│   ├── vertex_index_manager.rs  # 顶点索引
│   └── edge_index_manager.rs    # 边索引
└── ...
```

### 2.2 索引类型定义

当前支持的索引类型（`src/core/types/index.rs`）：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum IndexType {
    #[serde(rename = "tag")]
    TagIndex,       // 标签索引
    #[serde(rename = "edge")]
    EdgeIndex,      // 边索引
}
```

### 2.3 索引扫描执行器

`IndexScanExecutor` 支持的扫描类型：
- `UNIQUE`: 唯一索引查找
- `PREFIX`: 前缀索引查找
- `RANGE`: 范围索引查找

---

## 三、设计方案

### 3.1 总体架构

采用**混合架构**，同时支持 Tantivy 引擎和内置倒排索引：

```
src/storage/
├── fulltext/                    # 新增全文检索模块
│   ├── mod.rs                   # 模块入口
│   ├── provider.rs              # 全文检索提供者 trait
│   ├── types.rs                 # 类型定义
│   ├── tantivy_impl/            # Tantivy 实现
│   │   ├── mod.rs
│   │   ├── index_manager.rs
│   │   ├── searcher.rs
│   │   └── schema.rs
│   └── builtin_impl/            # 内置实现
│       ├── mod.rs
│       ├── inverted_index.rs
│       ├── tokenizer.rs
│       └── searcher.rs
└── ...
```

### 3.2 核心类型设计

#### 3.2.1 扩展索引类型

```rust
// src/core/types/index.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum IndexType {
    #[serde(rename = "tag")]
    TagIndex,
    #[serde(rename = "edge")]
    EdgeIndex,
    #[serde(rename = "fulltext")]      // 新增
    FulltextIndex,
}
```

#### 3.2.2 全文检索配置

```rust
// src/storage/fulltext/types.rs

/// 全文索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextIndexConfig {
    pub index_name: String,
    pub space_id: u64,
    pub schema_name: String,
    pub field_name: String,
    pub provider: FulltextProviderType,
    pub tokenizer: TokenizerType,
    pub options: FulltextOptions,
}

/// 全文检索提供者类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FulltextProviderType {
    Tantivy,        // Tantivy 引擎
    Builtin,        // 内置实现
}

/// 分词器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenizerType {
    Standard,       // 标准分词
    Cjk,           // CJK 字符分词
    Whitespace,    // 空格分词
    Raw,           // 不分词
}

/// 全文索引选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextOptions {
    pub store_doc: bool,            // 是否存储文档内容
    pub store_positions: bool,      // 是否存储词位置
    pub bm25_k1: f32,              // BM25 k1 参数
    pub bm25_b: f32,               // BM25 b 参数
}

impl Default for FulltextOptions {
    fn default() -> Self {
        Self {
            store_doc: true,
            store_positions: true,
            bm25_k1: 1.2,
            bm25_b: 0.75,
        }
    }
}
```

#### 3.2.3 全文检索提供者 Trait

```rust
// src/storage/fulltext/provider.rs

use crate::core::{Value, Result};
use crate::storage::fulltext::types::*;

/// 全文检索提供者接口
#[async_trait::async_trait]
pub trait FulltextProvider: Send + Sync {
    /// 创建索引
    async fn create_index(&self, config: FulltextIndexConfig) -> Result<()>;
    
    /// 删除索引
    async fn drop_index(&self, index_name: &str) -> Result<()>;
    
    /// 索引文档
    async fn index_document(
        &self,
        index_name: &str,
        doc_id: &Value,
        content: &str,
    ) -> Result<()>;
    
    /// 批量索引文档
    async fn batch_index_documents(
        &self,
        index_name: &str,
        documents: Vec<(Value, String)>,
    ) -> Result<usize>;
    
    /// 删除文档
    async fn delete_document(&self, index_name: &str, doc_id: &Value) -> Result<()>;
    
    /// 搜索
    async fn search(
        &self,
        index_name: &str,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults>;
    
    /// 获取索引统计信息
    async fn get_stats(&self, index_name: &str) -> Result<IndexStats>;
}

/// 搜索选项
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub highlight: bool,
    pub field_weights: Option<Vec<(String, f32)>>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            offset: 0,
            highlight: false,
            field_weights: None,
        }
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub total: usize,
    pub results: Vec<SearchResult>,
    pub query_time_ms: u64,
}

/// 单个搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub doc_id: Value,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// 索引统计信息
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub doc_count: usize,
    pub term_count: usize,
    pub avg_doc_length: f64,
}
```

### 3.3 Tantivy 实现方案

#### 3.3.1 索引管理器

```rust
// src/storage/fulltext/tantivy_impl/index_manager.rs

use tantivy::{
    schema::*,
    Index, IndexWriter, IndexReader, ReloadPolicy,
    query::{QueryParser, BooleanQuery},
    collector::TopDocs,
    TantivyDocument,
};
use std::path::PathBuf;
use std::collections::HashMap;

pub struct TantivyIndexManager {
    index_path: PathBuf,
    indexes: HashMap<String, Index>,
    writers: HashMap<String, IndexWriter>,
}

impl TantivyIndexManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            index_path: base_path,
            indexes: HashMap::new(),
            writers: HashMap::new(),
        }
    }
    
    /// 创建新索引
    pub fn create_index(&mut self, name: &str) -> Result<()> {
        let path = self.index_path.join(name);
        
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("doc_id", STRING | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        
        let schema = schema_builder.build();
        let index = Index::create_in_dir(&path, schema)?;
        
        self.indexes.insert(name.to_string(), index);
        Ok(())
    }
    
    /// 打开已有索引
    pub fn open_index(&mut self, name: &str) -> Result<()> {
        let path = self.index_path.join(name);
        let index = Index::open_in_dir(&path)?;
        self.indexes.insert(name.to_string(), index);
        Ok(())
    }
    
    /// 获取索引写入器
    pub fn get_writer(&mut self, name: &str) -> Result<&mut IndexWriter> {
        if !self.writers.contains_key(name) {
            let index = self.indexes.get(name)
                .ok_or_else(|| StorageError::IndexNotFound(name.to_string()))?;
            let writer = index.writer(50_000_000)?;
            self.writers.insert(name.to_string(), writer);
        }
        Ok(self.writers.get_mut(name).unwrap())
    }
    
    /// 获取索引读取器
    pub fn get_reader(&self, name: &str) -> Result<IndexReader> {
        let index = self.indexes.get(name)
            .ok_or_else(|| StorageError::IndexNotFound(name.to_string()))?;
        Ok(index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?)
    }
}
```

#### 3.3.2 搜索实现

```rust
// src/storage/fulltext/tantivy_impl/searcher.rs

use tantivy::{
    query::{QueryParser, BooleanQuery, TermQuery, Occur},
    collector::TopDocs,
    Term,
    schema::IndexRecordOption,
};

pub struct TantivySearcher {
    index_manager: Arc<Mutex<TantivyIndexManager>>,
}

impl TantivySearcher {
    pub fn search(
        &self,
        index_name: &str,
        query_text: &str,
        options: &SearchOptions,
    ) -> Result<SearchResults> {
        let reader = self.index_manager.lock().get_reader(index_name)?;
        let searcher = reader.searcher();
        let schema = searcher.schema();
        
        // 构建查询
        let query = self.build_query(query_text, &schema)?;
        
        // 执行搜索
        let limit = options.limit + options.offset;
        let top_docs = TopDocs::with_limit(limit);
        let results = searcher.search(&query, &top_docs)?;
        
        // 处理结果
        let mut search_results = Vec::new();
        for (score, doc_address) in results.into_iter().skip(options.offset) {
            let doc = searcher.doc(doc_address)?;
            let doc_id = self.extract_doc_id(&doc, &schema);
            
            search_results.push(SearchResult {
                doc_id: Value::String(doc_id),
                score,
                highlights: None,
            });
        }
        
        Ok(SearchResults {
            total: search_results.len(),
            results: search_results,
            query_time_ms: 0,
        })
    }
    
    fn build_query(
        &self,
        query_text: &str,
        schema: &Schema,
    ) -> Result<Box<dyn tantivy::query::Query>> {
        let terms: Vec<&str> = query_text.split_whitespace().collect();
        
        if terms.is_empty() {
            return Ok(Box::new(tantivy::query::EmptyQuery {}));
        }
        
        let content_field = schema.get_field("content")
            .ok_or_else(|| StorageError::FieldNotFound("content".to_string()))?;
        
        let mut clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
        
        for term in &terms {
            let term_text = term.to_lowercase();
            let term = Term::from_field_text(content_field, &term_text);
            let term_query: Box<dyn tantivy::query::Query> = Box::new(
                TermQuery::new(term, IndexRecordOption::WithFreqsAndPositions)
            );
            clauses.push((Occur::Should, term_query));
        }
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }
}
```

### 3.4 内置实现方案

#### 3.4.1 倒排索引结构

```rust
// src/storage/fulltext/builtin_impl/inverted_index.rs

use std::collections::{HashMap, BTreeMap};

/// 文档ID类型
pub type DocId = u64;

/// 词项位置
#[derive(Debug, Clone, Copy)]
pub struct TermPosition {
    pub doc_id: DocId,
    pub position: usize,
}

/// 倒排索引
pub struct InvertedIndex {
    /// 词项 -> 文档列表映射
    term_docs: HashMap<String, Vec<DocId>>,
    /// 词项 -> 位置信息映射
    term_positions: HashMap<String, Vec<TermPosition>>,
    /// 文档频率统计
    doc_freq: HashMap<String, usize>,
    /// 文档长度统计
    doc_lengths: HashMap<DocId, usize>,
    /// 总文档数
    total_docs: usize,
    /// 平均文档长度
    avg_doc_length: f64,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            term_docs: HashMap::new(),
            term_positions: HashMap::new(),
            doc_freq: HashMap::new(),
            doc_lengths: HashMap::new(),
            total_docs: 0,
            avg_doc_length: 0.0,
        }
    }
    
    /// 添加文档到索引
    pub fn add_document(&mut self, doc_id: DocId, terms: Vec<String>) {
        let doc_length = terms.len();
        self.doc_lengths.insert(doc_id, doc_length);
        
        for (pos, term) in terms.iter().enumerate() {
            // 更新文档列表
            self.term_docs
                .entry(term.clone())
                .or_insert_with(Vec::new)
                .push(doc_id);
            
            // 更新位置信息
            self.term_positions
                .entry(term.clone())
                .or_insert_with(Vec::new)
                .push(TermPosition { doc_id, position: pos });
        }
        
        self.total_docs += 1;
        self.update_avg_doc_length();
    }
    
    /// 计算 BM25 分数
    pub fn calculate_bm25(
        &self,
        term: &str,
        doc_id: DocId,
        k1: f32,
        b: f32,
    ) -> f32 {
        let tf = self.term_frequency(term, doc_id) as f32;
        let df = *self.doc_freq.get(term).unwrap_or(&0) as f32;
        let doc_len = *self.doc_lengths.get(&doc_id).unwrap_or(&0) as f32;
        
        // IDF 计算
        let idf = ((self.total_docs as f32 - df + 0.5) / (df + 0.5) + 1.0).ln();
        
        // TF 归一化
        let normalized_tf = (tf * (k1 + 1.0)) 
            / (tf + k1 * (1.0 - b + b * (doc_len / self.avg_doc_length as f32)));
        
        idf * normalized_tf
    }
    
    fn term_frequency(&self, term: &str, doc_id: DocId) -> usize {
        self.term_positions
            .get(term)
            .map(|positions| {
                positions.iter().filter(|p| p.doc_id == doc_id).count()
            })
            .unwrap_or(0)
    }
    
    fn update_avg_doc_length(&mut self) {
        if self.total_docs > 0 {
            let total_length: usize = self.doc_lengths.values().sum();
            self.avg_doc_length = total_length as f64 / self.total_docs as f64;
        }
    }
}
```

#### 3.4.2 分词器

```rust
// src/storage/fulltext/builtin_impl/tokenizer.rs

/// 分词器 trait
pub trait Tokenizer: Send + Sync {
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// 标准分词器（按空格和标点分词）
pub struct StandardTokenizer;

impl Tokenizer for StandardTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .flat_map(|word| {
                word.split(|c: char| !c.is_alphanumeric())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_lowercase())
            })
            .collect()
    }
}

/// CJK 分词器（支持中文、日文、韩文）
pub struct CjkTokenizer;

impl Tokenizer for CjkTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_word = String::new();
        
        for ch in text.chars() {
            if ch.is_ascii() {
                if ch.is_alphanumeric() {
                    current_word.push(ch);
                } else if !current_word.is_empty() {
                    tokens.push(current_word.to_lowercase());
                    current_word.clear();
                }
            } else if is_cjk(ch) {
                if !current_word.is_empty() {
                    tokens.push(current_word.to_lowercase());
                    current_word.clear();
                }
                tokens.push(ch.to_string());
            }
        }
        
        if !current_word.is_empty() {
            tokens.push(current_word.to_lowercase());
        }
        
        tokens
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF |    // CJK 统一表意文字
        0x3040..=0x309F |    // 平假名
        0x30A0..=0x30FF |    // 片假名
        0xAC00..=0xD7AF      // 韩文音节
    )
}
```

### 3.5 查询层集成

#### 3.5.1 全文扫描执行器

```rust
// src/query/executor/data_access/fulltext_scan.rs

use crate::query::executor::base::{BaseExecutor, Executor, DBResult};
use crate::storage::fulltext::{FulltextProvider, SearchOptions};

pub struct FulltextScanExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_id: u64,
    index_name: String,
    query: String,
    options: SearchOptions,
}

impl<S: StorageClient> FulltextScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_id: u64,
        index_name: String,
        query: String,
        limit: Option<usize>,
    ) -> Self {
        let options = SearchOptions {
            limit: limit.unwrap_or(100),
            ..Default::default()
        };
        
        Self {
            base: BaseExecutor::new(id, "FulltextScanExecutor".to_string(), storage),
            space_id,
            index_name,
            query,
            options,
        }
    }
}

#[async_trait::async_trait]
impl<S: StorageClient + Send + 'static> Executor for FulltextScanExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.base.storage.lock();
        
        // 执行全文搜索
        let results = storage
            .fulltext_search(&self.index_name, &self.query, &self.options)
            .await
            .map_err(|e| DBError::Storage(e))?;
        
        // 转换为执行结果
        let rows: Vec<Row> = results
            .results
            .into_iter()
            .map(|r| {
                let mut columns = Vec::new();
                columns.push(("doc_id".to_string(), r.doc_id));
                columns.push(("score".to_string(), Value::Float(r.score as f64)));
                Row::new(columns)
            })
            .collect();
        
        Ok(ExecutionResult::new(rows))
    }
}
```

#### 3.5.2 查询语法扩展

支持以下全文检索语法：

```sql
-- 基本全文搜索
MATCH (v:Post)
WHERE v.content CONTAINS "关键词"
RETURN v

-- 使用全文索引
LOOKUP ON fulltext_index WHERE QUERY("搜索文本")
RETURN *

-- 带评分的搜索
MATCH (v:Article)
WHERE v.title MATCH "BM25 算法"
RETURN v, score(v) as relevance
ORDER BY relevance DESC
```

---

## 四、集成步骤

### 第一阶段：基础框架（2-3 天）

1. **添加依赖**
   ```toml
   [dependencies]
   tantivy = { version = "0.24", optional = true }
   
   [features]
   fulltext-tantivy = ["dep:tantivy"]
   fulltext-builtin = []
   ```

2. **定义类型和 Trait**
   - 扩展 `IndexType` 枚举
   - 定义 `FulltextProvider` trait
   - 定义配置和选项类型

3. **创建模块结构**
   - 创建 `src/storage/fulltext/` 目录
   - 实现模块入口和类型定义

### 第二阶段：Tantivy 实现（5-7 天）

1. **索引管理**
   - 实现 `TantivyIndexManager`
   - 实现索引创建、打开、删除
   - 实现文档添加、删除

2. **搜索功能**
   - 实现查询构建
   - 实现搜索执行
   - 实现结果格式化

3. **集成到存储层**
   - 在 `StorageClient` 中添加全文检索接口
   - 实现与 Redb 存储的协调

### 第三阶段：查询层集成（3-4 天）

1. **执行器实现**
   - 实现 `FulltextScanExecutor`
   - 实现查询计划生成

2. **语法扩展**
   - 扩展查询解析器支持 `MATCH`、`CONTAINS`
   - 实现评分函数

3. **测试用例**
   - 单元测试
   - 集成测试

### 第四阶段：内置实现（可选，5-7 天）

1. **倒排索引**
   - 实现内存倒排索引
   - 实现持久化存储

2. **分词器**
   - 实现多种分词策略
   - 支持 CJK 字符

3. **BM25 评分**
   - 实现 BM25 算法
   - 支持参数调优

---

## 五、性能考虑

### 5.1 索引性能

| 操作 | Tantivy | 内置实现 |
|------|---------|----------|
| 索引速度 | 高（批量优化） | 中 |
| 查询速度 | 高 | 中 |
| 内存占用 | 中 | 高（全内存） |
| 磁盘占用 | 高 | 低 |

### 5.2 优化建议

1. **批量索引**：使用批量接口减少 IO 次数
2. **索引预热**：启动时预加载热数据
3. **查询缓存**：缓存常见查询结果
4. **异步处理**：索引更新异步执行

---

## 六、风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Tantivy 依赖问题 | 低 | 高 | 提供内置实现作为备选 |
| 性能不达标 | 中 | 中 | 充分测试，优化索引策略 |
| 存储空间膨胀 | 中 | 中 | 定期合并索引，清理过期数据 |
| 并发访问冲突 | 中 | 高 | 使用读写锁，控制并发度 |

---

## 七、总结

本设计方案提供了两种全文检索实现路径：

1. **Tantivy 方案**：功能完善、性能优秀，适合生产环境
2. **内置方案**：轻量级、无依赖，适合嵌入式场景

建议优先实现 Tantivy 方案，作为默认全文检索引擎。内置方案作为可选功能，在需要最小依赖的场景下使用。

**下一步行动**：
1. 评审设计方案
2. 确定实现优先级
3. 开始第一阶段开发
