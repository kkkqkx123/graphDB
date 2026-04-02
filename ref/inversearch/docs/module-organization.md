# Inversearch 模块组织架构设计

## 一、当前架构分析

### 现有模块结构

```
src/
├── lib.rs                    # 主入口，模块导出
├── index/                    # 单索引实现
│   ├── mod.rs
│   ├── builder.rs
│   └── remover.rs
├── search/                    # 搜索功能
│   ├── mod.rs               # 主搜索逻辑
│   ├── single_term.rs       # 单术语搜索
│   └── cache.rs             # 搜索缓存
├── resolver/                 # 结果处理
│   ├── mod.rs
│   ├── resolver.rs
│   ├── handler.rs
│   ├── enrich.rs
│   ├── and.rs / or.rs / not.rs / xor.rs
│   └── async_resolver.rs
├── storage/                  # 持久化存储
│   └── mod.rs
├── highlight/                # 高亮功能
│   ├── mod.rs
│   ├── core.rs
│   ├── processor.rs
│   └── tests.rs
├── encoder/                  # 编码器
│   ├── mod.rs
│   ├── validator.rs
│   └── transform.rs
├── charset/                  # 字符集处理
│   ├── mod.rs
│   ├── latin/
│   ├── cjk.rs
│   ├── normalize.rs
│   └── exact.rs
├── tokenizer/                 # 分词器
│   └── mod.rs
├── keystore/                 # 键值存储
│   └── mod.rs
├── intersect/                # 交集计算
│   ├── mod.rs
│   ├── core.rs
│   └── scoring.rs
├── common/                   # 通用工具
│   └── mod.rs
├── config/                    # 配置
│   └── mod.rs
├── type/                     # 类型定义
│   └── mod.rs
└── async_.rs                 # 异步支持
```

### 问题诊断

1. **resolver/mod.rs** - 职责过多
   - 同时包含集合运算、丰富化、异步支持
   - 应该拆分

2. **search/mod.rs** - 缺少多字段搜索
   - 只支持单字段搜索
   - 缺少搜索协调器

3. **缺少 Document 抽象层**
   - 无法管理多字段索引
   - 无法实现跨字段搜索

4. **common/mod.rs** - 缺少树形解析
   - 已有 `parse_simple`
   - 缺少 `parse_tree`

---

## 二、推荐模块架构

### 2.1 总体架构图

```
src/
├── lib.rs                    # 主入口（仅导出）
│
├── document/                 # 多字段文档抽象（新增）
│   ├── mod.rs               # Document 主结构
│   ├── field.rs             # 字段定义
│   ├── tree.rs              # 树形结构解析（新增）
│   ├── tag.rs               # 标签系统（新增）
│   └── batch.rs             # 批量操作（新增）
│
├── index/                    # 单索引（保持）
│   ├── mod.rs
│   ├── builder.rs
│   └── remover.rs
│
├── search/                   # 搜索功能（重构）
│   ├── mod.rs               # 主搜索接口
│   ├── coordinator.rs       # 搜索协调器（新增）
│   ├── multi_field.rs       # 多字段搜索（新增）
│   ├── single_term.rs       # 单术语（保持）
│   └── cache.rs             # 缓存（保持）
│
├── resolver/                 # 结果处理（重构）
│   ├── mod.rs               # 仅导出
│   ├── resolver.rs          # 核心解析器
│   └── ops/                  # 集合运算子模块
│       ├── mod.rs
│       ├── and.rs
│       ├── or.rs
│       ├── not.rs
│       └── xor.rs
│
├── enrich/                   # 结果丰富化（新增子模块）
│   ├── mod.rs
│   ├── basic.rs             # 基本丰富化
│   ├── highlight.rs         # 高亮丰富化
│   └── document.rs          # 文档丰富化
│
├── storage/                  # 持久化存储（重构）
│   ├── mod.rs               # 主接口
│   ├── memory.rs            # 内存存储
│   ├── redis.rs             # Redis 存储
│   └── interface.rs          # 存储接口
│
├── common/                   # 通用工具（扩展）
│   ├── mod.rs               # 导出入口
│   ├── parse.rs             # 字段解析
│   ├── tree.rs              # 树形解析（新增）
│   └── mod.rs
│
├── cache/                    # 缓存层（新增）
│   ├── mod.rs
│   ├── search.rs            # 搜索缓存
│   ├── encoder.rs           # 编码缓存
│   └── document.rs          # 文档缓存
│
├── highlight/                # 高亮功能（保持）
│   ├── mod.rs
│   ├── core.rs
│   ├── processor.rs
│   └── tests.rs
│
├── encoder/                  # 编码器（保持）
│   ├── mod.rs
│   ├── validator.rs
│   └── transform.rs
│
├── charset/                  # 字符集处理（保持）
│   ├── mod.rs
│   ├── latin/
│   ├── cjk.rs
│   ├── normalize.rs
│   └── exact.rs
│
├── tokenizer/                # 分词器（保持）
│   └── mod.rs
│
├── intersect/                # 交集计算（保持）
│   ├── mod.rs
│   ├── core.rs
│   └── scoring.rs
│
├── keystore/                 # 键值存储（保持）
│   └── mod.rs
│
├── config/                    # 配置（保持）
│   └── mod.rs
│
├── type/                     # 类型定义（保持）
│   └── mod.rs
│
├── async_.rs                 # 异步支持（保持）
└── error.rs                  # 错误处理（保持）
```

