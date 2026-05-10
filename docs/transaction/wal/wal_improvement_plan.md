# WAL 改进方案

## 1. 当前实现状态分析

### 1.1 已完整实现的功能

| 功能         | 状态    | 实现位置                                 | 说明                                        |
| ------------ | ------- | ---------------------------------------- | ------------------------------------------- |
| LSN 跟踪     | ✅ 完整 | `types.rs::Lsn`                          | 单调递增的字节偏移量，支持精确恢复点定位    |
| 文件轮转     | ✅ 完整 | `writer.rs::rotate()`                    | 基于文件大小的自动轮转                      |
| TTL 清理     | ✅ 完整 | `writer.rs::cleanup_old_wal_files()`     | 基于时间的过期删除                          |
| 大小限制清理 | ✅ 完整 | `writer.rs::cleanup_old_wal_files()`     | 基于总大小的删除策略                        |
| 归档支持     | ✅ 完整 | `writer.rs::archive_wal_file()`          | 支持移动/复制归档模式                       |
| Group Commit | ✅ 完整 | `writer.rs::GroupCommitManager`          | 批量提交优化吞吐量                          |
| 压缩         | ✅ 完整 | `types.rs::WalCompression`               | 支持 Zstd 压缩                              |
| CRC32 校验   | ✅ 完整 | `types.rs::WalHeader::verify_checksum()` | 数据完整性验证                              |
| 多种同步策略 | ✅ 完整 | `types.rs::SyncPolicy`                   | Never/EveryWrite/Periodic/Batch/GroupCommit |
| 记录分片     | ✅ 完整 | `types.rs::RecordType`                   | Full/First/Middle/Last 分片支持             |
| 并行恢复     | ✅ 完整 | `parser.rs::ParallelWalParser`           | 多线程并行解析 WAL 文件                     |
| 增强检查点   | ✅ 完整 | `checkpoint.rs::Checkpoint`              | 包含 LSN、活跃事务、脏页等信息              |

### 1.2 部分实现/待完善的功能

| 功能             | 状态      | 当前实现           | 缺失部分            |
| ---------------- | --------- | ------------------ | ------------------- |
| Dirty Tracking   | ❌ 未集成 | API 存在但未被调用 | 存储引擎集成        |
| Full Page Writes | ⚠️ 部分   | 类型定义完整       | 写入/恢复逻辑未集成 |
| Circular Buffer  | ❌ 未实现 | 仅配置项           | 完整实现缺失        |
| 后台刷新线程     | ❌ 未实现 | 无                 | 可选功能            |
| 并行恢复优化     | ⚠️ 可优化 | 基础实现           | 可增加流水线处理    |

### 1.3 当前架构评估

```
WAL 模块架构
├── types.rs          - 类型定义 (Lsn, WalHeader, WalConfig 等)
├── writer.rs         - WAL 写入器
│   ├── LocalWalWriter      - 本地文件写入器
│   └── GroupCommitManager  - 批量提交管理
├── parser.rs         - WAL 解析器
│   ├── ParallelWalParser   - 并行解析器
│   └── FragmentBuffer      - 分片重组缓冲区
└── checkpoint.rs     - 检查点管理
    └── CheckpointManager   - 检查点协调器
```

**优势**：

- 模块化设计清晰
- 功能覆盖全面
- 性能优化到位（Group Commit、并行恢复）
- 配置灵活

**不足**：

- Full Page Writes 未完全集成
- Circular Buffer 未实现
- 缺少性能监控指标暴露

---

## 2. 与业界最佳实践对比

### 2.1 功能对比矩阵

