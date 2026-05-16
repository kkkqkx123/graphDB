# Metrics 系统现状分析与架构调整方案

## 1. 现状分析

### 1.1 当前监控体系概览

当前项目存在**两套并行的 metrics 系统**，以及**多个散落的指标采集点**，整体缺乏统一架构。

```
┌─────────────────────────────────────────────────────────────────┐
│                    两套并行 Metrics 系统                          │
├─────────────────────────┬───────────────────────────────────────┤
│  系统 A: TelemetryRecorder │  系统 B: core::stats 体系            │
│  (metrics crate Recorder) │  (GlobalMetrics + StatsManager)      │
├─────────────────────────┼───────────────────────────────────────┤
│  src/api/core/telemetry.rs│  src/core/stats/                     │
│  • 自定义 Recorder 实现   │  • GlobalMetrics (Prometheus 风格)    │
│  • DashMap 存储          │  • StatsManager (统一管理器)          │
│  • /metrics 端点暴露      │  • QueryMetrics (轻量查询指标)        │
│  • JSON/Prometheus 格式   │  • QueryProfile (详细查询画像)        │
│  • EmbeddedTelemetry      │  • SlowQueryLogger (慢查询日志)       │
│                           │  • ErrorStatsManager (错误统计)       │
│                           │  • LatencyHistogram (延迟百分位)      │
│                           │  • AggregatedStatsManager (聚合统计)  │
└─────────────────────────┴───────────────────────────────────────┘
```

### 1.2 散落的指标采集点

| 位置                                               | 指标                 | 方式                           |
| -------------------------------------------------- | -------------------- | ------------------------------ |
| `src/sync/metrics.rs`                              | 同步系统指标         | 直接调用 `metrics::counter!`   |
| `src/search/metrics.rs`                            | 全文搜索指标         | 直接调用 `metrics::counter!`   |
| `src/query/query_manager.rs`                       | 查询生命周期指标     | 直接调用 `metrics::counter!`   |
| `src/query/cache/stats.rs`                         | 缓存命中/未命中/驱逐 | 直接调用 `metrics::counter!`   |
| `src/query/optimizer/stats/feedback/collector.rs`  | 优化器反馈           | 直接调用 `metrics::histogram!` |
| `src/storage/iterator/storage_iter.rs`             | 存储迭代器           | 直接调用 `metrics::counter!`   |
| `crates/inversearch/src/metrics.rs`                | Inversearch 指标     | 独立 Counter/Histogram         |
| `crates/inversearch/src/storage/common/metrics.rs` | Inversearch 存储指标 | 直接调用 `metrics::counter!`   |

### 1.3 当前架构的核心问题

#### 问题 1：两套系统重复且冲突

- `TelemetryRecorder` 实现了 `metrics::Recorder` trait，作为全局 recorder 注册
- `GlobalMetrics` 又通过 `metrics::counter!` / `metrics::histogram!` 宏直接创建指标
- 两套系统都使用 `metrics` crate 0.23，但 `GlobalMetrics` 的指标会通过 `TelemetryRecorder` 被重复采集
- 实际上 `GlobalMetrics` 维护了自己的 `AtomicU64` 副本，与 `TelemetryRecorder` 中的值可能不一致

#### 问题 2：无统一注册表

参考设计中的 `MetricsRegistry` 作为全局单例持有所有指标，当前项目没有等价物：

- `TelemetryRecorder` 用 `DashMap` 存储，但无结构化组织
- `GlobalMetrics` 是硬编码的字段集合，不可扩展
- 无法统一遍历/导出所有指标

#### 问题 3：无标签系统

- 当前指标名称为扁平字符串（如 `graphdb_query_total`）
- 无结构化标签（Label）系统
- 无标签键白名单验证
- 无预定义枚举标签（OperationType, Component 等）

#### 问题 4：无领域层封装

参考设计有 `EmbeddingMetrics`、`ParserMetrics`、`SearchMetrics` 等语义化结构体，当前项目：

- `SyncMetrics` 和 `FulltextMetrics` 只是简单的方法集合
- 无 `record_*` 语义方法封装
- 业务代码直接操作原始 `metrics::counter!` 宏

#### 问题 5：无聚合持久化

- 无 `MetricsAggregator` 后台任务
- 无定期将 Histogram 聚合写入 SQLite 的机制
- 无历史查询能力（`/api/metrics/history`）
- 无数据清理机制（`/api/metrics/cleanup`）

#### 问题 6：无运行时/系统指标

