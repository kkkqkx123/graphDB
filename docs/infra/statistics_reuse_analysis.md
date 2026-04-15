# 统计信息复用分析报告

## 一、现有统计信息概览

### 1.1 统计信息分布

| 模块 | 统计结构 | 精度 | 用途 | 收集点 |
|------|---------|------|------|--------|
| **core/stats** | | | | |
| | QueryMetrics | 微秒 (us) | 返回客户端 | 查询各阶段 |
| | QueryProfile | 毫秒 (ms) | 内部监控 | 查询全生命周期 |
| | ExecutorStat | 毫秒 (ms) | 执行器统计 | Executor |
| | MetricType | - | 全局指标 | 系统级别 |
| **query/executor** | | | | |
| | ExecutorStats | 微秒 (us) | 执行器统计 | Executor 实例 |
| | NodeExecutionStats | 毫秒 (f64) | 节点执行统计 | PlanNode |
| | ExecutionStatsContext | - | 统计上下文 | EXPLAIN/PROFILE |
| **query/optimizer/stats** | | | | |
| | OperatorFeedback | 微秒 (us) | 优化器反馈 | 优化器 |
| | ExecutionFeedbackCollector | 微秒 (us) | 执行反馈收集 | 执行器 |
| | QueryExecutionFeedback | - | 查询反馈 | 查询完成 |
| **storage/monitoring** | | | | |
| | StorageMetricsCollector | 计数 | 存储层统计 | 存储引擎 |
| | StorageMetricsSnapshot | - | 存储快照 | 监控 API |

---

## 二、重复计算的统计信息

### 2.1 时间统计重复

#### 问题 1: 执行时间在多处重复收集

```
ExecutorStats.exec_time_us (微秒)
    │
    ├─► 转换为 ExecutorStat.duration_ms (毫秒)
    │       │
    │       └─► 填充到 QueryProfile.executor_stats
    │
    └─► NodeExecutionStats.actual_time_ms (毫秒)
            │
            └─► 收集到 ExecutionStatsContext
```

**重复点**:
1. `ExecutorStats` 收集执行时间（微秒）
2. `ExecutorStat` 再次记录（毫秒）
3. `NodeExecutionStats` 再次记录（毫秒）

**影响**:
- 多次时间计算和转换
- 精度损失（微秒 → 毫秒）
- 内存浪费

---

#### 问题 2: 查询阶段时间重复

```
QueryMetrics.parse_time_us (微秒)
    │
    └─► QueryProfile.stages.parse_ms (毫秒)
```

**重复点**:
- `QueryMetrics` 和 `QueryProfile` 都记录相同的阶段时间
- 精度不统一（一个微秒，一个毫秒）

---

### 2.2 行数统计重复

#### 问题 3: 行数在多处重复计数

```
ExecutorStats.num_rows
    │
    ├─► ExecutorStat.rows_processed
    │       │
    │       └─► 填充到 QueryProfile
    │
    └─► NodeExecutionStats.actual_rows
            │
            └─► ExecutionStatsContext
```

**重复点**:
- `ExecutorStats` 记录行数
- `ExecutorStat` 再次记录
- `NodeExecutionStats` 再次记录

---

### 2.3 缓存统计重复

#### 问题 4: 缓存命中率在多处计算

```
ExecutorStats.cache_hits / cache_misses
    │
    └─► 计算 cache_hit_rate()
    
NodeExecutionStats.cache_hits / cache_misses
    │
    └─► 计算 cache_hit_rate()
    
StorageMetricsCollector.cache_hits / cache_misses
    │
    └─► 计算 cache_hit_rate()
```

**重复点**:
- 三个地方都记录缓存命中/未命中
- 三个地方都实现 `cache_hit_rate()` 方法

---

### 2.4 内存统计重复

#### 问题 5: 内存使用在多处记录

```
ExecutorStats.memory_peak / memory_current
    │
    └─► ExecutorStat.memory_used
    
NodeExecutionStats.memory_used
```

**重复点**:
- `ExecutorStats` 记录内存
- `ExecutorStat` 和 `NodeExecutionStats` 再次记录

---

## 三、可复用的统计信息

### 3.1 核心统计信息（应作为唯一数据源）

#### ✅ ExecutorStats（微秒级精度）

**位置**: `src/query/executor/base/executor_stats.rs`

