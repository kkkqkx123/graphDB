# GraphDB 索引系统架构分析

## 一、概述

GraphDB 的索引系统采用分层架构设计，将索引功能划分为四个主要层级：

```
┌─────────────────────────────────────────────────────────────┐
│                    查询层 (Query Layer)                   │
│  - 计划节点 (IndexScan)                                  │
│  - 执行器 (IndexScanExecutor, CreateTagIndexExecutor)        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  索引定义层 (Index Core)                  │
│  - 类型定义 (types.rs)                                    │
│  - 编码工具 (binary.rs)                                   │
│  - 错误定义 (error.rs)                                     │
│  - 配置 (config.rs)                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  服务层 (Index Service)                     │
│  - 缓存 (cache.rs)                                       │
│  - 统计 (stats.rs)                                       │
│  - 全文索引 (fulltext.rs)                                  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                存储层 (Storage Layer)                      │
│  ┌────────────────────┐  ┌──────────────────────────┐  │
│  │  元数据管理        │  │   数据管理              │  │
│  │  (metadata/)      │  │   (index/)             │  │
│  └────────────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    ┌───────────────┐
                    │  存储引擎     │
                    │  (redb)       │
                    └───────────────┘
```

---

## 二、索引定义层 (src/index/)

### 2.1 目录结构

```
src/index/
├── types.rs              # 索引类型定义
├── binary.rs             # 索引键二进制编码
├── config.rs             # 索引服务配置
├── error.rs              # 索引错误类型
├── mod.rs                # 模块导出
└── service/              # 服务层子模块
    ├── mod.rs
    ├── cache.rs          # 索引缓存
    ├── stats.rs          # 索引统计
    └── fulltext.rs       # 全文索引
```

### 2.2 核心类型定义 (types.rs)

#### IndexStatus - 索引状态

```rust
pub enum IndexStatus {
    Creating,           // 创建中
    Active,             // 活跃
    Dropped,            // 已删除
    Failed(String),      // 失败（带错误信息）
}
```

#### IndexType - 索引类型

```rust
pub enum IndexType {
    TagIndex,           // Tag 索引
    EdgeIndex,          // Edge 索引
    FulltextIndex,      // 全文索引
}
```

#### IndexField - 索引字段

```rust
pub struct IndexField {
    pub name: String,           // 字段名
    pub value_type: Value,      // 值类型
    pub is_nullable: bool,      // 是否可空
}
```

#### Index - 索引定义

```rust
pub struct Index {
    pub id: i32,                     // 索引 ID
    pub name: String,                 // 索引名称
    pub space_id: i32,                // 空间 ID
    pub schema_name: String,          // Schema 名称（Tag 或 Edge 类型名）
    pub fields: Vec<IndexField>,      // 索引字段列表
    pub properties: Vec<String>,       // 属性名称列表
    pub index_type: IndexType,        // 索引类型
    pub status: IndexStatus,          // 索引状态
    pub is_unique: bool,             // 是否唯一索引
    pub comment: Option<String>,      // 注释
}
```

#### IndexStats - 索引统计

```rust
pub struct IndexStats {
    pub index_id: i32,              // 索引 ID
    pub index_name: String,          // 索引名称
    pub total_entries: usize,        // 总条目数
    pub unique_entries: usize,       // 唯一条目数
    pub last_updated: i64,          // 最后更新时间
    pub memory_usage_bytes: usize,   // 内存使用（字节）
    pub query_count: u64,           // 查询次数
    pub avg_query_time_ms: f64,     // 平均查询时间（毫秒）
}
```

#### IndexOptimization - 索引优化建议

```rust
pub struct IndexOptimization {
    pub index_id: i32,              // 索引 ID
    pub index_name: String,          // 索引名称
    pub suggestions: Vec<String>,    // 优化建议列表
    pub priority: String,           // 优先级
}
```

### 2.3 二进制编码 (binary.rs)

#### IndexKey - 索引键

```rust
pub struct IndexKey {
    pub space_id: i32,             // 空间 ID
    pub index_id: i32,             // 索引 ID
    pub encoded_values: Vec<u8>,    // 编码后的值
}
```

