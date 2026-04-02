# Rust 版本序列化功能实现方案

## 概述

本文档详细说明了如何实现 Rust 版本缺失的序列化功能，包括每个功能应该在哪个模块实现，以及可以复用的现有功能。

## 模块职责划分

### 现有模块

| 模块 | 职责 | 可复用功能 |
|------|------|-----------|
| `serialize` | 序列化/反序列化核心逻辑 | 基础导入导出 |
| `compress` | 字符串压缩 | `compress_string`, `compress_with_cache` |
| `async_` | 异步操作支持 | `AsyncIndex`, `AsyncSearchTask` |
| `document` | 文档索引管理 | `Document`, `Field`, `TagSystem` |
| `storage` | 持久化存储 | `StorageInterface`, `FileStorage` |
| `keystore` | 键值存储 | `KeystoreMap`, `KeystoreSet` |
| `index` | 索引核心 | `Index`, `Register` |

---

## 功能实现方案

### 1. 异步导入导出

**实现模块**: `serialize` + `async_`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `async_` 模块的 `AsyncIndex` 和异步任务机制
- `tokio` 异步运行时

**实现方案**:
```rust
// 在 serialize.rs 中添加
pub struct AsyncSerializer {
    config: SerializeConfig,
}

impl AsyncSerializer {
    // 异步导出为 JSON
    pub async fn to_json_async(&self, index: &AsyncIndex) -> Result<String>;
    
    // 异步导出为二进制
    pub async fn to_binary_async(&self, index: &AsyncIndex) -> Result<Vec<u8>>;
    
    // 异步从 JSON 导入
    pub async fn from_json_async(&self, index: &mut AsyncIndex, json_str: &str) -> Result<()>;
    
    // 异步从二进制导入
    pub async fn from_binary_async(&self, index: &mut AsyncIndex, binary_data: &[u8]) -> Result<()>;
}
```

**依赖**: 
- 已有的 `AsyncIndex` 结构
- `tokio::task::spawn_blocking` 将同步序列化转为异步

**优先级**: 高

---

### 2. 分块处理机制

**实现模块**: `serialize`

**实现位置**: `src/serialize.rs`

**新增文件**: `src/serialize/chunk.rs` (可选，如果逻辑复杂)

**复用模块**: 无

**实现方案**:
```rust
// 在 serialize.rs 中添加
pub struct ChunkedSerializer {
    config: SerializeConfig,
    chunk_size: usize,
}

impl ChunkedSerializer {
    // 计算动态块大小
    fn calculate_chunk_size(&self, total_size: usize) -> usize;
    
    // 分块导出
    pub fn export_chunked<F>(&self, index: &Index, callback: F) -> Result<()>
    where F: Fn(ChunkData) -> Result<()>;
    
    // 分块导入
    pub fn import_chunked<F>(&self, index: &mut Index, mut provider: F) -> Result<()>
    where F: FnMut() -> Result<Option<ChunkData>>;
}

pub struct ChunkData {
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub data_type: ChunkDataType,
    pub data: Vec<u8>,
}

pub enum ChunkDataType {
    Registry,
    MainIndex,
    ContextIndex,
}
```

**关键点**:
- 参考 JavaScript 版本的 `map_to_json`、`reg_to_json` 等函数
- 实现动态块大小计算：`chunk_size_map * (chunk_size_reg / size) | 0`
- 支持流式回调处理

**优先级**: 高

---

### 3. Worker 支持

**实现模块**: `async_` + 新建 `worker` 模块

**实现位置**: `src/worker/mod.rs`

**复用模块**: 
- `async_` 模块的异步任务机制
- `rayon` 或 `tokio::task` 用于并行处理

**实现方案**:
```rust
// 新建 src/worker/mod.rs
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: tokio::sync::mpsc::UnboundedSender<WorkerTask>,
}

pub struct Worker {
    id: usize,
    index: Arc<RwLock<Index>>,
}

pub enum WorkerTask {
    Serialize { 
        callback: Box<dyn Fn(SerializeResult) + Send + Sync>,
        field: Option<String>,
        key: String,
    },
    Import {
        key: String,
        data: Vec<u8>,
    },
}

impl WorkerPool {
    // 创建 Worker 池
    pub fn new(worker_count: usize, index: Arc<RwLock<Index>>) -> Self;
    
    // 分发序列化任务
    pub async fn dispatch_serialize(&self, task: WorkerTask) -> Result<()>;
    
    // 分发导入任务
    pub async fn dispatch_import(&self, task: WorkerTask) -> Result<()>;
}
```