| 功能特性         | PostgreSQL      | RocksDB   | SQLite    | GraphDB 当前 | 改进建议 |
| ---------------- | --------------- | --------- | --------- | ------------ | -------- |
| LSN 跟踪         | ✅              | ✅        | ❌        | ✅           | 已达标   |
| 文件分段         | ✅ 16MB         | ✅ 可配置 | ❌ 单文件 | ✅ 16MB      | 已达标   |
| TTL 清理         | ❌              | ✅        | ❌        | ✅           | 已达标   |
| 大小限制         | ✅ max_wal_size | ✅        | ❌        | ✅           | 已达标   |
| Group Commit     | ✅              | ✅        | ❌        | ✅           | 已达标   |
| 记录分片         | ❌              | ✅        | ❌        | ✅           | 已达标   |
| 并行恢复         | ❌              | ✅        | ❌        | ✅           | 已达标   |
| Full Page Writes | ✅              | ❌        | ❌        | ⚠️ 部分      | 需完善   |
| Circular Buffer  | ❌              | ❌        | ❌        | ❌           | 可选实现 |
| 检查点模式       | ✅ 多种         | ✅        | ✅ 多种   | ⚠️ 基础      | 可扩展   |
| 压缩             | ❌              | ✅        | ❌        | ✅ Zstd      | 已达标   |

### 2.2 性能对比

| 指标     | PostgreSQL  | RocksDB        | GraphDB 目标          |
| -------- | ----------- | -------------- | --------------------- |
| 写入延迟 | ~1ms (sync) | ~0.1ms (async) | ~0.5ms (group commit) |
| 吞吐量   | ~100K TPS   | ~500K TPS      | ~200K TPS             |
| 恢复速度 | 线性        | 并行           | 并行 (4 线程)         |

---

## 3. 改进方案

### 3.1 Phase 0: 实现 Dirty Tracking (优先级: 最高)

**背景**：Dirty Tracking 是 Full Page Writes 和高效检查点的基础，当前 `CheckpointManager` 中的 `dirty_pages` 只是一个空壳实现。

**参考实现**：`ref/temp/dirty_tracker.rs` 提供了完整的实现。

**改进方案**：

```rust
// 1. 在 src/transaction/wal/ 中添加 dirty_tracker.rs

/// Page identifier with table context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId {
    pub table_type: TableType,
    pub label_id: u16,
    pub block_number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    Vertex = 1,
    Edge = 2,
    Schema = 4,
}

/// Thread-safe dirty page tracker
pub struct DirtyPageTracker {
    dirty_pages: RwLock<HashSet<PageId>>,
    last_flush: RwLock<Instant>,
    flush_threshold: usize,
    flush_interval: Duration,
}

impl DirtyPageTracker {
    pub fn new(flush_threshold: usize, flush_interval: Duration) -> Self {
        Self {
            dirty_pages: RwLock::new(HashSet::new()),
            last_flush: RwLock::new(Instant::now()),
            flush_interval,
            flush_threshold,
        }
    }

    pub fn mark_dirty(&self, page_id: PageId) {
        self.dirty_pages.write().insert(page_id);
    }

    pub fn mark_dirty_batch(&self, page_ids: &[PageId]) {
        let mut dirty = self.dirty_pages.write();
        for page_id in page_ids {
            dirty.insert(*page_id);
        }
    }

    pub fn should_flush(&self) -> bool {
        let dirty = self.dirty_pages.read();
        let threshold_reached = dirty.len() >= self.flush_threshold;
        drop(dirty);
        let time_reached = self.last_flush.read().elapsed() >= self.flush_interval;
        threshold_reached || time_reached
    }

    pub fn flush_and_reset(&self) -> Vec<PageId> {
        let pages: Vec<PageId> = self.dirty_pages.write().drain().collect();
        *self.last_flush.write() = Instant::now();
        pages
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_pages.read().len()
    }
}

// 2. 在 PropertyGraph 中集成
pub struct PropertyGraph {
    schema_ops: SchemaOps,
    edge_ops: EdgeOps,
    cache_manager: CacheManager,
    wal_manager: WalManager,
    dirty_tracker: Arc<DirtyPageTracker>,  // 新增
    config: PropertyGraphConfig,
    is_open: bool,
}

// 3. 在写入操作中标记脏页
impl PropertyGraph {
    pub fn add_vertex(...) -> ... {
        // ... existing logic ...

        // Mark page as dirty
        self.dirty_tracker.mark_dirty(PageId {
            table_type: TableType::Vertex,
            label_id: label,
            block_number: vid / PAGE_SIZE as u64,
        });
    }
}
```

**集成点**：

- `PropertyGraph::add_vertex` / `add_edge` / `update_*` 等写入操作
- `CheckpointManager::create_checkpoint` 获取脏页列表
- 检查点完成后清除脏页标记

### 3.2 Phase 1: 完善 Full Page Writes (优先级: 高)

**背景**：Full Page Writes 是防止部分页写入（torn page）的关键机制，PostgreSQL 在检查点后首次修改页面时会写入完整页面镜像。

**当前状态**：

- `FullPageWriteHeader` 类型已定义
- `WalOpType::FullPageWrite` 已定义
- 缺少写入和恢复逻辑

**改进方案**：

```rust
// 1. 在 writer.rs 中添加 full page write 方法
impl LocalWalWriter {
    /// Write a full page image for crash recovery
    pub fn write_full_page(
        &mut self,
        page_id: PageId,
        page_data: &[u8],
        page_lsn: Lsn,
    ) -> WalResult<Lsn> {
        let record_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));

        let header = FullPageWriteHeader::new(page_id, page_lsn, record_lsn, page_data.len() as u32)
            .with_checksum(crc32fast::hash(page_data));

        let header_bytes = header.serialize();
        let mut payload = header_bytes;
        payload.extend_from_slice(page_data);

        self.append_with_op_type(WalOpType::FullPageWrite, &payload, 0)
    }
}

// 2. 在 parser.rs 中添加 full page write 恢复处理
impl ParallelWalParser {
    fn handle_full_page_write(&self, header: &FullPageWriteHeader, page_data: &[u8]) -> WalResult<()> {
        // Verify checksum
        let checksum = crc32fast::hash(page_data);
        if checksum != header.page_checksum {
            return Err(WalError::ChecksumMismatch {
                expected: header.page_checksum,
                actual: checksum,
            });
        }

        // Apply page to storage
        // This would be implemented by the storage layer
        Ok(())
    }
}
```

**集成点**：

- 检查点后首次修改页面时触发
- 在 `PropertyGraph::flush()` 后重置跟踪状态

### 3.2 Phase 2: 实现 Circular Buffer 模式 (优先级: 中)

**背景**：Circular Buffer 模式可以减少磁盘空间使用，适用于不需要 PITR（时间点恢复）的场景。

**实现方案**：

```rust
// 在 writer.rs 中添加 circular buffer 支持
pub struct CircularWalWriter {
    /// Base directory for WAL files
    wal_dir: PathBuf,
    /// Fixed-size buffer file
    buffer_file: File,
    /// Buffer size
    buffer_size: usize,
    /// Current write position
    write_pos: AtomicU64,
    /// Current read position (for recovery)
    read_pos: AtomicU64,
    /// Current LSN
    current_lsn: AtomicU64,
    /// Configuration
    config: WalConfig,
}

impl CircularWalWriter {
    pub fn new(wal_dir: &Path, config: WalConfig) -> WalResult<Self> {
        let buffer_file = wal_dir.join("wal.circular");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&buffer_file)?;

        // Pre-allocate buffer
        file.set_len(config.circular_buffer_size as u64)?;

        Ok(Self {
            wal_dir: wal_dir.to_path_buf(),
            buffer_file: file,
            buffer_size: config.circular_buffer_size,
            write_pos: AtomicU64::new(0),
            read_pos: AtomicU64::new(0),
            current_lsn: AtomicU64::new(0),
            config,
        })
    }

    fn write_circular(&mut self, data: &[u8]) -> WalResult<()> {
        let pos = self.write_pos.load(Ordering::SeqCst);
        let wrapped_pos = (pos % self.buffer_size as u64) as usize;

        // Handle wrap-around
        if wrapped_pos + data.len() > self.buffer_size {
            // Split write across boundary
            let first_part = self.buffer_size - wrapped_pos;
            self.buffer_file.seek(SeekFrom::Start(wrapped_pos as u64))?;
            self.buffer_file.write_all(&data[..first_part])?;
            self.buffer_file.seek(SeekFrom::Start(0))?;
            self.buffer_file.write_all(&data[first_part..])?;
        } else {
            self.buffer_file.seek(SeekFrom::Start(wrapped_pos as u64))?;
            self.buffer_file.write_all(data)?;
        }

        self.write_pos.fetch_add(data.len() as u64, Ordering::SeqCst);
        Ok(())
    }
}
```

