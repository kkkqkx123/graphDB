# 性能监控系统改进方案（修订版）

## 一、现状分析

### 1.1 现有统计信息分布

当前项目的统计信息分布在三个主要模块中，各自负责不同层面的统计：

#### 1.1.1 查询统计层 (`src/core/stats/`)

**用途**: 查询级别的统计和监控

**核心组件**:

- **`QueryMetrics`** ([`src/core/stats/metrics.rs`](../../src/core/stats/metrics.rs)): 轻量级查询指标（微秒级精度）
  - 用于返回给客户端的查询结果
  - 包含：执行时间、节点数、结果数
- **`QueryProfile`** ([`src/core/stats/profile.rs`](../../src/core/stats/profile.rs)): 详细查询画像（毫秒级精度）
  - 用于内部分析和监控
  - 包含：各阶段时间、执行器统计、错误信息
- **`StatsManager`** ([`src/core/stats/manager.rs`](../../src/core/stats/manager.rs)): 统一统计管理器
  - 管理全局指标（MetricType）
  - 管理查询画像缓存
  - 管理错误统计
- **`ErrorStatsManager`** ([`src/core/stats/error_stats.rs`](../../src/core/stats/error_stats.rs)): 错误统计管理
  - 错误类型和阶段统计

**特点**:

- ✅ 已有完整的查询级别统计
- ✅ 支持慢查询日志（通过 StatsManager）
- ✅ 使用 DashMap 保证线程安全
- ⚠️ 精度不统一（QueryMetrics 用微秒，QueryProfile 用毫秒）

---

#### 1.1.2 执行器统计层 (`src/query/executor/`)

**用途**: 执行器执行过程中的详细统计

**核心组件**:

- **`ExecutorStats`** ([`src/query/executor/base/executor_stats.rs`](../../src/query/executor/base/executor_stats.rs)): 执行器基础统计
  - 处理行数、执行时间（微秒）
  - 内存使用、缓存命中率
  - 批处理次数
- **`NodeExecutionStats`** ([`src/query/executor/explain/execution_stats_context.rs`](../../src/query/executor/explain/execution_stats_context.rs)): 节点执行统计
  - 用于 EXPLAIN ANALYZE 和 PROFILE
  - 实际行数、实际时间（毫秒）
  - 启动时间、内存使用
  - I/O 统计（未实际使用）
- **`ExecutionStatsContext`** ([`src/query/executor/explain/execution_stats_context.rs`](../../src/query/executor/explain/execution_stats_context.rs)): 执行统计上下文
  - 管理所有节点的统计信息
  - 全局执行统计

**特点**:

- ✅ 微秒级精度（ExecutorStats）
- ⚠️ 毫秒级精度（NodeExecutionStats）
- ⚠️ I/O 统计字段存在但未使用
- ✅ 仅用于 EXPLAIN/PROFILE，非生产环境常规监控

---

#### 1.1.3 优化器统计层 (`src/query/optimizer/stats/`)

**用途**: 查询优化器的统计信息和执行反馈

**核心组件**:

- **`StatisticsManager`** ([`src/query/optimizer/stats/manager.rs`](../../src/query/optimizer/stats/manager.rs)): 优化器统计管理器
  - Tag 统计（顶点数）
  - Edge 统计（边数）
  - Property 统计（属性统计）
  - Histogram 统计（直方图）
- **`ExecutionFeedbackCollector`** ([`src/query/optimizer/stats/feedback/collector.rs`](../../src/query/optimizer/stats/feedback/collector.rs)): 执行反馈收集器
  - 收集查询执行反馈
  - 用于优化估计准确性
- **`QueryExecutionFeedback`** ([`src/query/optimizer/stats/feedback/query.rs`](../../src/query/optimizer/stats/feedback/query.rs)): 查询执行反馈
  - 估计成本 vs 实际成本
  - 估计行数 vs 实际行数

**特点**:

- ✅ 专注于优化器需要的统计
- ✅ 支持执行反馈机制
- ⚠️ 与查询统计层（StatsManager）无集成

---

#### 1.1.4 其他统计模块

**存储层监控** ([`src/storage/monitoring/storage_metrics.rs`](../../src/storage/monitoring/storage_metrics.rs)):

- StorageMetricsCollector: 扫描数、返回数、缓存命中率
- 独立于查询统计系统

**同步层监控** ([`src/sync/metrics.rs`](../../src/sync/metrics.rs)):

- SyncMetrics: 事务统计、重试统计、死信队列
- 独立于查询统计系统