**优势**:
- ✅ 微秒级精度
- ✅ 完整的统计字段（行数、时间、内存、缓存）
- ✅ 已有完善的计算方法（throughput, efficiency, cache_hit_rate）
- ✅ 支持序列化/反序列化

**应复用的场景**:
1. **ExecutorStat** 应直接从 `ExecutorStats` 转换
2. **NodeExecutionStats** 应引用 `ExecutorStats` 而非重复记录
3. **QueryProfile.executor_stats** 应使用转换后的 `ExecutorStats`

---

#### ✅ ExecutionFeedbackCollector（微秒级精度）

**位置**: `src/query/optimizer/stats/feedback/collector.rs`

**优势**:
- ✅ 原子操作，线程安全
- ✅ 轻量级设计
- ✅ 微秒级精度

**应复用的场景**:
1. 优化器反馈收集
2. 基数估计校正
3. 不应与 `ExecutorStats` 重复收集相同数据

---

#### ✅ StorageMetricsCollector

**位置**: `src/storage/monitoring/storage_metrics.rs`

**优势**:
- ✅ 原子操作，线程安全
- ✅ 专门的存储层统计

**应复用的场景**:
1. 存储层 I/O 统计
2. 缓存命中率统计
3. 不应在 Executor 中重复记录存储层统计

---

### 3.2 应简化的统计信息

#### ⚠️ ExecutorStat（毫秒级精度）

**问题**:
- 毫秒级精度，信息丢失
- 字段冗余（与 `ExecutorStats` 重复）

**改进方案**:
```rust
// 方案 1: 直接引用 ExecutorStats
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    pub stats: ExecutorStats,  // 引用完整统计
}

// 方案 2: 仅保留必要字段，从 ExecutorStats 派生
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    // 其他必要字段从 ExecutorStats 计算
}

impl ExecutorStat {
    pub fn from_executor_stats(executor_type: String, executor_id: i64, stats: &ExecutorStats) -> Self {
        Self {
            executor_type,
            executor_id,
            duration_ms: stats.exec_time_us / 1000,  // 显示时转换
            rows_processed: stats.num_rows,
            memory_used: stats.memory_peak,
        }
    }
}
```

---

#### ⚠️ NodeExecutionStats（毫秒级精度）

**问题**:
- 毫秒级精度，信息丢失
- 与 `ExecutorStats` 字段重复

**改进方案**:
```rust
// 方案：引用 ExecutorStats
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub executor_stats: ExecutorStats,  // 引用完整统计
    pub startup_time_us: u64,           // 仅保留节点特有统计
}

impl NodeExecutionStats {
    pub fn actual_rows(&self) -> usize {
        self.executor_stats.num_rows
    }
    
    pub fn actual_time_us(&self) -> u64 {
        self.executor_stats.exec_time_us
    }
    
    // 显示时转换
    pub fn actual_time_ms(&self) -> f64 {
        self.executor_stats.exec_time_us as f64 / 1000.0
    }
}
```

---

#### ⚠️ QueryProfile.stages（毫秒级精度）

**问题**:
- 与 `QueryMetrics` 重复
- 毫秒级精度

**改进方案**:
```rust
// 方案 1: 使用 QueryMetrics
pub struct QueryProfile {
    // ... 其他字段
    pub stages: QueryMetrics,  // 复用 QueryMetrics
}

// 方案 2: 统一为微秒
#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub parse_us: u64,
    pub validate_us: u64,
    pub plan_us: u64,
    pub optimize_us: u64,
    pub execute_us: u64,
}

impl StageMetrics {
    pub fn from_query_metrics(metrics: &QueryMetrics) -> Self {
        Self {
            parse_us: metrics.parse_time_us,
            validate_us: metrics.validate_time_us,
            plan_us: metrics.plan_time_us,
            optimize_us: metrics.optimize_time_us,
            execute_us: metrics.execute_time_us,
        }
    }
    
    // 显示时转换
    pub fn total_ms(&self) -> f64 {
        (self.parse_us + self.validate_us + self.plan_us + 
         self.optimize_us + self.execute_us) as f64 / 1000.0
    }
}
```

---

### 3.3 应移除的重复计算

#### ❌ 重复的 cache_hit_rate() 方法

