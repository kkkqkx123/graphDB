# Page 与 Persistence 模块集成方案

## 一、现状分析

### 1.1 模块职责

| 模块          | 职责           | 核心组件                                                                 |
| ------------- | -------------- | ------------------------------------------------------------------------ |
| `page`        | 页面级存储抽象 | Page, PageManager, PageLockManager, Record                               |
| `persistence` | 持久化策略     | DirtyPageTracker, FlushManager, FilePageWriter, SSTable, RecoveryManager |

### 1.2 当前问题

#### 问题 1：两套不兼容的 Page ID 体系

```
page::StoragePageId:
  - file_id: u32
  - page_number: u32
  - 语义：物理文件定位

persistence::DirtyPageId:
  - table_type: TableType (Vertex/Edge/Property/Schema)
  - label_id: u16
  - block_number: u64
  - 语义：逻辑表定位
```

两套 ID 无法互转，导致 `PageManager` 管理的页面与 `DirtyPageTracker` 追踪的脏页指向不同空间。

#### 问题 2：核心组件处于孤儿状态

| 组件                             | 状态           | 外部调用者       |
| -------------------------------- | -------------- | ---------------- |
| `PageManager`                    | 已实现，未集成 | 无（仅测试使用） |
| `FilePageWriter`                 | 已实现，未集成 | 无（仅测试使用） |
| `persistence::CheckpointManager` | 已实现，未集成 | 无（仅测试使用） |
| `SSTable`                        | 已实现，未集成 | 无               |
| `RecoveryManager`                | 已实现，未集成 | 无               |

#### 问题 3：Flush 链路断裂

```
当前状态:
DirtyPageTracker → FlushManager → [日志输出] → 丢弃

期望状态:
DirtyPageTracker → FlushManager → FilePageWriter → Disk
```

`FlushManager` 的后台线程只收集脏页列表，未调用 `PageWriter::write_page()`。

#### 问题 4：CheckpointManager 重复定义

| 位置                            | 关注点                                        |
| ------------------------------- | --------------------------------------------- |
| `persistence/page_writer.rs`    | 基于 DirtyPageTracker，管理 checkpoint 元数据 |
| `transaction/wal/checkpoint.rs` | 基于 WAL，管理 checkpoint 序列号和 WAL 清理   |

---

## 二、集成目标

### 2.1 目标架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        PropertyGraph                             │
│  (门面层：组合所有存储组件，提供统一 API)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│ VertexTable   │    │  EdgeTable    │    │  SchemaOps    │
│ (数据操作)     │    │  (数据操作)    │    │  (元数据)      │
└───────────────┘    └───────────────┘    └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PageManager                               │
│  (页面分配、缓存、生命周期管理)                                    │
│  - 使用统一的 PageId                                             │
│  - 管理页面缓存 (moka Cache)                                     │
│  - 触发脏页标记                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     DirtyPageTracker                             │
│  (脏页追踪)                                                      │
│  - 记录修改的页面                                                │
│  - 触发 flush 条件判断                                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       FlushManager                               │
│  (后台刷盘调度)                                                   │
│  - 后台线程定期检查                                               │
│  - 调用 PageWriter 写入                                          │
│  - 支持压缩                                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      FilePageWriter                              │
│  (页面持久化)                                                    │
│  - 原子写入（临时文件 + rename）                                  │
│  - 页面索引管理                                                  │
│  - checksum 校验                                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Disk                                    │
│  data/                                                           │
│  ├── vertex/label_{id}/block_{num}.page                         │
│  ├── edge/label_{id}/block_{num}.page                           │
│  ├── property/label_{id}/block_{num}.page                       │
│  └── schema/block_{num}.page                                    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
写入路径:
Client → PropertyGraph.insert_vertex()
       → VertexTable.insert()
       → PageManager.get_page_mut() + Page.write_record()
       → PageManager.mark_dirty()
       → DirtyPageTracker.mark_dirty()

刷盘路径:
DirtyPageTracker.should_flush() == true
       → FlushManager.flush_dirty_pages()
       → FilePageWriter.write_page()
       → DirtyPageTracker.unmark_dirty()

恢复路径:
RecoveryManager.recover()
       → WAL 重放
       → Full Page Write 恢复
       → DirtyPageTracker 状态恢复
