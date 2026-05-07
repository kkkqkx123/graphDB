# Metrics 体系分析与重构方案

## 执行摘要

本文档记录了 GraphDB 项目 metrics 体系的完整分析结果和重构方案。分析发现当前系统存在双重记录、功能重复、缺失 getter 等问题，并提出了统一的 metrics 架构设计。

**分析日期**: 2026-05-07  
**状态**: 待实施

---

## 一、现有组件清单

### 1.1 核心 Metrics 组件

| 组件 | 位置 | 状态 | 问题 |
|------|------|------|------|
| `GlobalMetrics` | `src/core/stats/global_metrics.rs` | ⚠️ 部分废弃 | `get_query_count()` 返回硬编码 `0` |
| `TelemetryRecorder` | `src/api/core/telemetry.rs` | ✅ 完整 | 自定义 Recorder 实现，支持快照导出 |
| `QueryMetrics` | `src/core/stats/metrics.rs` | ✅ 完整 | 轻量级查询指标，返回客户端 |
| `SyncMetrics` | `src/sync/metrics.rs` | ✅ 完整 | 使用 `metrics` crate |
| `FulltextMetrics` | `src/search/metrics.rs` | ✅ 完整 | 使用 `metrics` crate |
| `IterStats` | `src/storage/iterator/storage_iter.rs` | ⚠️ 未使用 | 有内部计数器 + metrics 双重记录 |
| `CacheStats (utils)` | `src/core/stats/utils.rs` | ✅ 完整 | 原子操作实现 |
| `CacheCounters` | `src/query/cache/stats.rs` | ⚠️ 重复 | 与 `CacheStats` 功能重叠 |
| `StorageMetricsCollector` | ~~`src/storage/monitoring/`~~ | ❌ 已删除 | 已废弃并删除 |

### 1.2 已删除组件

**`src/storage/monitoring/`** - 已删除

删除原因:
- 整个目录未被任何代码引用
- 属于预留设计，但从未集成
- 与现有 `GlobalMetrics` 和 `TelemetryRecorder` 架构重复

---

## 二、核心问题

### 2.1 双重记录

**位置**: `src/storage/iterator/storage_iter.rs`

```rust
// 问题代码
pub struct IterStats {
    pub items_scanned: u64,        // 内部计数器
    pub items_returned: u64,       // 内部计数器
    pub seek_operations: u64,      // 内部计数器
    pub cache_hits: u64,           // 内部计数器
    pub cache_misses: u64,         // 内部计数器
}

impl IterStats {
    pub fn record_scan(&mut self) {
        metrics::counter!("graphdb_storage_iter_items_scanned_total").increment(1);
        self.items_scanned += 1;  // 双重记录！
    }
}
```

**影响**: 
- 内存浪费（每个实例额外占用 40 bytes）
- 代码复杂性增加
- 可能导致数据不一致

### 2.2 功能重复

**`CacheStats` (utils) vs `CacheCounters` (cache/stats)**

| 特性 | `CacheStats` | `CacheCounters` |
|------|--------------|-----------------|
| 位置 | `src/core/stats/utils.rs` | `src/query/cache/stats.rs` |
| hits/misses | ✅ | ✅ |
| evictions | ❌ | ✅ |
| expirations | ❌ | ✅ |
| insertions | ❌ | ✅ |
| rejections | ❌ | ✅ |
| hit_rate() | ✅ | ✅ |
| reset() | ✅ | ✅ |

**影响**: 
- 维护两套相似逻辑
- 开发者容易混淆
- 代码膨胀

### 2.3 缺失 Getter

**位置**: `src/core/stats/global_metrics.rs`

```rust
/// Get total query count
pub fn get_query_count(&self) -> u64 {
    // Note: metrics::Counter doesn't expose a getter, so we track it separately
    // This is a placeholder - in real implementation, we'd need to track the count
    0  // 硬编码返回 0！
}
```

**影响**: 
- HTTP `/database` 端点无法获取真实查询数
- `statistics.rs` 中的 `global_query_total` 始终为 0

### 2.4 Storage 层无指标

删除 `monitoring` 目录后，存储层完全没有性能监控：
- 无法追踪扫描效率
- 无法监控缓存命中率
- 无法统计操作类型分布

### 2.5 TelemetryRecorder 未安装

代码中有 `init_global_recorder()` 和 `set_global_recorder()`，但未在启动流程中调用。

