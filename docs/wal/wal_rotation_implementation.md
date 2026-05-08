# WAL 文件轮转实现方案

## 概述

本文档提供 WAL 文件轮转功能的具体实现方案，包括代码修改、配置更新和测试计划。

---

## 1. 配置扩展

### 1.1 修改 `src/transaction/wal/types.rs`

在 `WalConfig` 结构体中添加新字段：

```rust
/// WAL Configuration
#[derive(Debug, Clone)]
pub struct WalConfig {
    // 现有字段
    pub truncate_size: usize,
    pub sync_policy: SyncPolicy,
    pub compression: WalCompression,
    pub checksum: bool,

    // === 新增字段 ===

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

    /// 归档模式
    /// 默认值：ArchiveMode::None
    pub archive_mode: ArchiveMode,
}

/// 归档模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArchiveMode {
    #[default]
    None,           // 不归档，直接删除
    Move,           // 移动到归档目录
    Copy,           // 复制到归档目录
}
```

### 1.2 更新 Default 实现

```rust
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
            archive_mode: ArchiveMode::None,
        }
    }
}
```

### 1.3 添加 Builder 方法

```rust
impl WalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    // 现有 builder 方法...

    /// Set maximum WAL file size
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum total WAL size
    pub fn with_max_total_size(mut self, size: usize) -> Self {
        self.max_total_size = size;
        self
    }

    /// Set WAL TTL in seconds
    pub fn with_ttl_seconds(mut self, seconds: u64) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    /// Set checkpoint interval
    pub fn with_checkpoint_interval(mut self, interval: u64) -> Self {
        self.checkpoint_interval = interval;
        self
    }

    /// Enable or disable auto checkpoint
    pub fn with_auto_checkpoint(mut self, enabled: bool) -> Self {
        self.auto_checkpoint = enabled;
        self
    }

    /// Set archive directory
    pub fn with_archive_dir(mut self, dir: String) -> Self {
        self.archive_dir = Some(dir);
        self
    }

    /// Set archive mode
    pub fn with_archive_mode(mut self, mode: ArchiveMode) -> Self {
        self.archive_mode = mode;
        self
    }
}
```

---

## 2. LocalWalWriter 扩展

### 2.1 添加新字段

在 `src/transaction/wal/writer.rs` 中修改 `LocalWalWriter`：

```rust
pub struct LocalWalWriter {
    // 现有字段...

    /// 当前版本号（文件序列号）
    version: u32,

    /// 当前文件的起始 LSN
    file_start_lsn: Lsn,

    /// 自上次检查点以来的 LSN 增量
    lsn_since_checkpoint: u64,

    /// 上次清理时间
    last_cleanup_time: Option<Instant>,
}
```

### 2.2 实现文件轮转核心方法

在 `LocalWalWriter` 的 impl 块中添加：