```

---

## 三、详细设计方案

### 3.1 统一 Page ID 系统

#### 方案：采用 `DirtyPageId` 作为统一标准

理由：

1. 语义更丰富，包含表类型和标签信息
2. 与 WAL 中的 PageId 格式一致
3. 支持按表类型组织存储目录

#### 实现步骤

**Step 1: 定义统一的 PageId**

在 `src/storage/page/mod.rs` 中：

```rust
// 重导出 DirtyPageId 作为统一的 PageId
pub use crate::storage::persistence::{DirtyPageId as PageId, TableType};

// 废弃 StoragePageId，提供兼容层
#[deprecated(since = "0.1.0", note = "Use PageId instead")]
pub use crate::storage::persistence::DirtyPageId as StoragePageId;
```

**Step 2: 修改 PageManager 使用新 ID**

```rust
// page/page_manager.rs
use crate::storage::persistence::{DirtyPageId, TableType};

pub struct PageManager {
    pages: Cache<DirtyPageId, Page>,
    // ...
}

impl PageManager {
    pub fn allocate_page(&self, table_type: TableType, label_id: u16) -> StorageResult<DirtyPageId> {
        let block_number = self.next_page_id.fetch_add(1, Ordering::SeqCst);
        let page_id = DirtyPageId::new(table_type, label_id, block_number);
        let page = Page::new(page_id.to_u64(), PageType::from_table_type(table_type));
        self.pages.insert(page_id, page);
        Ok(page_id)
    }

    pub fn get_page(&self, page_id: &DirtyPageId) -> StorageResult<Option<Page>> {
        // ...
    }

    pub fn mark_dirty(&self, page_id: &DirtyPageId) -> StorageResult<()> {
        if let Some(mut page) = self.pages.get(page_id) {
            page.mark_dirty();
            self.pages.insert(*page_id, page);
        }
        Ok(())
    }
}
```

**Step 3: 添加 PageId 工具方法**

```rust
// persistence/dirty_tracker.rs
impl DirtyPageId {
    pub fn to_u64(&self) -> u64 {
        ((self.table_type as u64) << 56)
            | ((self.label_id as u64) << 40)
            | (self.block_number & 0xFFFFFFFFFF)
    }

    pub fn from_u64(value: u64) -> Self {
        let table_type = match ((value >> 56) & 0xFF) as u8 {
            1 => TableType::Vertex,
            2 => TableType::Edge,
            3 => TableType::Property,
            _ => TableType::Schema,
        };
        Self {
            table_type,
            label_id: ((value >> 40) & 0xFFFF) as u16,
            block_number: value & 0xFFFFFFFFFF,
        }
    }

    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let table_dir = match self.table_type {
            TableType::Vertex => "vertex",
            TableType::Edge => "edge",
            TableType::Property => "property",
            TableType::Schema => "schema",
        };
        base_dir
            .join(table_dir)
            .join(format!("label_{}", self.label_id))
            .join(format!("block_{:08}.page", self.block_number))
    }
}
```

### 3.2 打通 Flush 链路

#### Step 1: 修改 FlushManager 持有 PageManager 引用

```rust
// persistence/flush_manager.rs
use crate::storage::page::PageManager;

pub struct FlushManager {
    dirty_tracker: Arc<DirtyPageTracker>,
    page_manager: Arc<PageManager>,  // 新增
    page_writer: Arc<dyn PageWriter>,
    compressor: Compressor,
    config: FlushConfig,
    running: Arc<AtomicBool>,
    background_thread: RwLock<Option<JoinHandle<()>>>,
}

impl FlushManager {
    pub fn new(
        config: FlushConfig,
        dirty_tracker: Arc<DirtyPageTracker>,
        page_manager: Arc<PageManager>,
        page_writer: Arc<dyn PageWriter>,
    ) -> Self {
        Self {
            dirty_tracker,
            page_manager,
            page_writer,
            compressor: Compressor::new(config.compression),
            config,
            running: Arc::new(AtomicBool::new(false)),
            background_thread: RwLock::new(None),
        }
    }

