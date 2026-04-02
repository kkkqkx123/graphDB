# Inversearch 缺失功能分析

## 概述

本文档详细分析 FlexSearch JavaScript 实现 (`document.js`) 与 Inversearch Rust 实现之间的功能差距，为后续功能补全提供参考。

参考实现: `src/document.js`

---

## 一、核心架构对比

### FlexSearch JavaScript 架构

```
Document (主入口)
├── field[] (多字段索引列表)
├── tree[] (字段树结构)
├── marker[] (嵌套字段标记)
├── tag[] (标签系统)
│   ├── tagfield[] (标签字段)
│   └── tagtree[] (标签树结构)
├── index (Map<字段名, Index>)
├── store (文档存储)
├── storetree (存储字段树)
├── keystore (键值存储)
├── cache (搜索缓存)
└── reg (注册表/文档ID集合)
```

### Inversearch 当前架构

```
Inversearch
├── Index (单索引实例)
├── search (搜索模块)
├── resolver (结果解析器)
├── highlight (高亮模块)
├── storage (存储模块)
├── encoder (编码器)
├── charset (字符集处理)
├── tokenizer (分词器)
└── keystore (键值存储)
```

**关键差异**: FlexSearch 使用 Document 类统一管理多字段，而 Inversearch 每次只处理单个索引实例。

---

## 二、缺失功能详细列表

### 2.1 Document 多字段抽象层

**状态**: ❌ 未实现

**JavaScript 实现**: `src/document.js:48-162`

**功能描述**:
- 统一管理多个字段的索引
- 支持动态字段配置
- 协调多字段搜索

**缺失方法**:
```rust
// 需要的接口
pub struct Document {
    pub fields: Vec<String>,
    pub indexes: HashMap<String, Index>,
    pub store: Option<Storage>,
    pub tag_indexes: HashMap<String, Index>,
}
```

**影响范围**:
- 无法实现多字段联合搜索
- 无法统一管理不同字段的索引配置

---

### 2.2 树形结构解析 (parse_tree)

**状态**: ❌ 未实现

**JavaScript 实现**: `src/document.js:311-341`

**功能描述**:
解析嵌套字段路径，支持数组索引和属性访问。

```javascript
// 支持的语法
"user.name"           // 嵌套属性
"users[0].name"       // 数组索引
"users[-1].name"      // 倒数第一个
"items[0-2].title"    // 数组范围
```

**实现示例**:

```rust
pub fn parse_tree(key: &str, marker: &mut Vec<bool>) -> TreePath {
    let parts: Vec<&str> = key.split(':').collect();
    let mut result = Vec::new();
    let mut count = 0;

    for part in parts {
        let mut field = part.to_string();
        
        // 处理数组索引 [0], [-1], [0-2]
        if field.ends_with(']') {
            let bracket_pos = field.rfind('[').unwrap();
            let index_part = &field[bracket_pos+1..field.len()-1];
            
            // 提取基础字段
            field = field[..bracket_pos].to_string();
            
            if !field.is_empty() {
                marker.push(true);
            }
            
            // 解析索引范围
            if index_part.contains('-') {
                let parts: Vec<&str> = index_part.split('-').collect();
                let start: usize = parts[0].parse().unwrap();
                let end: usize = parts[1].parse().unwrap();
                result.push(TreePath::Range(start, end, field));
            } else if index_part.starts_with('-') {
                let idx: usize = index_part[1..].parse().unwrap();
                result.push(TreePath::NegativeIndex(idx, field));
            } else {
                let idx: usize = index_part.parse().unwrap();
                result.push(TreePath::Index(idx, field));
            }
        } else {
            result.push(TreePath::Field(field));
        }
    }
    
    result
}

pub enum TreePath {
    Field(String),
    Index(usize, String),
    NegativeIndex(usize, String),
    Range(usize, usize, String),
}
```

**优先级**: 🔴 高

---

### 2.3 标签系统 (Tag System)

**状态**: ❌ 未实现

**JavaScript 实现**: `src/document.js:117-149`

**功能描述**:
为文档添加标签，支持基于标签的过滤和搜索。

**配置示例**:

```javascript
{
    document: {
        index: ["title", "content"],
        tag: ["category", "author"],
        store: ["id", "title", "content"]
    }
}
```