- 无 Tokio 运行时指标采集
- 无系统资源指标（CPU、内存、磁盘）
- 无 `RuntimeMetrics` / `SystemMetrics` 结构体

#### 问题 7：无 RAII 定时器

- 业务代码手动记录开始/结束时间
- 无 `OperationTimer` 在 Drop 时自动记录耗时
- 容易遗漏或错误计算耗时

#### 问题 8：无进度追踪

- 无 `ProgressTracker` 结构体
- 无法追踪扫描/解析/嵌入等阶段的进度
- 无进度快照导出能力

#### 问题 9：指标命名不一致

- `graphdb_query_total`（query_manager.rs）
- `graphdb_queries_total`（global_metrics.rs）
- `graphdb_fulltext_search_ops_total`（search/metrics.rs）
- `graphdb_sync_transactions_committed_total`（sync/metrics.rs）
- `inversearch_storage_operations_total`（crates/inversearch）
- 命名风格、前缀、单位均不统一

---

## 2. 目标架构

### 2.1 整体架构

采用参考设计的三层架构，逐步迁移：

```
┌──────────────────────────────────────────────────────────────────┐
│                        导出层 (Export Layer)                       │
│  JSON Exporter │ Prometheus Exporter │ File Writer │ SQLite      │
│  (复用现有 TelemetryServer)          │ (新增 MetricsAggregator)   │
├──────────────────────────────────────────────────────────────────┤
│                        领域层 (Domain Layer)                       │
│  Pipeline │ Search │ Storage │ Orchestrator │ Runtime │ System   │
│  (新增语义化结构体，封装原始指标)                                   │
├──────────────────────────────────────────────────────────────────┤
│                      基础设施层 (Infrastructure Layer)              │
│  MetricsRegistry │ Counter │ Gauge │ Histogram │ Labels          │
│  (统一注册表，替代两套并行系统)                                      │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 设计原则

| 原则           | 说明                                                                   |
| -------------- | ---------------------------------------------------------------------- |
| **统一注册表** | 所有指标通过 `MetricsRegistry` 创建和管理，替代两套并行系统            |
| **分层隔离**   | 领域层封装原始指标为语义化结构体，业务代码不直接操作原始 Counter/Gauge |
| **RAII 定时**  | 通过 `OperationTimer` 在 Drop 时自动记录操作耗时                       |
| **标签系统**   | Label Key 通过白名单验证，预定义枚举标签防止拼写错误                   |
| **渐进迁移**   | 不中断现有功能，分阶段迁移到新架构                                     |
| **无外部依赖** | 核心指标类型基于 `Arc<AtomicU64>` 手写实现，减少依赖树                 |

### 2.3 与参考设计的差异

由于项目实际情况，对参考设计做以下调整：

| 调整项     | 参考设计                 | 本项目方案                              | 理由                                             |
| ---------- | ------------------------ | --------------------------------------- | ------------------------------------------------ |
| 指标库     | 无外部依赖，手写实现     | 保留 `metrics` crate 作为底层，上层封装 | 已有大量代码依赖 `metrics` crate，完全重写成本高 |
| 注册表     | 独立 `MetricsRegistry`   | 基于 `TelemetryRecorder` 扩展           | 复用现有全局 recorder 机制                       |
| 导出层     | 独立 Exporter trait      | 复用现有 `TelemetryServer`              | 已有完整的 HTTP 端点                             |
| 聚合层     | 独立 `MetricsAggregator` | 新增，与参考设计一致                    | 当前完全缺失                                     |
| 运行时指标 | Tokio runtime            | 新增，与参考设计一致                    | 当前完全缺失                                     |
| 系统指标   | sysinfo crate            | 新增，与参考设计一致                    | 当前完全缺失                                     |

---

## 3. 分阶段迁移计划

### 阶段一：基础设施层重构（预计 2-3 天）

#### 3.1.1 统一注册表

基于现有 `TelemetryRecorder` 扩展，新增 `MetricsRegistry` 封装：

```rust
// 新增: src/metrics/registry.rs
pub struct MetricsRegistry {
    inner: Arc<TelemetryRecorder>,
    // 可选的标签元数据
    default_labels: Labels,
}

impl MetricsRegistry {
    pub fn global() -> &'static Self { ... }
    pub fn counter(&self, name: &str, labels: &[(&str, &str)]) -> LabeledCounter { ... }
    pub fn gauge(&self, name: &str, labels: &[(&str, &str)]) -> LabeledGauge { ... }
    pub fn histogram(&self, name: &str, labels: &[(&str, &str)]) -> LabeledHistogram { ... }
    pub fn export_all(&self) -> MetricsSnapshot { ... }
}
```

#### 3.1.2 标签系统

```rust
// 新增: src/metrics/labels.rs
pub struct Labels(BTreeMap<String, String>);