#### IndexBinaryEncoder - 编码器

提供以下编码方法：

| 方法 | 描述 |
|------|------|
| `encode_value(value: &Value)` | 编码单个值 |
| `encode_int64(i: i64)` | 编码 64 位整数 |
| `encode_float64(f: f64)` | 编码 64 位浮点数 |
| `encode_bool(b: bool)` | 编码布尔值 |
| `encode_string(s: &str)` | 编码字符串（带长度前缀） |
| `encode_date(d: &DateValue)` | 编码日期 |
| `encode_datetime(dt: &DateTimeValue)` | 编码日期时间 |
| `encode_composite_key(values: &[Value])` | 编码复合索引键 |
| `encode_prefix(values: &[Value], prefix_len: usize)` | 编码前缀 |
| `encode_range(start: &Value, end: &Value)` | 编码范围 |

### 2.4 配置 (config.rs)

```rust
pub struct IndexServiceConfig {
    pub max_memory_bytes: u64,           // 最大内存（字节）
    pub enable_auto_cleanup: bool,          // 启用自动清理
    pub cleanup_interval_secs: u64,        // 清理间隔（秒）
    pub exact_lookup_cache_size: usize,     // 精确查找缓存大小
    pub enable_cache_stats: bool,          // 启用缓存统计
    pub cache_ttl_secs: u64,              // 缓存 TTL（秒）
}
```

### 2.5 错误类型 (error.rs)

```rust
pub enum IndexError {
    IndexCreationError(String),    // 索引创建错误
    IndexUpdateError(String),     // 索引更新错误
    IndexQueryError(String),      // 索引查询错误
    IndexNotFound(String),        // 索引不存在
    IndexStatusError(String),     // 索引状态错误
}
```

---

## 三、服务层 (src/index/service/)

### 3.1 缓存模块 (cache.rs)

#### CacheStats - 缓存统计

```rust
pub struct CacheStats {
    pub hits: AtomicU64,           // 命中次数
    pub misses: AtomicU64,          // 未命中次数
    pub evictions: AtomicU64,       // 淘汰次数
    pub insertions: AtomicU64,      // 插入次数
    pub invalidations: AtomicU64,    // 失效次数
}
```

#### VersionedCache<V> - 版本化缓存

支持版本控制的缓存，当索引更新时自动失效相关缓存。

```rust
pub struct VersionedCache<V> {
    cache: DashMap<CacheKey, CacheEntry<V>>,    // 缓存数据
    index_versions: DashMap<i32, u64>,          // 索引版本
    config: IndexServiceConfig,                  // 配置
    stats: Arc<CacheStats>,                     // 统计
}
```

**主要方法：**

| 方法 | 描述 |
|------|------|
| `get(index_id, value)` | 获取缓存值 |
| `insert(index_id, value, result)` | 插入缓存值 |
| `invalidate_index(index_id)` | 失效整个索引的缓存 |
| `invalidate(index_id, value)` | 失效特定键的缓存 |
| `clear()` | 清空缓存 |

#### MemoryIndexCache - 内存索引缓存

基于 LRU 策略的内存缓存，用于缓存标签和属性。

```rust
pub struct MemoryIndexCache {
    label_cache: DashMap<String, Vec<Value>>,              // 标签缓存
    property_cache: DashMap<String, HashMap<Value, Vec<Value>>>, // 属性缓存
    access_count: DashMap<String, AtomicU64>,              // 访问计数
    last_accessed: DashMap<String, u64>,                  // 最后访问时间
    max_size: usize,                                       // 最大大小
    stats: Arc<CacheStats>,                               // 统计
}
```

### 3.2 统计模块 (stats.rs)

#### QueryType - 查询类型

```rust
pub enum QueryType {
    Exact,      // 精确查询
    Prefix,     // 前缀查询
    Range,      // 范围查询
}
```

#### IndexQueryStats - 索引查询统计

```rust
pub struct IndexQueryStats {
    exact_queries: AtomicU64,        // 精确查询次数
    prefix_queries: AtomicU64,       // 前缀查询次数
    range_queries: AtomicU64,       // 范围查询次数
    exact_hits: AtomicU64,          // 精确查询命中次数
    prefix_hits: AtomicU64,         // 前缀查询命中次数
    range_hits: AtomicU64,         // 范围查询命中次数
    total_query_time_us: AtomicU64,  // 总查询时间（微秒）
}
```