---

## 三、新模块详细设计

### 3.1 document/mod.rs - 文档模块

**职责**: 统一管理多字段索引和文档操作

```rust
//! Document 模块
//!
//! 提供多字段文档索引的统一管理
//!
//! # 模块结构
//!
//! - `mod.rs`: Document 主结构和公共接口
//! - `field.rs`: 字段定义和配置
//! - `tree.rs`: 树形结构解析
//! - `tag.rs`: 标签系统
//! - `batch.rs`: 批量操作

mod field;
mod tree;
mod tag;
mod batch;

pub use field::{Field, FieldConfig, FieldType};
pub use tree::{parse_tree, TreePath};
pub use tag::{TagSystem, TagConfig};
pub use batch::{Batch, BatchOperation};

use crate::{
    Index, IndexOptions, SearchOptions,
    DocId, SearchResult,
    storage::Storage,
    enrich::DocumentEnricher,
};
use serde_json::Value;
use std::collections::HashMap;

/// 文档搜索引擎主结构
pub struct Document {
    /// 字段配置列表
    fields: Vec<Field>,
    /// 字段名到索引的映射
    indexes: HashMap<String, Index>,
    /// 标签系统
    tag_system: Option<TagSystem>,
    /// 文档存储
    store: Option<Storage>,
    /// 文档丰富化器
    enricher: Option<DocumentEnricher>,
    /// 注册表（文档ID集合）
    reg: Register,
}

/// 注册表类型
enum Register {
    Set(crate::keystore::KeystoreSet<DocId>),
    Map(crate::keystore::KeystoreMap<DocId, ()>),
}

impl Document {
    /// 创建新的 Document 实例
    pub fn new(config: DocumentConfig) -> Result<Self> {
        // 实现...
        unimplemented!()
    }

    /// 添加文档
    pub fn add(&mut self, id: DocId, content: &Value) -> Result<()> {
        // 解析所有字段
        for field in &self.fields {
            let value = field.extract_value(content)?;
            field.index.add(id, &value, false)?;
        }
        Ok(())
    }

    /// 更新文档
    pub fn update(&mut self, id: DocId, content: &Value) -> Result<()> {
        self.remove(id)?;
        self.add(id, content)?;
        Ok(())
    }

    /// 删除文档
    pub fn remove(&mut self, id: DocId) -> Result<()> {
        for index in self.indexes.values() {
            index.remove(id, false)?;
        }
        Ok(())
    }

    /// 搜索
    pub fn search(&self, options: &SearchOptions) -> Result<SearchResult> {
        // 使用搜索协调器
        unimplemented!()
    }

    /// 获取文档
    pub fn get(&self, id: DocId) -> Option<&Value> {
        self.store.as_ref()?.get(&id.to_string())
    }

    /// 检查文档是否存在
    pub fn contains(&self, id: DocId) -> bool {
        match &self.reg {
            Register::Set(set) => set.has(&id),
            Register::Map(map) => map.has(&id),
        }
    }

    /// 清空所有索引
    pub fn clear(&mut self) {
        for index in self.indexes.values() {
            index.clear();
        }
        if let Some(tag_system) = &mut self.tag_system {
            tag_system.clear();
        }
    }
}

/// 文档配置
pub struct DocumentConfig {
    pub fields: Vec<FieldConfig>,
    pub tags: Vec<TagConfig>,
    pub store: bool,
    pub fastupdate: bool,
    pub cache: Option<usize>,
}
```

