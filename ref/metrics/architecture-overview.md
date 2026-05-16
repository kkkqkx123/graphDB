# Metrics 系统架构设计

## 1. 整体架构

Metrics 系统采用**三层架构**，自底向上分为基础设施层、领域层和导出层，各层职责明确、依赖单向。

```
┌─────────────────────────────────────────────────────────────┐
│                      导出层 (Export Layer)                    │
│  JSON Exporter │ Prometheus Exporter │ File Writer │ SQLite │
├─────────────────────────────────────────────────────────────┤
│                      领域层 (Domain Layer)                    │
│  Pipeline │ Search │ Storage │ Orchestrator │ Runtime │ System│
├─────────────────────────────────────────────────────────────┤
│                    基础设施层 (Infrastructure Layer)            │
│  Counter │ Gauge │ Histogram │ ProgressTracker │ Labels      │
└─────────────────────────────────────────────────────────────┘
```

**设计原则：**

- **无外部 metrics 依赖**：所有指标类型基于 `Arc<AtomicU64>` 手写实现，无 prometheus/metrics 等第三方 crate
- **分层隔离**：领域层封装原始指标为语义化结构体，业务代码不直接操作原始 Counter/Gauge
- **RAII 定时**：通过 `OperationTimer` 在 Drop 时自动记录操作耗时
- **基数保护**：Label Key 通过白名单验证，防止标签爆炸

---

## 2. 基础设施层

### 2.1 核心类型

所有指标类型均基于 `Arc<AtomicU64>` 实现线程安全：

| 类型 | 用途 | 核心操作 |
|------|------|----------|
| `Counter` | 单调递增计数器 | `inc()`, `increment_by(n)`, `get()` |
| `Gauge` | 可升降整数值 | `set(v)`, `get()` |
| `Histogram` | 延迟分布/百分位 | `observe(ms)`, `p50/p90/p95/p99()`, `get_buckets()` |
| `LabeledCounter` | 带标签的 Counter | `increment()`, `get()`, `labels()` |
| `LabeledGauge` | 带标签的 Gauge | `set(v)`, `get()`, `labels()` |
| `LabeledFloatGauge` | 带标签的浮点 Gauge | `set(v)`(bit-cast), `get()`, `labels()` |
| `LabeledHistogram` | 带标签的 Histogram | `observe(ms)`, `get_count()`, `labels()` |

### 2.2 标签系统

- `Labels`：基于 `BTreeMap<String, String>`，保证序列化顺序确定性
- `MetricKey(name: String, labels: Labels)`：注册表中的复合键
- 允许的标签键白名单：`["operation", "component", "status", "project_id", "language", "provider"]`

### 2.3 枚举类型标签

预定义枚举用于静态标签，防止拼写错误：

| 枚举 | 取值 |
|------|------|
| `OperationType` | Indexing, Querying, Embedding, Parsing, RelationExtraction, SummaryGeneration |
| `Component` | Scanner, Parser, VectorStore, SqliteStore, Api, Orchestrator |
| `Language` | Rust, TypeScript, JavaScript, Python, Go, Java, Unknown |
| `Provider` | OpenAI, HuggingFace, Local, Ollama |
| `ProgressPhase` | Scanning, Parsing, Embedding, RelationBuilding, SummaryGeneration |

### 2.4 核心文件

| 文件 | 职责 |
|------|------|
| `src/metrics/types.rs` | 核心类型定义 + 枚举标签 |
| `src/metrics/labels.rs` | 标签系统：Label, Labels, MetricKey |
| `src/metrics/progress.rs` | 进度追踪：ProgressTracker, ProgressSnapshot |
| `src/metrics/mod.rs` | 注册表：MetricsRegistry, MetricsSnapshot, HistogramStats |

---

## 3. 注册表 (MetricsRegistry)

### 3.1 数据结构

全局单例注册表，持有所有指标的运行时数据：

```rust
pub struct MetricsRegistry {
    counters: RwLock<HashMap<MetricKey, LabeledCounter>>,
    gauges: RwLock<HashMap<MetricKey, LabeledGauge>>,
    float_gauges: RwLock<HashMap<MetricKey, LabeledFloatGauge>>,
    histograms: RwLock<HashMap<MetricKey, LabeledHistogram>>,
}
```

### 3.2 核心方法