// 白名单验证
const ALLOWED_LABEL_KEYS: &[&str] = &[
    "operation", "component", "status", "project_id",
    "language", "provider", "space",
];

// 预定义枚举
pub enum OperationType { Indexing, Querying, Embedding, ... }
pub enum Component { Scanner, Parser, VectorStore, ... }
```

#### 3.1.3 核心类型封装

```rust
// 新增: src/metrics/types.rs
pub struct LabeledCounter { ... }
pub struct LabeledGauge { ... }
pub struct LabeledHistogram { ... }
pub struct OperationTimer { ... } // RAII 定时器
```

#### 3.1.4 文件结构

```
src/metrics/
├── mod.rs           # 模块入口，MetricsRegistry
├── types.rs         # 核心类型 + 枚举标签
├── labels.rs        # 标签系统
└── progress.rs      # 进度追踪
```

### 阶段二：领域层建设（预计 3-4 天）

#### 3.2.1 流水线指标

```rust
// 新增: src/metrics/domain/pipeline.rs
pub struct EmbeddingMetrics {
    requests_total: LabeledCounter,
    tokens_total: LabeledCounter,
    errors_total: LabeledCounter,
    latency_ms: LabeledHistogram,
}

impl EmbeddingMetrics {
    pub fn new(registry: &MetricsRegistry, model: &str) -> Self { ... }
    pub fn record_request(&self, tokens: u64, latency: Duration) { ... }
    pub fn record_error(&self) { ... }
}
```

#### 3.2.2 搜索指标

```rust
// 新增: src/metrics/domain/search.rs
pub struct SearchMetrics {
    queries_total: LabeledCounter,
    query_latency_ms: LabeledHistogram,
    index_size: LabeledGauge,
    // ...
}
```

#### 3.2.3 存储指标

```rust
// 新增: src/metrics/domain/storage.rs
pub struct Bm25Metrics { ... }
pub struct QdrantMetrics { ... }
pub struct SqliteMetrics { ... }
```

#### 3.2.4 运行时与系统指标

```rust
// 新增: src/metrics/domain/runtime.rs
pub struct RuntimeMetrics {
    // Tokio runtime 指标
    workers_total: LabeledGauge,
    active_tasks: LabeledGauge,
    // ...
}

// 新增: src/metrics/domain/system.rs
pub struct SystemMetrics {
    // sysinfo 指标
    cpu_usage_percent: LabeledGauge,
    memory_used_bytes: LabeledGauge,
    // ...
}
```

#### 3.2.5 文件结构

```
src/metrics/domain/
├── mod.rs           # 模块入口
├── pipeline.rs      # 流水线指标
├── search.rs        # 搜索指标
├── storage.rs       # 存储指标
├── orchestrator.rs  # 编排器指标
├── runtime.rs       # 运行时指标
└── system.rs        # 系统指标
```

### 阶段三：聚合与持久化（预计 2 天）

#### 3.3.1 MetricsAggregator

```rust
// 新增: src/metrics/aggregator.rs
pub struct MetricsAggregator {
    registry: Arc<MetricsRegistry>,
    sqlite: SqliteClient,
    config: MetricsAggregationConfig,
    // 后台任务 handle
}