### 3.2 document/tree.rs - 树形结构解析

**职责**: 解析嵌套字段路径

```rust
//! 树形结构解析
//!
//! 解析嵌套字段路径，支持数组索引和属性访问
//!
//! # 支持的语法
//!
//! ```rust
//! use inversearch::parse_tree;
//!
//! // 嵌套属性
//! parse_tree("user.name", &mut vec![]);
//!
//! // 数组索引
//! parse_tree("items[0].title", &mut vec![]);
//!
//! // 倒数索引
//! parse_tree("items[-1].name", &mut vec![]);
//!
//! // 范围索引
//! parse_tree("items[0-2].title", &mut vec![]);
//! ```

use serde_json::Value;

/// 树形路径项
#[derive(Debug, Clone)]
pub enum TreePath {
    /// 普通字段
    Field(String),
    /// 数组索引
    Index(usize, String),
    /// 负数索引（倒数）
    NegativeIndex(usize, String),
    /// 范围索引
    Range(usize, usize, String),
}

/// 解析树形路径
///
/// # 示例
///
/// ```
/// use inversearch::parse_tree;
///
/// let mut marker = vec![];
/// let result = parse_tree("user.name", &mut marker);
/// assert_eq!(result, vec!["user", "name"]);
/// ```
pub fn parse_tree(key: &str, marker: &mut Vec<bool>) -> Vec<TreePath> {
    let parts: Vec<&str> = key.split(':').collect();
    let mut result = Vec::new();
    let mut count = 0;

    for part in parts {
        let mut field = part.to_string();
        
        // 检查是否是数组索引语法
        if let Some(start) = field.rfind('[') {
            let end = field.len() - 1;
            let index_part = &field[start+1..end];
            field = field[..start].to_string();
            
            if !field.is_empty() {
                marker.push(true);
            }
            
            // 解析索引
            if index_part.contains('-') && !index_part.starts_with('-') {
                // 范围索引 [0-2]
                let range_parts: Vec<&str> = index_part.split('-').collect();
                let start_idx: usize = range_parts[0].parse().unwrap();
                let end_idx: usize = range_parts[1].parse().unwrap();
                result.push(TreePath::Range(start_idx, end_idx, field));
            } else if index_part.starts_with('-') {
                // 负数索引 [-1]
                let idx: usize = index_part[1..].parse().unwrap();
                result.push(TreePath::NegativeIndex(idx, field));
            } else {
                // 正数索引 [0]
                let idx: usize = index_part.parse().unwrap();
                result.push(TreePath::Index(idx, field));
            }
        } else {
            result.push(TreePath::Field(field));
        }
    }
    
    result
}

/// 从嵌套结构中提取值
pub fn extract_value<'a>(document: &'a Value, path: &[TreePath]) -> Option<&'a Value> {
    let mut current = document;
    
    for segment in path {
        current = match segment {
            TreePath::Field(name) => {
                current.get(name)?
            }
            TreePath::Index(idx, _) => {
                current.as_array()?.get(*idx)?
            }
            TreePath::NegativeIndex(idx, _) => {
                let arr = current.as_array()?;
                arr.len().checked_sub(*idx + 1)?.let(|i| arr.get(i))
            }
            TreePath::Range(start, end, _) => {
                unimplemented!("Range extraction returns multiple values")
            }
        };
    }
    
    Some(current)
}
```

### 3.3 document/tag.rs - 标签系统

**职责**: 支持文档标签和基于标签的过滤

```rust
//! 标签系统
//!
//! 为文档添加标签，支持基于标签的过滤和搜索
//!
//! # 示例
//!
//! ```rust
//! use inversearch::{DocId, TagSystem, TagConfig};
//!
//! let mut tag_system = TagSystem::new();
//! tag_system.add_config(TagConfig {
//!     field: "category".to_string(),
//!     filter: None,
//! });
//!
//! // 添加标签
//! tag_system.add_tags(1, &[("category", &json!("tech"))]);
//!
//! // 按标签查询
//! let ids = tag_system.query("category", "tech");
//! ```

