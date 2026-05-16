# Metrics 正式集成分析与改进方案

## 1. 分析范围

本文分析 graphDB 主 crate（`src/`）中 metrics 系统与搜索模块的正式集成情况，不包括 `crates/bm25` 和 `crates/inversearch` 内部的 `StorageMetrics`（它们作为内部实现细节保留）。

参考文档：

- [metrics-architecture-design.md](./metrics-architecture-design.md) — 架构设计方案
- [metrics_integration.md](./metrics_integration.md) — crate 层 metrics 分析（独立范围）

---

## 2. 现状总览

### 2.1 已实现组件

| 组件                                          | 状态      | 位置                                         |
| --------------------------------------------- | --------- | -------------------------------------------- |
| `MetricType` 搜索枚举值（含错误分类）         | ✅ 已定义 | `src/core/stats/manager.rs`                  |
| `StatsManager` 搜索记录方法                   | ✅ 已实现 | `src/core/stats/manager.rs`                  |
| `StatsManager` 错误分类方法                   | ✅ 已实现 | `src/core/stats/manager.rs`                  |
| `MetricsSearchEngine` Decorator（含错误分类） | ✅ 已实现 | `src/search/metrics.rs`                      |
| `FulltextIndexManager.wrap_engine()`          | ✅ 已实现 | `src/search/manager.rs`                      |
| `FulltextIndexManager.with_stats_manager()`   | ✅ 已实现 | `src/search/manager.rs`                      |
| `FulltextIndexManager.set_stats_manager()`    | ✅ 已实现 | `src/search/manager.rs`                      |
| API `/statistics/search` 端点（含百分位）     | ✅ 已实现 | `src/api/server/http/handlers/statistics.rs` |
| API `/statistics/database` 含搜索指标         | ✅ 已实现 | `src/api/server/http/handlers/statistics.rs` |
| IndexCache 缓存命中/未命中记录                | ✅ 已实现 | `src/search/index_cache.rs`                  |
| 搜索延迟百分位统计                            | ✅ 已实现 | `src/core/stats/manager.rs`                  |

### 2.2 架构对应关系

```
架构设计 (metrics-architecture-design.md)        当前实现
─────────────────────────────────────────────    ────────────
MetricsSearchEngine Decorator                    ✅ src/search/metrics.rs
StatsManager 扩展 MetricType                     ✅ manager.rs
SearchStatsCollector (DashMap 按引擎统计)         ❌ 未实现（被 StatsManager 方案替代）
API 暴露搜索 metrics                              ✅ handlers/statistics.rs
crate 内部 StorageMetrics 保留                    ✅ 已保留
```

**说明**：架构设计中提出的 `SearchStatsCollector`（`DashMap<EngineType, EngineSearchStats>`）方案已被更简洁的 `StatsManager` + `MetricType` 方案替代。当前方案复用已有基础设施，支持 space 级别隔离，与查询 metrics 共享同一套 API。

---

## 3. 关键问题

### 问题 1（P0）：启动时 StatsManager 未注入到 FulltextIndexManager

**状态**：✅ 已修复

**严重性**：阻塞性 — 整个搜索 metrics 系统完全失效。

**根因**：在 `src/api/mod.rs` 的 `start_service_with_config()` 中：

```
FulltextIndexManager::new(config)   ← 创建时无 stats_manager
        │
        ▼
SyncCoordinator::new(manager)       ← 包装进协调器
        │
        ▼
SyncManager::with_sync_config()     ← 包装进同步管理器
        │
        ▼
GraphService::create_service()      ← StatsManager 在此创建
        │                                   但 FulltextIndexManager
        ▼                                   已无法访问
start_http_server()
```

`FulltextIndexManager` 在 `GraphService` 创建 `StatsManager` **之前** 就已经创建完毕，且两者之间没有连接通道。结果是：

- `wrap_engine()` 中的 `if let Some(ref stats_manager) = self.stats_manager` 永远为 `None`
- `MetricsSearchEngine` 永远不会被创建
- **所有搜索/索引/删除操作的 metrics 均为零**

**修复方案**：