- `counter(name, &[("key","val")])` / `gauge(...)` / `float_gauge(...)` / `histogram(...)`：带标签数组的便捷方法，自动转换
- `counter_simple(name)` / `gauge_simple(name)` / `histogram_default_simple(name)`：无标签快捷方法
- `*_for_operation(name, op)` / `*_for_component(name, comp)`：预定义标签的便捷方法
- `export_all() -> MetricsSnapshot`：导出全量指标快照
- `get_all_counters_with_keys()` / `get_all_histograms_with_keys()`：带键遍历方法

---

## 4. 领域层

### 4.1 流水线指标 (Pipeline)

| 结构体 | 指标 | 文件 |
|--------|------|------|
| `EmbeddingMetrics` | requests_total, tokens_total, errors_total, latency_ms | `domain/pipeline.rs` |
| `ParserMetrics` | parse_attempts_total, parse_errors_total, parse_latency_ms | `domain/pipeline.rs` |
| `RelationMetrics` | relations_extracted_total, build_latency_ms, extraction_coverage_rate | `domain/pipeline.rs` |
| `SummaryMetrics` | summaries_generated_total, generation_latency_ms, avg_summary_length | `domain/pipeline.rs` |

### 4.2 搜索指标 (Search)

| 结构体 | 指标 | 文件 |
|--------|------|------|
| `SearchMetrics` | queries_total, query_latency_ms, index_size, index_operations_total | `domain/search.rs` |

### 4.3 存储指标 (Storage)

| 结构体 | 指标 | 文件 |
|--------|------|------|
| `Bm25Metrics` | documents_indexed_total, index_latency_ms, search_queries_total, search_latency_ms, documents_deleted_total, delete_latency_ms, index_size, errors_total | `domain/storage.rs` |
| `QdrantMetrics` | vectors_upserted_total, upsert_latency_ms, search_queries_total, search_latency_ms, vectors_deleted_total, delete_latency_ms, vector_count, collection_size_bytes, errors_total, active_connections | `domain/storage.rs` |
| `SqliteMetrics` | inserts_total, insert_latency_ms, queries_total, query_latency_ms, updates_total, update_latency_ms, deletes_total, delete_latency_ms, database_size_bytes, transactions_total, transaction_latency_ms, errors_total | `domain/storage.rs` |

### 4.4 编排器指标 (Orchestrator)

| 结构体 | 指标 | 文件 |
|--------|------|------|
| `QueryMetrics` | queries_executed_total, query_execution_latency_ms, cache_hits_total, cache_misses_total, cache_hit_rate, results_returned_total | `domain/orchestrator.rs` |
| `HotUpdateMetrics` | hot_update_cycles_total, hot_update_latency_ms, files_changed_total, files_processed_in_hot_update, files_failed_in_hot_update, entity_changes_detected | `domain/orchestrator.rs` |
| `IndexMetrics` | index_{op}_duration_ms, index_{op}_total, file_processing_duration_ms, files_processed_total, entities_indexed_total, batch_processing_duration_ms, batch_files_processed_total | `orchestrator/metrics.rs` |
| `StorageMetrics` | qdrant_upsert_duration_ms, bm25_index_duration_ms, sqlite_transaction_duration_ms, chunk_storage_duration_ms, entity_mapping_storage_duration_ms | `orchestrator/storage_metrics.rs` |

### 4.5 运行时与系统指标 (Runtime & System)

此类指标为**瞬时指标**，不持久化到 SQLite，仅内存中实时刷新：

| 结构体 | 数据来源 | 指标 |
|--------|----------|------|
| `RuntimeMetrics` | Tokio runtime | tokio_workers_total, tokio_active_tasks, tokio_worker_busy_duration_ms, tokio_worker_queue_depth |
| `SystemMetrics` | sysinfo crate | system_cpu_usage_percent, system_memory_used_bytes, system_memory_total_bytes, system_swap_used_bytes, system_disk_used_bytes |

### 4.6 领域层文件布局

```
src/metrics/domain/
├── mod.rs           # 模块入口，统一 re-export
├── pipeline.rs      # 流水线相关指标
├── search.rs        # 搜索相关指标
├── storage.rs       # 存储后端指标
├── orchestrator.rs  # 编排器指标
├── runtime.rs       # Tokio 运行时指标
└── system.rs        # 系统资源指标
```