**适用场景**：

- 嵌入式部署
- 磁盘空间受限
- 不需要 PITR

### 3.3 Phase 3: 增强检查点模式 (优先级: 中)

**背景**：SQLite 提供多种检查点模式（PASSIVE/FULL/RESTART/TRUNCATE），可以根据场景选择。

**改进方案**：

```rust
// 在 checkpoint.rs 中添加检查点模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckpointMode {
    /// Passive: checkpoint as many frames as possible without blocking
    Passive,
    /// Full: block until no writers, checkpoint all frames
    #[default]
    Full,
    /// Restart: same as Full, but ensures next writer restarts log
    Restart,
    /// Truncate: same as Restart, but also truncates WAL file
    Truncate,
}

impl CheckpointManager {
    /// Perform checkpoint with specified mode
    pub fn checkpoint(&mut self, mode: CheckpointMode) -> WalResult<CheckpointResult> {
        match mode {
            CheckpointMode::Passive => self.checkpoint_passive(),
            CheckpointMode::Full => self.checkpoint_full(),
            CheckpointMode::Restart => self.checkpoint_restart(),
            CheckpointMode::Truncate => self.checkpoint_truncate(),
        }
    }

    fn checkpoint_truncate(&mut self) -> WalResult<CheckpointResult> {
        // 1. Wait for all writers to complete
        // 2. Flush all dirty pages
        // 3. Write checkpoint record
        // 4. Truncate WAL file to header only
        // 5. Reset LSN counter
        Ok(CheckpointResult::default())
    }
}
```

### 3.4 Phase 4: 性能监控指标 (优先级: 低)

**背景**：暴露 WAL 性能指标有助于监控和调优。

**实现方案**：

```rust
// 在 types.rs 中扩展 WalStats
#[derive(Debug, Clone, Default)]
pub struct WalStats {
    // Existing fields
    pub total_rotations: u64,
    pub total_files_deleted: u64,
    pub total_files_archived: u64,
    pub last_rotation_time: Option<u64>,
    pub total_bytes_written: u64,
    pub total_entries_written: u64,

    // New metrics
    /// Average write latency in microseconds
    pub avg_write_latency_us: f64,
    /// Peak write latency in microseconds
    pub peak_write_latency_us: u64,
    /// Current WAL file count
    pub current_file_count: usize,
    /// Current total WAL size in bytes
    pub current_total_size: usize,
    /// Group commit batch size statistics
    pub group_commit_avg_batch_size: f64,
    /// Checkpoint count
    pub checkpoint_count: u64,
    /// Last checkpoint duration in microseconds
    pub last_checkpoint_duration_us: u64,
}

impl LocalWalWriter {
    /// Get current WAL statistics
    pub fn stats(&self) -> WalStats {
        self.stats.clone()
    }

    /// Get current metrics for monitoring
    pub fn metrics(&self) -> WalMetrics {
        WalMetrics {
            current_lsn: self.current_lsn.load(Ordering::SeqCst),
            last_synced_lsn: self.last_synced_lsn.load(Ordering::SeqCst),
            current_file_version: self.version,
            pending_writes: self.group_commit.as_ref()
                .map(|g| g.pending_count())
                .unwrap_or(0),
        }
    }
}
```