**核心方法**:

```rust
// 需要的标签接口
pub struct TagSystem {
    pub tag_fields: Vec<String>,
    pub tag_indexes: HashMap<String, HashMap<String, Vec<DocId>>>, // field -> tag -> ids
    pub tag_trees: Vec<TreePath>,
}

impl TagSystem {
    pub fn add_tags(&mut self, doc_id: DocId, tags: &[(&str, &Value)]);
    pub fn remove_tags(&mut self, doc_id: DocId);
    pub fn query_by_tag(&self, field: &str, tag: &str) -> Option<&Vec<DocId>>;
    pub fn query_by_tags(&self, field: &str, tags: &[&str]) -> Vec<DocId>;
}
```

**使用场景**:
- 文档分类和过滤
- 多维度搜索
- 权限控制

**优先级**: 🟡 中

---

### 2.4 多字段联合搜索 (Multi-field Search)

**状态**: ⚠️ 部分实现

**当前状态**:
- Inversearch 实现了单字段搜索
- 缺少跨字段搜索协调器

**JavaScript 实现**: `src/document/search.js`

**缺失功能**:

```rust
// 多字段搜索配置
pub struct MultiFieldSearchOptions {
    pub query: String,
    pub fields: Vec<FieldSearchOption>,
    pub boost: HashMap<String, f32>,  // 字段权重
    pub combine: CombineStrategy,      // 组合策略
}

pub enum CombineStrategy {
    And,       // 所有字段都必须匹配
    Or,        // 任一字段匹配即可
    Weight,    // 按权重组合评分
    BestField, // 最佳字段匹配
}

pub trait MultiFieldSearcher {
    fn search(&self, options: &MultiFieldSearchOptions) -> Result<SearchResult>;
    fn explain(&self, options: &MultiFieldSearchOptions) -> Result<Explanation>;
}
```

**优先级**: 🔴 高

---

### 2.5 动态字段解析 (Dynamic Field Resolution)

**状态**: ❌ 未实现

**JavaScript 实现**: `src/common.js:parse_simple()`

**功能描述**:
根据路径从嵌套对象中提取值。

```javascript
parse_simple({user: {name: "John"}}, "user.name") // "John"
parse_simple({items: [{name: "A"}, {name: "B"}]}, "items[0].name") // "A"
```

**Rust 实现**:

```rust
pub fn parse_simple<'a>(document: &'a Value, path: &str) -> Option<&'a str> {
    let mut current = document;
    
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(arr) => {
                // 处理索引
                if segment.ends_with(']') {
                    let idx_str = &segment[..segment.len()-1];
                    if idx_str == "-" {
                        // 最后一个元素
                        current = arr.last()?;
                    } else {
                        let idx: usize = idx_str.parse().ok()?;
                        current = arr.get(idx)?;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    
    current.as_str()
}
```

**优先级**: 🔴 高 (其他功能依赖此基础功能)

---

### 2.6 文档丰富化 (Document Enrichment)

**状态**: ⚠️ 部分实现

**当前状态**:
- Inversearch 有 `Enricher` 模块
- 仅支持简单的高亮和字段提取
- 缺少复杂嵌套文档的处理

**JavaScript 实现**: `src/serialize.js:exportDocument()`

**缺失功能**:

```rust
// 完整文档丰富化接口
pub struct DocumentEnricher {
    pub store: Option<Storage>,
    pub fields: Vec<String>,
    pub tag_fields: Vec<String>,
}

impl DocumentEnricher {
    pub fn enrich(&self, doc_id: DocId) -> Result<EnrichedDocument> {
        // 1. 从存储获取原始文档
        let doc = self.store.get(doc_id)?;
        
        // 2. 应用字段选择
        let selected = self.select_fields(&doc)?;
        
        // 3. 添加标签信息
        let tagged = self.apply_tags(selected)?;
        
        // 4. 添加高亮
        let highlighted = self.apply_highlights(tagged)?;
        
        Ok(highlighted)
    }
    
    pub fn enrich_batch(&self, doc_ids: &[DocId]) -> Vec<EnrichedDocument>;
}
```

**优先级**: 🟡 中

---

### 2.7 批量操作优化 (Batch Operations)