**关键点**:
- 参考 JavaScript 版本的 `SUPPORT_WORKER` 逻辑
- 支持字段级别的 Worker 分发
- 使用 `Arc<RwLock<Index>>` 共享索引

**优先级**: 中

---

### 4. 代码注入格式

**实现模块**: `serialize`

**实现位置**: `src/serialize.rs`

**复用模块**: 无

**实现方案**:
```rust
// 在 serialize.rs 中添加
pub struct CodeInjectionSerializer;

impl CodeInjectionSerializer {
    // 生成 JavaScript 代码注入格式
    pub fn to_javascript_injection(&self, index: &Index) -> Result<String>;
    
    // 从代码注入格式解析
    pub fn from_javascript_injection(&self, js_code: &str) -> Result<Index>;
}

impl Index {
    // 生成代码注入格式
    pub fn serialize_as_injection(&self) -> String {
        let mut result = String::new();
        
        // 生成 reg
        if let Register::Set(set) = &self.reg {
            result.push_str("index.reg=new Set([");
            // ... 生成 Set 内容
            result.push_str("]);");
        }
        
        // 生成 map
        result.push_str("index.map=new Map([");
        // ... 生成 Map 内容
        result.push_str("]);");
        
        // 生成 ctx
        result.push_str("index.ctx=new Map([");
        // ... 生成 Context 内容
        result.push_str("]);");
        
        format!("function inject(index){{{}}}", result)
    }
}
```

**关键点**:
- 参考 JavaScript 版本的 `serialize` 函数
- 生成可执行的 JavaScript 代码
- 支持 `withFunctionWrapper` 参数

**优先级**: 低

---

### 5. Document 类型序列化

**实现模块**: `document` + `serialize`

**实现位置**: 
- `src/document/serialize.rs` (新建)
- `src/serialize.rs` (添加 Document 支持)

**复用模块**: 
- `document` 模块的 `Document`、`Field`、`TagSystem`
- `serialize` 模块的现有序列化逻辑

**实现方案**:
```rust
// 新建 src/document/serialize.rs
use crate::document::Document;
use crate::serialize::{SerializeConfig, SerializeFormat};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExportData {
    pub version: String,
    pub created_at: String,
    pub document_info: DocumentInfo,
    pub fields: Vec<FieldExportData>,
    pub tags: Option<TagExportData>,
    pub store: Option<StoreExportData>,
    pub registry: RegistryExportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub field_count: usize,
    pub fastupdate: bool,
    pub store_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldExportData {
    pub name: String,
    pub index_data: IndexExportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagExportData {
    pub tags: HashMap<String, Vec<DocId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportData {
    pub documents: HashMap<DocId, serde_json::Value>,
}

impl Document {
    // 导出文档
    pub fn export(&self, config: &SerializeConfig) -> Result<DocumentExportData>;
    
    // 导入文档
    pub fn import(&mut self, data: DocumentExportData, config: &SerializeConfig) -> Result<()>;
    
    // 导出为 JSON
    pub fn to_json(&self, config: &SerializeConfig) -> Result<String>;
    
    // 从 JSON 导入
    pub fn from_json(json_str: &str, config: &SerializeConfig) -> Result<Document>;
}
```

**关键点**:
- 参考 JavaScript 版本的 `exportDocument` 和 `importDocument`
- 支持字段级别的序列化
- 支持标签（tag）序列化
- 支持文档存储（store）序列化

**优先级**: 高

---

### 6. 配置序列化

**实现模块**: `serialize`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `index` 模块的 `IndexOptions`
- `encoder` 模块的 `EncoderOptions`
- `type` 模块的配置结构

**实现方案**:
```rust
// 在 serialize.rs 中修改 IndexExportData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexExportData {
    pub version: String,
    pub created_at: String,
    pub index_info: IndexInfo,  // 已存在
    pub data: ExportData,
    pub config: IndexConfigExport,  // 新增
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfigExport {
    pub index_options: IndexOptions,
    pub encoder_options: EncoderOptions,
    pub tokenizer_config: TokenizerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub mode: String,
    pub separator: Option<String>,
    pub normalize: bool,
}

impl Index {
    // 导出时包含配置
    pub fn export_with_config(&self, config: &SerializeConfig) -> Result<IndexExportData> {
        // ... 现有导出逻辑
        
        let config_export = IndexConfigExport {
            index_options: self.get_index_options(),
            encoder_options: self.encoder.get_options(),
            tokenizer_config: self.get_tokenizer_config(),
        };
        
        // ...
    }
}
```

