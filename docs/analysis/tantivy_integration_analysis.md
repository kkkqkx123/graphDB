# Tantivy 集成与存储实现分析

## 一、架构总览

```
Query Layer (fulltext_search.rs / fulltext_planner.rs / fulltext_validator.rs)
    ↓
Storage Layer (FulltextStorage — 已废弃，无人调用)     Sync Layer (SyncManager → SyncCoordinator)
    ↓                                                   ↓
FulltextIndexManager (manager.rs)  管理 N 个 TantivySearchEngine 实例
    ↓
TantivySearchEngine (tantivy_index.rs)  实现 SearchEngine trait
    ↓
vendored tantivy (crates/tantivy/, 0.26.0 fork + jieba tokenizer)
    ↓
Directory Layer (MmapDirectory + ManagedDirectory) → Store Layer (LZ4/Zstd blocks)
```

### 关键设计决策

- tantivy 完整 fork 在 `crates/tantivy/`，添加了 `jieba` feature（中文分词）
- `EngineType::Bm25` 是唯一的搜索引擎类型（`inversearch` 已移除）
- 每个索引对应一个 `(space_id, tag, field)` 三元组，物理目录 `space_ft_{space_id}_{tag}_{field}`
- 索引固定双字段 schema：`id(STRING|STORED)` + `text(TextOptions with jieba)`

### 文件清单

| 文件 | 角色 |
|------|------|
| `src/search/tantivy_index.rs` | TantivySearchEngine — 核心适配器 |
| `src/search/engine.rs` | SearchEngine trait + EngineType::Bm25 |
| `src/search/manager.rs` | FulltextIndexManager — 索引生命周期管理 |
| `src/search/factory.rs` | SearchEngineFactory — 引擎工厂 |
| `src/search/config.rs` | FulltextConfig / TantivyConfig / SyncConfig |
| `src/search/result.rs` | SearchResult / IndexStats 等结果类型 |
| `src/search/metrics.rs` | MetricsSearchEngine — 监控装饰器 |
| `src/search/metadata.rs` | IndexKey / IndexMetadata |
| `src/search/error.rs` | SearchError |
| `src/storage/extend/fulltext_storage.rs` | FulltextStorage — 已废弃的遗留路径 |
| `src/sync/coordinator/coordinator.rs` | SyncCoordinator — 事务协调 |
| `src/sync/external_index/fulltext_client.rs` | FulltextClient — 同步适配器 |
| `src/sync/external_index/trait_def.rs` | ExternalIndexClient trait |
| `src/sync/batch/processor.rs` | GenericBatchProcessor — 批处理引擎 |
| `crates/tantivy/` | Vendored tantivy 0.26.0 |

---

## 二、数据同步流程

### 主路径：事务性批量同步

```
Graph 变更 → sync_wrapper.rs → SyncManager
    → SyncCoordinator::buffer_operation(txn_id, ChangeContext)
    → TransactionBatchBuffer (按 txn_id 暂存所有 IndexOperation)
    → SyncCoordinator::commit_transaction()
      → take_operations() 按 (space, tag, field) 分组
      → GenericBatchProcessor<FulltextClient>::execute_now()
        → TantivySearchEngine::index_batch() / delete_batch()
      → commit_all() → IndexWriter::commit()
```

### 遗留路径：FulltextStorage（已废弃）

```
FulltextStorage::index_vertex() / delete_vertex() 等
    → FulltextIndexManager::get_engine()
    → TantivySearchEngine::index() / delete()
```

**现状**：`FulltextStorage` 在 `GraphStorageContext` 中存在但 `fulltext_storage()` 从未被任何写路径调用。实际的全文索引写入全部通过 `SyncManager` 事务路径完成。

### 容错机制

- 指数退避重试 (`default_local_retry_config`)
- Dead Letter Queue 捕获永久失败的操作
- FailOpen / FailClosed 策略
- 后台定时 flush（1s 间隔）

---

## 三、现有问题与改进方案

### P0（紧急 — 数据安全）

| # | 问题 | 位置 | 描述 | 修复 |
|---|------|------|------|------|
| 1 | GC 竞争窗口 | `managed_directory.rs:109-191` | GC 计算完存活文件后过早释放 `META_LOCK`，此时若 merge 创建的 segment 文件已注册到 managed_paths 但尚未被 meta.json 引用，会被 GC 误删 | 将 `drop(_meta_lock)` 移到文件删除和 `save_managed_paths` 之后 |
| 2 | FulltextStorage 死代码 | `fulltext_storage.rs` | 存在从未被调用的直接写入路径，与 SyncManager 双路径并行但未使用 | 标记 `#[deprecated]`，引导未来的开发者使用 SyncManager |

### P1（重要 — 性能与正确性）

