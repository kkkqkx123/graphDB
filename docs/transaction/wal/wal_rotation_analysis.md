# WAL 文件轮转机制分析报告

## 执行摘要

本文档分析了主流数据库（PostgreSQL、RocksDB、SQLite）的 WAL（Write-Ahead Log）文件轮转实现策略，为 GraphDB 的 WAL 模块改进提供参考。

---

## 1. 各数据库 WAL 轮转策略对比

### 1.1 PostgreSQL

**轮转机制**：

- **固定大小分段**：WAL 文件被分成固定大小的段（默认 16MB）
- **自动轮转**：当一个段填满后，PostgreSQL 自动创建新的段文件
- **文件命名**：使用 24 字符十六进制文件名（如 `000000010000000000000001`）
- **回收机制**：旧的 WAL 段会被回收重用或移除

**关键特性**：

```sql
-- 配置参数
wal_segment_size = 16MB          -- 单个 WAL 段大小
wal_keep_segments = N            -- 保留的段数量（用于复制）
max_wal_size = 1GB               -- WAL 文件最大总大小
min_wal_size = 80MB              -- WAL 文件最小大小
```

**检查点触发**：

- 当 WAL 总量超过 `max_wal_size` 时触发检查点
- 检查点完成后，旧 WAL 文件可被回收

### 1.2 RocksDB

**轮转机制**：

- **多文件管理**：不采用固定大小轮转，而是基于多个配置参数管理
- **TTL 策略**：基于时间的过期删除
- **大小限制**：基于总大小的删除
- **归档目录**：支持单独的 WAL 目录

**关键配置**：

```cpp
DBOptions {
  wal_dir: string                    // WAL 文件目录
  WAL_ttl_seconds: uint64_t          // WAL 文件存活时间（秒）
  WAL_size_limit_MB: uint64_t        // WAL 总大小限制（MB）
  max_total_wal_size: uint64_t       // 最大总 WAL 大小
  manual_wal_flush: bool             // 是否手动刷新 WAL
}
```

**删除策略**：

1. **仅 TTL**：`WAL_ttl_seconds > 0` 且 `WAL_size_limit_MB == 0`
   - 定期删除超过 TTL 的 WAL 文件
2. **仅大小**：`WAL_ttl_seconds == 0` 且 `WAL_size_limit_MB > 0`
   - 每 10 分钟检查，删除最早的文件直到总大小在限制内
3. **两者结合**：两个参数都不为 0
   - 先执行 TTL 检查，再执行大小检查
4. **默认行为**：两个参数都为 0
   - WAL 文件在不再需要时立即删除

### 1.3 SQLite

**轮转机制**：

- **单文件模式**：SQLite WAL 通常是单个文件（`-wal` 后缀）
- **检查点截断**：通过检查点机制截断 WAL 文件
- **自动检查点**：默认每 1000 页触发一次自动检查点

**关键配置**：

```sql
PRAGMA wal_autocheckpoint = 1000;  -- 自动检查点阈值（页数）
PRAGMA wal_checkpoint(TRUNCATE);   -- 截断 WAL 文件
```

**特点**：

- 不支持多文件轮转
- 依赖检查点来管理 WAL 文件大小
- 简单但有效的设计

---

## 2. WAL 轮转的核心设计模式

### 2.1 固定大小轮转（PostgreSQL 模式）

**优点**：

- ✅ 可预测的文件管理
- ✅ 便于归档和备份
- ✅ 支持增量备份和 PITR（时间点恢复）
- ✅ 文件可以回收重用，减少文件系统开销

**缺点**：

- ❌ 固定大小可能不适合所有场景
- ❌ 需要复杂的文件命名和追踪系统
- ❌ 小文件过多可能影响文件系统性能

**适用场景**：

- 高吞吐写入场景
- 需要归档和长期保留 WAL 的场景
- 需要流复制的场景

### 2.2 TTL + 大小限制（RocksDB 模式）

**优点**：

- ✅ 灵活的时间控制
- ✅ 灵活的容量控制
- ✅ 实现相对简单
- ✅ 适合嵌入式场景

**缺点**：

- ❌ 不支持归档
- ❌ 文件数量不可控
- ❌ 可能产生大量小文件

**适用场景**：

- 嵌入式数据库
- 不需要长期归档的场景
- 本地存储场景

### 2.3 单文件检查点（SQLite 模式）

**优点**：