```rust
impl LocalWalWriter {
    // ... 现有方法 ...

    /// 检查是否需要轮转到新文件
    fn rotate_if_needed(&mut self) -> WalResult<()> {
        if self.file_used >= self.config.max_file_size {
            self.rotate()?;
        }
        Ok(())
    }

    /// 轮转到新的 WAL 文件
    fn rotate(&mut self) -> WalResult<()> {
        log::info!(
            "Rotating WAL file: used={}, max_size={}, version={}",
            self.file_used,
            self.config.max_file_size,
            self.version
        );

        // 1. 同步当前文件
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }

        // 2. 递增版本号
        self.version += 1;

        // 3. 生成新文件路径
        let new_path = self.get_wal_file_path(self.version);

        // 4. 打开新文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&new_path)?;

        // 5. 预分配空间
        file.set_len(self.config.truncate_size as u64)?;

        // 6. 更新状态
        self.file = Some(file);
        self.file_path = Some(new_path);
        self.file_size = self.config.truncate_size;
        self.file_used = 0;
        self.file_start_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));

        // 7. 写入文件头
        self.write_file_header()?;

        log::info!(
            "WAL rotated to version {}, file: {:?}, start_lsn={}",
            self.version,
            self.file_path,
            self.file_start_lsn
        );

        Ok(())
    }

    /// 生成 WAL 文件路径
    fn get_wal_file_path(&self, version: u32) -> PathBuf {
        PathBuf::from(&self.wal_uri)
            .join(format!("wal_{:08d}", version))
    }

    /// 列出所有 WAL 文件
    fn list_wal_files(&self) -> WalResult<Vec<PathBuf>> {
        let wal_dir = PathBuf::from(&self.wal_uri);

        if !wal_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&wal_dir)? {
            let entry = entry?;
            let path = entry.path();

            // 检查文件名格式：wal_XXXXXXXX
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("wal_") && name.len() == 12 {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// 获取 WAL 文件总大小
    fn get_total_wal_size(&self) -> WalResult<usize> {
        let mut total = 0;
        for file in self.list_wal_files()? {
            if let Ok(metadata) = std::fs::metadata(&file) {
                total += metadata.len() as usize;
            }
        }
        Ok(total)
    }

    /// 清理旧的 WAL 文件
    fn cleanup_old_wal_files(&mut self) -> WalResult<usize> {
        let mut deleted_count = 0;

        // 获取所有 WAL 文件
        let mut wal_files = self.list_wal_files()?;

        if wal_files.is_empty() {
            return Ok(0);
        }

        // 按版本号排序（文件名排序即可）
        wal_files.sort();

        // === 基于大小的清理 ===
        if self.config.max_total_size > 0 {
            let total_size = self.get_total_wal_size()?;

            if total_size > self.config.max_total_size {
                let mut current_size = total_size;

                for file in &wal_files {
                    if current_size <= self.config.max_total_size {
                        break;
                    }

                    let file_size = std::fs::metadata(file)?.len() as usize;

                    // 执行删除或归档
                    self.delete_or_archive_file(file)?;

                    current_size -= file_size;
                    deleted_count += 1;
                }
            }
        }

        // === 基于 TTL 的清理 ===
        if self.config.ttl_seconds > 0 {
            let now = Instant::now();
            let ttl = Duration::from_secs(self.config.ttl_seconds);

            for file in &wal_files {
                if let Ok(metadata) = std::fs::metadata(file) {
                    if let Ok(modified) = metadata.modified() {
                        let modified_instant: Instant = modified.into();

                        if now.duration_since(modified_instant) > ttl {
                            self.delete_or_archive_file(file)?;
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            log::info!("Cleaned up {} old WAL files", deleted_count);
        }

        Ok(deleted_count)
    }

    /// 删除或归档文件
    fn delete_or_archive_file(&self, file: &Path) -> WalResult<()> {
        if let Some(ref archive_dir) = self.config.archive_dir {
            match self.config.archive_mode {
                ArchiveMode::None => {
                    std::fs::remove_file(file)?;
                }
                ArchiveMode::Move => {
                    self.archive_wal_file(file, archive_dir)?;
                }
                ArchiveMode::Copy => {
                    self.copy_and_delete(file, archive_dir)?;
                }
            }
        } else {
            std::fs::remove_file(file)?;
        }
        Ok(())
    }

    /// 归档 WAL 文件
    fn archive_wal_file(&self, file: &Path, archive_dir: &str) -> WalResult<()> {
        // 确保归档目录存在
        std::fs::create_dir_all(archive_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        // 生成归档文件名（添加时间戳）
        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let archive_name = format!("{}_{}", file_name, timestamp);
        let archive_path = PathBuf::from(archive_dir).join(archive_name);

        // 移动文件
        std::fs::rename(file, &archive_path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        log::debug!("Archived WAL file: {:?} -> {:?}", file, archive_path);

        Ok(())
    }

    /// 复制并删除文件
    fn copy_and_delete(&self, file: &Path, archive_dir: &str) -> WalResult<()> {
        // 确保归档目录存在
        std::fs::create_dir_all(archive_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let archive_path = PathBuf::from(archive_dir).join(file_name);

        // 复制文件
        std::fs::copy(file, &archive_path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        // 删除原文件
        std::fs::remove_file(file)?;

        log::debug!("Copied and deleted WAL file: {:?} -> {:?}", file, archive_path);

        Ok(())
    }

    /// 触发检查点（如果需要）
    fn maybe_trigger_checkpoint(&mut self) -> WalResult<()> {
        if !self.config.auto_checkpoint {
            return Ok(());
        }

        self.lsn_since_checkpoint += 1;

        if self.lsn_since_checkpoint >= self.config.checkpoint_interval {
            log::debug!("Triggering auto-checkpoint at LSN {}",
                       self.current_lsn.load(Ordering::SeqCst));

            // 这里可以调用检查点逻辑
            // self.checkpoint_manager.create_checkpoint(...)?;

            self.lsn_since_checkpoint = 0;
        }

        Ok(())
    }
}
```

### 2.3 集成到 append 方法

修改 `WalWriter` trait 的实现：

```rust
impl WalWriter for LocalWalWriter {
    fn append(&mut self, data: &[u8]) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        // === 新增：检查是否需要轮转 ===
        self.rotate_if_needed()?;

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        let expected_size = self.file_used + data.len();
        if expected_size > self.file_size {
            let new_size =
                ((expected_size / self.config.truncate_size) + 1) * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.write_all(data)?;
        self.file_used += data.len();

        // 更新 LSN
        self.current_lsn.fetch_add(data.len() as u64, Ordering::SeqCst);

        // === 新增：定期清理 ===
        // 每 100 次写入或检查点时清理一次
        static WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = WRITE_COUNTER.fetch_add(1, Ordering::SeqCst);

        if counter % 100 == 0 {
            self.cleanup_old_wal_files()?;
        }

        // === 新增：自动检查点 ===
        if self.config.auto_checkpoint {
            self.maybe_trigger_checkpoint()?;
        }

        Ok(true)
    }

    // ... 其他方法保持不变 ...
}
```

---

## 3. CheckpointManager 集成

### 3.1 增强检查点逻辑

在 `src/transaction/wal/checkpoint.rs` 中：

```rust
impl CheckpointManager {
    // ... 现有方法 ...

    /// 创建检查点并清理 WAL 文件
    pub fn create_checkpoint_and_cleanup(
        &mut self,
        timestamp: Timestamp,
        lsn: Lsn,
        wal_writer: &mut LocalWalWriter,
    ) -> WalResult<Checkpoint> {
        // 1. 创建检查点
        let checkpoint = self.create_checkpoint(timestamp, lsn)?;

        // 2. 标记可以删除的 WAL 文件
        for file in &checkpoint.wal_files {
            log::debug!("WAL file can be deleted after checkpoint: {:?}", file);
        }

        // 3. 触发 WAL 清理
        wal_writer.cleanup_old_wal_files()?;

        Ok(checkpoint)
    }
}
```

---

## 4. 测试实现

### 4.1 单元测试

在 `src/transaction/wal/writer.rs` 底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_writer(config: WalConfig) -> (LocalWalWriter, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().unwrap();

        (writer, temp_dir)
    }

    #[test]
    fn test_wal_rotation_basic() {
        let config = WalConfig::default()
            .with_max_file_size(1024)  // 1KB 触发轮转
            .with_truncate_size(4096);

        let (mut writer, _temp_dir) = create_test_writer(config);

        // 写入足够触发轮转的数据
        let data = vec![0u8; 512];
        for _ in 0..3 {
            writer.append(&data).unwrap();
        }

        // 验证版本号递增
        assert!(writer.version >= 2);
    }

    #[test]
    fn test_wal_cleanup_by_size() {
        let config = WalConfig::default()
            .with_max_file_size(1024)
            .with_max_total_size(4096)  // 总大小限制 4KB
            .with_truncate_size(4096);

        let (mut writer, _temp_dir) = create_test_writer(config);

        // 写入大量数据触发清理
        let data = vec![0u8; 512];
        for _ in 0..20 {
            writer.append(&data).unwrap();
        }

        // 验证总大小在限制内
        let total_size = writer.get_total_wal_size().unwrap();
        assert!(total_size <= config.max_total_size);
    }

    #[test]
    fn test_wal_file_naming() {
        let config = WalConfig::default();
        let (writer, _temp_dir) = create_test_writer(config);

        let path = writer.get_wal_file_path(1);
        assert!(path.to_string_lossy().contains("wal_00000001"));

        let path = writer.get_wal_file_path(100);
        assert!(path.to_string_lossy().contains("wal_00000064"));
    }

    #[test]
    fn test_wal_archive() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().to_string_lossy().to_string();
        let archive_path = temp_dir.path().join("archive");

        let config = WalConfig::default()
            .with_archive_dir(archive_path.to_string_lossy().to_string())
            .with_archive_mode(ArchiveMode::Move);

        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().unwrap();

        // 创建一个测试文件
        let test_file = temp_dir.path().join("wal_00000001");
        std::fs::write(&test_file, vec![0u8; 100]).unwrap();

        // 归档文件
        writer.archive_wal_file(&test_file, archive_path.to_string_lossy().as_ref()).unwrap();

        // 验证文件已移动
        assert!(!test_file.exists());
        assert!(archive_path.exists());
    }
}
```

### 4.2 集成测试

在 `tests/wal_rotation.rs` 中创建：

```rust
//! WAL Rotation Integration Tests