---

## 三、统一 Metrics 体系架构

### 3.1 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    暴露层 (Exposure)                     │
├─────────────────────────────────────────────────────────┤
│  TelemetryRecorder (自定义 Recorder)                     │
│  - DashMap 存储                                         │
│  - 快照导出 (JSON/Prometheus)                           │
│  - HTTP /metrics 端点                                   │
├─────────────────────────────────────────────────────────┤
│                    收集层 (Collection)                   │
├─────────────────────────────────────────────────────────┤
│  GlobalMetrics (全局指标)                                │
│  - 查询指标 (total, duration, active)                    │
│  - 存储指标 (scan, cache hit/miss)                       │
│  - 执行器指标 (rows, memory)                             │
│  - 错误指标 (by type, by phase)                          │
├─────────────────────────────────────────────────────────┤
│                    业务层 (Business)                     │
├──────────────┬──────────────┬───────────────────────────┤
│ SyncMetrics  │FulltextMetrics│ StorageMetrics (集成)     │
│ - 事务同步   │ - 全文搜索    │ - 扫描/返回效率           │
│ - 索引操作   │ - 搜索延迟    │ - 操作类型计数            │
│ - 死信队列   │ - 缓存命中率  │ - 与 GlobalMetrics 统一   │
└──────────────┴──────────────┴───────────────────────────┘
```

### 3.2 设计原则

**1. 单一数据源**
- 所有指标统一通过 `metrics` crate 记录
- 移除所有内部计数器（`AtomicU64` 等）
- `TelemetryRecorder` 作为唯一存储后端

**2. 分层职责**

| 层级 | 职责 | 实现 |
|------|------|------|
| 暴露层 | 指标存储、导出、格式化 | `TelemetryRecorder` |
| 收集层 | 全局指标聚合 | `GlobalMetrics` |
| 业务层 | 领域指标记录 | 各模块 `*Metrics` |

**3. 命名规范**

```
graphdb_<模块>_<操作>_<类型>

示例:
- graphdb_query_total
- graphdb_storage_scan_duration_seconds
- graphdb_cache_hits_total
- graphdb_error_by_type_total{type="parse_error"}
```

**4. 指标类型选择**

| 类型 | 用途 | 示例 |
|------|------|------|
| Counter | 单调递增 | 查询总数、错误总数 |
| Gauge | 可增减 | 活跃查询数、内存使用 |
| Histogram | 分布统计 | 查询延迟、扫描时间 |

---

## 四、修复方案

### 4.1 P0: 修复 GlobalMetrics.get_query_count()

**方案**: 使用独立原子计数器跟踪查询总数

```rust
pub struct GlobalMetrics {
    query_total: Counter,
    query_total_count: AtomicU64,  // 新增
    // ...
}

pub fn get_query_count(&self) -> u64 {
    self.query_total_count.load(Ordering::Relaxed)
}
```

### 4.2 P0: Storage 层集成 Metrics

**方案**: 在 `GlobalMetrics` 中补充 storage 方法，在关键操作中记录

```rust
impl GlobalMetrics {
    pub fn record_storage_scan(&self, duration: Duration, count: u64) {
        self.storage_scan_total.increment(1);
        self.storage_items_scanned.increment(count);
        self.storage_scan_duration.record(duration.as_secs_f64());
    }
}
```

### 4.3 P1: 清理 IterStats 双重记录

**方案**: 移除内部计数器，仅保留 `metrics` 记录

```rust
pub struct IterStats {
    // 移除所有字段，改为空结构体或完全删除
}