**主要方法：**

| 方法 | 描述 |
|------|------|
| `record_query(found, duration, query_type)` | 记录查询 |
| `get_total_queries()` | 获取总查询次数 |
| `get_hit_rate()` | 获取命中率 |
| `get_average_query_time_us()` | 获取平均查询时间 |

### 3.3 全文索引模块 (fulltext.rs)

#### FulltextIndexError - 全文索引错误

```rust
pub enum FulltextIndexError {
    EngineError(String),          // 引擎错误
    IndexNotFound(String),        // 索引不存在
    IndexAlreadyExists(String),   // 索引已存在
    DocumentFormatError(String),  // 文档格式错误
    QuerySyntaxError(String),     // 查询语法错误
}
```

#### FulltextIndexConfig - 全文索引配置

```rust
pub struct FulltextIndexConfig {
    pub name: String,                              // 索引名称
    pub schema_type: FulltextSchemaType,              // Schema 类型（Tag/Edge）
    pub schema_name: String,                         // Schema 名称
    pub fields: Vec<String>,                        // 索引字段
    pub analyzer: Option<String>,                    // 分词器
    pub case_sensitive: bool,                        // 是否区分大小写
    pub created_at: chrono::DateTime<chrono::Utc>,   // 创建时间
}
```

#### FulltextDocument - 全文索引文档

```rust
pub struct FulltextDocument {
    pub id: String,                                // 文档 ID
    pub schema_type: FulltextSchemaType,              // Schema 类型
    pub schema_name: String,                         // Schema 名称
    pub content: HashMap<String, Value>,              // 文档内容
    pub indexed_at: chrono::DateTime<chrono::Utc>,   // 索引时间
}
```

#### FulltextQuery - 全文索引查询

```rust
pub struct FulltextQuery {
    pub index_name: String,           // 索引名称
    pub query_string: String,         // 查询字符串
    pub fields: Option<Vec<String>>,  // 查询字段
    pub limit: usize,                // 限制数量
    pub offset: usize,               // 偏移量
}
```

#### FulltextIndexEngine - 全文索引引擎 Trait

```rust
pub trait FulltextIndexEngine: Send + Sync {
    fn create_index(&mut self, config: &FulltextIndexConfig) -> DBResult<()>;
    fn drop_index(&mut self, name: &str) -> DBResult<()>;
    fn index_document(&mut self, doc: &FulltextDocument) -> DBResult<()>;
    fn delete_document(&mut self, id: &str) -> DBResult<()>;
    fn search(&mut self, query: &FulltextQuery) -> DBResult<Vec<FulltextSearchResult>>;
    fn index_exists(&self, name: &str) -> bool;
    fn get_index_config(&self, name: &str) -> Option<FulltextIndexConfig>;
    fn list_index_configs(&self) -> Vec<FulltextIndexConfig>;
}
```

#### SimpleFulltextEngine - 简单全文索引实现

基于倒排索引的简单实现，支持基本的全文搜索功能。

---

## 四、存储层数据管理 (src/storage/index/)

### 4.1 目录结构

```
src/storage/index/
├── mod.rs                    # 模块导出
└── index_data_manager.rs     # 索引数据管理
```

### 4.2 IndexDataManager Trait

```rust
pub trait IndexDataManager {
    // 顶点索引操作
    fn update_vertex_indexes(&self, space: &str, vertex_id: &Value, tag_name: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    fn delete_vertex_indexes(&self, space: &str, vertex_id: &Value) -> Result<(), StorageError>;
    fn lookup_tag_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;

    // 边索引操作
    fn update_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str, props: &[(String, Value)]) -> Result<(), StorageError>;
    fn delete_edge_indexes(&self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError>;
    fn lookup_edge_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError>;
    fn clear_edge_index(&self, space: &str, index_name: &str) -> Result<(), StorageError>;
    fn build_edge_index_entry(&self, space: &str, index: &Index, edge: &Edge) -> Result<(), StorageError>;
}
```