**搜索引擎监控**:

- Fulltext Metrics ([`src/search/metrics.rs`](../../src/search/metrics.rs))
- Inversearch Metrics ([`crates/inversearch/src/metrics.rs`](../../crates/inversearch/src/metrics.rs))

---

### 1.2 核心问题

#### 1.2.1 精度不统一

| 模块            | 统计结构           | 精度       | 问题      |
| --------------- | ------------------ | ---------- | --------- |
| core/stats      | QueryMetrics       | 微秒 (us)  | ✅ 统一   |
| core/stats      | QueryProfile       | 毫秒 (ms)  | ⚠️ 不统一 |
| query/executor  | ExecutorStats      | 微秒 (us)  | ✅ 统一   |
| query/executor  | NodeExecutionStats | 毫秒 (f64) | ⚠️ 不统一 |
| query/optimizer | ExecutionFeedback  | 毫秒 (f64) | ⚠️ 不统一 |

**影响**:

- 数据转换开销
- 精度损失
- 代码维护困难

---

#### 1.2.2 统计信息孤岛

```
QueryProfile (core/stats)
    ↓ 无关联
ExecutorStats (query/executor)
    ↓ 无关联
ExecutionFeedback (optimizer/stats)
```

**问题**:

- 三个统计系统互不关联
- 无法从 QueryProfile 获取 Executor 详细统计
- 无法从 ExecutionFeedback 追溯到 QueryProfile
- 重复收集统计信息

---

#### 1.2.3 缺少延迟分布统计

**现状**:

- 仅有平均值、总数统计
- 没有 P50/P95/P99 分位数
- 无法分析长尾延迟

**影响**:

- 难以发现性能瓶颈
- 无法准确评估系统性能

---

#### 1.2.4 I/O 统计未使用

**现状**:

- `NodeExecutionStats` 有 `io_reads` 和 `io_read_bytes` 字段
- `StorageMetricsCollector` 没有 I/O 统计方法
- 执行器中未收集 I/O 统计

**影响**:

- 无法分析 I/O 瓶颈
- 无法优化存储访问模式

---

#### 1.2.5 慢查询日志不完善

**现状**:

- StatsManager 有慢查询日志逻辑
- 但没有独立的日志文件（与主日志混用）
- 没有异步写入机制
- 没有日志轮转

**影响**:

- 慢查询分析困难
- 日志文件过大

---

### 1.3 设计约束

基于当前架构，改进方案需要遵循以下约束：

1. **不新增顶层模块**: 不使用 `infra` 等新的顶层模块
2. **利用现有模块**: 在现有 `core/stats` 和 `query/executor/explain` 基础上改进
3. **向后兼容**: 保持现有 API 的兼容性
4. **最小侵入**: 尽量减少对现有代码的修改

---

## 二、改进方案

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                   Query Execution Flow                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              ExecutorStats (执行器统计)                      │
│  - 微秒级精度                                                │
│  - 行数、时间、内存、缓存                                    │
│  - 每个 Executor 实例                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│         ExecutionStatsContext (执行统计上下文)               │
│  - 收集所有 NodeExecutionStats                               │
│  - 全局执行统计                                              │
│  - 用于 EXPLAIN/PROFILE                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              QueryProfile (查询画像)                         │
│  - 毫秒级精度（改为微秒）                                    │
│  - 各阶段时间                                                │
│  - ExecutorStat 列表                                         │
│  - 错误信息                                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│            StatsManager (统计管理器)                         │
│  - 全局指标 (MetricType)                                     │
│  - 查询画像缓存                                              │
│  - 慢查询日志（改进）                                        │
│  - 错误统计                                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│      Monitoring API (监控 API - 新增到 core/stats)           │
│  - Metrics API: 获取聚合指标                                 │
│  - Profile API: 获取查询画像                                 │
│  - Health API: 健康检查                                      │
└─────────────────────────────────────────────────────────────┘
```

**关键设计决策**:

1. **不新增顶层模块**: 所有改进在现有 `core/stats` 模块内完成
2. **ExecutorStats 为基础**: 保持 ExecutorStats 的微秒精度，作为底层统计
3. **统一 QueryProfile 精度**: 将 QueryProfile 从毫秒改为微秒
4. **增强 StatsManager**: 在 StatsManager 中增加监控 API 和延迟分布统计
5. **独立慢查询日志**: 在 core/stats 中实现 SlowQueryLogger

---

### 2.2 精度统一

#### 2.2.1 统一为微秒级精度

**修改文件**: `src/core/stats/profile.rs`

```rust
/// Statistics during the query execution phase (in microseconds)
#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub parse_us: u64,        // 原 parse_ms
    pub validate_us: u64,
    pub plan_us: u64,
    pub optimize_us: u64,
    pub execute_us: u64,
}

