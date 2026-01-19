# src/common 组件集成指南

## 概述

本文档说明了 `src/common` 目录中各个组件应该如何集成到 GraphDB 项目中，以及它们应该与哪些模块配合使用。

## 已修复的问题

### 1. memory.rs - 内存安全修复 ✅

**修复内容：**
- 将 `MemoryPool::allocate()` 返回的裸指针 `*mut u8` 改为安全的 `MemoryChunk` 类型
- `MemoryChunk` 实现了 `Drop` trait，自动管理内存释放
- 添加了 `as_slice()` 和 `as_mut_slice()` 方法提供安全的内存访问

**集成点：**
- `src/storage/memory_storage.rs` - 使用 MemoryPool 管理内存分配
- `src/query/executor/memory_manager.rs` - 使用 MemoryPool 管理查询执行时的内存
- `src/core/allocator.rs` - 可以使用 MemoryPool 作为底层分配器

**使用示例：**
```rust
use graphdb::common::memory::MemoryPool;

let pool = MemoryPool::new(1024 * 1024); // 1MB 池
let mut chunk = pool.allocate(100).expect("Allocation failed");
let slice = chunk.as_mut_slice();
slice[0] = 42;
// chunk 在 drop 时自动释放
```

### 2. base/id.rs - ID 溢出修复 ✅

**修复内容：**
- 为 `generate_tag_id()`, `generate_edge_type()`, `generate_space_id()`, `generate_index_id()` 添加溢出检查
- 当 ID 超过 `i32::MAX` 时会 panic 并给出明确的错误信息

**集成点：**
- `src/storage/memory_storage.rs` - 使用 ID 生成器创建顶点和边 ID
- `src/graph/schema.rs` - 使用 TagId 和 EdgeType 管理模式定义
- `src/index/storage.rs` - 使用 IndexId 管理索引

**使用示例：**
```rust
use graphdb::common::base::id::{gen_vertex_id, gen_tag_id};

let vertex_id = gen_vertex_id();
let tag_id = gen_tag_id();
```

### 3. thread.rs - ThreadPool 修复 ✅

**修复内容：**
- 使用 `tokio::sync::Notify` 替代轮询，实现事件驱动
- 添加了 `shutdown()` 和 `wait_for_completion()` 方法支持优雅关闭
- 修复了 `Lazy` 的并发初始化问题，使用 `RwLock` 替代

**集成点：**
- `src/query/executor/` - 使用 ThreadPool 并行执行查询操作
- `src/storage/` - 使用 ThreadPool 处理后台 I/O 操作
- `src/api/service/graph_service.rs` - 使用 ThreadPool 处理并发请求

**使用示例：**
```rust
use graphdb::common::thread::ThreadPool;

let pool = ThreadPool::new(4);
pool.execute(|| {
    // 执行任务
});
// pool 在 drop 时自动关闭
```

### 4. time.rs - 日期验证修复 ✅

**修复内容：**
- 添加了 `days_in_month()` 方法正确计算每个月的天数
- 添加了 `is_leap_year()` 方法判断闰年
- 修复了日期验证，现在会检查特定月份的有效天数

**集成点：**
- `src/core/value/` - 使用 Date 和 Time 类型表示时间值
- `src/query/parser/` - 解析日期时间字面量
- `src/storage/` - 存储和检索时间戳数据

**使用示例：**
```rust
use graphdb::common::time::Date;

let date = Date::new(2023, 2, 29).expect("Invalid date"); // 闰年2月29日
```

## 待修复的问题

### 5. network.rs - 空实现问题 ⚠️

**问题描述：**
- `handle_client()` 和 `serve_client()` 都是空实现
- 没有实际的请求处理逻辑
- 协议模块依赖不存在的 `crate::core::Value`

**集成点：**
- `src/api/mod.rs` - NetworkServer 应该在 `start_service()` 中启动
- `src/api/service/graph_service.rs` - NetworkServer 需要与 GraphService 集成
- `src/api/session/` - NetworkServer 需要管理客户端会话

**建议实现：**
```rust
// 在 src/api/mod.rs 中集成
pub async fn start_service(config_path: String) -> Result<()> {
    let config = Config::load(&config_path)?;
    let storage = Arc::new(MemoryStorage::new()?);
    let graph_service = Arc::new(GraphService::new(config.clone(), storage));

    let mut server = NetworkServer::new(config.network, graph_service);
    server.start().await?;

    Ok(())
}
```

### 6. process.rs - 跨平台问题 ⚠️

**问题描述：**
- Windows 平台信号处理为空实现
- `get_memory_usage()` 总是返回 0
- `get_system_usage()` 返回的都是占位值

**集成点：**
- `src/main.rs` - 使用 ProcessManager 获取进程信息
- `src/api/mod.rs` - 使用 ProcessManager 监控资源使用
- `src/config/` - 可以添加进程相关的配置项

**建议实现：**
```rust
// 使用 sysinfo crate 获取真实的系统信息
use sysinfo::{System, SystemExt};

pub fn get_system_usage() -> Result<SystemResourceUsage> {
    let mut sys = System::new_all();
    sys.refresh_all();

    Ok(SystemResourceUsage {
        total_memory: sys.total_memory(),
        available_memory: sys.available_memory(),
        used_memory: sys.used_memory(),
        cpu_count: num_cpus::get() as u8,
        load_avg: (0.0, 0.0, 0.0), // Windows 不支持
    })
}
```

### 7. fs.rs - FileLock 问题 ⚠️