1. 在 `FulltextIndexManager` 中添加 `set_stats_manager(&self, stats_manager: Arc<StatsManager>)` 方法，使用 `Mutex<Option<Arc<StatsManager>>>` 实现内部可变性，注入后自动重新包装所有现有引擎
2. 在 `SearchEngine` trait 中添加 `is_metrics_wrapped()` 默认方法返回 `false`，`MetricsSearchEngine` 覆盖返回 `true`，避免重复包装
3. 在 `src/api/mod.rs` 的 `start_service_with_config()` 中，`GraphService` 创建完成后通过 `sync_api → sync_manager → fulltext_manager` 链路注入 `StatsManager`

### 问题 2（P1）：IndexCache 未记录缓存指标

**状态**：✅ 已修复

`src/search/index_cache.rs` 中的缓存操作没有调用 `StatsManager::record_cache_hit()`，导致 `SearchCacheHitCount` 和 `SearchCacheMissCount` 始终为零。

**修复方案**：在 `IndexCache` 中添加 `stats_manager: Option<Arc<StatsManager>>` 字段和 `with_stats_manager()` 方法，在 `get()` 方法中记录缓存命中/未命中。

### 问题 3（P1）：搜索延迟缺少百分位统计

**状态**：✅ 已修复

查询 metrics 有 `LatencyHistogram` 支持 P50/P90/P99 计算，但搜索 metrics 只记录累计延迟（`SearchLatencyMs`），无法反映长尾延迟。

**修复方案**：在 `StatsManager` 中添加 `search_latency_histogram: Arc<RwLock<LatencyHistogram>>` 字段，在 `record_search()` 中记录延迟百分位，添加 `get_search_latency_percentiles()` 查询方法。

### 问题 4（P2）：搜索错误缺少分类

**状态**：✅ 已修复

当前只区分 success/failure，没有按错误类型（索引不存在、引擎错误、IO 错误等）分类统计。

**修复方案**：

1. 在 `MetricType` 中添加 5 个错误分类变体：`SearchErrorIndexNotFound`、`SearchErrorEngineError`、`SearchErrorIoError`、`SearchErrorSerialization`、`SearchErrorInternal`
2. 在 `StatsManager` 中添加 `classify_search_error()` 方法将 `SearchError` 映射到对应的 `MetricType`
3. 添加 `record_search_error()`、`record_index_error()`、`record_delete_error()` 方法记录分类错误
4. 在 `MetricsSearchEngine` 中所有操作失败时调用对应的错误分类记录方法

### 问题 5（P2）：record_search() 的 \_index_name 参数未使用

**状态**：✅ 已修复

在 `StatsManager::record_search()` 中，`_index_name` 参数被忽略，无法按索引粒度查询 metrics。

**修复方案**：

1. 在 `StatsManager` 中添加 `index_metrics: Arc<DashMap<String, SpaceMetrics>>` 字段，支持按索引名称存储 metrics
2. 添加 `add_index_metric()`、`add_index_metric_with_amount()`、`get_index_value()`、`get_all_index_metrics()` 方法
3. 在 `record_search()`、`record_index_operation()`、`record_delete_operation()` 中使用 `index_name` 参数记录索引级 metrics
4. 在 `record_search_error()`、`record_index_error()`、`record_delete_error()` 中添加 `index_name` 参数，记录索引级错误分类 metrics
5. 在 `record_search_result_count()` 和 `record_cache_hit()` 中使用 `space_id` 记录空间级 metrics

---

## 4. 改进方案

### 阶段一（P0）：修复启动连接 ✅ 已完成

#### 4.1.1 给 FulltextIndexManager 添加 set_stats_manager()

在 `src/search/manager.rs` 中添加方法，支持在运行时注入 `StatsManager` 并重新包装所有现有引擎。使用 `Mutex<Option<Arc<StatsManager>>>` 实现内部可变性，通过 `SearchEngine::is_metrics_wrapped()` 避免重复包装。

#### 4.1.2 在启动流程中建立连接

在 `src/api/mod.rs` 的 `start_service_with_config()` 中，`GraphService` 创建完成后注入：