impl StageMetrics {
    pub fn record_parse(&mut self, duration: Duration) {
        self.parse_us = duration.as_micros() as u64;
    }

    pub fn record_validate(&mut self, duration: Duration) {
        self.validate_us = duration.as_micros() as u64;
    }

    pub fn record_plan(&mut self, duration: Duration) {
        self.plan_us = duration.as_micros() as u64;
    }

    pub fn record_optimize(&mut self, duration: Duration) {
        self.optimize_us = duration.as_micros() as u64;
    }

    pub fn record_execute(&mut self, duration: Duration) {
        self.execute_us = duration.as_micros() as u64;
    }

    /// 转换为毫秒（用于显示）
    pub fn total_ms(&self) -> f64 {
        (self.parse_us + self.validate_us + self.plan_us +
         self.optimize_us + self.execute_us) as f64 / 1000.0
    }
}
```

**修改文件**: `src/query/executor/explain/execution_stats_context.rs`

```rust
/// Node-level execution statistics
#[derive(Debug, Clone, Default)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub actual_rows: usize,
    pub actual_time_us: f64,    // 原 actual_time_ms
    pub startup_time_us: f64,   // 原 startup_time_ms
    pub total_time_us: f64,     // 原 total_time_ms
    pub memory_used: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub io_reads: usize,
    pub io_read_bytes: usize,
}
```

---

#### 2.2.2 精度转换工具

**新增文件**: `src/core/stats/time_utils.rs`

```rust
//! Time utility functions for statistics
//!
//! Provide conversion between different time units.

use std::time::Duration;

/// Convert duration to microseconds (as f64 for precision)
pub fn duration_to_micros(duration: Duration) -> f64 {
    duration.as_micros() as f64
}

/// Convert duration to milliseconds (as f64 for display)
pub fn duration_to_millis(duration: Duration) -> f64 {
    duration.as_millis() as f64
}

/// Format microseconds to human-readable string
pub fn format_micros(us: f64) -> String {
    if us >= 1_000_000.0 {
        format!("{:.2}s", us / 1_000_000.0)
    } else if us >= 1_000.0 {
        format!("{:.2}ms", us / 1_000.0)
    } else {
        format!("{:.2}us", us)
    }
}

/// Calculate average from total and count
pub fn calculate_average(total_us: f64, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total_us / count as f64
    }
}
```

---

### 2.3 延迟分布统计

#### 2.3.1 添加 HDR Histogram 支持

**Cargo.toml** 添加依赖:

```toml
[dependencies]
hdrhistogram = "7.5"
```

#### 2.3.2 实现 LatencyHistogram

**新增文件**: `src/core/stats/histogram.rs`

```rust
//! Latency histogram for performance analysis
//!
//! Use HDR Histogram to track latency distribution.

use hdrhistogram::Histogram;
use parking_lot::Mutex;
use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};

/// Latency histogram using HDR Histogram
pub struct LatencyHistogram {
    histogram: Mutex<Histogram<u64>>,
    count: AtomicU64,
}

impl LatencyHistogram {
    /// Create a new histogram
    /// Supports latency from 1us to 1h with 3 significant digits
    pub fn new() -> Self {
        Self {
            histogram: Mutex::new(Histogram::new_with_bounds(1, 3_600_000_000, 3).unwrap()),
            count: AtomicU64::new(0),
        }
    }