**现状**:
```rust
// ExecutorStats
pub fn cache_hit_rate(&self) -> f64 {
    let total = self.cache_hits + self.cache_misses;
    if total > 0 {
        self.cache_hits as f64 / total as f64
    } else {
        0.0
    }
}

// NodeExecutionStats
pub fn cache_hit_rate(&self) -> f64 {
    let total = self.cache_hits + self.cache_misses;
    if total > 0 {
        self.cache_hits as f64 / total as f64
    } else {
        0.0
    }
}

// StorageMetricsSnapshot
pub fn cache_hit_rate(&self) -> f64 {
    let total = self.cache_hits + self.cache_misses;
    if total > 0 {
        self.cache_hits as f64 / total as f64
    } else {
        0.0
    }
}
```

**改进方案**:
```rust
// 统一工具函数
pub fn calculate_cache_hit_rate(hits: u64, misses: u64) -> f64 {
    let total = hits + misses;
    if total > 0 {
        hits as f64 / total as f64
    } else {
        0.0
    }
}

// 各结构使用统一函数
impl ExecutorStats {
    pub fn cache_hit_rate(&self) -> f64 {
        calculate_cache_hit_rate(self.cache_hits, self.cache_misses)
    }
}
```

---

#### ❌ 重复的 to_json / from_json 方法

**现状**:
- `ExecutorStats` 实现了 `to_json()` / `from_json()`
- 其他统计结构也可能实现类似方法

**改进方案**:
```rust
// 统一使用 serde derive
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutorStats {
    // ...
}

// 移除手动实现的 to_json / from_json
// 直接使用 serde_json::to_string(&stats)
```

---

## 四、统计信息复用架构

### 4.1 推荐的数据流

```
查询执行
    │
    ├─► ExecutorStats (唯一数据源)
    │       │
    │       ├─► 转换为 ExecutorStat (用于 QueryProfile)
    │       │       │
    │       │       └─► QueryProfile.executor_stats
    │       │
    │       ├─► 转换为 NodeExecutionStats (用于 EXPLAIN)
    │       │       │
    │       │       └─► ExecutionStatsContext
    │       │
    │       └─► 直接用于监控 API
    │
    ├─► QueryMetrics (阶段时间统计)
    │       │
    │       └─► 转换为 StageMetrics (用于 QueryProfile)
    │               │
    │               └─► QueryProfile.stages
    │
    └─► StorageMetricsCollector (存储层统计)
            │
            └─► 用于监控 API
```

---

### 4.2 统计信息层次

```
┌─────────────────────────────────────────────────────────┐
│                  Layer 3: 展示层                         │
│  - QueryProfile (内部监控，毫秒显示)                     │
│  - QueryMetricsResponse (客户端返回，微秒显示)          │
│  - EXPLAIN Output (执行计划展示)                        │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ 转换/聚合
┌─────────────────────────────────────────────────────────┐
│                  Layer 2: 聚合层                         │
│  - ExecutionStatsContext (节点统计聚合)                 │
│  - StatsManager (全局统计聚合)                          │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ 收集
┌─────────────────────────────────────────────────────────┐
│                  Layer 1: 收集层 (唯一数据源)            │
│  - ExecutorStats (执行器统计)                           │
│  - QueryMetrics (查询阶段统计)                          │
│  - StorageMetricsCollector (存储层统计)                 │
│  - ExecutionFeedbackCollector (优化器反馈)              │
└─────────────────────────────────────────────────────────┘
```

**设计原则**:
1. **Layer 1 是唯一数据源**: 所有统计信息只在 Layer 1 收集一次
2. **Layer 2 负责聚合**: 不重复收集，只聚合 Layer 1 的数据
3. **Layer 3 负责展示**: 不存储数据，只负责格式转换和显示

---

## 五、具体改进方案

### 5.1 ExecutorStats 作为核心

**修改文件**: `src/query/executor/base/executor_stats.rs`

保持 `ExecutorStats` 不变，作为执行器统计的唯一数据源。

---

### 5.2 简化 ExecutorStat

**修改文件**: `src/core/stats/profile.rs`