    fn do_flush(&self, page_ids: &[PageId]) -> StorageResult<usize> {
        let mut flushed = 0;
        for page_id in page_ids {
            if let Some(page) = self.page_manager.get_page(page_id)? {
                let data = page.to_bytes();
                self.page_writer.write_page(page_id, &data)?;
                self.page_manager.clear_dirty(page_id)?;
                flushed += 1;
            }
        }
        Ok(flushed)
    }
}
```

#### Step 2: 修改后台线程执行实际写入

```rust
impl FlushManager {
    pub fn start_background_flush(&self) -> StorageResult<()> {
        if !self.config.background_flush_enabled {
            return Ok(());
        }

        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let dirty_tracker = self.dirty_tracker.clone();
        let page_manager = self.page_manager.clone();
        let page_writer = self.page_writer.clone();
        let interval = self.config.flush_interval;
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                thread::sleep(interval);

                if dirty_tracker.should_flush() {
                    let pages = dirty_tracker.flush_and_reset();
                    if !pages.is_empty() {
                        for page_id in &pages {
                            if let Some(page) = page_manager.get_page(page_id).ok().flatten() {
                                if page.is_dirty() {
                                    let data = page.to_bytes();
                                    if let Err(e) = page_writer.write_page(page_id, &data) {
                                        log::error!("Failed to flush page {:?}: {}", page_id, e);
                                        dirty_tracker.mark_dirty(*page_id);
                                    }
                                }
                            }
                        }
                        log::info!("Background flush completed: {} pages", pages.len());
                    }
                }
            }
        });

        *self.background_thread.write() = Some(handle);
        Ok(())
    }
}
```

### 3.3 集成 PageManager 到 PropertyGraph

#### Step 1: 修改 PropertyGraph 结构

```rust
// engine/property_graph.rs
use crate::storage::page::{PageManager, PageManagerConfig};
use crate::storage::persistence::{FilePageWriter, FlushManager, DirtyPageTracker};

pub struct PropertyGraph {
    schema_ops: SchemaOps,
    edge_ops: EdgeOps,
    cache_manager: CacheManager,
    wal_manager: WalManager,

    // 新增：统一管理页面
    page_manager: Arc<PageManager>,
    dirty_tracker: Arc<DirtyPageTracker>,
    flush_manager: FlushManager,
    lock_manager: PageLockManager,

    config: PropertyGraphConfig,
    is_open: bool,
}
```

#### Step 2: 修改初始化逻辑

```rust
impl PropertyGraph {
    pub fn with_config(config: PropertyGraphConfig) -> Self {
        let memory_tracker = Arc::new(MemoryTracker::new(config.memory_config.clone()));

        // 创建页面管理器
        let page_manager = Arc::new(PageManager::with_config(PageManagerConfig {
            max_pages: config.cache_memory / 4096, // 按 4KB 页面计算
            base_path: config.work_dir.join("pages"),
        }));

        // 创建脏页追踪器
        let dirty_tracker = Arc::new(DirtyPageTracker::with_config(
            crate::storage::persistence::DirtyTrackerConfig {
                flush_threshold: config.flush_config.flush_threshold,
                flush_interval: config.flush_config.flush_interval,
            },
        ));

        // 创建页面写入器
        let page_writer = Arc::new(
            FilePageWriter::new(
                config.work_dir.clone(),
                config.flush_config.compression,
            ).expect("Failed to create page writer")
        );

        // 创建 FlushManager（连接所有组件）
        let flush_manager = FlushManager::new(
            config.flush_config.clone(),
            dirty_tracker.clone(),
            page_manager.clone(),
            page_writer,
        );

        let cache_manager = CacheManager::new(
            config.enable_cache,
            config.cache_memory,
            memory_tracker.clone(),
        );

        let lock_manager = PageLockManager::new();

        Self {
            schema_ops: SchemaOps::new(),
            edge_ops: EdgeOps::new(),
            cache_manager,
            wal_manager: WalManager::new(),
            page_manager,
            dirty_tracker,
            flush_manager,
            lock_manager,
            config,
            is_open: true,
        }
    }
}
```

### 3.4 协调 CheckpointManager

#### 方案：合并为统一的 CheckpointManager

将 `persistence::CheckpointManager` 和 `wal::CheckpointManager` 合并，放在 `transaction/wal/checkpoint.rs` 中。

```rust
// transaction/wal/checkpoint.rs
use crate::storage::persistence::{DirtyPageTracker, DirtyPageId};
use crate::storage::page::PageManager;

pub struct CheckpointManager {
    // WAL 相关
    wal_dir: PathBuf,
    current_seq: u64,
    last_checkpoint_ts: Timestamp,
    last_checkpoint_lsn: Lsn,

    // 页面相关
    dirty_tracker: Arc<DirtyPageTracker>,
    page_manager: Arc<PageManager>,

    // 活跃事务
    active_transactions: Vec<TransactionId>,
}