    /// Record a latency measurement
    pub fn record(&self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        if let Ok(mut hist) = self.histogram.lock().try_record(micros) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get P50 latency (microseconds)
    pub fn p50(&self) -> u64 {
        self.histogram.lock().value_at_percentile(50.0)
    }

    /// Get P95 latency (microseconds)
    pub fn p95(&self) -> u64 {
        self.histogram.lock().value_at_percentile(95.0)
    }

    /// Get P99 latency (microseconds)
    pub fn p99(&self) -> u64 {
        self.histogram.lock().value_at_percentile(99.0)
    }

    /// Get mean latency (microseconds)
    pub fn mean(&self) -> f64 {
        self.histogram.lock().mean()
    }

    /// Get sample count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get a snapshot of histogram statistics
    pub fn snapshot(&self) -> HistogramSnapshot {
        let hist = self.histogram.lock();
        HistogramSnapshot {
            p50: hist.value_at_percentile(50.0),
            p95: hist.value_at_percentile(95.0),
            p99: hist.value_at_percentile(99.0),
            mean: hist.mean(),
            count: hist.len(),
        }
    }

    /// Reset the histogram
    pub fn reset(&self) {
        self.histogram.lock().reset();
        self.count.store(0, Ordering::Relaxed);
    }
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram statistics snapshot
#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub mean: f64,
    pub count: u64,
}
```

---

#### 2.3.3 集成到 StatsManager

**修改文件**: `src/core/stats/manager.rs`

```rust
use super::histogram::LatencyHistogram;

pub struct StatsManager {
    // ... 现有字段
    global_latency_histogram: Arc<LatencyHistogram>,
}

impl StatsManager {
    pub fn with_config(config: crate::config::MonitoringConfig) -> Self {
        Self {
            // ... 现有初始化
            global_latency_histogram: Arc::new(LatencyHistogram::new()),
        }
    }

    /// Record query latency to histogram
    pub fn record_query_latency(&self, duration: Duration) {
        self.global_latency_histogram.record(duration);
    }

    /// Get latency histogram snapshot
    pub fn get_latency_snapshot(&self) -> HistogramSnapshot {
        self.global_latency_histogram.snapshot()
    }
}
```

---

### 2.4 监控 API

#### 2.4.1 在 core/stats 中增加监控 API

**新增文件**: `src/core/stats/api.rs`

```rust
//! Monitoring API for querying statistics
//!
//! Provide APIs for querying metrics, profiles, and health status.

use std::sync::Arc;
use super::{StatsManager, QueryProfile};
use super::histogram::{LatencyHistogram, HistogramSnapshot};

/// Monitoring metrics snapshot
#[derive(Debug, Clone)]
pub struct MonitoringMetrics {
    pub query_metrics: QueryMetricsSnapshot,
    pub latency_histogram: LatencySnapshot,
}

#[derive(Debug, Clone, Default)]
pub struct QueryMetricsSnapshot {
    pub total_queries: u64,
    pub active_queries: u64,
    pub failed_queries: u64,
    pub avg_parse_time_us: f64,
    pub avg_execute_time_us: f64,
    pub avg_total_time_us: f64,
}

#[derive(Debug, Clone)]
pub struct LatencySnapshot {
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub mean_us: f64,
    pub count: u64,
}

/// Monitoring API provider
pub struct MonitoringApi {
    stats_manager: Arc<StatsManager>,
}

impl MonitoringApi {
    pub fn new(stats_manager: Arc<StatsManager>) -> Self {
        Self { stats_manager }
    }

    /// Get all monitoring metrics
    pub fn get_metrics(&self) -> MonitoringMetrics {
        MonitoringMetrics {
            query_metrics: self.get_query_metrics(),
            latency_histogram: self.get_latency_snapshot(),
        }
    }

    /// Get query metrics
    pub fn get_query_metrics(&self) -> QueryMetricsSnapshot {
        // TODO: 从 StatsManager 获取
        QueryMetricsSnapshot::default()
    }

    /// Get latency snapshot
    pub fn get_latency_snapshot(&self) -> LatencySnapshot {
        let hist = self.stats_manager.get_latency_snapshot();
        LatencySnapshot {
            p50: hist.p50,
            p95: hist.p95,
            p99: hist.p99,
            mean: hist.mean,
            count: hist.count,
        }
    }
}

/// Profile API for querying query profiles
pub struct ProfileApi {
    stats_manager: Arc<StatsManager>,
}

impl ProfileApi {
    pub fn new(stats_manager: Arc<StatsManager>) -> Self {
        Self { stats_manager }
    }

    /// Get recent query profiles
    pub fn get_recent_profiles(&self, limit: usize) -> Vec<QueryProfile> {
        // TODO: 从 StatsManager 获取
        Vec::new()
    }

    /// Get slow queries
    pub fn get_slow_queries(&self, threshold_ms: u64, limit: usize) -> Vec<QueryProfile> {
        // TODO: 过滤慢查询
        Vec::new()
    }
}

/// Health API for health checks
pub struct HealthApi {
    monitoring_api: Arc<MonitoringApi>,
}

impl HealthApi {
    pub fn new(monitoring_api: Arc<MonitoringApi>) -> Self {
        Self { monitoring_api }
    }