**关键点**:
- 导出完整的索引配置
- 确保导入后配置一致性
- 支持配置版本升级

**优先级**: 高

---

### 7. 压缩支持

**实现模块**: `serialize` + `compress`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `compress` 模块的 `compress_string`、`compress_with_cache`
- `flate2` 或 `zstd` 库用于二进制压缩

**实现方案**:
```rust
// 在 serialize.rs 中修改
use crate::compress::{compress_string, decompress_string};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

impl Index {
    // 序列化为二进制（带压缩）
    pub fn to_binary(&self, config: &SerializeConfig) -> Result<Vec<u8>> {
        let data = self.export(config)?;
        let serialized = bincode::serialize(&data)?;
        
        if config.compression {
            self.compress_data(&serialized)
        } else {
            Ok(serialized)
        }
    }
    
    // 从二进制反序列化（带解压缩）
    pub fn from_binary(binary_data: &[u8], config: &SerializeConfig) -> Result<Index> {
        let data = if config.compression {
            self.decompress_data(binary_data)?
        } else {
            binary_data.to_vec()
        };
        
        let export_data: IndexExportData = bincode::deserialize(&data)?;
        // ... 导入逻辑
    }
    
    // 压缩数据
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
    
    // 解压缩数据
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}
```

**关键点**:
- 复用 `compress` 模块的字符串压缩
- 使用 `flate2` 或 `zstd` 进行二进制压缩
- 支持多种压缩算法

**优先级**: 中

---

### 8. 增量更新

**实现模块**: `serialize`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `index` 模块的 `Index::add`、`Index::update`
- `document` 模块的 `Document::add`

**实现方案**:
```rust
// 在 serialize.rs 中添加
pub struct IncrementalSerializer {
    config: SerializeConfig,
}

impl IncrementalSerializer {
    // 导出增量数据
    pub fn export_incremental(&self, index: &Index, since: DateTime<Utc>) -> Result<IncrementalExportData>;
    
    // 导入增量数据
    pub fn import_incremental(&self, index: &mut Index, data: IncrementalExportData) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalExportData {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub operations: Vec<IncrementalOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncrementalOperation {
    Add { id: DocId, content: String },
    Update { id: DocId, content: String },
    Remove { id: DocId },
}

impl Index {
    // 追加导入
    pub fn import_append(&mut self, data: IndexExportData, config: &SerializeConfig) -> Result<()> {
        // 不清空现有索引，直接追加
        self.import_main_index_append(&data.data.main_index)?;
        self.import_context_index_append(&data.data.context_index)?;
        self.import_registry_append(&data.data.registry)?;
        Ok(())
    }
    
    // 合并索引
    pub fn merge(&mut self, other: &Index) -> Result<()> {
        // 合并两个索引
        // ...
    }
}
```

**关键点**:
- 不清空现有索引
- 支持追加新数据
- 支持合并操作

**优先级**: 中

---

### 9. 键值反向映射

**实现模块**: `keystore`

**实现位置**: `src/keystore/mod.rs`

**复用模块**: 
- `keystore` 模块的 `KeystoreMap`、`KeystoreSet`

**实现方案**:
```rust
// 在 keystore/mod.rs 中修改
pub struct KeystoreMap<V> {
    pub index: HashMap<usize, HashMap<String, V>>,
    reverse_map: HashMap<usize, String>,  // 新增：哈希到字符串的反向映射
    capacity: usize,
}

impl<V> KeystoreMap<V> {
    // 新增：获取反向映射
    pub fn get_key_by_hash(&self, hash: usize) -> Option<&String> {
        self.reverse_map.get(&hash)
    }
    
    // 新增：设置键值对（同时更新反向映射）
    pub fn insert_with_reverse(&mut self, key: &str, value: V) -> Result<()> {
        let hash = self.hash_key(key)?;
        self.reverse_map.insert(hash, key.to_string());
        self.index.entry(hash).or_insert_with(HashMap::new).insert(key.to_string(), value);
        Ok(())
    }
}

// 在 serialize.rs 中使用
impl Index {
    // 获取上下文键字符串
    fn get_ctx_key_string(&self, key: &usize) -> Option<String> {
        self.ctx.get_key_by_hash(*key).cloned()
    }
}
```

**关键点**:
- 在 `KeystoreMap` 中维护反向映射
- 支持从哈希值获取原始字符串
- 增加少量内存开销

**优先级**: 中

---

### 10. 流式处理

**实现模块**: `serialize` + `async_`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `async_` 模块的异步流支持
- `tokio::io::AsyncReadExt`、`tokio::io::AsyncWriteExt`