**状态**: ⚠️ 部分实现

**JavaScript 实现**: `src/document/add.js`

**缺失功能**:

```rust
pub struct BatchOperations {
    pending_adds: Vec<(DocId, String)>,
    pending_updates: HashMap<DocId, String>,
    pending_deletes: Vec<DocId>,
    batch_size: usize,
}

impl BatchOperations {
    pub fn add(&mut self, doc_id: DocId, content: &str) {
        self.pending_adds.push((doc_id, content.to_string()));
        self.flush_if_full();
    }
    
    pub fn update(&mut self, doc_id: DocId, content: &str) {
        self.pending_updates.insert(doc_id, content.to_string());
        self.flush_if_full();
    }
    
    pub fn remove(&mut self, doc_id: DocId) {
        self.pending_deletes.push(doc_id);
        self.flush_if_full();
    }
    
    pub fn flush(&mut self) -> Result<()>;
    pub fn flush_if_full(&mut self);
    pub fn clear(&mut self);
}
```

**优先级**: 🟡 中

---

### 2.8 Worker 并行处理支持

**状态**: ❌ 未实现

**JavaScript 实现**: `src/worker.js`, `src/document.js:84-107`

**功能描述**:
- 后台线程处理索引操作
- 异步初始化
- 主线程与 Worker 通信

**Rust 实现思路**:

```rust
// 使用 tokio::task 进行并行处理
pub struct WorkerIndex {
    tx: Sender<IndexRequest>,
    rx: Receiver<IndexResponse>,
    worker_handle: JoinHandle<()>,
}

pub enum IndexRequest {
    Add(DocId, String),
    Remove(DocId),
    Search(SearchOptions),
    Clear,
}

pub enum IndexResponse {
    AddResult(Result<()>),
    RemoveResult(Result<()>),
    SearchResult(Result<SearchResult>),
    ClearResult(Result<()>),
}

impl WorkerIndex {
    pub async fn new(options: IndexOptions) -> Self {
        let (tx, rx) = channel(100);
        let worker_handle = tokio::spawn(Self::worker_loop(rx));
        
        WorkerIndex { tx, rx, worker_handle }
    }
    
    pub async fn add(&self, id: DocId, content: &str) -> Result<()> {
        self.tx.send(IndexRequest::Add(id, content.to_string())).await?;
        match self.rx.recv().await? {
            IndexResponse::AddResult(r) => r,
            _ => Err(InversearchError::UnexpectedResponse),
        }
    }
}
```

**优先级**: 🟢 低 (异步处理已通过 tokio 实现)

---

### 2.9 持久化集成 (Persistent Storage Integration)

**状态**: ⚠️ 部分实现

**JavaScript 实现**: `src/db/interface.js`

**当前状态**:
- Inversearch 有 `storage` 模块
- 实现了基本的 Redis 集成
- 缺少数据库抽象接口

**缺失功能**:

```rust
// 数据库抽象接口
pub trait StorageInterface {
    fn mount(&mut self, id: &str) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn get(&self, id: &str) -> Option<String>;
    fn set(&mut self, id: &str, value: &str);
    fn remove(&mut self, id: &str);
    fn has(&self, id: &str) -> bool;
}

// 支持的存储后端
pub enum StorageBackend {
    Memory(HashMap<String, String>),
    Redis(RedisClient),
    SQLite(SqliteConnection),
    MongoDB(MongoCollection),
}
```

**优先级**: 🟡 中

---

### 2.10 缓存策略 (Caching Strategy)

**状态**: ⚠️ 部分实现

**JavaScript 实现**: `src/cache.js`

**当前状态**:
- Inversearch 有 `search/cache.rs`
- 实现了搜索结果缓存
- 缺少编码结果缓存

**缺失功能**:

```rust
pub enum CacheStrategy {
    None,
    Search,      // 仅缓存搜索结果
    Encoder,     // 缓存编码中间结果
    Document,    // 缓存文档解析结果
    All,         // 所有缓存
}

pub struct ComprehensiveCache {
    search_cache: SearchCache,
    encoder_cache: LruCache<String, Vec<String>>,
    document_cache: LruCache<DocId, ParsedDocument>,
    strategy: CacheStrategy,
}

impl ComprehensiveCache {
    pub fn get_encoded(&mut self, content: &str) -> Option<&Vec<String>> {
        if self.strategy.supports(CacheLevel::Encoder) {
            self.encoder_cache.get(content)
        } else {
            None
        }
    }
    
    pub fn store_encoded(&mut self, content: &str, encoded: Vec<String>) {
        if self.strategy.supports(CacheLevel::Encoder) {
            self.encoder_cache.insert(content.to_string(), encoded);
        }
    }
}
```