    /// Check system health
    pub fn check_health(&self) -> HealthStatus {
        // TODO: 实现健康检查
        HealthStatus::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct HealthStatus {
    pub status: HealthState,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}
```

---

### 2.5 慢查询日志系统

#### 2.5.1 独立的慢查询日志

**新增文件**: `src/core/stats/slow_query.rs`

```rust
//! Slow query logging system
//!
//! Independent slow query log file with async writing and log rotation.

use std::sync::mpsc;
use std::thread;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use chrono::Local;
use super::profile::QueryProfile;
use super::profile::QueryStatus;

/// Slow query log configuration
#[derive(Debug, Clone)]
pub struct SlowQueryConfig {
    pub enabled: bool,
    pub threshold_ms: u64,
    pub log_file_path: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub verbose_format: bool,
    pub buffer_size: usize,
}

impl Default for SlowQueryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_ms: 1000,
            log_file_path: "logs/slow_query.log".to_string(),
            max_file_size_mb: 100,
            max_files: 5,
            verbose_format: false,
            buffer_size: 100,
        }
    }
}

/// Slow query logger
pub struct SlowQueryLogger {
    config: SlowQueryConfig,
    tx: mpsc::Sender<String>,
    writer_handle: Option<thread::JoinHandle<()>>,
    current_file_size: AtomicU64,
    current_file_path: Mutex<PathBuf>,
}

impl SlowQueryLogger {
    pub fn new(config: SlowQueryConfig) -> Result<Self, std::io::Error> {
        // 创建日志目录
        if let Some(parent) = Path::new(&config.log_file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // 创建异步通道
        let (tx, rx) = mpsc::channel::<String>(config.buffer_size);

        // 启动后台写入线程
        let writer_handle = Some(Self::spawn_writer_thread(rx, config.clone()));

        Ok(Self {
            config,
            tx,
            writer_handle,
            current_file_size: AtomicU64::new(0),
            current_file_path: Mutex::new(PathBuf::from(&config.log_file_path)),
        })
    }

    /// Log a slow query
    pub fn log_slow_query(&self, profile: &QueryProfile) {
        if !self.config.enabled {
            return;
        }

        if profile.total_duration_ms < self.config.threshold_ms {
            return;
        }

        let log_entry = if self.config.verbose_format {
            self.format_verbose_log(profile)
        } else {
            self.format_simple_log(profile)
        };

        // 异步发送（非阻塞）
        let _ = self.tx.try_send(log_entry);
    }

    fn format_simple_log(&self, profile: &QueryProfile) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let status_str = match profile.status {
            QueryStatus::Success => "success",
            QueryStatus::Failed => "failed",
        };

        format!(
            "[{}] [SLOW_QUERY] [trace_id={}] [session_id={}] [duration={}ms] [status={}] {}\n",
            timestamp,
            profile.trace_id,
            profile.session_id,
            profile.total_duration_ms,
            status_str,
            profile.query_text
        )
    }

    fn format_verbose_log(&self, profile: &QueryProfile) -> String {
        // 实现详细格式
        String::new()
    }

    fn spawn_writer_thread(
        rx: mpsc::Receiver<String>,
        config: SlowQueryConfig,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            // 实现后台写入和日志轮转逻辑
        })
    }
}
```

---

#### 2.5.2 集成到 StatsManager

**修改文件**: `src/core/stats/manager.rs`

```rust
use super::slow_query::{SlowQueryConfig, SlowQueryLogger};

pub struct StatsManager {
    // ... 现有字段
    slow_query_logger: Option<Arc<SlowQueryLogger>>,
}

impl StatsManager {
    pub fn with_slow_query_logger(
        config: crate::config::MonitoringConfig,
        slow_query_config: SlowQueryConfig,
    ) -> Self {
        let slow_query_logger = Arc::new(SlowQueryLogger::new(slow_query_config).unwrap());

        Self {
            // ... 现有初始化
            slow_query_logger: Some(slow_query_logger),
        }
    }

    pub fn record_query_profile(&self, profile: QueryProfile) {
        // ... 现有逻辑

        // 记录到慢查询日志
        if let Some(ref logger) = self.slow_query_logger {
            logger.log_slow_query(&profile);
        }
    }
}
```

---

### 2.6 I/O 统计集成

#### 2.6.1 扩展存储层统计

**修改文件**: `src/storage/monitoring/storage_metrics.rs`

```rust
pub struct StorageMetricsCollector {
    // ... 现有字段
    io_reads: AtomicU64,
    io_read_bytes: AtomicU64,
    io_writes: AtomicU64,
    io_write_bytes: AtomicU64,
    io_time_us: AtomicU64,
}

