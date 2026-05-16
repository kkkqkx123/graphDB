# BM25 Metrics 系统设计与实现方案

## 1. 现状分析

### 1.1 当前问题

| 存储后端 | 操作计数 | 延迟统计 | 错误分类 | 内存使用 | 状态 |
|---------|---------|---------|---------|---------|------|
| **RedisStorage** | ✅ | ✅ | ✅ | ✅ (Redis INFO) | 有实现但孤立 |
| **TantivyStorage** | ❌ | ❌ | ❌ | ❌ | **完全缺失** |
| **inversearch Storage** | ✅ | ✅ | ✅ | ⚠️ (估算) | 良好参考 |

### 1.2 核心问题

1. **TantivyStorage 无 metrics**：作为默认存储后端，没有任何可观测性
2. **代码重复**：`redis.rs` 中重新定义了与 inversearch 几乎相同的 `StorageMetrics`
3. **不一致的错误处理**：Redis 手动调用 `record_error()`，缺乏统一管理
4. **缺少延迟分布**：仅有平均值，无法反映长尾延迟

---

## 2. 设计目标

### 2.1 原则

- ✅ **无外部依赖**：仅使用 `std::sync::atomic`
- ✅ **统一接口**：所有存储后端实现相同的 metrics API
- ✅ **低开销**：原子操作，无锁竞争
- ✅ **向后兼容**：不修改现有公共 API

### 2.2 目标架构

```
┌─────────────────────────────────────────────────────────┐
│                  crates/bm25                            │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │         storage::common::metrics (新增)           │ │
│  │  StorageMetricsCollector                          │ │
│  │  StorageMetrics                                   │ │
│  │  ErrorType enum                                   │ │
│  └───────────────────────────────────────────────────┘ │
│                    ▲              ▲                     │
│                    │              │                     │
│         ┌──────────┘              └──────────┐         │
│         ▼                                    ▼         │
│  ┌─────────────────┐               ┌─────────────────┐ │
│  │ RedisStorage    │               │ TantivyStorage  │ │
│  │ (已有 metrics)  │               │ (新增 metrics)  │ │
│  │ 重构为使用      │               │ 使用统一        │ │
│  │ StorageMetricsCollector        │ collector       │ │
│  └─────────────────┘               └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## 3. 类型定义

### 3.1 StorageMetricsCollector

```rust
// crates/bm25/src/storage/common/metrics.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 错误类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    Connection,
    Serialization,
    Deserialization,
    Timeout,
    Other,
}

/// 存储指标收集器
#[derive(Debug, Default)]
pub struct StorageMetricsCollector {
    operation_count: AtomicU64,
    total_latency: AtomicU64,      // microseconds
    error_count: AtomicU64,
    connection_errors: AtomicU64,
    serialization_errors: AtomicU64,
    deserialization_errors: AtomicU64,
    timeout_errors: AtomicU64,
    other_errors: AtomicU64,
}