use serde_json::Value;
use crate::DocId;

/// 标签配置
#[derive(Debug, Clone)]
pub struct TagConfig {
    pub field: String,
    pub filter: Option<Box<dyn Fn(&Value) -> bool + Send + Sync>>,
}

/// 标签系统
pub struct TagSystem {
    /// 标签字段配置
    configs: Vec<TagConfig>,
    /// 标签索引: field -> tag -> doc_ids
    indexes: Vec<HashMap<String, Vec<DocId>>>,
    /// 标签树路径
    trees: Vec<Vec<crate::document::TreePath>>,
}

impl TagSystem {
    /// 创建新的标签系统
    pub fn new() -> Self {
        TagSystem {
            configs: Vec::new(),
            indexes: Vec::new(),
            trees: Vec::new(),
        }
    }

    /// 添加标签配置
    pub fn add_config(&mut self, config: TagConfig) {
        let field = config.field.clone();
        self.configs.push(config);
        self.indexes.push(HashMap::new());
        self.trees.push(crate::document::parse_tree(&field, &mut vec![]));
    }

    /// 为文档添加标签
    pub fn add_tags(&mut self, doc_id: DocId, tags: &[(&str, &Value)]) {
        for (i, tag_data) in tags.iter().enumerate() {
            let (field, value) = tag_data;
            let tag_str = value.as_str().unwrap_or("");
            
            if let Some(index) = self.indexes.get_mut(i) {
                let ids = index.entry(tag_str.to_string()).or_default();
                if !ids.contains(&doc_id) {
                    ids.push(doc_id);
                }
            }
        }
    }

    /// 移除文档的标签
    pub fn remove_tags(&mut self, doc_id: DocId) {
        for index in &mut self.indexes {
            for ids in index.values_mut() {
                if let Some(pos) = ids.iter().position(|&id| id == doc_id) {
                    ids.swap_remove(pos);
                }
            }
        }
    }

    /// 按标签查询文档
    pub fn query(&self, field: &str, tag: &str) -> Option<&Vec<DocId>> {
        let idx = self.configs.iter()
            .position(|c| c.field == field)?;
        self.indexes[idx].get(tag)
    }

    /// 按多个标签查询（交集）
    pub fn query_multi(&self, field: &str, tags: &[&str]) -> Vec<DocId> {
        let idx = self.configs.iter()
            .position(|c| c.field == field)?;
        
        let mut result = None;
        for tag in tags {
            if let Some(ids) = self.indexes[idx].get(*tag) {
                if let Some(ref mut combined) = result {
                    // 计算交集
                    let set: std::collections::HashSet<&DocId> = combined.iter().collect();
                    *combined = ids.iter()
                        .filter(|id| set.contains(id))
                        .copied()
                        .collect();
                } else {
                    result = Some(ids.clone());
                }
            }
        }
        
        result.unwrap_or_default()
    }

    /// 清空所有标签
    pub fn clear(&mut self) {
        for index in &mut self.indexes {
            index.clear();
        }
    }
}
```

### 3.4 document/batch.rs - 批量操作

**职责**: 高效的批量文档操作

```rust
//! 批量操作
//!
//! 提供高效的批量文档添加、更新、删除操作
//!
//! # 使用示例
//!
//! ```rust
//! use inversearch::{Document, Batch};
//!
//! let mut batch = Batch::new(1000); // 批量大小 1000
//!
//! // 添加操作
//! batch.add(1, &json!({"title": "Doc 1"}));
//! batch.add(2, &json!({"title": "Doc 2"}));
//!
// // 执行批量操作
//! document.execute_batch(&mut batch)?;
//! ```

use serde_json::Value;
use crate::DocId;

/// 批量操作类型
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Add(DocId, Value),
    Update(DocId, Value),
    Remove(DocId),
}