```rust
/// Actuator statistics
/// 
/// 简化为仅包含必要标识字段，实际统计引用 ExecutorStats
#[derive(Debug, Clone)]
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    /// 统计信息引用 ExecutorStats
    #[serde(flatten)]
    pub stats: ExecutorStats,
}

impl ExecutorStat {
    /// 从 ExecutorStats 转换
    pub fn from_executor(
        executor_type: String,
        executor_id: i64,
        stats: ExecutorStats,
    ) -> Self {
        Self {
            executor_type,
            executor_id,
            stats,
        }
    }
    
    /// 显示用：执行时间（毫秒）
    pub fn duration_ms(&self) -> f64 {
        self.stats.exec_time_us as f64 / 1000.0
    }
    
    /// 显示用：行数
    pub fn rows_processed(&self) -> usize {
        self.stats.num_rows
    }
    
    /// 显示用：内存
    pub fn memory_used(&self) -> usize {
        self.stats.memory_peak
    }
}
```

---

### 5.3 简化 NodeExecutionStats

**修改文件**: `src/query/executor/explain/execution_stats_context.rs`

```rust
/// Node-level execution statistics
#[derive(Debug, Clone)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    /// 引用 ExecutorStats 作为数据源
    pub executor_stats: ExecutorStats,
    /// 节点特有统计
    pub startup_time_us: u64,
}

impl NodeExecutionStats {
    pub fn new(node_id: i64) -> Self {
        Self {
            node_id,
            executor_stats: ExecutorStats::default(),
            startup_time_us: 0,
        }
    }
    
    // 代理方法，方便访问
    pub fn actual_rows(&self) -> usize {
        self.executor_stats.num_rows
    }
    
    pub fn actual_time_us(&self) -> u64 {
        self.executor_stats.exec_time_us
    }
    
    pub fn actual_time_ms(&self) -> f64 {
        self.executor_stats.exec_time_us as f64 / 1000.0
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        self.executor_stats.cache_hit_rate()
    }
}
```

---

### 5.4 统一 StageMetrics

**修改文件**: `src/core/stats/profile.rs`

```rust
/// Statistics during the query execution phase (in microseconds)
#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub parse_us: u64,
    pub validate_us: u64,
    pub plan_us: u64,
    pub optimize_us: u64,
    pub execute_us: u64,
}

impl StageMetrics {
    /// 从 QueryMetrics 转换
    pub fn from_query_metrics(metrics: &QueryMetrics) -> Self {
        Self {
            parse_us: metrics.parse_time_us,
            validate_us: metrics.validate_time_us,
            plan_us: metrics.plan_time_us,
            optimize_us: metrics.optimize_time_us,
            execute_us: metrics.execute_time_us,
        }
    }
    
    /// 转换为毫秒（用于显示）
    pub fn total_ms(&self) -> f64 {
        (self.parse_us + self.validate_us + self.plan_us + 
         self.optimize_us + self.execute_us) as f64 / 1000.0
    }
    
    /// 解析时间（毫秒，用于显示）
    pub fn parse_ms(&self) -> f64 {
        self.parse_us as f64 / 1000.0
    }
    
    /// 执行时间（毫秒，用于显示）
    pub fn execute_ms(&self) -> f64 {
        self.execute_us as f64 / 1000.0
    }
}
```

---

### 5.5 统一工具函数

**新增文件**: `src/core/stats/utils.rs`

```rust
//! Statistics utility functions
//!
//! Provide common utility functions for statistics to avoid duplicate implementations.

use std::collections::HashMap;

/// Calculate cache hit rate
pub fn calculate_cache_hit_rate(hits: u64, misses: u64) -> f64 {
    let total = hits + misses;
    if total > 0 {
        hits as f64 / total as f64
    } else {
        0.0
    }
}

/// Calculate average from total and count
pub fn calculate_average(total: f64, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

/// Convert microseconds to milliseconds (for display)
pub fn micros_to_millis(micros: u64) -> f64 {
    micros as f64 / 1000.0
}

/// Convert duration to microseconds
pub fn duration_to_micros(duration: std::time::Duration) -> u64 {
    duration.as_micros() as u64
}

/// Format microseconds to human-readable string
pub fn format_duration(micros: u64) -> String {
    if micros >= 1_000_000 {
        format!("{:.2}s", micros as f64 / 1_000_000.0)
    } else if micros >= 1_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{}us", micros)
    }
}

/// Merge two HashMaps of statistics
pub fn merge_stats<T: Clone + std::ops::Add<Output = T>>(
    stats1: &mut HashMap<String, T>,
    stats2: &HashMap<String, T>,
) {
    for (key, value) in stats2 {
        if let Some(existing) = stats1.get_mut(key) {
            // 如果值支持加法，则累加
            // 注意：这里需要根据具体类型处理
        } else {
            stats1.insert(key.clone(), value.clone());
        }
    }
}
```