**优先级**: 🟢 低 (性能优化功能)

---

## 三、功能实现优先级

### 🔴 高优先级 (核心功能)

1. **树形结构解析** (`parse_tree`)
   - 依赖: 无
   - 被依赖: 标签系统、文档丰富化
   - 难度: ⭐⭐

2. **动态字段解析** (`parse_simple`)
   - 依赖: 树形结构解析
   - 被依赖: 多字段搜索、文档丰富化
   - 难度: ⭐

3. **Document 多字段抽象层**
   - 依赖: 树形结构解析、动态字段解析
   - 被依赖: 多字段搜索、标签系统
   - 难度: ⭐⭐⭐

4. **多字段联合搜索**
   - 依赖: Document 抽象层
   - 被依赖: 完整文档搜索体验
   - 难度: ⭐⭐⭐

### 🟡 中优先级 (重要功能)

5. **标签系统**
   - 依赖: Document 抽象层
   - 被依赖: 文档分类、过滤
   - 难度: ⭐⭐

6. **文档丰富化**
   - 依赖: 动态字段解析、标签系统
   - 被依赖: 搜索结果展示
   - 难度: ⭐⭐

7. **持久化集成**
   - 依赖: 存储抽象
   - 被依赖: 生产环境部署
   - 难度: ⭐⭐

8. **批量操作优化**
   - 依赖: 无
   - 被依赖: 大数据量导入
   - 难度: ⭐⭐

### 🟢 低优先级 (优化功能)

9. **Worker 并行处理**
   - 依赖: tokio 异步runtime
   - 被依赖: 高并发场景
   - 难度: ⭐⭐⭐

10. **缓存策略增强**
    - 依赖: 无
    - 被依赖: 性能优化
    - 难度: ⭐

---

## 四、推荐实现路径

### 阶段一: 基础架构完善

```
1. 实现 parse_tree 工具函数
   └─ services/inversearch/src/common/tree.rs

2. 实现 parse_simple 工具函数
   └─ services/inversearch/src/common/mod.rs (扩展)

3. 创建 Document 抽象层
   └─ services/inversearch/src/document/mod.rs
```

### 阶段二: 核心功能

```
4. 实现标签系统
   └─ services/inversearch/src/tag/mod.rs

5. 实现多字段搜索协调器
   └─ services/inversearch/src/search/multi_field.rs

6. 完善文档丰富化
   └─ services/inversearch/src/resolver/document_enrich.rs
```

### 阶段三: 高级功能

```
7. 实现批量操作
   └─ services/inversearch/src/batch/mod.rs

8. 增强持久化支持
   └─ services/inversearch/src/storage/interface.rs

9. 添加 Worker 支持 (如需要)
   └─ services/inversearch/src/worker/mod.rs
```

---

## 五、测试覆盖建议

### 单元测试

- [ ] 树形结构解析边界情况
- [ ] 动态字段解析嵌套对象
- [ ] 标签系统增删改查
- [ ] 多字段搜索评分准确性

### 集成测试

- [ ] 完整 Document 生命周期
- [ ] 多字段搜索与单字段搜索对比
- [ ] 标签过滤与搜索结合
- [ ] 持久化存储恢复

### 性能测试

- [ ] 批量导入性能
- [ ] 多字段搜索响应时间
- [ ] 标签查询延迟
- [ ] 内存使用情况

---

## 六、参考资源

- JavaScript 实现: `src/document.js`
- JavaScript 工具函数: `src/common.js`
- 序列化实现: `src/serialize.js`
- 文档搜索: `src/document/search.js`
- 文档添加: `src/document/add.js`

---

## 更新日志

| 日期 | 版本 | 变更 |
|------|------|------|
| 2024-01-06 | 1.0 | 初始文档创建 |