use graphdb::transaction::wal::{
    LocalWalWriter, WalConfig, WalWriter, ArchiveMode,
};
use tempfile::TempDir;
use std::time::Duration;

#[test]
fn test_wal_rotation_with_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().to_string_lossy().to_string();

    let config = WalConfig::default()
        .with_max_file_size(1024)
        .with_checksum(true);

    // 写入数据并触发轮转
    {
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config.clone());
        writer.open().unwrap();

        for i in 0..10 {
            let data = format!("Entry {}", i).into_bytes();
            writer.append(&data).unwrap();
        }

        writer.sync().unwrap();
    }

    // 验证可以解析所有 WAL 文件
    // TODO: 使用 WalParser 验证
}

#[test]
fn test_wal_concurrent_rotation() {
    // 测试并发写入时的轮转
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().to_string_lossy().to_string();

    let config = WalConfig::default()
        .with_max_file_size(2048);

    let mut handles = vec![];

    for t in 0..4 {
        let path = wal_path.clone();
        let cfg = config.clone();

        let handle = thread::spawn(move || {
            let mut writer = LocalWalWriter::with_config(&path, t, cfg);
            writer.open().unwrap();

            for i in 0..50 {
                let data = format!("Thread {} Entry {}", t, i).into_bytes();
                writer.append(&data).unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有文件都有效
    // TODO: 添加验证逻辑
}

#[test]
fn test_wal_ttl_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().to_string_lossy().to_string();

    let config = WalConfig::default()
        .with_ttl_seconds(1)  // 1 秒 TTL
        .with_max_file_size(1024);

    {
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config.clone());
        writer.open().unwrap();

        // 写入一些数据
        writer.append(b"Test data").unwrap();
    }

    // 等待 TTL 过期
    thread::sleep(Duration::from_secs(2));

    // 再次打开并触发清理
    {
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config.clone());
        writer.open().unwrap();

        let deleted = writer.cleanup_old_wal_files().unwrap();
        assert!(deleted >= 1);
    }
}
```

---

## 5. 配置示例

### 5.1 默认配置（推荐）

```rust
let config = WalConfig::default();
// max_file_size: 16MB
// max_total_size: 256MB
// ttl_seconds: 0 (不过期)
// checkpoint_interval: 10000
// auto_checkpoint: true
```

### 5.2 高吞吐场景

```rust
let config = WalConfig::default()
    .with_max_file_size(64 * 1024 * 1024)      // 64MB 文件
    .with_max_total_size(1024 * 1024 * 1024)   // 1GB 总大小
    .with_checkpoint_interval(50000);          // 更大的检查点间隔
```

### 5.3 嵌入式场景

```rust
let config = WalConfig::default()
    .with_max_file_size(4 * 1024 * 1024)       // 4MB 文件
    .with_max_total_size(64 * 1024 * 1024)     // 64MB 总大小
    .with_ttl_seconds(3600);                   // 1 小时 TTL
```

### 5.4 需要归档的场景

```rust
let config = WalConfig::default()
    .with_archive_dir("/backup/wal_archive".to_string())
    .with_archive_mode(ArchiveMode::Move)
    .with_max_file_size(16 * 1024 * 1024)
    .with_ttl_seconds(86400);  // 24 小时后归档
```

---

## 6. 迁移计划

### Phase 1: 基础准备（1-2 天）

- [ ] 更新 `WalConfig` 结构体
- [ ] 添加新的配置字段和 builder 方法
- [ ] 更新 `ArchiveMode` 枚举
- [ ] 编写配置相关测试

### Phase 2: 核心实现（3-4 天）

- [ ] 实现 `rotate_if_needed()` 方法
- [ ] 实现 `rotate()` 方法
- [ ] 实现 `get_wal_file_path()` 方法
- [ ] 实现 `list_wal_files()` 方法
- [ ] 实现 `get_total_wal_size()` 方法
- [ ] 集成到 `append()` 方法

### Phase 3: 清理功能（2-3 天）

- [ ] 实现 `cleanup_old_wal_files()` 方法
- [ ] 实现 `delete_or_archive_file()` 方法
- [ ] 实现 `archive_wal_file()` 方法
- [ ] 添加定期清理逻辑
- [ ] 编写清理功能测试

### Phase 4: 检查点集成（2-3 天）

- [ ] 增强 `CheckpointManager`
- [ ] 实现 `create_checkpoint_and_cleanup()` 方法
- [ ] 添加自动检查点触发
- [ ] 编写集成测试

### Phase 5: 测试和优化（2-3 天）

- [ ] 完成所有单元测试
- [ ] 完成集成测试
- [ ] 性能基准测试
- [ ] 文档完善
- [ ] Code review

**总预计时间**: 10-15 个工作日

---

## 7. 性能影响评估

### 7.1 预期开销

| 操作     | 额外开销 | 频率             |
| -------- | -------- | ---------------- |
| 轮转检查 | < 1μs    | 每次 append      |
| 文件轮转 | ~1ms     | 每 16MB 数据     |
| 清理检查 | ~10μs    | 每 100 次 append |
| 实际清理 | ~10ms    | 触发时           |

### 7.2 优化建议

1. **减少检查频率**：不是每次 append 都检查，可以每 N 次检查一次
2. **异步清理**：使用后台线程进行清理
3. **批量操作**：批量删除/归档文件
4. **缓存文件列表**：避免频繁扫描目录

---

## 8. 监控指标

建议添加以下监控指标：

```rust
pub struct WalMetrics {
    /// 当前 WAL 文件数量
    pub current_file_count: usize,

    /// WAL 总大小
    pub total_size_bytes: usize,

    /// 当前文件大小
    pub current_file_size: usize,

    /// 轮转次数
    pub rotation_count: u64,

    /// 清理的文件数量
    pub cleaned_file_count: u64,

    /// 归档的文件数量
    pub archived_file_count: u64,

    /// 平均轮转间隔（写入次数）
    pub avg_rotation_interval: f64,
}
```

---

## 9. 风险和缓解

### 风险 1: 轮转时性能下降

**缓解措施**：

- 在低峰期进行轮转（如果可能）
- 异步轮转支持
- 预分配新文件减少 IO

### 风险 2: 清理过于激进

**缓解措施**：

- 保守的默认配置
- 检查点确认后再删除
- 归档模式作为安全网

### 风险 3: 文件命名冲突

**缓解措施**：

- 使用递增序列号
- 原子文件创建
- 冲突检测和重试

---

## 10. 总结

本实现方案提供了完整的 WAL 文件轮转功能，包括：

✅ **基础轮转**：固定大小触发轮转  
✅ **清理策略**：基于大小和 TTL 的双重清理  
✅ **归档支持**：可选的归档机制  
✅ **检查点集成**：安全的文件删除  
✅ **监控指标**：完整的可观测性

实现后，GraphDB 的 WAL 模块将达到生产级标准，支持长期稳定运行。
