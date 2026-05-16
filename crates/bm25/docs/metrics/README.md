# BM25 Metrics 系统

## 概述

BM25 模块现已集成了完整的 metrics 收集系统，提供对存储操作的全面可观测性。

## 主要特性

### ✅ 已实现功能

1. **统一的指标收集器** (`StorageMetricsCollector`)
   - 线程安全的原子操作
   - 零外部依赖（仅使用 `std::sync::atomic`）
   - 支持操作计数、延迟统计、错误分类

2. **完整的存储后端支持**
   - ✅ TantivyStorage: 新增完整 metrics 支持
   - ✅ RedisStorage: 重构为使用统一 collector
   - ✅ 所有 StorageInterface 实现均支持 metrics

3. **详细的错误分类**
   - Connection errors
   - Serialization errors
   - Deserialization errors
   - Timeout errors
   - Other errors

4. **RAII 风格定时器** (`OperationTimer`)
   - 自动记录操作延迟
   - Drop 时自动触发

## 快速开始

### 基础使用

```rust
use bm25::storage::common::{StorageMetricsCollector, ErrorType};
use std::time::Instant;

// 创建收集器
let collector = StorageMetricsCollector::new();

// 记录操作
let start = Instant::now();
// ... 执行操作 ...
collector.record_operation(start);

// 记录错误
collector.record_error(ErrorType::Connection);

// 获取指标
let metrics = collector.get_metrics(memory_usage);
println!("Operations: {}", metrics.operation_count);
println!("Avg Latency: {} μs", metrics.average_latency);
println!("Error Rate: {:.2}%", metrics.error_rate() * 100.0);
```

### RAII 风格

```rust
use bm25::storage::common::{StorageMetricsCollector, OperationTimer};

let collector = StorageMetricsCollector::new();

{
    let _timer = OperationTimer::new(&collector);
    // ... 执行操作 ...
} // 自动记录延迟
```

## API 参考

### StorageMetricsCollector

| 方法 | 描述 |
|------|------|
| `new()` | 创建新的收集器 |
| `record_operation(start)` | 记录操作完成 |
| `record_error(error_type)` | 记录错误 |
| `get_operation_count()` | 获取操作总数 |
| `get_average_latency()` | 获取平均延迟 (μs) |
| `get_error_count()` | 获取错误总数 |
| `get_connection_errors()` | 获取连接错误数 |
| `get_serialization_errors()` | 获取序列化错误数 |
| `get_deserialization_errors()` | 获取反序列化错误数 |
| `reset()` | 重置所有指标 |
| `get_metrics(memory_usage)` | 获取聚合的 StorageMetrics |

### StorageMetrics

| 字段 | 类型 | 描述 |
|------|------|------|
| `operation_count` | `u64` | 总操作数 |
| `average_latency` | `u64` | 平均延迟 (微秒) |
| `memory_usage` | `u64` | 内存使用 (字节) |
| `error_count` | `u64` | 总错误数 |
| `connection_errors` | `u64` | 连接错误数 |
| `serialization_errors` | `u64` | 序列化错误数 |
| `deserialization_errors` | `u64` | 反序列化错误数 |

| 方法 | 描述 |
|------|------|
| `error_rate()` | 计算错误率 (0.0-1.0) |
| `success_count()` | 获取成功操作数 |
| `success_rate()` | 计算成功率 (0.0-1.0) |

### ErrorType

```rust
pub enum ErrorType {
    Connection,
    Serialization,
    Deserialization,
    Timeout,
    Other,
}
```

## 存储后端集成

### TantivyStorage

```rust
use bm25::storage::{TantivyStorage, StorageInterface};

let mut storage = TantivyStorage::new(config);
storage.init().await?;

// 所有操作自动记录 metrics
storage.commit_stats("term", 1.5, 10).await?;
let stats = storage.get_stats("term").await?;

// 获取 metrics
let metrics = storage.get_operation_stats();
println!("Operations: {}", metrics.operation_count);
```

