# Tantivy 存储层设计与改进方案

## 一、存储架构总览

tantivy 的存储层分为两个逻辑层级：

```
┌─────────────────────────────────────────────────┐
│  Store Layer (文档存储)                           │
│  StoreWriter / StoreReader                       │
│  BlockCompressor (LZ4/Zstd/None)                 │
│  SkipIndex (跳表) + BlockCache (LRU)             │
├─────────────────────────────────────────────────┤
│  Directory Layer (虚拟文件系统)                    │
│  Directory trait                                 │
│  ├── RamDirectory (测试用)                       │
│  ├── MmapDirectory (生产用, mmap + 文件)          │
│  └── ManagedDirectory (装饰器: GC + CRC校验)      │
└─────────────────────────────────────────────────┘
```

- **Directory 层**：WORM (Write Once Read Many) 虚拟文件系统抽象，提供统一的文件读写接口
- **Store 层**：行式文档存储，按块压缩 (LZ4/Zstd/None) + 跳表索引 + LRU 缓存

## 二、Directory 层分析

### 核心抽象

`Directory` trait (`src/directory/directory.rs:107`) 定义了虚拟文件系统操作：

- `open_read` / `open_write` / `atomic_write` / `delete` / `exists`
- `acquire_lock` / `watch` (文件变更通知)
- `sync_directory` (持久化保证)

写入端统一通过 `WritePtr = BufWriter<Box<dyn TerminatingWrite + Send + Sync>>` 抽象。

### 三种实现

| 实现 | 读策略 | 写策略 | 用途 |
|---|---|---|---|
| **RamDirectory** | `HashMap<PathBuf, FileSlice>` + `RwLock` | `VecWriter` 积累，flush 写回 HashMap | 单元测试 |
| **MmapDirectory** | `MmapCache` (Weak 引用缓存) + `memmap2::Mmap` | `SafeFileWriter` + `BufWriter`，terminate 时 `sync_data` | 生产环境 |
| **ManagedDirectory** | 装饰器：剥离 Footer，校验 CRC + 版本号 | 注册文件到 `.managed.json`，添加 CRC Footer | 包装上述实现 |

### 关键机制

1. **文件 Footer** (`src/directory/footer.rs`)：每个文件末尾附加 CRC32 + 版本号，读取时校验完整性
2. **GC 安全机制** (`managed_directory.rs:109-200`)：两阶段加锁防止读写竞争
3. **mmap 缓存** (`mmap_directory/mod.rs:81-151`)：基于 `Weak<dyn Deref<Target=[u8]>>` 自动释放
4. **文件变更监听** (`mmap_directory/file_watcher.rs`)：基于轮询 (polling) 的 CRC 校验变更检测

## 三、Store 层分析

### 写入流程

```
document → current_block 积累 → 超 ~16KB → compress(LZ4/Zstd)
  → CountingWriter<WritePtr> → 注册 Checkpoint 到 SkipIndexBuilder
```

**关键设计**：
- `DedicatedThread` 模式：通过 `SyncSender<BlockCompressorMessage>` 通道 (容量 3) 将压缩卸载到后台线程 (`store_compressor.rs:167-227`)
- Merge 优化 (`stack`)：直接拷贝压缩块 + 偏移量平移，避免解压再压缩 (`store_compressor.rs:125-143`)
- SkipIndex：多层跳表 (`CHECKPOINT_PERIOD=8`) 实现 O(log n) 定位

### 读取流程

```
doc_id → SkipIndex::seek → Checkpoint → 读压缩块 → decompress
  → LRU Cache (100 blocks 硬编码) → block slice → 提取目标文档
```

## 四、设计合理性评价

### 合理之处

1. **WORM + 装饰器模式**：`ManagedDirectory` 叠加在具体实现上，GC/CRC校验/版本兼容与底层存储解耦，清晰可复用
2. **SkipIndex + BlockCache 读写分离**：写时顺序追加，读时随机访问 + LRU 缓存，典型的日志结构读优化策略
3. **Merge 零拷贝 Stacking**：避免 merge 时解压再压缩，大幅降低 CPU
4. **文件级 CRC 校验**：每个文件独立校验，损坏范围可控；版本号嵌入 Footer 可处理向前兼容
5. **Weak 引用 mmap 缓存**：无需手动释放，mmap 随最后持有者自动解除

### 可改进之处

1. **文件监听器代码重复**：`src/directory/file_watcher.rs` 和 `src/directory/mmap_directory/file_watcher.rs` 完全相同，前者无人引用
2. **VecWriter drop 行为与文档不符**：文档说未 flush 时 panic，实际代码只是 warn，导致测试中可能遗漏 bug
3. **MmapCache 无自动清理**：死亡 `Weak` 引用只在 `get_cache_info()` 中清理，正常读写路径不清理，导致缓存持续膨胀
4. **GC 存在竞争窗口**：merge 创建文件后 meta.json 更新前若触发 GC，新创建的 segment 文件可能被误删
5. **BlockCache 容量硬编码**：`DOCSTORE_CACHE_CAPACITY = 100` 固定，不支持根据可用内存自适应
6. **WritePtr 双重动态分发**：`BufWriter<Box<dyn TerminatingWrite>>` 每字节至少两次间接调用
7. **异步支持是 "bolt-on"**：`quickwit` feature 下只是 `spawn_blocking` 卸载，非真正异步 I/O

## 五、改进方案

### 方案 A：删除重复的 file_watcher.rs

- 文件：`src/directory/file_watcher.rs`
- 操作：直接删除，`mmap_directory/file_watcher.rs` 是唯一的实际使用者
- 风险：无

### 方案 B：修复 VecWriter drop 行为

- 文件：`src/directory/ram_directory.rs`
- 操作：将 `warn!` 改为 `panic!`，与文档声明的行为一致
- 风险：极小，仅在测试中触发，且符合文档预期

### 方案 C：MmapCache 增加死亡引用自动清理

- 文件：`src/directory/mmap_directory/mod.rs`
- 操作：在 `get_mmap()` 中当缓存大小超过阈值时触发 `remove_weak_ref()` 清理
- 风险：低

### 方案 D：修复 GC 竞争窗口

- 文件：`src/directory/managed_directory.rs`
- 操作：将 GC 中的 `read()` 锁改为 `write()` 锁，并扩展锁范围覆盖整个 GC 过程
- 风险：中，`register_file_as_managed` 在 GC 期间会被阻塞

### 可选方案（当前暂缓实施）

- BlockCache 容量配置化：已有 `cache_num_blocks` 参数，对外 API 够用
- 消除 WritePtr 动态分发：涉及过多文件修改，收益有限
