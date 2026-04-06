# Inversearch 数据持久化机制分析

## 一、整体架构

该包提供了**三层持久化机制**：

```
┌─────────────────────────────────────────────────────────────┐
│                    持久化架构层次                            │
├─────────────────────────────────────────────────────────────┤
│  第一层: StorageInterface (存储后端抽象)                     │
│    ├── MemoryStorage  (内存存储)                             │
│    ├── FileStorage    (文件存储)                             │
│    └── RedisStorage   (Redis存储)                            │
├─────────────────────────────────────────────────────────────┤
│  第二层: Serialize (序列化/反序列化)                         │
│    ├── JSON格式                                             │
│    └── Binary格式 (bincode)                                 │
├─────────────────────────────────────────────────────────────┤
│  第三层: Keystore (底层数据结构)                             │
│    ├── KeystoreMap<K, V>                                    │
│    ├── KeystoreSet<T>                                       │
│    └── KeystoreArray<T>                                     │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、存储接口层 (`src/storage/mod.rs`)

### 1. `StorageInterface` Trait 定义

```rust
pub trait StorageInterface: Send + Sync {
    async fn mount(&mut self, index: &Index) -> Result<()>;      // 挂载索引
    async fn open(&mut self) -> Result<()>;                      // 打开连接
    async fn close(&mut self) -> Result<()>;                     // 关闭连接
    async fn destroy(&mut self) -> Result<()>;                   // 销毁数据库
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;  // 提交变更
    async fn get(...) -> Result<SearchResults>;                  // 获取术语结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;  // 富化结果
    async fn has(&self, id: DocId) -> Result<bool>;              // 检查ID存在
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;     // 删除ID
    async fn clear(&mut self) -> Result<()>;                     // 清空数据
    async fn info(&self) -> Result<StorageInfo>;                 // 获取存储信息
}
```

### 2. 三种存储实现

| 存储类型 | 用途 | 持久化方式 |
|---------|------|-----------|
| `MemoryStorage` | 测试/开发 | 内存HashMap，无持久化 |
| `FileStorage` | 单机应用 | JSON文件，`save_to_file()`/`load_from_file()` |
| `RedisStorage` | 分布式/生产 | Redis键值存储，通过`SET`/`GET`命令 |

---

## 三、序列化层 (`src/serialize.rs`)

### 1. 导出数据结构

```rust
pub struct IndexExportData {
    pub version: String,           // 版本号
    pub created_at: String,        // 创建时间
    pub index_info: IndexInfo,     // 索引配置信息
    pub config: IndexConfigExport, // 配置导出
    pub data: ExportData,          // 核心数据
}

pub struct ExportData {
    pub main_index: HashMap<String, Vec<u64>>,           // 主索引: term -> doc_ids
    pub context_index: HashMap<String, HashMap<String, Vec<u64>>>,  // 上下文索引
    pub registry: RegistryData,                          // 注册表
}
```

### 2. 序列化方法

| 方法 | 格式 | 特点 |
|-----|------|-----|
| `to_json()` | JSON | 可读性好，便于调试 |
| `from_json()` | JSON | 反序列化 |
| `to_binary()` | bincode | 高性能，体积小 |
| `from_binary()` | bincode | 反序列化 |

---

## 四、底层数据结构 (`src/keystore/mod.rs`)

### 1. KeystoreMap - 核心索引存储

```rust
pub struct KeystoreMap<K, V> {
    pub index: HashMap<usize, HashMap<K, V>>,  // 哈希分桶存储
    pub refs: Vec<HashMap<K, V>>,               // 引用数组
    pub size: usize,
    pub bit: usize,                             // 哈希位数
}
```

**关键特性**：
- 使用 **LCG哈希函数** 进行分桶：`crc(key) % (1 << bit)`
- 支持快速查找：`get()`, `set()`, `has()`, `delete()`
- 自动扩容管理

### 2. KeystoreSet - 文档ID集合

```rust
pub struct KeystoreSet<T> {
    pub index: HashMap<usize, HashSet<T>>,
    pub refs: Vec<HashSet<T>>,
    pub size: usize,
    pub bit: usize,
}
```

### 3. KeystoreArray - 数组存储

```rust
pub struct KeystoreArray<T> {
    pub index: HashMap<usize, Vec<T>>,
    pub refs: Vec<Vec<T>>,
    pub size: usize,
    pub bit: usize,
}
```

---

## 五、数据持久化流程

### 1. 提交流程 (Commit)

```
Index.add() → 内存索引更新
     ↓
Storage.commit(index) → 从Index导出数据
     ↓
┌─────────────────────────────────────┐
│  FileStorage: save_to_file()        │
│    → JSON序列化 → 写入文件          │
├─────────────────────────────────────┤
│  RedisStorage: SET命令              │
│    → JSON序列化 → Redis存储         │
└─────────────────────────────────────┘
```

### 2. 加载流程 (Load)

```
Storage.open() → 建立连接
     ↓
┌─────────────────────────────────────┐
│  FileStorage: load_from_file()      │
│    → 读取文件 → JSON反序列化        │
├─────────────────────────────────────┤
│  RedisStorage: GET命令              │
│    → 从Redis读取 → JSON反序列化     │
└─────────────────────────────────────┘
     ↓
Index.import(data) → 恢复内存索引
```

---

## 六、Redis存储实现细节 (`src/storage/redis.rs`)

### 键命名规则

| 数据类型 | 键格式 | 示例 |
|---------|-------|-----|
| 索引数据 | `{prefix}:index:{term}` | `inversearch:index:hello` |
| 上下文数据 | `{prefix}:ctx:{context}:{term}` | `inversearch:ctx:default:hello` |
| 文档数据 | `{prefix}:doc:{doc_id}` | `inversearch:doc:1` |

### 配置选项

```rust
pub struct RedisStorageConfig {
    pub url: String,                    // Redis连接URL
    pub pool_size: usize,               // 连接池大小
    pub connection_timeout: Duration,   // 连接超时
    pub key_prefix: String,             // 键前缀
}
```

---

## 七、Document级持久化 (`src/document/serialize.rs`)

支持多字段文档的完整持久化：

```rust
pub struct DocumentExportData {
    pub version: String,
    pub created_at: String,
    pub document_info: DocumentInfo,
    pub fields: Vec<FieldExportData>,      // 多字段索引数据
    pub tags: Option<TagExportData>,       // 标签数据
    pub store: Option<StoreExportData>,    // 原始文档存储
    pub registry: RegistryExportData,       // 注册表
}
```

---

## 八、总结

Inversearch 的数据持久化具有以下特点：

1. **分层设计**：存储接口 → 序列化 → 底层数据结构，职责清晰
2. **多后端支持**：内存、文件、Redis三种存储后端可切换
3. **双格式序列化**：JSON（可读）和Binary（高性能）两种格式
4. **哈希分桶优化**：Keystore使用LCG哈希分桶，提高查找效率
5. **异步支持**：所有存储操作均为异步，适合高并发场景
6. **完整数据导出**：支持索引配置、数据、注册表的完整导出/导入