/// 批量操作缓冲
pub struct Batch {
    operations: Vec<BatchOperation>,
    max_size: usize,
    current_size: usize,
}

impl Batch {
    /// 创建新的批量操作
    pub fn new(max_size: usize) -> Self {
        Batch {
            operations: Vec::with_capacity(max_size),
            max_size,
            current_size: 0,
        }
    }

    /// 添加文档
    pub fn add(&mut self, id: DocId, content: &Value) {
        self.operations.push(BatchOperation::Add(id, content.clone()));
        self.current_size += 1;
    }

    /// 更新文档
    pub fn update(&mut self, id: DocId, content: &Value) {
        self.operations.push(BatchOperation::Update(id, content.clone()));
        self.current_size += 1;
    }

    /// 删除文档
    pub fn remove(&mut self, id: DocId) {
        self.operations.push(BatchOperation::Remove(id));
        self.current_size += 1;
    }

    /// 检查是否需要刷新
    pub fn should_flush(&self) -> bool {
        self.current_size >= self.max_size
    }

    /// 获取操作数量
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// 清空操作队列
    pub fn clear(&mut self) {
        self.operations.clear();
        self.current_size = 0;
    }

    /// 取出所有操作
    pub fn drain(&mut self) -> Vec<BatchOperation> {
        self.current_size = 0;
        self.operations.drain(..).collect()
    }
}
```

### 3.5 search/coordinator.rs - 搜索协调器

**职责**: 协调多字段搜索

```rust
//! 搜索协调器
//!
//! 协调多字段搜索，管理字段权重和结果合并
//!
//! # 示例
//!
//! ```rust
//! use inversearch::{Document, SearchCoordinator, MultiFieldSearchOptions};
//!
//! let coordinator = SearchCoordinator::new();
//! coordinator.add_field("title", 2.0);  // title 权重 2.0
//! coordinator.add_field("content", 1.0); // content 权重 1.0
//!
//! let result = coordinator.search("rust programming")?;
//! ```

use crate::{SearchResult, SearchOptions, DocId, Document};
use std::collections::HashMap;

/// 字段搜索配置
struct FieldSearch {
    name: String,
    weight: f32,
    query: Option<String>,
}

/// 多字段搜索协调器
pub struct SearchCoordinator {
    document: Document,
    fields: Vec<FieldSearch>,
    boost: HashMap<String, f32>,
}

impl SearchCoordinator {
    /// 创建新的搜索协调器
    pub fn new(document: Document) -> Self {
        SearchCoordinator {
            document,
            fields: Vec::new(),
            boost: HashMap::new(),
        }
    }

    /// 添加搜索字段
    pub fn add_field(&mut self, name: &str, weight: f32) {
        self.fields.push(FieldSearch {
            name: name.to_string(),
            weight,
            query: None,
        });
    }

    /// 设置字段的搜索查询（用于不同字段不同查询）
    pub fn set_field_query(&mut self, name: &str, query: &str) {
        if let Some(field) = self.fields.iter_mut().find(|f| f.name == name) {
            field.query = Some(query.to_string());
        }
    }

    /// 设置字段权重
    pub fn set_boost(&mut self, name: &str, boost: f32) {
        self.boost.insert(name.to_string(), boost);
    }

    /// 执行多字段搜索
    pub fn search(&self, query: &str) -> Result<SearchResult> {
        // 收集各字段的搜索结果
        let mut field_results: Vec<(String, Vec<DocId>, f32)> = Vec::new();
        
        for field in &self.fields {
            let field_query = field.query.as_ref().unwrap_or(&query.to_string());
            
            // 从 Document 获取索引并搜索
            // ... 实现搜索逻辑
            
            let results: Vec<DocId> = Vec::new(); // 实际从索引获取
            field_results.push((field.name.clone(), results, field.weight));
        }
        
        // 按权重合并结果
        let merged = self.merge_results(&field_results);
        
        Ok(SearchResult {
            results: merged,
            total: merged.len(),
            query: query.to_string(),
        })
    }