impl CheckpointManager {
    pub fn create_checkpoint(
        &mut self,
        timestamp: Timestamp,
        lsn: Lsn,
        mode: CheckpointMode,
    ) -> WalResult<CheckpointResult> {
        // 1. 获取脏页列表
        let dirty_pages = self.dirty_tracker.get_dirty_pages();

        // 2. 刷写脏页（Full/Truncate 模式）
        if mode == CheckpointMode::Full || mode == CheckpointMode::Truncate {
            for page_id in &dirty_pages {
                self.page_manager.flush_page(page_id)?;
            }
        }

        // 3. 创建 checkpoint 记录
        self.current_seq += 1;
        self.last_checkpoint_ts = timestamp;
        self.last_checkpoint_lsn = lsn;

        // 4. 保存 checkpoint 元数据
        self.save_checkpoint_meta()?;

        // 5. 清理旧 WAL 文件（Truncate 模式）
        if mode == CheckpointMode::Truncate {
            self.truncate_old_wal_files()?;
        }

        // 6. 清空脏页追踪
        self.dirty_tracker.clear();

        Ok(CheckpointResult {
            pages_written: dirty_pages.len(),
            wal_files_processed: 0,
            duration_us: 0,
            mode,
            success: true,
        })
    }
}
```

### 3.5 SSTable 定位

#### 方案：SSTable 作为元数据和索引的持久化格式

SSTable 不替代页面存储，而是用于：

1. **Schema 元数据持久化**：VertexSchema、EdgeSchema 的序列化存储
2. **索引持久化**：二级索引的磁盘格式
3. **统计信息持久化**：ColumnStatistics 的存储

```
data/
├── pages/           # 页面存储（FilePageWriter）
│   ├── vertex/
│   ├── edge/
│   └── property/
├── schema/          # Schema 元数据（SSTable）
│   └── schema.sst
├── index/           # 二级索引（SSTable）
│   └── {index_name}.sst
└── stats/           # 统计信息（SSTable）
    └── stats.sst
```

---

## 四、实施计划

### Phase 1: 统一 Page ID（预计 1 天）

1. 在 `persistence/dirty_tracker.rs` 中添加 `to_u64()` 和 `from_u64()` 方法
2. 修改 `page/page_manager.rs` 使用 `DirtyPageId`
3. 添加废弃标记和兼容层
4. 更新相关测试

### Phase 2: 打通 Flush 链路（预计 2 天）

1. 修改 `FlushManager` 添加 `page_manager` 和 `page_writer` 依赖
2. 实现后台线程的实际写入逻辑
3. 添加错误处理和重试机制
4. 编写集成测试

### Phase 3: 集成 PageManager（预计 2 天）

1. 修改 `PropertyGraph` 持有 `PageManager`
2. 修改 `VertexTable` 和 `EdgeTable` 使用 `PageManager`
3. 连接脏页标记链路
4. 编写端到端测试

### Phase 4: 合并 CheckpointManager（预计 1 天）

1. 创建统一的 `CheckpointManager`
2. 删除 `persistence/page_writer.rs` 中的 `CheckpointManager`
3. 更新所有调用点
4. 编写测试

### Phase 5: 集成测试与文档（预计 1 天）

1. 编写完整的集成测试
2. 更新架构文档
3. 性能基准测试

---

## 五、风险与缓解

| 风险                        | 影响 | 缓解措施                  |
| --------------------------- | ---- | ------------------------- |
| Page ID 变更导致兼容性问题  | 高   | 提供兼容层，渐进式迁移    |
| Flush 性能影响写入延迟      | 中   | 异步刷盘，批量写入        |
| 并发访问 PageManager 的竞争 | 中   | 使用细粒度锁，减少临界区  |
| Checkpoint 期间的写阻塞     | 中   | 采用 Passive 模式，非阻塞 |

---

## 六、验收标准

1. **功能验收**
   - [ ] `PageManager` 被正确集成到 `PropertyGraph`
   - [ ] 脏页能正确追踪并刷写到磁盘
   - [ ] 崩溃恢复能正确恢复数据
   - [ ] Checkpoint 功能正常工作

2. **性能验收**
   - [ ] 写入吞吐量不低于当前水平
   - [ ] 后台刷盘不影响前台写入延迟

3. **代码质量**
   - [ ] 所有测试通过
   - [ ] 无 clippy 警告
   - [ ] 文档更新完整
