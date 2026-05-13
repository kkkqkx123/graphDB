# compression.rs 与 lob.rs 代码分析及集成方案

## 分析日期

2026-05-13

## 分析范围

- `src/storage/compression.rs`：通用压缩模块
- `src/storage/lob.rs`：大对象存储模块
- `src/transaction/wal/writer/compression.rs`：WAL 压缩模块（作为对比）

---

## 一、compression.rs 分析

### 1.1 现状

定义了两个实体：

- `CompressionType` 枚举：`None` | `Zstd { level: i32 }`
- `Compressor` 结构体：包装 `CompressionType`，提供 `compress()`、`decompress()`、`compress_size_estimate()`

### 1.2 实际使用追踪

| 使用者                           | 使用内容                        | 方式                                    |
| -------------------------------- | ------------------------------- | --------------------------------------- |
| `src/storage/engine/config.rs:7` | `CompressionType`               | 作为 `FlushConfig.compression` 字段类型 |
| `src/storage/mod.rs:49`          | `CompressionType`, `Compressor` | re-export                               |
| 其余位置                         | 无                              | —                                       |

**`Compressor` 结构体及其所有方法在生产代码中零使用。** 没有任何模块构造 `Compressor` 实例或调用 `compress/decompress`。

### 1.3 存在的问题

#### ① Compressor 是死代码

`Compressor` 结构体定义了完整的压缩/解压/大小估算接口，但从未被集成到任何写入路径（flush、编码、存储）中。仅 `CompressionType` 作为配置类型在 `FlushConfig` 中存在。

#### ② 与 WAL 压缩方案重复且不一致

WAL 模块在 `src/transaction/wal/writer/compression.rs` 中有一套完全独立的压缩实现：

| 维度            | storage::compression       | WAL compression                                            |
| --------------- | -------------------------- | ---------------------------------------------------------- |
| 组织方式        | 结构体 + 静态 dispatch     | `Compressor` trait + 动态 dispatch (`Box<dyn Compressor>`) |
| 大小阈值检查    | 无                         | 有（`min_size`，小于阈值不压缩）                           |
| 压缩无效回退    | 无                         | 有（压缩后变大则回退原始数据）                             |
| Level 保留      | `from_u8/to_u8` 丢失 level | 保留                                                       |
| 公共解压 helper | 无                         | 有 `decompress_payload()`                                  |

两个模块使用同一个第三方库 `zstd`，却有两套接口抽象，增加维护成本。

#### ③ `from_u8/to_u8` 序列化丢失精度

```rust
// 序列化时：所有 Zstd 都输出 2，丢弃 level
pub fn to_u8(&self) -> u8 {
    match self {
        CompressionType::None => 0,
        CompressionType::Zstd { .. } => 2,  // level 信息丢失
    }
}
// 反序列化时：硬编码 level=3
pub fn from_u8(value: u8) -> Self {
    match value {
        2 => CompressionType::Zstd { level: 3 }, // level 信息无法恢复
        _ => CompressionType::None,
    }
}
```

若未来需要多级压缩配置，现有序列化方案无法支持。

#### ④ `compress_size_estimate` 粗糙且无用

```rust
CompressionType::Zstd { .. } => data_len + (data_len / 10),
```

仅是一个 `110%` 的线性估算，不准确且未被任何代码调用。

---

## 二、lob.rs 分析

### 2.1 现状

定义了：

- `LargeObjectStore`：`HashMap<LobId, Vec<u8>>`
- `LobId = u64`
- `DEFAULT_LOB_THRESHOLD = 1024`（1KB）
- 提供 `store/load/load_owned/delete/update/contains/should_store_large/stats/clear`

### 2.2 实际使用追踪

| 使用者                  | 使用内容                                                         | 方式      |
| ----------------------- | ---------------------------------------------------------------- | --------- |
| `src/storage/mod.rs:79` | `LargeObjectStore`, `LobId`, `LobStats`, `DEFAULT_LOB_THRESHOLD` | re-export |
| 其余位置                | 无                                                               | —         |

**整个模块零生产引用。** 没有模块 import `LargeObjectStore` 或 `LobId`，`should_store_large` 从未被调用。

### 2.3 存在的问题

#### ① 纯内存实现，无持久化

整个 store 就是一个 `HashMap<u64, Vec<u8>>`。数据库进程退出后所有数据丢失，无法作为真正的数据库大对象存储使用。

- 无序列化/反序列化
- 无 WAL 日志
- 无快照/恢复机制

#### ② 与存储引擎完全割裂

在 graph database 中，LOB 存储需要与列存储层打通：

