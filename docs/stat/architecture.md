# GraphDB 统计、监控与遥测体系架构

## 执行摘要

本文档详细描述了 GraphDB 项目的统计、监控和遥测体系架构，包括各模块的职责、相互关系、指标收集流程和数据流。

**架构特点**：

- 🎯 **三层架构**：统计层、监控层、遥测层
- 🔧 **统一指标收集**：基于 `metrics` crate 的 Prometheus 风格指标
- 📊 **多样化输出**：支持 JSON、Plain Text 等多种格式
- ⚡ **高性能**：使用原子操作和 DashMap 减少锁竞争
- 🔄 **模块化设计**：各模块职责清晰，松耦合

---

## 一、整体架构

### 1.1 三层架构

```
┌─────────────────────────────────────────────────────────┐
│                    应用层 (Application)                  │
├─────────────────────────────────────────────────────────┤
│  查询管理 | 执行器 | 存储引擎 | 同步系统 | 搜索引擎     │
├─────────────────────────────────────────────────────────┤
│                  监控层 (Monitoring)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ GlobalMetrics│  │  Telemetry  │  │  CacheStats │     │
│  │  (全局指标) │  │  (遥测系统) │  │  (缓存统计) │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                  统计层 (Statistics)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ StatsManager│  │ QueryProfile│  │ErrorStats   │     │
│  │ (统计管理) │  │ (查询画像)  │  │ (错误统计)  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

### 1.2 模块分层

| 层级       | 模块             | 职责                                  | 指标类型                  |
| ---------- | ---------------- | ------------------------------------- | ------------------------- |
| **统计层** | `core::stats`    | 查询统计、错误统计、统计管理          | 计数器、直方图            |
| **监控层** | `api::telemetry` | 指标收集、暴露、HTTP 接口             | Counter、Gauge、Histogram |
| **业务层** | 各业务模块       | 业务指标记录（Sync、Search、Storage） | 领域特定指标              |

---

## 二、核心模块详解

### 2.1 统计层 (Statistics Layer)

#### 2.1.1 `core::stats` 模块结构

```
src/core/stats/
├── mod.rs              # 模块入口和导出
├── manager.rs          # StatsManager - 统一统计管理器
├── metrics.rs          # QueryMetrics - 轻量级查询指标
├── profile.rs          # QueryProfile - 详细查询画像
├── error_stats.rs      # ErrorStatsManager - 错误统计
├── latency_histogram.rs# LatencyHistogram - 延迟直方图
├── global_metrics.rs   # GlobalMetrics - 全局指标
└── utils.rs            # 工具函数和 CacheStats
```

#### 2.1.2 核心结构体

**QueryMetrics（轻量级查询指标）**

- **用途**：返回给客户端的查询指标
- **精度**：微秒 (us)
- **内容**：
  - `total_queries`: 总查询数
  - `running_queries`: 运行中的查询数
  - `failed_queries`: 失败的查询数
  - `avg_duration_ms`: 平均执行时间 (ms)
  - `slow_queries`: 慢查询列表

**QueryProfile（详细查询画像）**

- **用途**：内部性能分析和问题诊断
- **精度**：毫秒 (ms)
- **内容**：
  - 执行时间分解（规划时间、执行时间）
  - 执行器统计信息
  - 错误信息
  - 慢查询日志

**StatsManager（统计管理器）**

- **职责**：
  - 管理全局统计指标
  - 维护慢查询日志
  - 提供统计查询接口
  - 管理会话统计

**ErrorStatsManager（错误统计管理器）**

- **职责**：
  - 记录错误信息
  - 按查询阶段分类统计
  - 提供错误汇总和分析

#### 2.1.3 GlobalMetrics（全局指标）

使用 `metrics` crate 实现的 Prometheus 风格指标：

**查询指标**：

- `graphdb_query_total`: 总查询数
- `graphdb_query_duration_seconds`: 查询延迟直方图
- `graphdb_query_active`: 活跃查询数
- `graphdb_query_*_total`: 按查询类型分类（MATCH, CREATE, UPDATE, DELETE 等）

**存储指标**：

- `graphdb_storage_scan_total`: 存储扫描次数
- `graphdb_storage_scan_duration_seconds`: 扫描延迟
- `graphdb_storage_cache_hits_total`: 缓存命中数
- `graphdb_storage_cache_misses_total`: 缓存未命中数

**执行器指标**：

- `graphdb_executor_rows_processed_total`: 处理的行数
- `graphdb_executor_memory_used_bytes`: 内存使用量

**错误指标**：

- `graphdb_error_total`: 错误总数
- `graphdb_error_by_type_total{type}`: 按类型分类的错误数

---

### 2.2 监控层 (Monitoring Layer)

#### 2.2.1 Telemetry 系统

**位置**：`src/api/telemetry/`

**核心组件**：

```
src/api/telemetry/
├── mod.rs           # 主模块，包含 Recorder 实现
├── server.rs        # HTTP 服务器（可选）
├── embedded.rs      # 嵌入式 API
└── c_api.rs         # C API（可选）
```

**MetricsStore**：

- 使用 `DashMap` 存储指标，最小化锁竞争
- 支持三种指标类型：
  - `Counters`: 单调递增计数器
  - `Gauges`: 可增减的仪表
  - `Histograms`: 直方图（支持百分位数计算）

**TelemetryRecorder**：

- 实现 `metrics::Recorder` trait
- 作为 `metrics` crate 的全局记录器
- 将指标记录到 `MetricsStore`

**指标输出格式**：

1. **JSON 格式**：结构化数据，适合程序化处理
2. **Plain Text 格式**：Prometheus 兼容格式
3. **过滤支持**：按前缀过滤指标

#### 2.2.2 指标数据流

```
代码调用 metrics::counter!("name").increment(1)
         ↓