---

## 5. 导出层

### 5.1 导出机制

五种导出/输出方式：

| 方式 | 端点/目标 | 格式 | 说明 |
|------|-----------|------|------|
| 系统健康摘要 | `GET /api/metrics` | JSON | 子系统状态汇总 |
| 全量指标快照 | `GET /api/metrics/json` | JSON | register.export_all() 输出 |
| 历史聚合数据 | `GET /api/metrics/history` | JSON | 从 SQLite 查询 |
| 数据清理 | `DELETE /api/metrics/cleanup` | - | 删除 SQLite 历史数据 |
| 文件日志 | `metrics.log` | JSON Lines | 离线分析用 |
| Prometheus | 可通过扩展接入 | Prometheus 格式 | 内置 PrometheusExporter |

### 5.2 Export Trait

```rust
#[async_trait]
pub trait MetricExporter: Send + Sync {
    async fn export(&self, registry: &MetricsRegistry) -> Result<String, ExportError>;
    fn name(&self) -> &'static str;
}
```

当前实现：
- `JsonExporter`：全量指标 JSON 序列化
- `PrometheusExporter`：Prometheus 文本格式输出（含 `_bucket`/`_sum`/`_count`）

### 5.3 文件日志 (Writer)

`writer.rs` 提供全局单例的文件写入器，基于 `OnceLock<Mutex<BufWriter<File>>>`，每个进程仅一个 metrics.log 文件。通过 `log_metric_to_file(name, value, labels)` 记录任意指标。

### 5.4 核心文件

| 文件 | 职责 |
|------|------|
| `src/metrics/exporter.rs` | MetricExporter trait + JsonExporter/PrometheusExporter/ExporterManager |
| `src/metrics/serialization.rs` | MetricValue, MetricData, MetricsSnapshot (API 响应结构) |
| `src/metrics/writer.rs` | 文件日志写入器 |
| `src/metrics/logger.rs` | 便捷日志函数：log_metric, log_counter, log_histogram |

---

## 6. 聚合层 (MetricsAggregator)

### 6.1 职责

后台周期性任务，从注册表读取 Histogram 统计数据，聚合后写入 SQLite `metrics_aggregated` 表，提供历史查询能力。

### 6.2 配置

```rust
pub struct MetricsAggregationConfig {
    pub enabled: bool,
    pub interval_secs: u64,  // 默认 300s (5分钟)
}
```

### 6.3 数据结构

```rust
pub struct AggregatedMetric {
    pub timestamp: DateTime<Utc>,
    pub metric_name: String,
    pub labels_json: String,
    pub count: u64,
    pub avg: f64,
    pub median: f64,
    pub max: f64,
    pub p90: f64,
    pub p99: f64,
    pub project_id: Option<String>,
    pub operation_type: Option<String>,
}
```

### 6.4 核心文件

| 文件 | 职责 |
|------|------|
| `src/metrics/aggregator.rs` | MetricsAggregator 实现 |

---

## 7. 数据流

```
业务代码 (Embedder/Storage/Parser/Query)
    │
    ├─► 领域指标 (record_* 方法) ──► MetricsRegistry (内存)
    │                                      │
    │                                      ├─► export_all() ──► /api/metrics/json (快照)
    │                                      ├─► PrometheusExporter ──► Prometheus 格式
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

## 8. 集成点

### 8.1 启动流程 (engine.rs EngineBuilder::build)

1. `MetricsRegistry::new()` — 创建全局注册表
2. `ProgressTracker::new(0)` — 创建进度追踪器
3. `EmbeddingMetrics::new(&registry, &model_name)` — 创建嵌入指标
4. `embedder.with_metrics(embedding_metrics)` — 注入到嵌入器
5. 条件创建 `MetricsAggregator::new(sqlite_client, registry, config)` — 聚合引擎
6. `RuntimeMetrics::new(&registry)` / `SystemMetrics::new(&registry)` — 运行时/系统指标
7. 启动后台任务：`start_metrics_aggregation()` / `start_runtime_metrics_collection()` / `start_system_metrics_collection()`

### 8.2 各存储后端集成模式

每个后端 (BM25/Qdrant/SQLite/Parser/Query/HotUpdate) 遵循统一模式：

```rust
pub struct SomeBackend {
    metrics: Option<Arc<DomainMetrics>>,  // 可选指标
    // ... 其他字段
}