- Vertex/Edge 的属性列需要能引用 `LobId`
- 属性编码器需要判断 `should_store_large`，将大值路由到 LOB 存储
- 查询时需要能透明地从 LOB 恢复数据

当前没有任何字段/类型引用 `LobId`，LOBs 的概念在整个存储引擎中不存在。

#### ③ ID 生成永不回收

```rust
next_id: AtomicU64,
// store 时：
let id = self.next_id.fetch_add(1, Ordering::SeqCst);
```

ID 永远单调递增，即使 `delete` 后也不回收。长期运行下 ID 空间浪费，且 `AtomicU64` 无实际作用——数据访问需要 `&mut self`，是独占的。

#### ④ 虚假的并发设计

- `next_id` 用 `AtomicU64` 带 `SeqCst` 排序——暗示多线程共享
- 但 `objects: HashMap` 的所有访问都需要 `&mut self` ——实际上是独占访问
- 两者矛盾。要么统一用 `Mutex<LargeObjectStoreInner>`，要么用 `RwLock<HashMap<LobId, Vec<u8>>>`

#### ⑤ 无内存管理

- 没有 eviction 策略（LRU、TTL 等）
- 没有大小上限
- 没有 disk offloading
- 大对象的无限累加会导致 OOM

#### ⑥ 无引用计数 / GC

无法追踪哪些 Vertex/Edge 记录引用了哪个 LobId。delete 后 ID 虽然可复用，但 orphan 对象无从识别。

#### ⑦ `load_owned` 冗余

等价于 `load(id).map(|v| v.to_vec())`，提供 `Clone` 约束即可。

---

## 三、正确集成方案

### 3.1 compression.rs 集成方案

#### 目标

消除与 WAL 压缩的重复，建立统一的通用压缩抽象层，并实际集成到 Flush / 编码路径中。

#### 方案：废弃当前 Compressor，统一到 trait 设计

**Step 1：在 storage 层定义统一的压缩 trait**

```rust
// src/storage/compression.rs
pub trait StorageCompressor: Send + Sync {
    fn compress(&self, data: &[u8]) -> StorageResult<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> StorageResult<Vec<u8>>;
}

pub struct ZstdCompressor {
    level: i32,
    min_compress_size: usize,
}

impl StorageCompressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        if data.len() < self.min_compress_size {
            return Ok(data.to_vec());
        }
        let compressed = zstd::encode_all(data, self.level)
            .map_err(|e| StorageError::compress_error(e.to_string()))?;
        if compressed.len() < data.len() {
            Ok(compressed)
        } else {
            Ok(data.to_vec())  // 压缩后更大则回退
        }
    }
    fn decompress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // 无法区分是否压缩过，需要调用方通过 CompressionType 标记决定
        // 或者在 encode/decode 时写 header 标记
        zstd::decode_all(data)
            .map_err(|e| StorageError::decompress_error(e.to_string()))
    }
}
```

**Step 2：将 `CompressionType` 保留为元信息，在 FlushConfig 和 Record 头部中使用**

优化 `from_u8/to_u8` 以保留 level：

```rust
pub fn to_u8(&self) -> u8 {
    match self {
        CompressionType::None => 0,
        CompressionType::Zstd { level } => {
            // 使用低 4 位存 compression type，高 4 位存 level
            0x20 | (level.min(15) as u8) << 4
        }
    }
}
```

**Step 3：在 Flush 路径中使用 Compressor**

```
FlushConfig.compression -> 构建 ZstdCompressor -> encode 时调用 compress()
```

**Step 4：废弃 WAL 中独立的 `writer::compression`，统一引用 storage 层**

- 将 `NoopCompressor` 保留（作为默认实现）
- WAL 通过 `StorageCompressor` trait 获取压缩能力，无需重复定义

---

### 3.2 lob.rs 集成方案

#### 目标

将 `LargeObjectStore` 从死代码重构为真正可用的 LOB 存储层，并与列存储引擎打通。

#### 方案：分层重构

```
┌────────────────────────────────────────────────┐
│              Storage Engine                      │
│  ┌──────────────────────────────────────────┐   │
│  │  ColumnStore / PropertyTable              │   │
│  │  (检测大值 -> 写入 LOB -> 存储 LobId)     │   │
│  └──────────────┬───────────────────────────┘   │
│                 ↓ LobId                          │
│  ┌──────────────────────────────────────────┐   │
│  │           LargeObjectStore                │   │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────┐  │   │
│  │  │ MemTable │  │ WAL Log  │  │  SST /  │  │   │
│  │  │ (活跃)   │  │ (持久化) │  │ 磁盘文件 │  │   │
│  │  └─────────┘  └──────────┘  └─────────┘  │   │
│  └──────────────────────────────────────────┘   │
└────────────────────────────────────────────────┘
```