| # | 问题 | 位置 | 描述 | 修复 |
|---|------|------|------|------|
| 1 | 每次 search 创建新 Reader | `tantivy_index.rs:125` | `self.index.reader()` 每次触发 reopen，没有缓存 | 添加 `CachedReader`：缓存 reader 实例，通过 `watch` 监听 meta.json 变更自动刷新 |
| 2 | MmapCache 死亡引用清理 | `mmap_directory/mod.rs` | 分析文档称缓存膨胀，但实际代码已在 `get_mmap()` 中检查 `max_entries` 并调用 `remove_weak_ref()` | **已修复** — 当前代码正确 |

### P2（改进 — 功能完整性）

| # | 问题 | 位置 | 描述 | 修复 |
|---|------|------|------|------|
| 1 | index_size 恒为 0 | `tantivy_index.rs:189` | stats() 返回的 index_size 始终是 0 | 扫描索引目录中 segment 文件并求和 |
| 2 | Highlights 未实现 | `tantivy_index.rs:149` | `SearchResult.highlights` 始终为 None | 使用 tantivy 的 `SnippetGenerator` 生成高亮片段 |
| 3 | VecWriter drop 行为 | `ram_directory.rs:38-49` | 分析文档称 "代码只 warn"，实际代码 **正确 panic** | **已正确实现** — 无需修改 |
| 4 | BlockCache 容量 | `store/reader.rs:25` | 分析文档称硬编码，实际 `IndexReaderBuilder.doc_store_cache_num_blocks()` 已暴露配置接口 | **已可配置** — 无需修改 |

### P3（长远 — 架构演进）

| # | 问题 | 位置 | 描述 | 方向 | 推荐 |
|---|------|------|------|------|------|
| 1 | Schema 硬编码 | `tantivy_index.rs:37-50` | 固定双字段，不支持每个索引自定义 schema | 启用 `TantivyConfig.tokenizer_name`，使 `build_schema()` 使用配置的分词器 | 实施 |
| 2 | WritePtr 双动态分发 | `directory/mod.rs:55` | `BufWriter<Box<dyn TerminatingWrite>>` 每字节两次 vtable 间接调用 | 改为枚举分发，消除 `dyn` | 不推荐（收益极小） |
| 3 | 异步 I/O 非原生 | `quickwit feature` | 仅 spawn_blocking 卸载，实际 `read_bytes_async` 全为同步 stub | 实现真正的异步 I/O（tokio/io_uring） | 不推荐（单机同步即可） |

---

#### P3.1 详细实施方案：Schema 硬编码 → 支持自定义分词器

**现状分析**：
- `TantivyConfig.tokenizer_name: Option<String>` 已定义但**完全未使用**
- `build_schema()` 在 `tantivy_index.rs:37-50` 硬编码 `jieba` 分词器
- `index.tokenizers().register("jieba", JiebaTokenizer::default())` 固定注册在 `open_or_create`:81
- tantivy 内置分词器（`raw`、`default`、`en_stem` 等）从未注册

**修改方案**：

1. **`src/search/tantivy_index.rs` — `build_schema()` 接受 `&TantivyConfig`**:
   - 当 `config.tokenizer_name` 为 `Some(name)` 时，使用该名称作为分词器
   - 当为 `None` 时，默认使用 `"jieba"`（向后兼容）
   - 注册对应分词器：jieba → `JiebaTokenizer`，其他 → tantivy 内置

2. **`src/search/tantivy_index.rs` — 内置分词器注册**：
   - tantivy 0.26 内置 `raw` 和 `default` 分词器已自动注册
   - 仅在 `tokenizer_name == "jieba"` 时才注册 `JiebaTokenizer`
   - 其余名称直接透传给 tantivy 内置查找机制

3. **`src/search/tantivy_index.rs` — `open_or_create()`**：
   - 将 `_config` 改为 `config`（去掉下划线前缀）
   - 传入 `&config` 给 `build_schema()`

4. **存储路径**：
   - Schema 由 tantivy 自动持久化到 `meta.json`
   - 重建时 `Index::open_in_dir()` 从 `meta.json` 加载 schema
   - 分词器名称在每次 `open_or_create()` 时重新配置即可

**涉及文件**：`src/search/tantivy_index.rs`

**代码变更示例**：
```rust
fn build_schema(config: &TantivyConfig) -> (Schema, Field, Field) {
    let tokenizer_name = config.tokenizer_name.as_deref().unwrap_or("jieba");
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer_name)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let text_field = schema_builder.add_text_field("text", text_options);
    let schema = schema_builder.build();
    (schema, id_field, text_field)
}
```

---

#### P3.2 详细分析：WritePtr 双动态分发

**现状分析**：
- `WritePtr = BufWriter<Box<dyn TerminatingWrite + Send + Sync>>`（`directory/mod.rs:55`）
- `ManagedDirectory::open_write()` 进一步包装：
  ```
  内层目录.open_write(path)        → BufWriter<Box<dyn TWrite>>
     .into_inner()                 → Box<dyn TWrite>
   FooterProxy::new(inner)         → FooterProxy<Box<dyn TWrite>>
   Box::new(footer_proxy)          → Box<dyn TWrite>
   BufWriter::new(boxed)           → BufWriter<Box<dyn TWrite>>
  ```