### 4.3 RedbIndexDataManager 实现

基于 Redb 存储引擎的索引数据管理器实现。

#### 索引键格式

- **顶点索引键**: `space:idx:v:index_name:tag_name:prop_value:vertex_id`
- **边索引键**: `space:idx:e:index_name:edge_type:prop_value:src:dst`

#### 主要方法实现

| 方法 | 功能 | 键格式 |
|------|------|---------|
| `update_vertex_indexes` | 更新顶点索引 | `space:idx:v:default:tag_name:prop_name:prop_value:vertex_id` |
| `update_edge_indexes` | 更新边索引 | `space:idx:e:default:edge_type:prop_name:prop_value:src:dst` |
| `delete_vertex_indexes` | 删除顶点索引 | 扫描并删除所有匹配的键 |
| `delete_edge_indexes` | 删除边索引 | 扫描并删除所有匹配的键 |
| `lookup_tag_index` | 查询 Tag 索引 | 前缀扫描 + 值匹配 |
| `lookup_edge_index` | 查询 Edge 索引 | 前缀扫描 + 值匹配 |

---

## 五、元数据管理层 (src/storage/metadata/)

### 5.1 目录结构

```
src/storage/metadata/
├── mod.rs                           # 模块导出
├── schema_manager.rs                 # Schema 管理 Trait
├── extended_schema.rs                # 扩展 Schema Trait
├── redb_schema_manager.rs           # Redb Schema 实现
├── redb_extended_schema.rs          # Redb 扩展 Schema 实现
├── index_metadata_manager.rs         # 索引元数据 Trait
└── redb_index_metadata_manager.rs   # Redb 索引元数据实现
```

### 5.2 IndexMetadataManager Trait

```rust
pub trait IndexMetadataManager: Send + Sync + std::fmt::Debug {
    // Tag 索引操作
    fn create_tag_index(&self, space: &str, index: &Index) -> Result<bool, StorageError>;
    fn drop_tag_index(&self, space: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError>;
    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError>;
    fn drop_tag_indexes_by_tag(&self, space: &str, tag_name: &str) -> Result<(), StorageError>;

    // Edge 索引操作
    fn create_edge_index(&self, space: &str, index: &Index) -> Result<bool, StorageError>;
    fn drop_edge_index(&self, space: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError>;
    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError>;
    fn drop_edge_indexes_by_type(&self, space: &str, edge_type: &str) -> Result<(), StorageError>;
}
```

### 5.3 SchemaManager Trait

```rust
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // 空间操作
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn get_space_by_id(&self, space_id: i32) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    // Tag 操作
    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError>;
    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError>;
    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError>;
    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError>;

    // Edge 类型操作
    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError>;
    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError>;
    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError>;

    // Schema 获取
    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError>;
    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError>;
}
```

### 5.4 RedbIndexMetadataManager 实现

基于 Redb 存储引擎的索引元数据管理器。

#### 存储表

| 表名 | 键格式 | 值格式 |
|------|---------|---------|
| `TAG_INDEXES_TABLE` | `space:index_name` | `Index` 序列化 |
| `EDGE_INDEXES_TABLE` | `space:index_name` | `Index` 序列化 |

#### 主要方法实现

| 方法 | 功能 |
|------|------|
| `create_tag_index` | 创建 Tag 索引元数据 |
| `drop_tag_index` | 删除 Tag 索引元数据 |
| `get_tag_index` | 获取 Tag 索引元数据 |
| `list_tag_indexes` | 列出所有 Tag 索引 |
| `drop_tag_indexes_by_tag` | 删除指定 Tag 的所有索引 |
| `create_edge_index` | 创建 Edge 索引元数据 |
| `drop_edge_index` | 删除 Edge 索引元数据 |
| `get_edge_index` | 获取 Edge 索引元数据 |
| `list_edge_indexes` | 列出所有 Edge 索引 |
| `drop_edge_indexes_by_type` | 删除指定 Edge 类型的所有索引 |

---

## 六、调用链与依赖关系

### 6.1 索引创建调用链