    /// 合并多字段结果
    fn merge_results(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        // 使用加权评分合并
        let mut scored: Vec<(DocId, f32)> = Vec::new();
        let mut seen: HashMap<DocId, usize> = HashMap::new();
        
        for (field_name, docs, weight) in results {
            for &doc_id in docs {
                let score = weight * self.boost.get(field_name).unwrap_or(&1.0);
                
                if let Some(&pos) = seen.get(&doc_id) {
                    scored[pos].1 += score;
                } else {
                    seen.insert(doc_id, scored.len());
                    scored.push((doc_id, score));
                }
            }
        }
        
        // 按分数排序
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        scored.into_iter()
            .map(|(id, _)| id)
            .collect()
    }
}
```

### 3.6 enrich/mod.rs - 结果丰富化

**职责**: 搜索结果的丰富化处理

```rust
//! 结果丰富化模块
//!
//! 提供搜索结果的丰富化功能，包括文档提取、高亮、标签等
//!
//! # 模块结构
//!
//! - `mod.rs`: 主接口和公共类型
//! - `basic.rs`: 基本丰富化
//! - `highlight.rs`: 高亮丰富化
//! - `document.rs`: 文档丰富化

mod basic;
mod highlight;
mod document;

pub use basic::BasicEnricher;
pub use highlight::HighlightEnricher;
pub use document::DocumentEnricher;
```

### 3.7 cache/mod.rs - 缓存层

**职责**: 统一管理各种缓存

```rust
//! 缓存模块
//!
//! 提供统一的缓存管理，包括搜索缓存、编码缓存、文档缓存
//!
//! # 模块结构
//!
//! - `mod.rs`: 主接口
//! - `search.rs`: 搜索结果缓存
//! - `encoder.rs`: 编码中间结果缓存
//! - `document.rs`: 文档解析缓存

mod search;
mod encoder;
mod document;

pub use search::{SearchCache, CacheStats};
pub use encoder::EncoderCache;
pub use document::DocumentCache;

/// 缓存类型
#[derive(Clone)]
pub enum CacheType {
    None,
    Search,
    Encoder,
    Document,
    All,
}

/// 统一缓存管理器
pub struct CacheManager {
    search_cache: Option<SearchCache>,
    encoder_cache: Option<EncoderCache>,
    document_cache: Option<DocumentCache>,
    cache_type: CacheType,
}

impl CacheManager {
    /// 创建缓存管理器
    pub fn new(cache_type: CacheType, max_size: usize) -> Self {
        let search_cache = match cache_type {
            CacheType::Search | CacheType::All => Some(SearchCache::new(max_size)),
            _ => None,
        };
        
        let encoder_cache = match cache_type {
            CacheType::Encoder | CacheType::All => Some(EncoderCache::new(max_size)),
            _ => None,
        };
        
        let document_cache = match cache_type {
            CacheType::Document | CacheType::All => Some(DocumentCache::new(max_size)),
            _ => None,
        };
        
        CacheManager {
            search_cache,
            encoder_cache,
            document_cache,
            cache_type,
        }
    }
}
```

---

## 四、lib.rs 导出设计

### 4.1 主入口文件 (lib.rs)

```rust
//!
//! Inversearch - 高性能全文搜索库
//!

// 导出公共 API
pub use document::{
    Document,
    DocumentConfig,
    Field, FieldConfig,
    parse_tree, TreePath,
    TagSystem, TagConfig,
    Batch, BatchOperation,
};

pub use search::{
    search,
    SearchResult,
    SearchCoordinator,
    MultiFieldSearchOptions,
};

pub use enrich::{
    Enricher,
    BasicEnricher,
    HighlightEnricher,
    DocumentEnricher,
};

pub use cache::{
    CacheManager,
    SearchCache,
    EncoderCache,
    DocumentCache,
    CacheType,
    CacheStats,
};

// 重新导出内部模块
pub mod document;
pub mod search;
pub mod enrich;
pub mod cache;