pub struct MetricsAggregationConfig {
    pub enabled: bool,
    pub interval_secs: u64,  // 默认 300
}
```

#### 3.3.2 聚合数据表

```sql
CREATE TABLE metrics_aggregated (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    labels_json TEXT NOT NULL,
    count INTEGER NOT NULL,
    avg REAL NOT NULL,
    median REAL NOT NULL,
    max REAL NOT NULL,
    p90 REAL NOT NULL,
    p99 REAL NOT NULL,
    project_id TEXT,
    operation_type TEXT
);
```

### 阶段四：现有代码迁移（预计 3-4 天）

#### 3.4.1 迁移优先级

| 优先级 | 模块                                   | 迁移方式                     | 影响范围   |
| ------ | -------------------------------------- | ---------------------------- | ---------- |
| P0     | `src/sync/metrics.rs`                  | 重写为 `SyncDomainMetrics`   | 同步系统   |
| P0     | `src/search/metrics.rs`                | 重写为 `SearchDomainMetrics` | 搜索系统   |
| P1     | `src/query/query_manager.rs`           | 替换内联指标调用             | 查询系统   |
| P1     | `src/query/cache/stats.rs`             | 替换内联指标调用             | 缓存系统   |
| P2     | `src/core/stats/global_metrics.rs`     | 逐步废弃，迁移到新注册表     | 全局指标   |
| P2     | `src/storage/iterator/storage_iter.rs` | 替换内联指标调用             | 存储系统   |
| P3     | `crates/inversearch/src/metrics.rs`    | 统一指标命名                 | 外部 crate |

#### 3.4.2 迁移策略

1. **新增不删旧**：先创建新架构代码，旧代码保持运行
2. **双写过渡**：过渡期同时写入新旧两套系统，验证数据一致性
3. **逐步切换**：每个模块独立迁移，逐个验证
4. **最终清理**：所有模块迁移完成后，删除旧代码

### 阶段五：API 与导出层整合（预计 1-2 天）

#### 3.5.1 API 端点

| 方法   | 路径                   | 说明           | 状态     |
| ------ | ---------------------- | -------------- | -------- |
| GET    | `/api/metrics`         | 系统健康摘要   | 复用现有 |
| GET    | `/api/metrics/json`    | 全量指标快照   | 新增     |
| GET    | `/api/metrics/history` | 历史聚合数据   | 新增     |
| DELETE | `/api/metrics/cleanup` | 删除历史数据   | 新增     |
| GET    | `/metrics`             | Telemetry 端点 | 保留现有 |

#### 3.5.2 导出格式统一

- JSON 格式：统一使用 `MetricsSnapshot` 结构
- Prometheus 格式：复用现有 `to_text_format()` 方法
- 文件日志：新增 `metrics.log` JSON Lines 输出

---

## 4. 数据流设计

```
业务代码 (Query/Sync/Search/Storage)
    │
    ├─► 领域指标 (record_* 方法) ──► MetricsRegistry (内存)
    │                                      │
    │                                      ├─► export_all() ──► /api/metrics/json (快照)
    │                                      ├─► TelemetryServer ──► /metrics (Prometheus/JSON)
    │                                      ├─► MetricsAggregator (后台) ──► SQLite (历史)
    │                                      │       │
    │                                      │       ├─► query_history() ──► /api/metrics/history
    │                                      │       └─► cleanup() ──► /api/metrics/cleanup
    │                                      │
    │                                      └─► log_metric_to_file() ──► metrics.log
    │
    ├─► RuntimeMetrics::collect() ──► Tokio 指标 (瞬时，不入 SQLite)
    │
    └─► SystemMetrics::collect() ──► sysinfo 指标 (瞬时，不入 SQLite)

ProgressTracker (独立于 Registry)
    │
    ├─► get_progress() ──► /api/metrics (在系统健康摘要中)
    │
    └─► (被 API、CLI 输出使用)
```

---

## 5. 配置设计

```rust
// src/config/global.rs 扩展
pub struct MetricsConfig {
    pub aggregation: MetricsAggregationConfig,
    pub runtime_metrics: RuntimeMetricsConfig,
    pub system_metrics: SystemMetricsConfig,
}

pub struct MetricsAggregationConfig {
    pub enabled: bool,        // 默认 false
    pub interval_secs: u64,   // 默认 300
}

pub struct RuntimeMetricsConfig {
    pub enabled: bool,        // 默认 false
    pub collection_interval_secs: u64,  // 默认 60
}