- ✅ 实现最简单
- ✅ 文件管理开销最小
- ✅ 适合轻量级应用

**缺点**：

- ❌ 单文件大小受限
- ❌ 检查点时可能阻塞
- ❌ 不支持并行处理

**适用场景**：

- 轻量级嵌入式应用
- 写入量不大的场景
- 移动端应用

---

## 3. GraphDB 当前实现分析

### 3.1 现有代码状态

根据代码分析，GraphDB 的 WAL 模块位于 `src/transaction/wal/writer.rs`：

**当前配置**：

```rust
pub struct LocalWalWriter {
    wal_uri: String,              // WAL 文件路径
    file_size: usize,             // 当前文件大小
    file_used: usize,             // 已使用大小
    version: u32,                 // 版本计数器
    config: WalConfig,            // 配置
    // ...
}
```

**已删除的代码**（刚刚清理）：

```rust
// 这两个方法已被删除，因为未被使用
fn rotate_if_needed(&mut self) -> WalResult<()> {
    if self.file_used >= self.config.max_file_size {
        self.rotate()?;
    }
    Ok(())
}

fn rotate(&mut self) -> WalResult<()> {
    // 创建新文件的逻辑
}
```

**当前问题**：

1. ❌ 缺少文件轮转逻辑
2. ❌ WAL 文件会无限增长
3. ❌ 没有归档机制
4. ❌ 没有清理过期 WAL 文件的策略
5. ❌ 不支持检查点截断

### 3.2 配置参数分析

当前 `WalConfig` 包含：

```rust
pub struct WalConfig {
    max_file_size: usize,         // 最大文件大小（但未使用）
    truncate_size: usize,         // 预分配大小
    sync_policy: SyncPolicy,      // 同步策略
    compression: WalCompression,  // 压缩配置
    checksum: bool,               // 校验和
    // ...
}
```

**缺失的配置**：

- TTL 配置
- 最大总大小配置
- 归档目录配置
- 检查点间隔配置

---

## 4. 推荐方案

基于 GraphDB 的使用场景（单机图数据库，嵌入式部署），推荐采用 **RocksDB 模式 + PostgreSQL 部分特性** 的混合方案。

### 4.1 方案概述

**核心策略**：

1. **多文件轮转**：采用固定大小轮转（类似 PostgreSQL）
2. **TTL + 大小限制**：双重清理策略（类似 RocksDB）
3. **检查点机制**：支持检查点截断（类似 SQLite）
4. **归档支持**：可选的归档目录

### 4.2 详细设计

#### 4.2.1 文件轮转策略

```rust
pub struct WalConfig {
    // 轮转相关
    max_file_size: usize,         // 单个 WAL 文件最大大小（默认 16MB）
    max_total_size: usize,        // WAL 文件总大小限制（默认 256MB）
    ttl_seconds: u64,             // WAL 文件存活时间（默认 0，不过期）

    // 检查点相关
    checkpoint_interval: u64,     // 检查点间隔（LSN 增量，默认 10000）
    auto_checkpoint: bool,        // 是否自动检查点（默认 true）

    // 归档相关
    archive_dir: Option<String>,  // 归档目录（默认 None）
    archive_mode: ArchiveMode,    // 归档模式（默认 None）

    // 现有配置
    truncate_size: usize,         // 预分配大小
    sync_policy: SyncPolicy,      // 同步策略
    compression: WalCompression,  // 压缩配置
    checksum: bool,               // 校验和
}
```

#### 4.2.2 轮转触发条件

```rust
impl LocalWalWriter {
    fn should_rotate(&self) -> bool {
        // 条件 1: 当前文件达到最大大小
        self.file_used >= self.config.max_file_size
    }

    fn should_cleanup(&self) -> bool {
        // 条件 1: 总大小超过限制
        self.get_total_wal_size() > self.config.max_total_size
        ||
        // 条件 2: 存在过期文件
        self.has_expired_wal_files()
    }
}
```

#### 4.2.3 文件命名规范

采用 PostgreSQL 风格的命名：

```
wal_00000001  // 序列号，8 位十六进制
wal_00000002
wal_00000003
...
```

或采用 LSN 基础命名：

```
wal_0000000100000000  // 起始 LSN，16 位十六进制
wal_0000000100004000  // 下一个段
```

#### 4.2.4 清理策略