TelemetryRecorder::register_counter()
         ↓
MetricsStore.counters.insert()
         ↓
HTTP GET /metrics
         ↓
MetricsStore::snapshot()
         ↓
MetricsSnapshot::to_text_format() / to_json()
```

---

### 2.3 业务层指标 (Business Metrics)

#### 2.3.1 SyncMetrics（同步系统指标）

**位置**：`src/sync/metrics.rs`

**职责**：记录事务和索引同步的指标

**指标**：

- 事务提交/回滚数
- 索引操作数（插入/更新/删除）
- 重试尝试/成功/失败数
- 死信队列大小
- 活跃事务数
- 补偿操作统计

**实现方式**：

- 使用 `metrics` crate 直接记录
- 使用 `CacheStats` 记录缓存统计
- 无内部计数器（避免双重计数）

#### 2.3.2 FulltextMetrics（全文搜索指标）

**位置**：`src/search/metrics.rs`

**职责**：记录全文搜索的指标

**指标**：

- 索引操作数
- 搜索操作数
- 搜索延迟
- 索引文档数
- 队列大小
- 缓存命中率

**实现方式**：

- 使用 `metrics` crate 直接记录
- 使用 `CacheStats` 记录缓存统计

#### 2.3.3 StorageMetricsCollector（存储指标收集器）

**位置**：`src/storage/monitoring/storage_metrics.rs`

**职责**：记录存储引擎的指标

**指标**：

- 扫描项目数
- 返回项目数
- 缓存命中率
- 操作类型统计

**实现方式**：

- 使用原子计数器（AtomicU64）
- 使用 `CacheStats` 记录缓存统计
- 使用 `DashMap` 存储操作类型计数

---

### 2.4 缓存统计 (Cache Statistics)

#### 2.4.1 CacheStats

**位置**：`src/core/stats/utils.rs`

**用途**：统一的缓存统计实现

**功能**：

- 记录命中/未命中次数
- 计算命中率
- 支持批量记录
- 支持重置

**使用场景**：

- `StorageMetricsCollector`
- `SyncMetrics`
- `FulltextMetrics`
- `PlanCache`
- `CteCache`

---

## 三、指标收集流程

### 3.1 查询执行流程中的指标收集

```
1. 查询开始
   ↓
   StatsManager::record_query_start()
   - 增加活跃查询数
   - 记录开始时间
   ↓
2. 查询解析和规划
   ↓
   GlobalMetrics::record_query_type("MATCH")
   - 记录查询类型
   ↓
3. 查询执行
   ↓
   ExecutorStats::add_exec_time()
   - 记录执行器执行时间
   ↓
   StorageMetricsCollector::record_scan()
   - 记录存储扫描
   ↓
   StorageMetricsCollector::record_cache_hit/miss()
   - 记录缓存命中情况
   ↓
4. 查询完成
   ↓
   GlobalMetrics::record_query(duration)
   - 记录总执行时间
   ↓
   StatsManager::add_slow_query()
   - 如果是慢查询，加入慢查询日志
   ↓
5. 返回结果
   ↓
   QueryMetrics::new()
   - 生成查询指标返回给客户端
```

### 3.2 指标暴露流程

```
1. 外部请求 /metrics 端点
   ↓
2. Telemetry 系统处理请求
   ↓