// ... 现有模块保持不变
pub mod index;
pub mod storage;
pub mod highlight;
pub mod encoder;
pub mod charset;
pub mod tokenizer;
pub mod intersect;
pub mod keystore;
pub mod common;
pub mod config;
pub mod type as type_module;
pub mod async_;
pub mod error;
```

---

## 五、模块职责总结

| 模块 | 职责 | 依赖 |
|------|------|------|
| **document** | 多字段文档抽象 | index, storage |
| **├── Field** | 字段定义和配置 | encoder |
| **├── tree** | 树形路径解析 | common |
| **├── tag** | 标签系统 | tree |
| **└── batch** | 批量操作 | - |
| **search** | 搜索功能 | index, resolver |
| **├── coordinator** | 多字段搜索协调 | document |
| **├── single_term** | 单术语搜索 | index |
| **└── cache** | 搜索缓存 | - |
| **enrich** | 结果丰富化 | storage, highlight |
| **├── basic** | 基本丰富化 | - |
| **├── highlight** | 高亮丰富化 | highlight |
| **└── document** | 文档丰富化 | storage |
| **cache** | 缓存层 | - |
| **├── search** | 搜索缓存 | - |
| **├── encoder** | 编码缓存 | encoder |
| **└── document** | 文档缓存 | document |
| **storage** | 持久化存储 | keystore |
| **common** | 通用工具 | - |

---

## 六、迁移计划

### 阶段一: 基础设施

1. **创建 document/mod.rs 框架**
   ```
   src/document/mod.rs      # 空结构
   src/document/field.rs    # 字段定义
   src/document/tree.rs     # 树形解析
   src/document/tag.rs      # 标签系统
   src/document/batch.rs    # 批量操作
   ```

2. **重构 search/mod.rs**
   ```
   src/search/mod.rs        # 简化，仅导出
   src/search/coordinator.rs # 新增
   src/search/multi_field.rs # 新增
   ```

3. **创建 enrich/mod.rs**
   ```
   src/enrich/mod.rs
   src/enrich/basic.rs
   src/enrich/highlight.rs
   src/enrich/document.rs
   ```

### 阶段二: 功能实现

4. **实现树形解析** (document/tree.rs)
5. **实现字段抽象** (document/field.rs)
6. **实现标签系统** (document/tag.rs)
7. **实现搜索协调器** (search/coordinator.rs)

### 阶段三: 集成测试

8. **集成测试**
9. **性能测试**
10. **文档更新**

---

## 七、注意事项

1. **mod.rs 仅用于导出**
   - 避免在 mod.rs 中实现复杂逻辑
   - 每个子模块应有独立文件

2. **单一职责原则**
   - 每个模块只做一件事
   - 避免模块职责膨胀

3. **依赖方向**
   - 上层模块依赖下层模块
   - 避免循环依赖

4. **公开 API 最小化**
   - 只导出必要的公共接口
   - 内部实现保持私有

---

## 八、文件清单

### 新增文件

| 文件 | 优先级 | 描述 |
|------|--------|------|
| `src/document/mod.rs` | 🔴 高 | Document 主结构 |
| `src/document/field.rs` | 🔴 高 | 字段定义 |
| `src/document/tree.rs` | 🔴 高 | 树形解析 |
| `src/document/tag.rs` | 🟡 中 | 标签系统 |
| `src/document/batch.rs` | 🟡 中 | 批量操作 |
| `src/search/coordinator.rs` | 🔴 高 | 搜索协调器 |
| `src/search/multi_field.rs` | 🔴 高 | 多字段搜索 |
| `src/enrich/mod.rs` | 🟡 中 | 丰富化模块 |
| `src/enrich/basic.rs` | 🟡 中 | 基本丰富化 |
| `src/enrich/highlight.rs` | 🟡 中 | 高亮丰富化 |
| `src/enrich/document.rs` | 🟡 中 | 文档丰富化 |
| `src/cache/mod.rs` | 🟢 低 | 缓存模块 |
| `src/cache/search.rs` | 🟢 低 | 搜索缓存 |
| `src/cache/encoder.rs` | 🟢 低 | 编码缓存 |
| `src/cache/document.rs` | 🟢 低 | 文档缓存 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/lib.rs` | 添加新的模块导出 |
| `src/common/mod.rs` | 扩展通用工具 |
| `src/resolver/mod.rs` | 拆分子模块 |
| `src/search/mod.rs` | 重构为导出模块 |

---

*文档版本: 1.0*
*创建日期: 2024-01-06*