```rust
pub enum CleanupPolicy {
    /// 立即删除不再需要的 WAL 文件
    Immediate,

    /// 移动到归档目录
    Archive,

    /// 保留最近 N 个文件
    KeepLastN(usize),

    /// 保留最近 N 秒的文件
    KeepLastNSeconds(u64),
}
```

### 4.3 实现优先级

#### Phase 1: 基础轮转（必须）

- [ ] 实现 `rotate()` 方法
- [ ] 实现 `rotate_if_needed()` 方法
- [ ] 添加文件序列号管理
- [ ] 在 `append()` 中调用轮转检查

#### Phase 2: 清理策略（重要）

- [ ] 实现基于大小的清理
- [ ] 实现基于 TTL 的清理
- [ ] 添加 `cleanup_old_wal_files()` 方法
- [ ] 在检查点时触发清理

#### Phase 3: 检查点集成（重要）

- [ ] 增强 `CheckpointManager`
- [ ] 记录每个检查点的 WAL 文件列表
- [ ] 实现安全的文件删除逻辑
- [ ] 支持检查点截断

#### Phase 4: 归档支持（可选）

- [ ] 实现归档目录管理
- [ ] 添加归档配置
- [ ] 实现归档文件移动
- [ ] 支持归档恢复

---

## 5. 具体实现建议

### 5.1 修改 WalConfig

```rust
pub struct WalConfig {
    // 现有字段
    pub truncate_size: usize,
    pub sync_policy: SyncPolicy,
    pub compression: WalCompression,
    pub checksum: bool,

    // 新增字段
    /// 单个 WAL 文件最大大小（字节）
    /// 默认值：16MB (16 * 1024 * 1024)
    pub max_file_size: usize,

    /// WAL 文件总大小限制（字节）
    /// 默认值：256MB
    /// 0 表示无限制
    pub max_total_size: usize,

    /// WAL 文件存活时间（秒）
    /// 默认值：0（不过期）
    pub ttl_seconds: u64,

    /// 检查点间隔（LSN 增量）
    /// 默认值：10000
    pub checkpoint_interval: u64,

    /// 是否启用自动检查点
    /// 默认值：true
    pub auto_checkpoint: bool,

    /// 归档目录路径
    /// 默认值：None
    pub archive_dir: Option<String>,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            truncate_size: 64 * 1024 * 1024,  // 64MB 预分配
            sync_policy: SyncPolicy::Immediate,
            compression: WalCompression::None,
            checksum: true,
            max_file_size: 16 * 1024 * 1024,   // 16MB
            max_total_size: 256 * 1024 * 1024, // 256MB
            ttl_seconds: 0,                     // 不过期
            checkpoint_interval: 10000,
            auto_checkpoint: true,
            archive_dir: None,
        }
    }
}
```

### 5.2 恢复 rotate 方法

```rust
impl LocalWalWriter {
    /// 检查是否需要轮转到新文件
    fn rotate_if_needed(&mut self) -> WalResult<()> {
        if self.file_used >= self.config.max_file_size {
            self.rotate()?;
        }
        Ok(())
    }

    /// 轮转到新的 WAL 文件
    fn rotate(&mut self) -> WalResult<()> {
        // 同步当前文件
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }

        // 递增版本号
        self.version += 1;

        // 生成新文件路径
        let new_path = self.get_wal_file_path(self.version);

        // 打开新文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&new_path)?;

        // 预分配空间
        file.set_len(self.config.truncate_size as u64)?;

        // 更新状态
        self.file = Some(file);
        self.file_path = Some(new_path);
        self.file_size = self.config.truncate_size;
        self.file_used = 0;

        // 写入文件头
        self.write_file_header()?;

        log::info!("WAL rotated to version {}, file: {:?}",
                   self.version, self.file_path);

        Ok(())
    }

    /// 生成 WAL 文件路径
    fn get_wal_file_path(&self, version: u32) -> PathBuf {
        PathBuf::from(&self.wal_uri)
            .join(format!("wal_{:08d}", version))
    }

    /// 清理旧的 WAL 文件
    fn cleanup_old_wal_files(&mut self) -> WalResult<usize> {
        let mut deleted_count = 0;

        // 获取所有 WAL 文件
        let mut wal_files = self.list_wal_files()?;

        // 按版本号排序
        wal_files.sort();

        // 基于大小的清理
        let total_size = self.get_total_wal_size()?;
        if total_size > self.config.max_total_size && self.config.max_total_size > 0 {
            let mut current_size = total_size;

            for file in &wal_files {
                if current_size <= self.config.max_total_size {
                    break;
                }

                let file_size = std::fs::metadata(file)?.len() as usize;

                // 如果是归档模式，移动到归档目录
                if let Some(ref archive_dir) = self.config.archive_dir {
                    self.archive_wal_file(file, archive_dir)?;
                } else {
                    std::fs::remove_file(file)?;
                }

                current_size -= file_size;
                deleted_count += 1;
            }
        }

        // 基于 TTL 的清理
        if self.config.ttl_seconds > 0 {
            let now = std::time::Instant::now();
            let ttl = std::time::Duration::from_secs(self.config.ttl_seconds);

            for file in &wal_files {
                let metadata = std::fs::metadata(file)?;
                let modified = metadata.modified()?;

                if now.duration_since(modified.into()) > ttl {
                    if let Some(ref archive_dir) = self.config.archive_dir {
                        self.archive_wal_file(file, archive_dir)?;
                    } else {
                        std::fs::remove_file(file)?;
                    }
                    deleted_count += 1;
                }
            }
        }

        if deleted_count > 0 {
            log::info!("Cleaned up {} old WAL files", deleted_count);
        }

        Ok(deleted_count)
    }
}
```