pub struct SystemMetricsConfig {
    pub enabled: bool,        // 默认 false
    pub collection_interval_secs: u64,  // 默认 60
}
```

---

## 6. 与参考设计的对照

| 参考设计组件                     | 当前状态    | 目标状态                    | 优先级 |
| -------------------------------- | ----------- | --------------------------- | ------ |
| MetricsRegistry                  | ❌ 无       | 基于 TelemetryRecorder 扩展 | P0     |
| Counter/Gauge/Histogram          | ⚠️ 有但分散 | 统一封装 + Labeled 变体     | P0     |
| Labels 标签系统                  | ❌ 无       | 白名单 + 预定义枚举         | P0     |
| ProgressTracker                  | ❌ 无       | 新增                        | P1     |
| 领域层 (Pipeline/Search/Storage) | ❌ 无       | 新增语义化结构体            | P1     |
| 领域层 (Orchestrator)            | ❌ 无       | 新增                        | P1     |
| RuntimeMetrics                   | ❌ 无       | 新增 Tokio 指标             | P2     |
| SystemMetrics                    | ❌ 无       | 新增 sysinfo 指标           | P2     |
| MetricsAggregator                | ❌ 无       | 新增后台聚合任务            | P1     |
| MetricExporter trait             | ⚠️ 部分有   | 统一 Export trait           | P2     |
| JsonExporter                     | ⚠️ 部分有   | 统一 JSON 格式              | P2     |
| PrometheusExporter               | ⚠️ 部分有   | 复用现有                    | P2     |
| File Writer                      | ❌ 无       | 新增 metrics.log            | P2     |
| SQLite 持久化                    | ❌ 无       | 新增聚合表                  | P1     |
| API 端点                         | ⚠️ 部分有   | 补齐缺失端点                | P1     |
| OperationTimer (RAII)            | ❌ 无       | 新增                        | P0     |

---

## 7. 风险与缓解措施

| 风险             | 影响                      | 缓解措施                                   |
| ---------------- | ------------------------- | ------------------------------------------ |
| 迁移期间指标丢失 | 监控数据不完整            | 双写策略，新旧系统同时运行                 |
| 性能开销增加     | 额外锁竞争                | 使用 `DashMap` + `AtomicU64`，最小化锁粒度 |
| 与现有代码冲突   | 编译错误                  | 新增模块路径，不修改现有模块路径           |
| 外部 crate 依赖  | inversearch 等 crate 耦合 | 提供兼容层，逐步统一命名                   |
| 配置兼容性       | 旧配置失效                | 保持旧配置字段，新增字段可选               |

---

## 8. 验收标准

### 阶段一完成标志

- [ ] `MetricsRegistry` 全局单例可用
- [ ] `LabeledCounter`/`LabeledGauge`/`LabeledHistogram` 可用
- [ ] 标签白名单验证生效
- [ ] 预定义枚举标签可用
- [ ] `OperationTimer` RAII 定时器可用
- [ ] 现有 `TelemetryServer` 指标不受影响

### 阶段二完成标志

- [ ] `EmbeddingMetrics` 可用，业务代码可注入
- [ ] `SearchMetrics` 可用
- [ ] `StorageMetrics` (BM25/Qdrant/SQLite) 可用
- [ ] `RuntimeMetrics` 可采集 Tokio 指标
- [ ] `SystemMetrics` 可采集系统资源指标

### 阶段三完成标志

- [ ] `MetricsAggregator` 后台任务正常运行
- [ ] 聚合数据写入 SQLite `metrics_aggregated` 表
- [ ] `/api/metrics/history` 返回历史数据
- [ ] `/api/metrics/cleanup` 可清理历史数据

### 阶段四完成标志

- [ ] `src/sync/metrics.rs` 迁移完成
- [ ] `src/search/metrics.rs` 迁移完成
- [ ] `src/query/` 中内联指标调用迁移完成
- [ ] `src/core/stats/global_metrics.rs` 废弃标记
- [ ] 所有指标命名统一为 `graphdb_*` 前缀

### 阶段五完成标志

- [ ] 所有 API 端点可用
- [ ] JSON/Prometheus 格式统一
- [ ] 文件日志输出正常
- [ ] 旧代码清理完成，无编译警告

---

## 9. 文件清单（最终状态）

### 核心模块（新增/重构）

- `src/metrics/mod.rs` — 模块根，MetricsRegistry
- `src/metrics/types.rs` — 核心类型 + 枚举标签
- `src/metrics/labels.rs` — 标签系统
- `src/metrics/progress.rs` — 进度追踪
- `src/metrics/aggregator.rs` — 指标聚合
- `src/metrics/exporter.rs` — 导出器
- `src/metrics/serialization.rs` — 序列化结构
- `src/metrics/writer.rs` — 文件日志写入
- `src/metrics/logger.rs` — 便捷日志函数

### 领域模块（新增）

- `src/metrics/domain/mod.rs` — 模块入口
- `src/metrics/domain/pipeline.rs` — 流水线指标
- `src/metrics/domain/search.rs` — 搜索指标
- `src/metrics/domain/storage.rs` — 存储指标
- `src/metrics/domain/orchestrator.rs` — 编排器指标
- `src/metrics/domain/runtime.rs` — 运行时指标
- `src/metrics/domain/system.rs` — 系统指标

### 现有模块（迁移/清理）

- `src/core/stats/global_metrics.rs` — 逐步废弃
- `src/sync/metrics.rs` — 迁移到领域层
- `src/search/metrics.rs` — 迁移到领域层
- `src/api/core/telemetry.rs` — 作为底层存储保留
- `src/api/server/telemetry_server.rs` — 保留并扩展

### 配置

- `src/config/global.rs` — MetricsConfig 扩展