**实现方案**:
```rust
// 在 serialize.rs 中添加
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::Stream;

pub struct StreamSerializer {
    config: SerializeConfig,
}

impl StreamSerializer {
    // 流式导出到写入器
    pub async fn export_to_stream<W>(&self, index: &Index, writer: &mut W) -> Result<()>
    where W: AsyncWriteExt + Unpin;
    
    // 从读取器流式导入
    pub async fn import_from_stream<R>(&self, index: &mut Index, reader: &mut R) -> Result<()>
    where R: AsyncReadExt + Unpin;
    
    // 流式导出为数据流
    pub fn export_as_stream(&self, index: &Index) -> impl Stream<Item = Result<Vec<u8>>>;
}

impl Index {
    // 流式导出到文件
    pub async fn export_to_file_async(&self, path: &str, config: &SerializeConfig) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        let mut file = File::create(path).await?;
        let data = self.to_json(config)?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }
    
    // 从文件流式导入
    pub async fn import_from_file_async(&mut self, path: &str, config: &SerializeConfig) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;
        
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        self.from_json(&contents, config)?;
        Ok(())
    }
}
```

**关键点**:
- 使用 `tokio` 异步 I/O
- 支持流式读写
- 避免一次性加载全部数据

**优先级**: 中

---

### 11. 跨格式兼容

**实现模块**: `serialize`

**实现位置**: `src/serialize/compat.rs` (新建)

**复用模块**: 无

**实现方案**:
```rust
// 新建 src/serialize/compat.rs
pub struct JsCompatSerializer;

impl JsCompatSerializer {
    // 从 JavaScript 格式导入
    pub fn from_js_format(&self, js_data: &JsFormatData) -> Result<IndexExportData>;
    
    // 转换为 JavaScript 格式
    pub fn to_js_format(&self, data: &IndexExportData) -> Result<JsFormatData>;
    
    // 验证 JavaScript 格式
    pub fn validate_js_format(&self, js_data: &JsFormatData) -> Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct JsFormatData {
    pub reg: Option<JsRegData>,
    pub map: Option<JsMapData>,
    pub ctx: Option<JsCtxData>,
}

#[derive(Debug, Deserialize)]
pub struct JsRegData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub data: Vec<String>,
}

// 使用示例
impl Index {
    // 从 JavaScript 导出导入
    pub fn import_from_js(&mut self, js_data: &JsFormatData) -> Result<()> {
        let compat = JsCompatSerializer;
        let data = compat.from_js_format(js_data)?;
        self.import(data, &SerializeConfig::default())
    }
}
```

**关键点**:
- 实现格式转换器
- 支持双向转换
- 验证数据完整性

**优先级**: 低

---

### 12. 版本迁移

**实现模块**: `serialize`

**实现位置**: `src/serialize.rs`

**复用模块**: 无

**实现方案**:
```rust
// 在 serialize.rs 中添加
pub struct VersionManager {
    current_version: String,
    supported_versions: Vec<String>,
}

impl VersionManager {
    // 检查版本兼容性
    pub fn check_compatibility(&self, version: &str) -> Result<()>;
    
    // 升级数据格式
    pub fn upgrade(&self, data: IndexExportData, target_version: &str) -> Result<IndexExportData>;
    
    // 降级数据格式
    pub fn downgrade(&self, data: IndexExportData, target_version: &str) -> Result<IndexExportData>;
}

impl Index {
    // 导入时自动迁移版本
    pub fn import_with_migration(&mut self, data: IndexExportData, config: &SerializeConfig) -> Result<()> {
        let manager = VersionManager::default();
        
        // 检查版本
        manager.check_compatibility(&data.version)?;
        
        // 如果需要，升级到当前版本
        let upgraded_data = if data.version != "0.1.0" {
            manager.upgrade(data, "0.1.0")?
        } else {
            data
        };
        
        self.import(upgraded_data, config)
    }
}
```

**关键点**:
- 版本兼容性检查
- 自动升级/降级
- 支持多版本共存

**优先级**: 低

---

### 13. 内存优化

**实现模块**: `serialize` + 新建 `memory` 模块

**实现位置**: `src/memory/mod.rs` (新建)

**复用模块**: 
- `compress` 模块的压缩功能
- `keystore` 模块的键值存储