- 每次 `write()` 的 vtable 调用链（BufWriter 缓冲满时，约每 8KB）：
  ```
  BufWriter::write()                     [static]
    → Box<dyn TWrite>::write()           [vtable #1: FooterProxy]
      → FooterProxy::write()             [CRC + delegate]
        → Box<dyn TWrite>::write()       [vtable #2: SafeFileWriter]
  ```
- `terminate()` 时另有 2 次 vtable 调用

**枚举去虚拟化方案**：
- 定义 `WriterKind` 枚举包含所有具体 writer 类型：
  ```rust
  pub enum WriterKind {
      File(SafeFileWriter),
      Vec(VecWriter),
  }
  impl Write for WriterKind { ... }
  impl TerminatingWrite for WriterKind { ... }
  ```
- 定义 `WritePtr` 为包含或不含 FooterProxy 的 enum：
  ```rust
  pub enum WritePtr {
      Plain(BufWriter<WriterKind>),
      Managed(BufWriter<FooterProxy<WriterKind>>),
  }
  ```
- 效果：`write()` 零 vtable 调用（仅 enum match，可被编译器去虚拟化）
- 涉及文件约 10 个：`mod.rs`、`directory.rs`、`ram_directory.rs`、`managed_directory.rs`、`mmap_directory/mod.rs`、`footer.rs`、`composite_file.rs` 等

**评估结论**：**不推荐实施**
- 收益：2 vtable 调用 / 8KB 数据 ≈ 纳秒级影响
- 在 graphDB 写入路径中，网络序列化（JSON/Postcard）和磁盘 I/O 占 >99% 时间
- 分析文档原文已指出「涉及过多文件修改，收益有限」
- `FooterProxy` 本身是合理的抽象，消除它带来的复杂度不值得

---

#### P3.3 详细分析：异步 I/O 非原生

**现状分析**：
- `crates/tantivy/Cargo.toml` 中 `quickwit` feature 启用 `sstable` + `futures-util` + `futures-channel`
- `FileHandle::read_bytes_async()` 在 `common/src/file_slice.rs:29` 定义，但**所有实现都是同步的**：
  - `WrapFile`：返回 `Unsupported` 错误（含 `// todo implement async` 注释）
  - `OwnedBytes` 和 `&'static [u8]`：直接委托给同步 `read_bytes()`
- `quickwit` feature 下使用 `spawn_blocking` 仅用于将 CPU 密集型操作（解压缩）迁移到 rayon 线程池
- `Directory` trait **没有任何异步方法**，整个存储层是同步的

**评估结论**：**不推荐实施**
- graphDB 定位为本地单节点部署，mmap 同步 I/O 是最优选择
- 引入 tokio/io_uring 会增加依赖体积和复杂性
- 真正的异步 I/O 需要重写 `Directory` trait + 所有实现 → 巨大工程量
- 当前通过 tokio `spawn_blocking` 包装 tantivy 操作已满足异步运行时集成需求

---

## 四、Tantivy 存储层评估

### 架构

```
Directory trait
    ├── RamDirectory (测试)
    ├── MmapDirectory (生产: mmap + 文件)
    └── ManagedDirectory (装饰器: GC + CRC32 Footer)

Store Layer
    ├── StoreWriter: 块压缩 (LZ4/Zstd) + SkipIndex
    ├── StoreReader: LRU BlockCache + 跳表定位
    └── Merge: 零拷贝 stack (不解压直接拼接)
```

### 设计评价

**合理之处**：
- WORM + ManagedDirectory 装饰器模式，GC/CRC 与底层存储解耦
- SkipIndex + BlockCache 读写分离，典型日志结构读优化
- Merge 零拷贝 stacking，避免 CPU 开销
- 文件级 CRC32 校验，损坏范围可控

**可改进之处**（除上述已修复项目）：
- GC 锁范围不足（P0.1）
- WritePtr 双动态分发（P3.2，影响极小）
- 删除的 file_watcher 重复代码不存在（分析文档有误，已验证）

---

## 五、改进优先级总表

| 优先级 | 项目 | 状态 | 影响 |
|--------|------|------|------|
| P0.1 | GC 竞争窗口修复 | ✓ 已实施 | 防止索引损坏 |
| P0.2 | FulltextStorage 废弃标记 | ✓ 已实施 | 防止未来误用 |
| P1.1 | Reader 缓存 | ✓ 已实施 | 提升搜索性能 |
| P2.1 | index_size 计算 | ✓ 已实施 | 监控指标完整 |
| P2.2 | Highlights 支持 | ✓ 已实施 | 搜索体验提升 |
| P3.1 | Schema 自定义（分词器） | 待定 | 支持非中文索引 |
| P3.2 | WritePtr 去虚拟化 | 不推荐 | 收益极小 |
| P3.3 | 异步 I/O | 不推荐 | 单机无需 |