### RedisStorage

```rust
use bm25::storage::{RedisStorage, RedisStorageConfig, StorageInterface};

let config = RedisStorageConfig::default();
let mut storage = RedisStorage::new(config).await?;

// 所有操作自动记录 metrics
storage.commit_batch(&stats).await?;

// 获取 metrics
let metrics = storage.get_operation_stats();
println!("Memory: {} bytes", metrics.memory_usage);
println!("Error Rate: {:.2}%", metrics.error_rate() * 100.0);
```

## 性能影响

### 预期开销

| 操作 | 额外开销 | 说明 |
|------|---------|------|
| `record_operation()` | ~50ns | 两次 atomic load + 一次 store |
| `record_error()` | ~30ns | 两次 atomic fetch_add |
| `get_metrics()` | ~100ns | 多次 atomic load |

### 优化建议

1. **生产环境可选禁用** (未来计划)
   ```toml
   [features]
   default = []
   metrics = []  # 启用时包含 metrics 代码
   ```

2. **高吞吐场景采样** (未来计划)
   ```rust
   if random() % 100 < 10 {  // 10% 采样
       collector.record_operation(start);
   }
   ```

## 测试

### 运行单元测试

```bash
cd crates/bm25
cargo test --lib storage::common::metrics
```

### 测试覆盖

- ✅ 基础指标收集
- ✅ 错误分类记录
- ✅ 并发访问安全
- ✅ RAII 定时器
- ✅ 指标重置
- ✅ 克隆语义

## 设计文档

详细设计文档请参考:
- [完整设计文档](./docs/metrics/design.md)

## 与 inversearch 的对比

| 特性 | inversearch | bm25 (新) |
|------|-------------|-----------|
| 原子类型 | `AtomicUsize` | `AtomicU64` |
| 错误分类 | 5 种 | 5 种 |
| RAII Timer | 定义未用 | ✅ 完整实现 |
| 延迟单位 | 微秒 | 微秒 |
| 内存单位 | 字节 | 字节 |

## 未来计划

### Phase 1 (已完成)
- [x] 核心 metrics 基础设施
- [x] TantivyStorage 集成
- [x] RedisStorage 重构
- [x] 单元测试

### Phase 2 (计划中)
- [ ] 添加延迟直方图 (P50/P90/P99)
- [ ] 条件编译支持 (feature flag)
- [ ] 导出到 Prometheus
- [ ] 与 graphdb 主 crate 的 StatsManager 集成

### Phase 3 (长期)
- [ ] 提取共享 metrics crate
- [ ] 统一 inversearch 和 bm25 的原子类型
- [ ] 添加标签系统支持

## 贡献指南

### 添加新的 metrics 收集点

1. 在操作开始前创建 timer:
   ```rust
   let start = Instant::now();
   ```

2. 在操作完成后记录:
   ```rust
   self.metrics.record_operation(start);
   ```

3. 遇到错误时记录:
   ```rust
   self.metrics.record_error(ErrorType::Connection);
   ```

### 确保线程安全

所有 metrics 操作都使用 `Ordering::Relaxed`，因为:
- 计数器不需要严格顺序一致性
- 性能优先，允许轻微的数据竞争窗口
- 对于监控指标，近似值通常足够

## 故障排除

### Q: 为什么 memory_usage 总是 0？

A: Tantivy 的内存管理是内部的，难以精确估算。如需准确数据，可以:
1. 通过 tantivy API 查询
2. 使用操作系统级别的监控工具
3. 暂时忽略此字段

### Q: 如何禁用 metrics 以减少开销？

A: 当前版本始终启用，未来将通过 feature flag 支持:
```toml
[dependencies]
bm25 = { version = "*", default-features = false, features = ["storage-tantivy"] }
# 不启用 metrics feature
```

## License

与项目主许可证保持一致。