**实现方案**:
```rust
// 新建 src/memory/mod.rs
use std::sync::Arc;
use std::collections::VecDeque;

pub struct MemoryPool {
    chunks: VecDeque<Vec<u8>>,
    chunk_size: usize,
    max_chunks: usize,
}

impl MemoryPool {
    // 创建内存池
    pub fn new(chunk_size: usize, max_chunks: usize) -> Self;
    
    // 获取内存块
    pub fn acquire(&mut self) -> Vec<u8>;
    
    // 释放内存块
    pub fn release(&mut self, chunk: Vec<u8>);
    
    // 清空池
    pub fn clear(&mut self);
}

pub struct ZeroCopySerializer {
    memory_pool: Arc<Mutex<MemoryPool>>,
}

impl ZeroCopySerializer {
    // 零拷贝序列化
    pub fn serialize_zero_copy(&self, index: &Index) -> Result<Vec<u8>>;
    
    // 零拷贝反序列化
    pub fn deserialize_zero_copy(&self, data: &[u8]) -> Result<Index>;
}
```

**关键点**:
- 内存池复用
- 减少内存分配
- 零拷贝优化

**优先级**: 低

---

### 14. 并发优化

**实现模块**: `serialize` + `async_`

**实现位置**: `src/serialize.rs`

**复用模块**: 
- `async_` 模块的并发支持
- `rayon` 并行迭代器

**实现方案**:
```rust
// 在 serialize.rs 中添加
use rayon::prelude::*;
use std::sync::{Arc, RwLock};

pub struct ConcurrentSerializer {
    config: SerializeConfig,
    thread_pool: rayon::ThreadPool,
}

impl ConcurrentSerializer {
    // 并发导出
    pub fn export_concurrent(&self, index: &Index) -> Result<IndexExportData> {
        let index = Arc::new(RwLock::new(index.clone()));
        
        // 并行导出主索引
        let main_index = self.thread_pool.install(|| {
            index.read().unwrap()
                .map.index
                .par_iter()
                .map(|(hash, map)| {
                    // 并行处理每个哈希桶
                    (*hash, map.clone())
                })
                .collect()
        });
        
        // 并行导出上下文索引
        let context_index = self.thread_pool.install(|| {
            index.read().unwrap()
                .ctx.index
                .par_iter()
                .map(|(hash, map)| {
                    (*hash, map.clone())
                })
                .collect()
        });
        
        // ...
    }
    
    // 并发导入
    pub fn import_concurrent(&self, index: &mut Index, data: IndexExportData) -> Result<()> {
        // 并行导入主索引
        self.thread_pool.install(|| {
            data.data.main_index.par_iter().for_each(|(term, ids)| {
                // 并行插入
                let term_hash = index.keystore_hash_str(term);
                // ...
            });
        });
        
        // ...
    }
}
```

**关键点**:
- 使用 `rayon` 并行处理
- 读写锁保护共享数据
- 提升大数据集处理速度

**优先级**: 低

---

## 实现优先级总结

### 第一阶段（核心功能）
1. **配置序列化** - `serialize` 模块，高优先级
2. **Document 类型序列化** - `document/serialize.rs`，高优先级
3. **异步导入导出** - `serialize` + `async_`，高优先级
4. **分块处理机制** - `serialize`，高优先级

### 第二阶段（性能优化）
5. **压缩支持** - `serialize` + `compress`，中优先级
6. **增量更新** - `serialize`，中优先级
7. **键值反向映射** - `keystore`，中优先级
8. **流式处理** - `serialize` + `async_`，中优先级

### 第三阶段（高级特性）
9. **Worker 支持** - `worker` 模块，中优先级
10. **并发优化** - `serialize` + `async_`，低优先级
11. **跨格式兼容** - `serialize/compat.rs`，低优先级
12. **版本迁移** - `serialize`，低优先级
13. **内存优化** - `memory` 模块，低优先级
14. **代码注入格式** - `serialize`，低优先级

---

## 模块依赖关系

```
serialize
├── async_ (异步支持)
├── compress (压缩)
├── document (Document 序列化)
├── keystore (键值反向映射)
├── worker (Worker 支持)
└── memory (内存优化)

document/serialize
├── document (Document 结构)
└── serialize (基础序列化)

worker
├── async_ (异步任务)
└── index (索引共享)

memory
├── compress (压缩)
└── keystore (键值存储)
```

---

## 测试策略

每个功能实现后需要添加相应的测试：

1. **单元测试**: 测试单个函数的正确性
2. **集成测试**: 测试模块间的协作
3. **性能测试**: 测试大数据集的处理能力
4. **兼容性测试**: 测试与 JavaScript 版本的兼容性

---

## 总结

本方案详细说明了 14 个缺失功能的实现位置、复用模块和具体实现方案。建议按照优先级分阶段实现，优先完成核心功能，再逐步优化性能和添加高级特性。