**问题描述：**
- `FileLock` 只是打开文件，没有实际锁定
- 跨进程无法协调文件访问

**集成点：**
- `src/storage/` - 使用 FileLock 保护数据文件
- `src/config/` - 使用 FileLock 保护配置文件
- `src/api/` - 使用 FileLock 保护日志文件

**建议实现：**
```rust
// 使用 fs2 crate 实现真正的文件锁定
use fs2::FileExt;

impl FileLock {
    pub fn acquire_exclusive<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        let file = File::create(path.as_ref())?;
        file.lock_exclusive()?;
        Ok(Self { _file: file, _path: path.as_ref().to_path_buf() })
    }
}
```

### 8. log.rs - 日志轮转问题 ⚠️

**问题描述：**
- `FileWriter` 没有实现日志文件轮转
- 配置中有 `max_file_size` 和 `max_files` 但未使用

**集成点：**
- `src/api/mod.rs` - 在 `init_log_level()` 中初始化日志系统
- `src/main.rs` - 使用日志系统记录启动信息
- 所有模块 - 使用日志宏记录运行时信息

**建议实现：**
```rust
impl FileWriter {
    fn write(&self, entry: &LogEntry) -> Result<()> {
        let file_size = self.file.metadata()?.len();
        if file_size > self.max_file_size {
            self.rotate_files()?;
        }

        writeln!(file, "{}", entry)?;
        Ok(())
    }

    fn rotate_files(&self) -> Result<()> {
        // 删除最旧的日志文件
        // 重命名现有日志文件
        // 创建新的日志文件
    }
}
```

### 9. charset.rs - 编码检测问题 ⚠️

**问题描述：**
- `detect_encoding()` 只能检测 UTF-8 和带 BOM 的 UTF-16
- 无法准确检测 GBK、Big5 等编码

**集成点：**
- `src/storage/` - 使用 CharsetUtils 处理导入的数据
- `src/query/parser/` - 使用 CharsetUtils 处理查询字符串
- `src/api/` - 使用 CharsetUtils 处理客户端请求

**建议实现：**
```rust
// 使用 chardetng 库改进编码检测
use chardetng::CharsetDetector;

pub fn detect_encoding(bytes: &[u8]) -> Option<Encoding> {
    let mut detector = CharsetDetector::new();
    let (charset, confidence) = detector.detect(bytes, true)?;

    if confidence > 0.6 {
        match charset.as_str() {
            "UTF-8" => Some(Encoding::Utf8),
            "GBK" => Some(Encoding::Gbk),
            "Big5" => Some(Encoding::Big5),
            _ => None,
        }
    } else {
        None
    }
}
```

## 组件集成矩阵

| 组件 | 集成模块 | 优先级 | 状态 |
|------|---------|--------|------|
| memory.rs | storage, query/executor, core/allocator | 高 | ✅ 已修复 |
| base/id.rs | storage, graph/schema, index/storage | 高 | ✅ 已修复 |
| thread.rs | query/executor, storage, api/service | 高 | ✅ 已修复 |
| time.rs | core/value, query/parser, storage | 中 | ✅ 已修复 |
| network.rs | api/mod, api/service, api/session | 高 | ⚠️ 待实现 |
| process.rs | main, api/mod, config | 中 | ⚠️ 待实现 |
| fs.rs | storage, config, api | 中 | ⚠️ 待实现 |
| log.rs | api/mod, main, 所有模块 | 高 | ⚠️ 待实现 |
| charset.rs | storage, query/parser, api | 低 | ⚠️ 待实现 |

## 建议的集成顺序

1. **第一阶段（高优先级）：**
   - 在 `src/api/mod.rs` 中集成 `log.rs` 和 `network.rs`
   - 在 `src/storage/memory_storage.rs` 中集成 `memory.rs` 和 `base/id.rs`
   - 在 `src/query/executor/` 中集成 `thread.rs`

2. **第二阶段（中优先级）：**
   - 在 `src/storage/` 中集成 `fs.rs` 的 FileLock
   - 在 `src/main.rs` 中集成 `process.rs` 的系统监控
   - 在所有模块中集成 `time.rs` 的时间处理

3. **第三阶段（低优先级）：**
   - 在数据导入功能中集成 `charset.rs`
   - 添加完整的测试覆盖所有 common 组件

## 测试建议

为每个集成点添加测试：

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_memory_pool_integration() {
        let pool = MemoryPool::new(1024);
        let mut chunk = pool.allocate(100).expect("Allocation failed");
        assert_eq!(chunk.len(), 100);
    }

    #[test]
    fn test_id_generator_integration() {
        let id1 = gen_vertex_id();
        let id2 = gen_vertex_id();
        assert_ne!(id1.as_i64(), id2.as_i64());
    }

    #[test]
    fn test_thread_pool_integration() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
```

## 总结

已修复的严重问题：
1. ✅ memory.rs - 内存安全问题已解决
2. ✅ base/id.rs - ID 溢出问题已解决
3. ✅ thread.rs - ThreadPool 实现问题已解决
4. ✅ time.rs - 日期验证问题已解决

待实现的功能：
1. ⚠️ network.rs - 需要实现完整的网络协议处理
2. ⚠️ process.rs - 需要实现跨平台的系统监控
3. ⚠️ fs.rs - 需要实现真正的文件锁定
4. ⚠️ log.rs - 需要实现日志轮转
5. ⚠️ charset.rs - 需要改进编码检测

建议优先实现高优先级的集成点，然后逐步完善其他功能。