impl IterStats {
    pub fn record_scan(&self) {
        metrics::counter!("graphdb_storage_iter_items_scanned_total").increment(1);
    }
}
```

### 4.4 P1: 统一 CacheStats 和 CacheCounters

**方案**: 扩展 `CacheStats` (utils) 支持所有缓存统计类型，删除 `CacheCounters`

```rust
pub struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,      // 新增
    expirations: AtomicU64,    // 新增
    insertions: AtomicU64,     // 新增
    rejections: AtomicU64,     // 新增
}
```

### 4.5 P2: 安装 TelemetryRecorder

**方案**: 在服务器启动时调用 `set_global_recorder()`

```rust
pub fn start_server(addr: &str) -> Result<(), Error> {
    let recorder = TelemetryRecorder::new();
    set_global_recorder(recorder)?;
    // ...
}
```

---

## 五、实施优先级

| 优先级 | 任务 | 工作量 | 风险 |
|--------|------|--------|------|
| **P0** | 修复 `get_query_count()` | 小 | 低 |
| **P0** | Storage 层集成 metrics | 中 | 低 |
| **P1** | 清理 `IterStats` 双重记录 | 小 | 低 |
| **P1** | 统一 `CacheStats` 和 `CacheCounters` | 中 | 中 |
| **P2** | 安装 `TelemetryRecorder` | 小 | 低 |

---

## 六、指标清单

### 6.1 查询指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_query_total` | Counter | 总查询数 |
| `graphdb_query_duration_seconds` | Histogram | 查询延迟 |
| `graphdb_query_active` | Gauge | 活跃查询数 |
| `graphdb_query_match_total` | Counter | MATCH 查询数 |
| `graphdb_query_create_total` | Counter | CREATE 查询数 |
| `graphdb_query_update_total` | Counter | UPDATE 查询数 |
| `graphdb_query_delete_total` | Counter | DELETE 查询数 |
| `graphdb_query_insert_total` | Counter | INSERT 查询数 |
| `graphdb_query_go_total` | Counter | GO 查询数 |
| `graphdb_query_fetch_total` | Counter | FETCH 查询数 |
| `graphdb_query_lookup_total` | Counter | LOOKUP 查询数 |
| `graphdb_query_show_total` | Counter | SHOW 查询数 |

### 6.2 存储指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_storage_scan_total` | Counter | 存储扫描次数 |
| `graphdb_storage_scan_duration_seconds` | Histogram | 扫描延迟 |
| `graphdb_storage_cache_hits_total` | Counter | 缓存命中数 |
| `graphdb_storage_cache_misses_total` | Counter | 缓存未命中数 |
| `graphdb_storage_iter_items_scanned_total` | Counter | 迭代器扫描项数 |
| `graphdb_storage_iter_items_returned_total` | Counter | 迭代器返回项数 |
| `graphdb_storage_iter_seek_operations_total` | Counter | 迭代器 seek 操作数 |

### 6.3 执行器指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_executor_rows_processed_total` | Counter | 处理的行数 |
| `graphdb_executor_memory_used_bytes` | Gauge | 内存使用量 |

### 6.4 错误指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_error_total` | Counter | 错误总数 |
| `graphdb_error_by_type_total{type}` | Counter | 按类型分类的错误数 |
| `graphdb_error_by_phase_total{phase}` | Counter | 按阶段分类的错误数 |

### 6.5 慢查询指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_slow_query_total` | Counter | 慢查询总数 |
| `graphdb_slow_query_duration_seconds` | Histogram | 慢查询延迟 |
| `graphdb_slow_query_active` | Gauge | 活跃慢查询数 |
| `graphdb_slow_query_errors_total` | Counter | 慢查询错误数 |

### 6.6 同步系统指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_sync_transactions_committed_total` | Counter | 提交的事务数 |
| `graphdb_sync_transactions_rolled_back_total` | Counter | 回滚的事务数 |
| `graphdb_sync_index_operations_total` | Counter | 索引操作总数 |
| `graphdb_sync_retry_attempts_total` | Counter | 重试尝试数 |
| `graphdb_sync_dead_letter_queue_size` | Gauge | 死信队列大小 |
| `graphdb_sync_active_transactions` | Gauge | 活跃事务数 |

### 6.7 全文搜索指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `graphdb_fulltext_index_ops_total` | Counter | 索引操作数 |
| `graphdb_fulltext_indexed_docs_total` | Counter | 索引文档数 |
| `graphdb_fulltext_search_ops_total` | Counter | 搜索操作数 |
| `graphdb_fulltext_search_duration_seconds` | Histogram | 搜索延迟 |
| `graphdb_fulltext_queue_size` | Gauge | 队列大小 |
| `graphdb_fulltext_cache_hits_total` | Counter | 缓存命中数 |
| `graphdb_fulltext_cache_misses_total` | Counter | 缓存未命中数 |

---

## 七、参考文档

- [使用指南](usage_guide.md) - metrics 使用指南
- [架构文档](architecture.md) - 详细架构说明（已过时，待更新）
- [迁移总结](migration_summary.md) - 迁移过程和经验（已过时，待更新）

---

**文档版本**: 1.0  
**最后更新**: 2026-05-07  
**维护者**: GraphDB Team