3. MetricsStore::snapshot()
   - 收集所有计数器
   - 收集所有仪表
   - 收集所有直方图（计算百分位数）
   ↓
4. 格式化输出
   - JSON 格式：适合程序化处理
   - Plain Text 格式：Prometheus 兼容
   ↓
5. 返回给客户端
```

---

## 四、数据流图

### 4.1 指标收集数据流

```
┌─────────────────┐
│  业务代码模块   │
│ (Query/Storage) │
└────────┬────────┘
         │ 调用 metrics::counter/gauge/histogram
         ↓
┌─────────────────┐
│ TelemetryRecorder│
│  (metrics crate) │
└────────┬────────┘
         │ 分发到对应的指标类型
         ↓
┌─────────────────────────────────┐
│         MetricsStore            │
│  ┌─────────┐ ┌─────────┐ ┌───┐ │
│  │Counters │ │ Gauges  │ │...│ │
│  │DashMap  │ │DashMap  │ │   │ │
│  └─────────┘ └─────────┘ └───┘ │
└─────────────────────────────────┘
```

### 4.2 统计数据流

```
┌──────────────────┐
│  QueryExecutor   │
└────────┬─────────┘
         │ 记录执行统计
         ↓
┌──────────────────┐
│  ExecutorStats   │
│  - num_rows      │
│  - exec_time_us  │
│  - memory_peak   │
└────────┬─────────┘
         │ 汇总到查询画像
         ↓
┌──────────────────┐
│   QueryProfile   │
│  - planning_time │
│  - execution_time│
│  - executor_stats│
└────────┬─────────┘
         │ 提交到统计管理器
         ↓
┌──────────────────┐
│   StatsManager   │
│  - 慢查询日志    │
│  - 全局统计      │
└──────────────────┘
```

---

## 五、关键设计决策

### 5.1 为什么使用 metrics crate？

**优点**：

1. **标准化**：Prometheus 风格的指标命名和类型
2. **生态系统**：与 Rust 生态系统无缝集成
3. **灵活性**：支持标签、直方图、百分位数
4. **零开销**：指标存储在 recorder 中，无额外内存占用
5. **可扩展**：易于集成到各种监控系统

### 5.2 为什么保留 Atomic 计数器？

**使用场景**：

- `StorageMetricsCollector`：需要快速获取快照
- `CacheStats`：需要原子操作的缓存统计
- `PlanCache`：需要线程安全的缓存统计

**原因**：

- 低开销：原子操作比锁更轻量
- 简单直接：适合简单的计数需求
- 无需全局注册：独立于 `metrics` crate

### 5.3 为什么移除内部计数器？

**迁移前的问题**：

- 双重计数：每个指标记录 2 次
- 内存浪费：每个实例额外占用 30-50 bytes
- 代码复杂性：需要维护两套逻辑

**迁移后的优势**：

- 统一记录：只使用 `metrics` crate
- 减少开销：消除原子操作的重复
- 简化代码：更容易维护和测试

---

## 六、性能考虑

### 6.1 锁优化

**使用 DashMap**：

- `MetricsStore` 使用 `DashMap` 而非 `RwLock<HashMap>`
- 减少锁竞争，提高并发性能
- 适合读多写少的场景

### 6.2 原子操作

**使用 AtomicU64**：

- `CacheStats` 使用原子操作而非锁
- `Ordering::Relaxed`：最弱的内存序，性能最优
- 适合统计类应用（不要求严格的顺序）

### 6.3 内存控制

**直方图清理**：

- 定期清理旧的直方图数据
- 防止内存无限增长
- 保留最近的统计数据

---

## 七、扩展性

### 7.1 添加新指标

**步骤**：

1. 在代码中调用 `metrics::counter/gauge/histogram!("name")`
2. 指标自动注册到 `TelemetryRecorder`
3. 通过 `/metrics` 端点自动暴露

**示例**：

```rust
// 记录查询数
metrics::counter!("graphdb_custom_query_total").increment(1);

// 记录延迟
metrics::histogram!("graphdb_custom_duration_seconds")
    .record(duration.as_secs_f64());