impl StorageMetricsCollector {
    pub fn record_io_read(&self, bytes: u64) {
        self.io_reads.fetch_add(1, Ordering::Relaxed);
        self.io_read_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_io_write(&self, bytes: u64) {
        self.io_writes.fetch_add(1, Ordering::Relaxed);
        self.io_write_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_io_time(&self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        self.io_time_us.fetch_add(micros, Ordering::Relaxed);
    }
}
```

---

## 三、模块结构

### 3.1 最终的模块组织

```
src/core/stats/
├── mod.rs              // 模块导出
├── metrics.rs          // QueryMetrics（轻量级指标）
├── profile.rs          // QueryProfile（查询画像，改为微秒精度）
├── manager.rs          // StatsManager（统计管理器）
├── error_stats.rs      // ErrorStatsManager（错误统计）
├── histogram.rs        // [新增] LatencyHistogram（延迟分布）
├── api.rs              // [新增] 监控 API（Metrics/Profile/Health）
├── slow_query.rs       // [新增] 慢查询日志
└── time_utils.rs       // [新增] 时间工具函数

src/query/executor/
├── base/
│   └── executor_stats.rs  // ExecutorStats（微秒精度，保持不变）
└── explain/
    ├── execution_stats_context.rs  // NodeExecutionStats（改为微秒）
    ├── instrumented_executor.rs
    └── profile_executor.rs

src/query/optimizer/stats/
├── manager.rs          // StatisticsManager（优化器统计）
└── feedback/           // 执行反馈（保持独立）
```

---

### 3.2 数据流

```
查询执行
    │
    ├─► ExecutorStats (微秒)
    │       │
    │       └─► 收集到 ExecutionStatsContext
    │               │
    │               └─► 转换为 ExecutorStat
    │                       │
    │                       └─► 填充到 QueryProfile
    │                               │
    │                               └─► 记录到 StatsManager
    │                                       │
    │                                       ├─► 更新全局指标
    │                                       ├─► 记录到延迟直方图
    │                                       └─► 写入慢查询日志
    │
    └─► 监控 API 提供查询接口
```

---

## 四、关键改进点总结

### 4.1 精度统一

| 改进项             | 改进前     | 改进后       |
| ------------------ | ---------- | ------------ |
| QueryProfile       | 毫秒 (ms)  | 微秒 (us)    |
| NodeExecutionStats | 毫秒 (f64) | 微秒 (f64)   |
| 时间工具           | 无         | 统一转换函数 |

### 4.2 延迟分布

- ✅ 添加 HDR Histogram 支持
- ✅ 提供 P50/P95/P99 统计
- ✅ 集成到 StatsManager

### 4.3 监控 API

- ✅ Metrics API: 获取聚合指标
- ✅ Profile API: 获取查询画像
- ✅ Health API: 健康检查
- ✅ 所有 API 在 `core/stats/api.rs` 中实现

### 4.4 慢查询日志

- ✅ 独立日志文件（`logs/slow_query.log`）
- ✅ 异步写入机制
- ✅ 日志轮转支持
- ✅ 与主日志分离

### 4.5 I/O 统计

- ✅ 扩展存储层 I/O 统计
- ✅ 在执行器中收集 I/O 数据

---

## 五、与现有代码的集成

### 5.1 最小化修改

1. **ExecutorStats**: 保持不变（已经是微秒精度）
2. **ExecutionStatsContext**: 仅修改字段类型（ms → us）
3. **StatsManager**: 增加 histogram 和 slow_query_logger 字段
4. **QueryProfile**: 修改 StageMetrics 为微秒

### 5.2 向后兼容

1. **API 兼容**: 保持现有方法签名
2. **数据兼容**: 提供毫秒显示函数
3. **配置兼容**: 使用默认配置

---

## 六、总结

本改进方案基于现有模块结构，不进行大的架构调整：

1. ✅ **不新增顶层模块**: 所有改进在 `core/stats` 内完成
2. ✅ **精度统一**: 统一为微秒级精度
3. ✅ **延迟分布**: 添加 HDR Histogram 支持
4. ✅ **监控 API**: 在 core/stats 中实现
5. ✅ **慢查询日志**: 独立日志文件，异步写入
6. ✅ **I/O 统计**: 完善存储层和执行器统计

通过渐进式改进，可以在保持现有代码稳定的前提下，提升系统的可观测性和性能诊断能力。