#### 详细步骤

**Step 1：重构数据结构**

```rust
pub struct LargeObjectStore {
    /// 活跃内存表（当前写入和读取的热数据）
    memtable: HashMap<LobId, Arc<[u8]>>,
    /// 持久化存储后端（可选）
    persistent: Option<Box<dyn LobPersistence>>,
    /// 引用计数（追踪 record -> LobId 的引用）
    refcount: HashMap<LobId, usize>,
    /// WAL 日志（崩溃恢复用）
    wal: Option<Arc<dyn LobWalLogger>>,
    /// 配置
    config: LobConfig,
    stats: LobStoreStats,
}

pub struct LobConfig {
    pub threshold: usize,           // 大对象阈值（默认 1KB）
    pub max_memory: usize,          // 内存上限
    pub persist_dir: Option<PathBuf>, // 持久化目录
}

/// 持久化后端接口（支持文件系统 / SST 等多种实现）
pub trait LobPersistence: Send + Sync {
    fn write(&self, id: LobId, data: &[u8]) -> StorageResult<()>;
    fn read(&self, id: LobId) -> StorageResult<Option<Vec<u8>>>;
    fn delete(&self, id: LobId) -> StorageResult<()>;
    fn flush(&self) -> StorageResult<()>;
}
```

**Step 2：打通列存储 -> LOB 路径**

在属性编码器（如 `vertex::encoding`）中添加 LOB 检测逻辑：

```rust
// 伪代码：在属性值序列化时
fn encode_property_value(value: &Value) -> StorageResult<EncodedValue> {
    let raw = serialize(value);
    if lob_store.should_store_large(raw.len()) {
        let lob_id = lob_store.store(raw);
        EncodedValue::LobRef(lob_id)  // 新的变体
    } else {
        EncodedValue::Inline(raw)
    }
}

// 在属性值反序列化时
fn decode_property_value(encoded: &EncodedValue) -> StorageResult<Value> {
    match encoded {
        EncodedValue::Inline(data) => deserialize(data),
        EncodedValue::LobRef(id) => {
            let data = lob_store.load(id)?;
            deserialize(data)
        }
    }
}
```

需要在 `EncodingFormat` 中新增 `LobRef` 变体。

**Step 3：引用计数 + GC**

```rust
impl LargeObjectStore {
    pub fn store(&mut self, data: Vec<u8>) -> LobId {
        let id = self.allocate_id();
        self.memtable.insert(id, Arc::from(data));
        self.refcount.insert(id, 1);
        id
    }

    pub fn ref_inc(&mut self, id: LobId) -> StorageResult<()> {
        let count = self.refcount.get_mut(&id)
            .ok_or(StorageError::not_found(format!("LOB {} not found", id)))?;
        *count += 1;
        Ok(())
    }

    pub fn ref_dec(&mut self, id: LobId) -> StorageResult<()> {
        let count = self.refcount.get_mut(&id)
            .ok_or(StorageError::not_found(format!("LOB {} not found", id)))?;
        *count -= 1;
        if *count == 0 {
            self.evict_or_delete(id);
        }
        Ok(())
    }
}
```

**Step 4：持久化 + WAL**

- 每次 `store` 写入 WAL（`LobPut(id, data)`）
- 每次 `delete` 写入 WAL（`LobDel(id)`）
- Crash Recovery 时回放 WAL 重建 LOB 状态
- `flush` 时将 `memtable` 中的数据写入持久化后端，完成后截断 WAL

**Step 5：内存上限控制**

```rust
fn check_memory(&mut self) {
    while self.memtable_size > self.config.max_memory {
        // LRU 淘汰：将最久未访问的 LOB 写入持久化后端，从内存移除
        let victim = self.lru_list.pop_back();
        if let Some(backend) = &self.persistent {
            backend.write(victim.id, &victim.data)?;
        }
        self.memtable.remove(&victim.id);
    }
}
```

---

## 四、优先级建议

| 优先级 | 模块           | 行动                                                                                                              |
| ------ | -------------- | ----------------------------------------------------------------------------------------------------------------- |
| P0     | compression.rs | 统一 trait 设计，消除与 WAL 的重复。工作量小，收益明确                                                            |
| P1     | lob.rs         | 先确认列存储层是否需要 LOB（当前数据量下 1KB 阈值是否合理）；若需要，按上述方案分层重构；若不需要，直接移除该模快 |