// 记录活跃连接数
metrics::gauge!("graphdb_active_connections").increment(1.0);
```

### 7.2 集成外部监控系统

**Prometheus**：

- 使用 Plain Text 格式
- 配置 Prometheus 抓取 `/metrics` 端点

**Grafana**：

- 使用 Prometheus 作为数据源
- 创建仪表板展示指标

**自定义监控**：

- 使用 JSON 格式
- 自行解析和处理指标数据

---

## 八、最佳实践

### 8.1 指标命名规范

**格式**：`graphdb_<模块>_<操作>_<类型>`

**示例**：

- `graphdb_query_total`
- `graphdb_storage_scan_duration_seconds`
- `graphdb_executor_rows_processed_total`

**类型后缀**：

- `_total`: 计数器（单调递增）
- `_seconds`: 时间（秒）
- `_bytes`: 大小（字节）
- 无前缀：仪表（可增减）

### 8.2 标签使用

**推荐标签**：

- `type`: 类型分类（如查询类型）
- `status`: 状态（success/failed）
- `operation`: 操作类型

**示例**：

```rust
metrics::counter!("graphdb_error_by_type_total", "type" => error_type)
```

### 8.3 直方图桶选择

**时间类指标**：

- 使用秒为单位
- 桶：0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0

**大小类指标**：

- 使用字节为单位
- 桶：100, 1000, 10000, 100000, 1000000

---

## 九、故障排查

### 9.1 指标未记录

**检查清单**：

1. 确认 `TelemetryRecorder` 已设置为全局 recorder
2. 确认指标名称正确
3. 确认调用了 `increment()` 或 `record()` 方法

### 9.2 内存占用过高

**可能原因**：

- 直方图数据过多
- 标签组合过多（基数爆炸）

**解决方案**：

- 启用直方图清理
- 减少标签的使用
- 限制标签的取值范围

### 9.3 性能下降

**可能原因**：

- 指标记录过于频繁
- DashMap 冲突过多

**解决方案**：

- 批量记录指标
- 优化指标名称分布

---

## 十、未来改进方向

### 10.1 短期改进

- [ ] 添加指标文档自动生成
- [ ] 实现指标异常检测
- [ ] 优化直方图内存占用

### 10.2 中期改进

- [ ] 集成分布式追踪（OpenTelemetry）
- [ ] 实现动态指标采样
- [ ] 添加指标告警功能

### 10.3 长期改进

- [ ] 支持流式指标导出
- [ ] 实现指标压缩和归档
- [ ] 集成机器学习进行异常预测

---

## 附录 A：模块文件清单

### A.1 统计层

| 文件                                  | 行数 | 职责       |
| ------------------------------------- | ---- | ---------- |
| `src/core/stats/mod.rs`               | ~50  | 模块入口   |
| `src/core/stats/manager.rs`           | ~300 | 统计管理   |
| `src/core/stats/metrics.rs`           | ~100 | 查询指标   |
| `src/core/stats/profile.rs`           | ~200 | 查询画像   |
| `src/core/stats/error_stats.rs`       | ~150 | 错误统计   |
| `src/core/stats/latency_histogram.rs` | ~100 | 延迟直方图 |
| `src/core/stats/global_metrics.rs`    | ~200 | 全局指标   |
| `src/core/stats/utils.rs`             | ~150 | 工具函数   |

### A.2 监控层

| 文件                            | 行数 | 职责        |
| ------------------------------- | ---- | ----------- |
| `src/api/telemetry/mod.rs`      | ~400 | 遥测核心    |
| `src/api/telemetry/server.rs`   | -    | HTTP 服务器 |
| `src/api/telemetry/embedded.rs` | -    | 嵌入式 API  |

### A.3 业务层

| 文件                                        | 行数 | 职责     |
| ------------------------------------------- | ---- | -------- |
| `src/sync/metrics.rs`                       | ~140 | 同步指标 |
| `src/search/metrics.rs`                     | ~110 | 搜索指标 |
| `src/storage/monitoring/storage_metrics.rs` | ~160 | 存储指标 |

### A.4 缓存统计

| 文件                                | 行数 | 职责         |
| ----------------------------------- | ---- | ------------ |
| `src/query/cache/plan_cache.rs`     | ~600 | 计划缓存     |
| `src/query/cache/cte_cache.rs`      | ~800 | CTE 缓存     |
| `src/query/cache/global_manager.rs` | ~400 | 全局缓存管理 |

---

## 附录 B：指标类型说明

### B.1 Counter（计数器）

**特点**：

- 单调递增
- 只能增加，不能减少
- 重启后归零

**用途**：

- 请求总数
- 错误总数
- 处理项目数

### B.2 Gauge（仪表）

**特点**：

- 可增可减
- 表示当前状态
- 重启后归零

**用途**：

- 活跃连接数
- 队列长度
- 内存使用量

### B.3 Histogram（直方图）

**特点**：

- 记录分布
- 支持百分位数
- 可计算总和和平均值

**用途**：

- 请求延迟
- 响应大小
- 处理时间

---

**文档版本**：1.0  
**最后更新**：2026-04-15  
**维护者**：GraphDB Team