---

## 4. 实现优先级和时间规划

### 4.1 优先级排序

| 优先级 | 功能                  | 原因                            | 预计工作量 |
| ------ | --------------------- | ------------------------------- | ---------- |
| P0     | 实现 Dirty Tracking   | Full Page Writes 和检查点的基础 | 1-2 天     |
| P1     | 完善 Full Page Writes | 崩溃恢复安全性                  | 2-3 天     |
| P2     | 增强检查点模式        | 灵活性和性能                    | 1-2 天     |
| P3     | 性能监控指标          | 可观测性                        | 1 天       |
| P4     | 后台刷新线程          | 可选功能，减少写入延迟          | 1-2 天     |
| P5     | Circular Buffer       | 可选功能                        | 2-3 天     |

### 4.2 实施路线图

```
Week 1:
├── Day 1-2: Dirty Tracking 实现
│   ├── 创建 dirty_tracker.rs
│   ├── 集成到 PropertyGraph
│   └── 单元测试
├── Day 3-4: Full Page Writes 写入逻辑
└── Day 5: Full Page Writes 恢复逻辑

Week 2:
├── Day 1: Full Page Writes 集成测试
├── Day 2-3: 检查点模式增强
├── Day 4: 性能监控指标
└── Day 5: 后台刷新线程 (可选)
```

---

## 5. 测试计划

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dirty_tracker_basic() {
        // Test basic mark_dirty and is_dirty
    }

    #[test]
    fn test_dirty_tracker_threshold() {
        // Test flush threshold trigger
    }

    #[test]
    fn test_dirty_tracker_time_based() {
        // Test time-based flush trigger
    }

    #[test]
    fn test_full_page_write_basic() {
        // Test basic full page write functionality
    }

    #[test]
    fn test_full_page_write_recovery() {
        // Test recovery with full page writes
    }

    #[test]
    fn test_checkpoint_modes() {
        // Test all checkpoint modes
    }
}
```

### 5.2 集成测试

```rust
#[test]
fn test_crash_recovery_with_full_page_writes() {
    // Simulate crash and verify recovery
}

#[test]
fn test_checkpoint_truncate_mode() {
    // Verify truncate mode properly resets WAL
}
```

---

## 6. 总结

### 6.1 当前状态评估

GraphDB 的 WAL 实现已达到业界主流数据库水平，大部分核心功能已完整实现：

**优势**：

- LSN 跟踪完整实现
- 文件轮转和清理策略完善
- Group Commit 性能优化到位
- 并行恢复支持
- 记录分片支持大事务

**待改进**：

- Full Page Writes 需要完整集成
- Circular Buffer 可选实现
- 检查点模式可扩展

### 6.2 改进建议

1. **立即实施**：完善 Full Page Writes，确保崩溃恢复安全性
2. **短期实施**：增强检查点模式，提供更灵活的选择
3. **中期实施**：添加性能监控指标，提升可观测性
4. **长期可选**：Circular Buffer 模式，适用于特定场景

### 6.3 与业界对比结论

| 维度       | 评分       | 说明                                     |
| ---------- | ---------- | ---------------------------------------- |
| 功能完整性 | ⭐⭐⭐⭐☆  | 核心功能齐全，Full Page Writes 待完善    |
| 性能优化   | ⭐⭐⭐⭐⭐ | Group Commit、并行恢复到位               |
| 可靠性     | ⭐⭐⭐⭐☆  | CRC32、LSN 完整，Full Page Writes 待集成 |
| 可配置性   | ⭐⭐⭐⭐⭐ | 配置项丰富，策略灵活                     |
| 可观测性   | ⭐⭐⭐☆☆   | 基础统计已有，监控指标可扩展             |

总体而言，GraphDB 的 WAL 实现已达到生产可用水平，建议优先完善 Full Page Writes 以确保崩溃恢复的安全性。