impl SomeBackend {
    pub fn with_metrics(mut self, metrics: Arc<DomainMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
}
```

调用点统一调用 `metrics.record_*(...)`：

| 后端 | 调用时机 |
|------|----------|
| Embedder | 每次嵌入请求完成 (record_request) |
| Parser | 每次解析成功/失败 (record_parse) |
| BM25 | 索引/搜索/删除操作完成 |
| Qdrant | Upsert/Search/Delete 操作完成 |
| SQLite | Insert/Query/Update/Delete/Transaction 操作完成 |
| Query | Query/Cache hit or miss |
| HotUpdate | 每次热更新周期完成 |
| Index | 批量处理完成/文件处理完成 |

---

## 9. 路由注册 (API)

在 `src/api/router.rs` 中注册：

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/api/metrics` | `handle_get_metrics` | 系统健康摘要 |
| GET | `/api/metrics/json` | `handle_get_metrics_json` | 全量指标快照 |
| GET | `/api/metrics/history` | `handle_get_metrics_history` | 历史聚合数据 |
| DELETE | `/api/metrics/cleanup` | `handle_cleanup_metrics` | 删除历史数据 |

---

## 10. 配置

```rust
// config/global.rs
pub struct MetricsConfig {
    pub aggregation: MetricsAggregationConfig,
}

pub struct MetricsAggregationConfig {
    pub enabled: bool,        // 默认 false
    pub interval_secs: u64,   // 默认 300
}
```

---

## 11. 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 指标库选型 | 无外部依赖，手写实现 | 减少依赖树，完全控制语义和性能 |
| 线程安全模型 | `Arc<AtomicU64>` + `RwLock<HashMap>` | 读写分离，高频写入时无锁竞争 |
| 标签系统 | 白名单 + 预定义枚举 | 防止标签爆炸导致内存泄露 |
| 持久化策略 | 仅 Histogram 聚合后入 SQLite | 减少写入量，聚焦有分析价值的指标 |
| 运行时/系统指标 | 仅内存，不持久化 | 瞬时值无需历史，降低存储开销 |
| 进度追踪 | 独立于 Registry | 关注工作流状态而非业务 KPI |
| 文件写入器 | 全局单例 OnceLock | 简化使用，避免多文件竞争 |

---

## 12. 文件清单

### 核心模块
- `src/metrics/mod.rs` — 模块根，MetricsRegistry, MetricsSnapshot, HistogramStats
- `src/metrics/types.rs` — 核心类型 + 枚举标签
- `src/metrics/labels.rs` — 标签系统
- `src/metrics/progress.rs` — 进度追踪
- `src/metrics/aggregator.rs` — 指标聚合
- `src/metrics/exporter.rs` — 导出器
- `src/metrics/serialization.rs` — 序列化结构
- `src/metrics/writer.rs` — 文件日志写入
- `src/metrics/logger.rs` — 便捷日志函数

### 领域模块
- `src/metrics/domain/mod.rs` — 模块入口
- `src/metrics/domain/pipeline.rs` — 流水线指标
- `src/metrics/domain/search.rs` — 搜索指标
- `src/metrics/domain/storage.rs` — 存储指标
- `src/metrics/domain/orchestrator.rs` — 编排器指标
- `src/metrics/domain/runtime.rs` — 运行时指标
- `src/metrics/domain/system.rs` — 系统指标

### 编排器层
- `src/orchestrator/metrics.rs` — IndexMetrics + OperationTimer
- `src/orchestrator/storage_metrics.rs` — StorageMetrics + StorageOperationTimer

### API
- `src/api/handlers/metrics.rs` — API 端点处理器
- `src/api/router.rs` — 路由注册

### CLI
- `cce-cli/src/commands/metrics.rs` — CLI 指标命令

### 前端
- `frontend/src/lib/api/metrics.ts` — API 客户端
- `frontend/src/lib/stores/metrics.ts` — Svelte Store

### 测试
- `tests/integration_metrics.rs` — 集成测试
- `tests/index_orchestrator/metrics.rs` — 编排器测试
- `tests/common/metrics_helpers.rs` — 测试辅助

### 配置
- `src/config/global.rs` — MetricsConfig 定义