impl StorageMetricsCollector {
    /// 创建新的收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录操作完成
    pub fn record_operation(&self, start: Instant) {
        let latency = start.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// 记录错误
    pub fn record_error(&self, error_type: ErrorType) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        match error_type {
            ErrorType::Connection => {
                self.connection_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Serialization => {
                self.serialization_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Deserialization => {
                self.deserialization_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Timeout => {
                self.timeout_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Other => {
                self.other_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// 获取操作计数
    pub fn get_operation_count(&self) -> u64 {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// 获取平均延迟（微秒）
    pub fn get_average_latency(&self) -> u64 {
        let count = self.get_operation_count();
        if count > 0 {
            self.total_latency.load(Ordering::Relaxed) / count
        } else {
            0
        }
    }

    /// 获取总错误数
    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// 重置所有指标
    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        self.total_latency.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.connection_errors.store(0, Ordering::Relaxed);
        self.serialization_errors.store(0, Ordering::Relaxed);
        self.deserialization_errors.store(0, Ordering::Relaxed);
        self.timeout_errors.store(0, Ordering::Relaxed);
        self.other_errors.store(0, Ordering::Relaxed);
    }

    /// 获取聚合后的 StorageMetrics
    pub fn get_metrics(&self, memory_usage: u64) -> StorageMetrics {
        StorageMetrics {
            operation_count: self.get_operation_count(),
            average_latency: self.get_average_latency(),
            memory_usage,
            error_count: self.get_error_count(),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            serialization_errors: self.serialization_errors.load(Ordering::Relaxed),
            deserialization_errors: self.deserialization_errors.load(Ordering::Relaxed),
        }
    }
}
```

### 3.2 StorageMetrics

```rust
/// 存储性能指标（对外暴露的只读结构体）
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: u64,
    pub average_latency: u64,      // microseconds
    pub memory_usage: u64,         // bytes
    pub error_count: u64,
    pub connection_errors: u64,
    pub serialization_errors: u64,
    pub deserialization_errors: u64,
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 计算错误率
    pub fn error_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
     self.error_count as f64 / self.operation_count as f64
        }
    }
}
```

---

## 4. 实现指南

### 4.1 TantivyStorage 实现步骤

#### 步骤 1: 添加 metrics 字段

```rust
pub struct TantivyStorage {
    config: TantivyStorageConfig,
    index: Option<Arc<RwLock<Index>>>,
    schema: Schema,
    writer: Option<Arc<RwLock<IndexWriter>>>,
    reader: Option<Arc<RwLock<IndexReader>>>,
    metrics: Arc<StorageMetricsCollector>,  // 新增
}
```

#### 步骤 2: 初始化 metrics

```rust
impl TantivyStorage {
    pub fn new(config: TantivyStorageConfig) -> Self {
        Self {
            config,
            index: None,
            schema: Self::build_schema(),
            writer: None,
            reader: None,
            metrics: Arc::new(StorageMetricsCollector::default()),  // 新增
        }
    }

    /// 获取 metrics
    pub fn get_operation_stats(&self) -> StorageMetrics {
        self.metrics.get_metrics(0)  // Tantivy 内存使用难以精确估算
    }
}
```

#### 步骤 3: 在所有操作中记录 metrics

```rust
#[async_trait::async_trait]
impl StorageInterface for TantivyStorage {
    async fn init(&mut self) -> Result<()> {
        let start = Instant::now();
        
        if self.index.is_none() {
            // ... existing logic ...
        }
        
        self.metrics.record_operation(start);  // 记录
        Ok(())
    }

    async fn commit_stats(&mut self, _term: &str, _tf: f32, _df: u64) -> Result<()> {
        let start = Instant::now();
        // Tantivy manages word frequency statistics automatically
        self.metrics.record_operation(start);  // 记录
        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let start = Instant::now();
        
        let reader = self.reader.as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = reader.read().await;
        let searcher = reader.searcher();
        
        // ... existing logic ...
        
        self.metrics.record_operation(start);  // 记录
        Ok(result)
    }

    // 其他方法同理...
}
```

### 4.2 RedisStorage 重构步骤

#### 步骤 1: 移除内联 metrics 字段

```rust
pub struct RedisStorage {
    pool: Pool<RedisConnectionManager>,
    key_prefix: String,
    memory_usage: Arc<AtomicUsize>,
    // 移除以下字段，改用统一的 collector:
    // operation_count: Arc<AtomicU64>,
    // total_latency: Arc<AtomicU64>,
    // error_stats: Arc<ErrorStats>,
    
    metrics: Arc<StorageMetricsCollector>,  // 新增
}
```

#### 步骤 2: 更新错误记录逻辑

```rust
async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>> {
    self.pool.get().await.map_err(|e| {
        self.metrics.record_error(ErrorType::Connection);  // 使用统一方法
        Bm25Error::StorageError(e.to_string())
    })
}
```

#### 步骤 3: 更新 get_operation_stats

```rust
pub fn get_operation_stats(&self) -> StorageMetrics {
    self.metrics.get_metrics(self.memory_usage.load(Ordering::Relaxed))
}
```

---

## 5. 使用示例

### 5.1 基础使用

```rust
use bm25::storage::common::metrics::{StorageMetricsCollector, ErrorType};
use std::time::Instant;

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

### 5.2 RAII 风格（可选扩展）

```rust
// 未来可扩展为 RAII 定时器
pub struct OperationTimer<'a> {
    start: Instant,
    collector: &'a StorageMetricsCollector,
}

impl<'a> OperationTimer<'a> {
    pub fn new(collector: &'a StorageMetricsCollector) -> Self {
        Self {
            start: Instant::now(),
            collector,
        }
    }
}

impl<'a> Drop for OperationTimer<'a> {
    fn drop(&mut self) {
        self.collector.record_operation(self.start);
    }
}

// 使用
async fn get(&self, key: &str) -> Result<()> {
    let _timer = OperationTimer::new(&self.metrics);
    // 操作完成时自动记录
    Ok(())
}
```

---

## 6. 测试计划

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new();
        
        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);
        
        assert_eq!(collector.get_operation_count(), 1);
        assert!(collector.get_average_latency() > 0);
    }

    #[test]
    fn test_error_recording() {
        let collector = StorageMetricsCollector::new();
        
        collector.record_error(ErrorType::Connection);
        collector.record_error(ErrorType::Serialization);
        
        assert_eq!(collector.get_error_count(), 2);
    assert_eq!(collector.connection_errors.load(Ordering::Relaxed), 1);
        assert_eq!(collector.serialization_errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_concurrent_access() {
        let collector = Arc::new(StorageMetricsCollector::new());
        let mut handles = vec![];
        
        for _ in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let start = Instant::now();
                    thread::sleep(Duration::from_micros(10));
                    collector_clone.record_operation(start);
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(collector.get_operation_count(), 1000);
    }
}
```

---

## 7. 迁移检查清单

### Phase 1: 基础设施（优先级 P0）
- [ ] 创建 `crates/bm25/src/storage/common/metrics.rs`
- [ ] 实现 `StorageMetricsCollector`
- [ ] 实现 `StorageMetrics`
- [ ] 实现 `ErrorType` 枚举
- [ ] 添加单元测试

### Phase 2: TantivyStorage 集成（优先级 P0）
- [ ] 添加 `metrics` 字段到 `TantivyStorage`
- [ ] 在 `init()` 中记录 metrics
- [ ] 在 `commit_stats()` 中记录 metrics
- [ ] 在 `get_stats()` 中记录 metrics
- [ ] 在 `get_df()` 中记录 metrics
- [ ] 在 `get_tf()` 中记录 metrics
- [ ] 添加 `get_operation_stats()` 方法
- [ ] 编写集成测试

### Phase 3: RedisStorage 重构（优先级 P1）
- [ ] 移除内联 metrics 字段
- [ ] 使用 `StorageMetricsCollector`
- [ ] 更新 `record_error()` 调用
- [ ] 更新 `get_operation_stats()` 实现
- [ ] 确保功能等价

### Phase 4: 文档与验证（优先级 P2）
- [ ] 更新 README.md
- [ ] 添加使用示例
- [ ] 运行完整测试套件
- [ ] 性能基准测试

---

## 8. 性能影响评估

### 8.1 预期开销

| 操作 | 额外开销 | 说明 |
|------|---------|------|
| `record_operation()` | ~50ns | 两次 atomic load + 一次 store |
| `record_error()` | ~30ns | 两次 atomic fetch_add |
| `get_metrics()` | ~100ns | 多次 atomic load |

### 8.2 优化建议

1. **条件编译**：生产环境可通过 feature flag 禁用 metrics
   ```toml
   [features]
   default = []
   metrics = []  # 启用时包含 metrics 代码
   ```

2. **采样策略**：高吞吐场景可考虑采样记录
   ```rust
   if random() % 100 < 10 {  // 10% 采样
       collector.record_operation(start);
   }
   ```

3. **批量记录**：批操作可在结束时记录一次总延迟

---

## 9. 与 inversearch 的对比

| 特性 | inversearch | bm25 (新) | 一致性 |
|------|-------------|-----------|--------|
| 原子类型 | `AtomicUsize` | `AtomicU64` | ⚠️ 建议统一为 u64 |
| 错误分类 | 5 种 | 5 种 | ✅ |
| RAII Timer | 定义未用 | 可选扩展 | ⚠️ |
| 延迟单位 | 微秒 | 微秒 | ✅ |
| 内存单位 | 字节 | 字节 | ✅ |

**建议**：后续可将两个 crate 的 metrics 提取到共享 crate `storage-common-metrics`

---

## 10. 附录：完整文件清单

### 新增文件
- `crates/bm25/src/storage/common/metrics.rs` - 核心实现
- `crates/bm25/docs/metrics/design.md` - 本文档

### 修改文件
- `crates/bm25/src/storage/common/mod.rs` - 导出新模块
- `crates/bm25/src/storage/tantivy.rs` - 集成 metrics
- `crates/bm25/src/storage/redis.rs` - 重构 metrics
- `crates/bm25/Cargo.toml` - 可选添加 dev-dependencies

### 导出更新
```rust
// crates/bm25/src/storage/common/mod.rs
pub mod metrics;

pub use metrics::{StorageMetrics, StorageMetricsCollector, ErrorType};
```