```rust
// GraphService 创建之后
let stats_manager = graph_service.get_stats_manager().clone();
if let Some(sync_api) = graph_service.sync_api() {
    let fulltext_manager = sync_api.sync_manager().fulltext_manager();
    fulltext_manager.set_stats_manager(stats_manager);
    info!("StatsManager injected into FulltextIndexManager for search metrics");
}
```

### 阶段二（P1）：添加搜索延迟百分位统计 ✅ 已完成

#### 4.2.1 StatsManager 新增字段

```rust
pub struct StatsManager {
    // ... 现有字段 ...
    search_latency_histogram: Arc<RwLock<LatencyHistogram>>,
}
```

#### 4.2.2 record_search() 中记录百分位

```rust
pub fn record_search(&self, space_id: u64, index_name: &str, latency_ms: u64, success: bool) {
    // ... 现有逻辑 ...

    // 记录延迟百分位
    {
        let mut histogram = self.search_latency_histogram.write();
        histogram.record_micros(latency_ms * 1000);
    }
}
```

#### 4.2.3 新增查询方法

```rust
pub fn get_search_latency_percentiles(&self) -> (u64, u64, u64, u64) {
    let histogram = self.search_latency_histogram.write();
    (histogram.avg(), histogram.p50(), histogram.p95(), histogram.p99())
}
```

### 阶段三（P1）：IndexCache 集成缓存指标 ✅ 已完成

在 `src/search/index_cache.rs` 中添加 `stats_manager` 字段，在 get 操作时记录缓存命中/未命中。

### 阶段四（P2）：增强 API 端点 ✅ 已完成

在 `/statistics/search` 端点中添加延迟百分位和错误分类详情。

### 阶段五（P2）：搜索错误分类 ✅ 已完成

在 `MetricType` 中扩展搜索错误类型，在 `MetricsSearchEngine` 中根据 `SearchError` 类型分类记录。

---

## 5. 修改文件清单

| 优先级 | 文件                                         | 修改内容                                                                     | 状态 |
| ------ | -------------------------------------------- | ---------------------------------------------------------------------------- | ---- |
| **P0** | `src/search/engine.rs`                       | 在 `SearchEngine` trait 中添加 `is_metrics_wrapped()` 默认方法               | ✅   |
| **P0** | `src/search/metrics.rs`                      | `MetricsSearchEngine` 覆盖 `is_metrics_wrapped()` 返回 `true`                | ✅   |
| **P0** | `src/search/manager.rs`                      | 添加 `set_stats_manager()` 方法，`stats_manager` 改为 `Mutex` 支持内部可变性 | ✅   |
| **P0** | `src/api/mod.rs`                             | 在 `start_service_with_config()` 中注入 StatsManager                         | ✅   |
| **P1** | `src/core/stats/manager.rs`                  | 添加 `search_latency_histogram` 字段和百分位查询方法                         | ✅   |
| **P1** | `src/search/index_cache.rs`                  | 添加 stats_manager 字段，记录缓存命中/未命中                                 | ✅   |
| **P2** | `src/api/server/http/handlers/statistics.rs` | 在 `/statistics/search` 中添加百分位                                         | ✅   |
| **P2** | `src/core/stats/manager.rs`                  | 扩展 MetricType 搜索错误分类，添加错误分类方法                               | ✅   |
| **P2** | `src/search/metrics.rs`                      | 添加错误分类记录逻辑                                                         | ✅   |
| **P3** | `src/core/stats/manager.rs`                  | 添加 `index_metrics` 字段，支持按索引粒度查询 metrics                        | ✅   |
| **P3** | `src/search/metrics.rs`                      | 错误分类方法传递 `index_name` 参数                                           | ✅   |

---

## 6. 实施顺序

```
阶段一 (P0) ──► 阶段二 (P1) ──► 阶段三 (P1) ──► 阶段四 (P2) ──► 阶段五 (P2) ──► 阶段六 (P3)
  修复启动连接     延迟百分位       缓存指标          API 增强          错误分类        索引级 metrics
  ✅ 已完成        ✅ 已完成       ✅ 已完成        ✅ 已完成         ✅ 已完成       ✅ 已完成
```

所有阶段均已实施完成，编译通过。当前 metrics 系统已具备完整的三层粒度：全局级、空间级、索引级。