### 5.3 集成到 append 方法

```rust
impl WalWriter for LocalWalWriter {
    fn append(&mut self, data: &[u8]) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        // 检查是否需要轮转
        self.rotate_if_needed()?;

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        // 写入数据
        file.write_all(data)?;
        self.file_used += data.len();

        // 检查是否需要清理
        if self.config.max_total_size > 0 || self.config.ttl_seconds > 0 {
            self.cleanup_old_wal_files()?;
        }

        // 自动检查点
        if self.config.auto_checkpoint {
            self.maybe_trigger_checkpoint()?;
        }

        Ok(true)
    }
}
```

---

## 6. 测试建议

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_wal_rotation_basic() {
        // 测试基本的文件轮转功能
    }

    #[test]
    fn test_wal_rotation_max_size() {
        // 测试达到最大大小时的轮转
    }

    #[test]
    fn test_wal_cleanup_by_size() {
        // 测试基于大小的清理
    }

    #[test]
    fn test_wal_cleanup_by_ttl() {
        // 测试基于 TTL 的清理
    }

    #[test]
    fn test_wal_archive() {
        // 测试归档功能
    }
}
```

### 6.2 集成测试

```rust
#[test]
fn test_wal_rotation_with_recovery() {
    // 测试轮转后的恢复功能
}

#[test]
fn test_wal_rotation_concurrent() {
    // 测试并发写入时的轮转
}
```

---

## 7. 性能考虑

### 7.1 文件大小选择

- **太小**（< 4MB）：文件数量过多，文件系统开销大
- **太大**（> 64MB）：单个文件过大，恢复时间长
- **推荐**：16MB（PostgreSQL 默认值）

### 7.2 轮转时机

- 在 `append()` 时检查会引入额外开销
- 建议在写入大对象前检查
- 可以考虑异步轮转

### 7.3 清理频率

- 每次 append 都检查开销太大
- 建议每 N 次写入或检查点时清理
- 可以使用计数器控制

---

## 8. 总结

### 推荐方案总结

| 特性         | 推荐实现   | 优先级     |
| ------------ | ---------- | ---------- |
| 固定大小轮转 | 16MB 分段  | P0（必须） |
| 总大小限制   | 256MB 默认 | P0（必须） |
| TTL 清理     | 可配置     | P1（重要） |
| 检查点集成   | LSN 触发   | P1（重要） |
| 归档支持     | 可选目录   | P2（可选） |
| 文件重用     | 序列号命名 | P1（重要） |

### 下一步行动

1. **立即**：恢复 `rotate()` 和 `rotate_if_needed()` 方法
2. **本周**：实现基础轮转和清理逻辑
3. **下周**：集成检查点机制
4. **后续**：添加归档支持和性能优化

---

## 参考文献

1. PostgreSQL WAL Documentation: https://www.postgresql.org/docs/current/wal.html
2. RocksDB WAL Configuration: https://github.com/facebook/rocksdb/wiki/Write-Ahead-Log-(WAL)
3. SQLite WAL Mode: https://www.sqlite.org/wal.html
4. Understanding Write-Ahead Logging: https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation
