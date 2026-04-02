# Flexsearch 与 BM25 合并可行性分析

## 一、背景

用户提出：考虑将 flexsearch (inversearch) 合并到 bm25 中，因为两者数据持久化、索引需求类似，可以共用 tantivy 来存储数据。

---

## 二、当前架构对比

### BM25 架构

```
BM25 Service
├── Tantivy 搜索引擎 (核心存储)
│   ├── Schema: document_id, title, content
│   ├── IndexWriter (50MB heap)
│   └── IndexReader (OnCommitWithDelay)
├── PersistenceManager
│   ├── 备份/恢复
│   ├── 导入/导出
│   └── 索引压缩
├── Redis 缓存
└── gRPC 服务
```

**持久化特点**：
- 依赖 **Tantivy 原生索引格式**（二进制，高效）
- 索引自动持久化到磁盘目录
- 支持增量写入和合并

### Inversearch 架构

```
Inversearch Service
├── 自定义倒排索引
│   ├── KeystoreMap (term -> doc_ids)
│   ├── KeystoreMap (context -> term -> doc_ids)
│   └── Register (doc_id tracking)
├── StorageInterface
│   ├── MemoryStorage (无持久化)
│   ├── FileStorage (JSON文件)
│   └── RedisStorage (Redis键值)
├── Serialize
│   ├── JSON 格式
│   └── Binary 格式 (bincode)
└── gRPC 服务
```

**持久化特点**：
- **自定义索引结构**，不依赖外部搜索引擎
- 多种存储后端可选
- 序列化层独立设计

---

## 三、核心差异分析

### 1. 索引结构本质不同

| 特性 | BM25 (Tantivy) | Inversearch (自定义) |
|-----|---------------|---------------------|
| **索引格式** | Tantivy 专有二进制 | JSON/Binary 自定义 |
| **倒排列表** | Tantivy 内部管理 | `Vec<DocId>` 手动管理 |
| **词典存储** | Tantivy FST | `KeystoreMap` 哈希分桶 |
| **位置信息** | 支持（用于短语查询） | 不支持 |
| **列式存储** | 支持（DocValues） | 不支持 |

### 2. 搜索算法差异

| 特性 | BM25 | Inversearch |
|-----|------|-------------|
| **评分算法** | BM25 概率模型 | 自定义匹配度 |
| **上下文搜索** | 不支持 | 支持 (depth, bidirectional) |
| **分词模式** | Tantivy Tokenizer | 自定义 (strict/forward/reverse/full) |
| **字符集处理** | Tantivy Analyzer | 自定义 CJK/Latin 处理 |

### 3. 持久化机制差异

```
BM25 持久化流程:
  Document → Tantivy IndexWriter → 磁盘索引文件
  (自动管理，无需手动序列化)

Inversearch 持久化流程:
  Document → 内存索引 → Storage.commit() → 序列化 → 存储后端
  (手动管理，需要显式序列化)
```

---

## 四、合并方案评估

### 方案 A: Inversearch 使用 Tantivy 作为存储后端

**实现思路**：
```rust
// 新增 TantivyStorage 实现 StorageInterface
pub struct TantivyStorage {
    index: tantivy::Index,
    schema: Schema,
    writer: IndexWriter,
}

impl StorageInterface for TantivyStorage {
    async fn commit(&mut self, index: &Index, ...) -> Result<()> {
        // 将 Inversearch 的 KeystoreMap 数据导入 Tantivy
        for (term, doc_ids) in &index.map {
            for doc_id in doc_ids {
                let mut doc = TantivyDocument::new();
                doc.add_text(self.schema.term, term);
                doc.add_u64(self.schema.doc_id, *doc_id);
                self.writer.add_document(doc)?;
            }
        }
        self.writer.commit()?;
    }
}
```

**优点**：
- 利用 Tantivy 的高效索引格式
- 统一存储技术栈
- 减少维护成本

**缺点**：
- **丢失 Inversearch 的核心优势**：
  - 上下文搜索无法直接映射到 Tantivy
  - 自定义分词模式需要重新实现
  - KeystoreMap 的哈希分桶优化无法利用
- **性能可能下降**：
  - Tantivy 针对全文搜索优化，不适合精确关键词匹配
  - Inversearch 的内存索引对小数据集更快
- **复杂度增加**：
  - 需要维护 Inversearch → Tantivy 的数据映射
  - 查询逻辑需要完全重写

### 方案 B: 统一服务，双引擎并存

**实现思路**：
```rust
pub enum SearchEngine {
    BM25(TantivyIndex),
    Inversearch(InversearchIndex),
}

pub struct UnifiedSearchService {
    engine: SearchEngine,
    cache: RedisCache,
    persistence: PersistenceManager,
}
```

**优点**：
- 保持两种算法的独立性
- 统一服务入口
- 共享基础设施（缓存、监控、配置）

**缺点**：
- 服务复杂度增加
- 需要维护两套索引逻辑
- 资源占用增加

### 方案 C: 提取共享组件，保持独立服务（推荐）

**实现思路**：
```
services/
├── search-common/          # 共享基础库
│   ├── storage/           # 统一存储抽象
│   │   ├── traits.rs      # StorageInterface
│   │   ├── tantivy.rs     # Tantivy 后端
│   │   ├── redis.rs       # Redis 后端
│   │   └── file.rs        # 文件后端
│   ├── cache/             # 缓存抽象
│   ├── config/            # 配置管理
│   └── metrics/           # 监控指标
├── bm25/                   # BM25 服务
│   └── 使用 search-common
└── inversearch/            # Inversearch 服务
    └── 使用 search-common
```

**优点**：
- 保持算法独立性
- 共享基础设施代码
- 各服务可独立演进
- 部署灵活

**缺点**：
- 需要额外的模块管理
- 版本协调成本

---

## 五、技术可行性详细分析

### 1. 能否用 Tantivy 存储 Inversearch 数据？

**理论上可行，但需要大量适配工作**：

```rust
// Inversearch 索引结构
KeystoreMap<String, Vec<DocId>>  // term -> [doc_ids]

// 映射到 Tantivy Schema
Schema:
  - term: TEXT (indexed, stored)
  - doc_ids: TEXT (stored, comma-separated)
  // 或者
  - term: STRING
  - doc_id: U64  // 每个文档一行
```

**问题**：
1. **查询效率**：Inversearch 的 `get(term)` 是 O(1) 哈希查找，Tantivy 需要倒排索引查询
2. **上下文索引**：`ctx:keyword:term -> doc_ids` 的二级索引结构难以直接映射
3. **增量更新**：Inversearch 的 `fastupdate` 模式依赖 Register 追踪，Tantivy 需要删除+重新添加

### 2. 持久化需求是否真的类似？

**表面相似，实际不同**：

| 需求 | BM25 | Inversearch |
|-----|------|-------------|
| 索引存储 | Tantivy 自动管理 | 手动序列化 |
| 备份恢复 | 目录复制 | JSON/Binary 导出 |
| 分布式存储 | 不需要（单机） | RedisStorage 支持 |
| 数据迁移 | Tantivy 格式 | 通用 JSON 格式 |

---

## 六、结论与建议

### 结论

**不建议将 Inversearch 合并到 BM25 中**，原因如下：

1. **算法本质不同**：
   - BM25 是概率模型，适合语义搜索
   - Inversearch 是精确匹配，适合关键词搜索
   - 合并会丢失各自的核心优势

2. **索引结构不兼容**：
   - Tantivy 的索引格式针对 BM25 优化
   - Inversearch 的 KeystoreMap 针对快速查找优化
   - 强行统一会导致性能下降

3. **持久化机制差异大**：
   - BM25 依赖 Tantivy 自动持久化
   - Inversearch 需要灵活的存储后端（内存/文件/Redis）
   - 统一存储会限制 Inversearch 的灵活性

### 建议

采用 **方案 C：提取共享组件，保持独立服务**

```
services/
├── search-common/          # 共享基础库
│   ├── storage/           # 统一存储抽象
│   ├── cache/             # 缓存抽象
│   ├── config/            # 配置管理
│   └── metrics/           # 监控指标
├── bm25/                   # BM25 服务（保持 Tantivy）
└── inversearch/            # Inversearch 服务（保持自定义索引）
```

**实施步骤**：

1. **第一阶段**：提取 `search-common` 库
   - 统一 StorageInterface trait
   - 共享 Redis 缓存实现
   - 统一配置和监控

2. **第二阶段**：优化各自持久化
   - BM25：增强 Tantivy 备份恢复
   - Inversearch：优化序列化性能

3. **第三阶段**：可选增强
   - Inversearch 可选支持 Tantivy 作为存储后端（仅用于持久化，不用于搜索）
   - BM25 可选支持 Redis 分布式缓存

---

## 七、附录：代码示例

### 共享存储接口设计

```rust
// search-common/src/storage/traits.rs
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

// BM25 使用 Tantivy 后端
// inversearch 可选 File/Redis/Tantivy 后端
```

### Inversearch 可选 Tantivy 存储（仅持久化）

```rust
// inversearch/src/storage/tantivy.rs
pub struct TantivyPersistence {
    index: tantivy::Index,
}

impl StorageInterface for TantivyPersistence {
    async fn commit(&mut self, index: &Index, ...) -> Result<()> {
        // 仅用于持久化，搜索仍使用内存索引
        let serialized = index.to_binary(&SerializeConfig::default())?;
        // 存储到 Tantivy 作为 blob
    }
}
```

这样既保持了 Inversearch 的搜索性能，又可以利用 Tantivy 的持久化能力。