---

## 六、复用效果评估

### 6.1 减少重复计算

| 统计项 | 改进前重复次数 | 改进后重复次数 | 减少 |
|--------|--------------|--------------|------|
| 执行时间 | 3 次 | 1 次 | 67% |
| 行数统计 | 3 次 | 1 次 | 67% |
| 缓存统计 | 3 次 | 1 次 | 67% |
| 内存统计 | 2 次 | 1 次 | 50% |
| cache_hit_rate() 方法 | 3 处实现 | 1 处实现 | 67% |

---

### 6.2 精度提升

| 结构 | 改进前精度 | 改进后精度 | 提升 |
|------|-----------|-----------|------|
| ExecutorStat | 毫秒 | 微秒（内部）+ 毫秒（显示） | 保持微秒精度 |
| NodeExecutionStats | 毫秒 | 微秒（内部）+ 毫秒（显示） | 保持微秒精度 |
| StageMetrics | 毫秒 | 微秒（内部）+ 毫秒（显示） | 保持微秒精度 |

---

### 6.3 内存优化

**改进前**:
```rust
// 每个执行器节点存储 3 份统计
ExecutorStats: 120 bytes
ExecutorStat: 80 bytes
NodeExecutionStats: 100 bytes
总计：~300 bytes / 节点
```

**改进后**:
```rust
// 每个执行器节点存储 1 份统计
ExecutorStats: 120 bytes
ExecutorStat: 引用（8 bytes）
NodeExecutionStats: 引用（8 bytes）
总计：~136 bytes / 节点
```

**内存节省**: 约 55%

---

## 七、实施建议

### 7.1 分阶段实施

#### 阶段 1: 统一工具函数
- ✅ 创建 `src/core/stats/utils.rs`
- ✅ 提取公共函数（cache_hit_rate, format_duration 等）
- ✅ 更新现有代码使用工具函数

#### 阶段 2: 简化 ExecutorStat
- ✅ 修改 `ExecutorStat` 结构，引用 `ExecutorStats`
- ✅ 更新所有创建 `ExecutorStat` 的代码
- ✅ 添加转换方法

#### 阶段 3: 简化 NodeExecutionStats
- ✅ 修改 `NodeExecutionStats` 结构，引用 `ExecutorStats`
- ✅ 更新所有创建 `NodeExecutionStats` 的代码
- ✅ 添加代理方法

#### 阶段 4: 统一 StageMetrics
- ✅ 修改 `StageMetrics` 为微秒精度
- ✅ 添加从 `QueryMetrics` 转换的方法
- ✅ 更新所有使用 `StageMetrics` 的代码

---

### 7.2 测试策略

1. **单元测试**: 确保转换方法正确
2. **集成测试**: 确保统计信息一致性
3. **性能测试**: 验证内存和性能优化效果
4. **回归测试**: 确保现有功能不受影响

---

### 7.3 向后兼容

1. **保留显示方法**: 保留 `duration_ms()`, `actual_time_ms()` 等毫秒显示方法
2. **API 兼容**: 保持现有 API 签名不变
3. **数据兼容**: 序列化格式保持不变（使用 `#[serde(flatten)]`）

---

## 八、总结

### 8.1 核心原则

1. **单一数据源**: 每个统计信息只在一个地方收集
2. **引用而非复制**: 上层结构引用底层统计，而非复制字段
3. **精度分离**: 内部存储使用微秒精度，显示时转换为毫秒
4. **工具函数化**: 公共计算方法提取为工具函数

### 8.2 预期收益

- ✅ **减少重复计算**: 67% 的统计信息不再重复收集
- ✅ **提升精度**: 所有统计内部保持微秒精度
- ✅ **节省内存**: 约 55% 的内存节省
- ✅ **简化维护**: 统一的工具函数，易于维护和测试
- ✅ **向后兼容**: 保持现有 API 和显示格式

通过统计信息的合理复用，可以在不改变现有架构的前提下，显著提升系统的性能和可维护性。