```
用户执行 CREATE TAG INDEX
        ↓
Parser 解析为 AST
        ↓
Planner 生成 CreateTagIndexNode (计划节点)
        ↓
Factory 创建 CreateTagIndexExecutor (执行器)
        ↓
Executor.execute()
        ↓
StorageClient.create_tag_index()
        ↓
IndexMetadataManager.create_tag_index()
        ↓
RedbIndexMetadataManager.create_tag_index()
        ↓
写入 TAG_INDEXES_TABLE (元数据存储)
```

**代码路径：**

1. [CreateTagIndexExecutor](file:///d:/项目/database/graphDB/src/query/executor/admin/index/tag_index.rs#L85)
   ```rust
   let result = storage_guard.create_tag_index("default", &self.index_info);
   ```

2. [RedbStorage.create_tag_index](file:///d:/项目/database/graphDB/src/storage/redb_storage.rs#L621)
   ```rust
   fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
       self.index_metadata_manager.create_tag_index(space, info)
   }
   ```

3. [RedbIndexMetadataManager.create_tag_index](file:///d:/项目/database/graphDB/src/storage/metadata/redb_index_metadata_manager.rs#L34)
   ```rust
   fn create_tag_index(&self, space: &str, index: &Index) -> Result<bool, StorageError> {
       // 写入 TAG_INDEXES_TABLE
   }
   ```

### 6.2 索引查询调用链

```
用户执行 MATCH 查询（带索引条件）
        ↓
Parser 解析为 AST
        ↓
Planner 生成 IndexScanNode (计划节点)
        ↓
Optimizer 应用索引优化规则
        ↓
Factory 创建 IndexScanExecutor (执行器)
        ↓
Executor.execute()
        ↓
StorageClient.lookup_index_with_score()
        ↓
IndexDataManager.lookup_tag_index() / lookup_edge_index()
        ↓
RedbIndexDataManager.lookup_tag_index()
        ↓
扫描 INDEX_DATA_TABLE (索引数据存储)
```

**代码路径：**

1. [IndexScanExecutor](file:///d:/项目/database/graphDB/src/query/executor/search_executors.rs#L723)
   ```rust
   let index_results = storage.lookup_index_with_score(&space_name, &self.index_name, &Value::String(self.query.clone()))
   ```

2. [RedbStorage.lookup_index_with_score](file:///d:/项目/database/graphDB/src/storage/redb_storage.rs#L890)
   ```rust
   let indexed_values = self.index_data_manager.lookup_tag_index(space, &index, value)?;
   ```

3. [RedbIndexDataManager.lookup_tag_index](file:///d:/项目/database/graphDB/src/storage/index/index_data_manager.rs#L219)
   ```rust
   fn lookup_tag_index(&self, space: &str, index: &Index, value: &Value) -> Result<Vec<Value>, StorageError> {
       // 扫描 INDEX_DATA_TABLE
   }
   ```

### 6.3 索引更新调用链

```
用户执行 INSERT/UPDATE 操作
        ↓
Parser 解析为 AST
        ↓
Planner 生成计划节点
        ↓
Executor.execute()
        ↓
StorageClient.insert_vertex() / insert_edge()
        ↓
IndexDataManager.update_vertex_indexes() / update_edge_indexes()
        ↓
RedbIndexDataManager.update_vertex_indexes()
        ↓
写入 INDEX_DATA_TABLE (索引数据存储)
```

**代码路径：**

1. [RedbStorage.insert_vertex](file:///d:/项目/database/graphDB/src/storage/redb_storage.rs)
   ```rust
   self.index_data_manager.update_vertex_indexes(space, vertex_id, tag_name, props)?;
   ```

2. [RedbIndexDataManager.update_vertex_indexes](file:///d:/项目/database/graphDB/src/storage/index/index_data_manager.rs#L64)
   ```rust
   fn update_vertex_indexes(&self, space: &str, vertex_id: &Value, tag_name: &str, props: &[(String, Value)]) -> Result<(), StorageError> {
       // 构建索引键并写入 INDEX_DATA_TABLE
   }
   ```

### 6.4 依赖关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                    查询层 (Query Layer)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Executor (CreateTagIndexExecutor, IndexScanExecutor) │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│              存储客户端接口 (StorageClient Trait)             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RedbStorage (实现)                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│              元数据管理 (IndexMetadataManager)                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RedbIndexMetadataManager (实现)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│              数据管理 (IndexDataManager)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RedbIndexDataManager (实现)                           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│              索引类型定义 (Index, IndexField, etc.)        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  src/index/types.rs                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│              存储引擎 (redb)                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  redb::Database, Table, ReadableTable                 │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 6.5 模块依赖关系

| 模块 | 依赖 | 被依赖 |
|------|------|---------|
| `src/index/types.rs` | `core::Value`, `serde`, `bincode` | 所有索引相关模块 |
| `src/index/binary.rs` | `core::Value`, `core::DateTimeValue`, etc. | `src/index/types.rs` |
| `src/index/service/cache.rs` | `core::Value`, `index::IndexServiceConfig` | 查询层 |
| `src/index/service/stats.rs` | 无 | 查询层 |
| `src/index/service/fulltext.rs` | `core::Value`, `core::error` | 查询层 |
| `src/storage/index/index_data_manager.rs` | `core::Value`, `core::Edge`, `index::Index` | `RedbStorage` |
| `src/storage/metadata/index_metadata_manager.rs` | `core::StorageError`, `index::Index` | `RedbStorage` |
| `src/storage/redb_storage.rs` | `storage::index::RedbIndexDataManager`, `storage::metadata::RedbIndexMetadataManager` | 查询层 |
| `src/query/executor/admin/index/tag_index.rs` | `core::Value`, `index::Index`, `storage::StorageClient` | - |
| `src/query/executor/search_executors.rs` | `core::Value`, `storage::StorageClient` | - |

---

## 七、数据流分析

### 7.1 索引创建数据流

```
CREATE TAG INDEX person_name_idx ON person(name)

1. Parser 解析
   → CreateTagIndexStatement { index_name: "person_name_idx", tag_name: "person", fields: ["name"] }

2. Planner 生成计划
   → CreateTagIndexNode { info: Index { name: "person_name_idx", schema_name: "person", ... } }

3. Factory 创建执行器
   → CreateTagIndexExecutor { index_info: Index { ... } }

4. Executor 执行
   → storage.create_tag_index("default", &index_info)

5. 存储层处理
   → index_metadata_manager.create_tag_index("default", &index_info)
   → 写入 TAG_INDEXES_TABLE: key="default:person_name_idx", value=Index 序列化

6. 返回结果
   → ExecutionResult::Success
```

### 7.2 索引查询数据流

```
MATCH (p:person {name: "Alice"}) RETURN p

1. Parser 解析
   → MatchStatement { patterns: [...], where: Some(Equals(Property("name"), String("Alice"))) }

2. Planner 生成计划
   → IndexScanNode { index_id: 1, scan_type: "EXACT", scan_limits: [...] }

3. Optimizer 优化
   → 应用 IndexScanRule，优化扫描参数

4. Factory 创建执行器
   → IndexScanExecutor { index_id: 1, scan_type: "EXACT", ... }

5. Executor 执行
   → storage.lookup_index_with_score("default", "person_name_idx", Value::String("Alice"))

6. 存储层处理
   → index_data_manager.lookup_tag_index("default", &index, &Value::String("Alice"))
   → 扫描 INDEX_DATA_TABLE，前缀="default:idx:v:default:person_name_idx:"
   → 匹配包含 "Alice" 的键
   → 返回匹配的顶点 ID 列表

7. 获取顶点数据
   → storage.get_vertex("default", vertex_id)

8. 返回结果
   → ExecutionResult::Values([Vertex { ... }])
```

### 7.3 索引更新数据流

```
INSERT VERTEX "v1" WITH LABELS person SET name = "Alice"

1. Parser 解析
   → InsertVertexStatement { vertex_id: "v1", tag: "person", props: [("name", "Alice")] }

2. Planner 生成计划
   → InsertVertexNode { vertex_id: "v1", tag: "person", props: [...] }

3. Factory 创建执行器
   → InsertVertexExecutor { vertex_id: "v1", tag: "person", props: [...] }

4. Executor 执行
   → storage.insert_vertex("default", &vertex_id, &tag, &props)

5. 存储层处理
   → 写入 VERTEXES_TABLE: key="default:v1", value=Vertex 序列化
   → index_data_manager.update_vertex_indexes("default", &vertex_id, "person", &props)
   → 构建索引键: "default:idx:v:default:person:name:Alice:v1"
   → 写入 INDEX_DATA_TABLE: key="default:idx:v:default:person:name:Alice:v1", value=v1

6. 返回结果
   → ExecutionResult::Success
```

---

## 八、存储结构

### 8.1 元数据存储表

| 表名 | 键格式 | 值格式 | 用途 |
|------|---------|---------|------|
| `TAG_INDEXES_TABLE` | `space:index_name` | `Index` 序列化 | Tag 索引元数据 |
| `EDGE_INDEXES_TABLE` | `space:index_name` | `Index` 序列化 | Edge 索引元数据 |
| `TAGS_TABLE` | `space:tag_name` | `TagInfo` 序列化 | Tag 定义 |
| `EDGE_TYPES_TABLE` | `space:edge_type_name` | `EdgeTypeInfo` 序列化 | Edge 类型定义 |
| `SPACES_TABLE` | `space_name` | `SpaceInfo` 序列化 | 空间定义 |

### 8.2 索引数据存储表

| 表名 | 键格式 | 值格式 | 用途 |
|------|---------|---------|------|
| `INDEX_DATA_TABLE` | `space:idx:v/e:index_name:schema_name:prop_name:prop_value:vertex_id/edge_key` | `vertex_id` 或 `src` | 索引数据 |

### 8.3 索引键示例

#### Tag 索引键
```
default:idx:v:person_name_idx:person:name:Alice:v1
default:idx:v:person_name_idx:person:name:Bob:v2
default:idx:v:person_age_idx:person:age:25:v1
```

#### Edge 索引键
```
default:idx:e:friend_weight_idx:friend:weight:0.8:v1:v2
default:idx:e:friend_weight_idx:friend:weight:0.9:v2:v3
```

---

## 九、设计特点

### 9.1 分层架构

1. **清晰的职责划分**：每层专注于特定功能
2. **低耦合**：通过 Trait 接口解耦
3. **高内聚**：相关功能集中在同一模块

### 9.2 Trait 驱动设计

1. **可扩展性**：易于添加新的存储引擎实现
2. **可测试性**：可以轻松创建 Mock 实现
3. **灵活性**：运行时可以切换实现

### 9.3 性能优化

1. **缓存机制**：VersionedCache 和 MemoryIndexCache
2. **二进制编码**：高效的索引键编码/解码
3. **前缀扫描**：支持范围查询优化

### 9.4 并发安全

1. **DashMap**：并发安全的哈希表
2. **Arc<Mutex<>>**：共享可变状态
3. **AtomicU64**：原子计数器

---

## 十、与 Nebula-Graph 对比

| 维度 | Nebula-Graph | GraphDB |
|------|-------------|---------|
| **元数据管理** | `src/meta/` | `src/storage/metadata/` |
| **索引数据管理** | `src/storage/` | `src/storage/index/` |
| **索引执行** | `src/storage/exec/` | `src/query/executor/` |
| **索引优化** | 存储层执行计划 | 查询层优化规则 |
| **编码方式** | IndexKeyUtils | IndexBinaryEncoder |
| **存储引擎** | RocksDB | Redb |
| **架构风格** | 面向对象（继承） | Trait 驱动（组合） |

---

## 十一、总结

GraphDB 的索引系统采用分层架构设计，具有以下特点：

1. **清晰的分层**：定义层、服务层、存储层各司其职
2. **灵活的扩展**：通过 Trait 接口支持多种实现
3. **高效的查询**：缓存、二进制编码、前缀扫描等优化
4. **完整的元数据**：索引定义、Schema 定义分离管理
5. **并发安全**：使用 DashMap、Arc、Mutex 等并发原语

该设计既保持了与 Nebula-Graph 功能对齐，又充分利用了 Rust 的类型系统和所有权模型，实现了高性能、高安全性的索引系统。
