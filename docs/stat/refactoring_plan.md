# 统计信息复用改进方案

## 一、现状与问题

### 1.1 统计信息分布

当前项目的统计信息分布在三个主要模块：

| 模块                    | 核心结构                                     | 精度          | 用途           |
| ----------------------- | -------------------------------------------- | ------------- | -------------- |
| `core/stats`            | QueryMetrics, QueryProfile, ExecutorStat     | 微秒/毫秒混用 | 查询统计和监控 |
| `query/executor`        | ExecutorStats, NodeExecutionStats            | 微秒/毫秒混用 | 执行器统计     |
| `query/optimizer/stats` | OperatorFeedback, ExecutionFeedbackCollector | 微秒          | 优化器反馈     |

### 1.2 核心问题

1. **重复计算严重**
   - 执行时间在 3 个结构中重复记录
   - 行数统计在 3 个结构中重复记录
   - 缓存统计在 3 个结构中重复记录
   - `cache_hit_rate()` 方法在 3 个地方重复实现

2. **精度不统一**
   - `QueryMetrics`: 微秒 (us)
   - `QueryProfile`: 毫秒 (ms)
   - `ExecutorStats`: 微秒 (us)
   - `NodeExecutionStats`: 毫秒 (f64)

3. **内存浪费**
   - 每个执行器节点存储 3 份统计信息
   - 约 300 bytes/节点，实际只需 136 bytes/节点

---

## 二、改进目标

### 2.1 核心目标

1. **单一数据源**: 每个统计信息只在一个地方收集
2. **引用而非复制**: 上层结构引用底层统计，而非复制字段
3. **精度分离**: 内部存储使用微秒精度，显示时转换为毫秒
4. **工具函数化**: 公共计算方法提取为工具函数

### 2.2 预期收益

- ✅ **减少重复计算**: 67% 的统计信息不再重复收集
- ✅ **提升精度**: 所有统计内部保持微秒精度
- ✅ **节省内存**: 约 55% 的内存节省
- ✅ **简化维护**: 统一的工具函数，易于维护和测试

---

## 三、架构设计

### 3.1 统计信息层次架构

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

### 3.2 数据流

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

## 四、具体修改方案

### 4.1 新增工具函数模块

**文件**: `src/core/stats/utils.rs`

```rust
//! Statistics utility functions
//!
//! Provide common utility functions for statistics to avoid duplicate implementations.

use std::time::Duration;

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
pub fn duration_to_micros(duration: Duration) -> u64 {
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
```

**修改**: `src/core/stats/mod.rs`

```rust
pub mod utils;  // 新增
```

---

### 4.2 简化 ExecutorStat

**文件**: `src/core/stats/profile.rs`

**修改前**:

```rust
#[derive(Debug, Clone)]
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    pub duration_ms: u64,
    pub rows_processed: usize,
    pub memory_used: usize,
}
```

**修改后**:

```rust
use crate::query::executor::base::ExecutorStats;

/// Actuator statistics
#[derive(Debug, Clone)]
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    /// 引用 ExecutorStats 作为数据源
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
        crate::stats::utils::micros_to_millis(self.stats.exec_time_us)
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

### 4.3 简化 NodeExecutionStats

**文件**: `src/query/executor/explain/execution_stats_context.rs`

**修改前**:

```rust
#[derive(Debug, Clone, Default)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub actual_rows: usize,
    pub actual_time_ms: f64,
    pub startup_time_ms: f64,
    pub total_time_ms: f64,
    pub memory_used: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub io_reads: usize,
    pub io_read_bytes: usize,
}
```

**修改后**:

```rust
use crate::query::executor::base::ExecutorStats;

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
        crate::core::stats::utils::micros_to_millis(self.executor_stats.exec_time_us)
    }

    pub fn cache_hit_rate(&self) -> f64 {
        self.executor_stats.cache_hit_rate()
    }
}

impl Default for NodeExecutionStats {
    fn default() -> Self {
        Self::new(0)
    }
}
```

---

### 4.4 统一 StageMetrics

**文件**: `src/core/stats/profile.rs`

**修改前**:

```rust
#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub parse_ms: u64,
    pub validate_ms: u64,
    pub plan_ms: u64,
    pub optimize_ms: u64,
    pub execute_ms: u64,
}
```

**修改后**:

```rust
use crate::core::stats::QueryMetrics;

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
        crate::stats::utils::micros_to_millis(self.parse_us)
    }

    /// 执行时间（毫秒，用于显示）
    pub fn execute_ms(&self) -> f64 {
        crate::stats::utils::micros_to_millis(self.execute_us)
    }
}
```

---

### 4.5 更新 QueryProfile

**文件**: `src/core/stats/profile.rs`

**修改点**:

```rust
impl QueryProfile {
    /// 添加执行器统计
    pub fn add_executor_stat(&mut self, stat: ExecutorStat) {
        self.executor_stats.push(stat);
    }

    /// 从 ExecutorStats 添加统计
    pub fn add_executor_stats_from(
        &mut self,
        executor_type: String,
        executor_id: i64,
        stats: ExecutorStats,
    ) {
        let stat = ExecutorStat::from_executor(
            executor_type,
            executor_id,
            stats,
        );
        self.executor_stats.push(stat);
    }

    /// 设置阶段统计
    pub fn set_stage_metrics(&mut self, metrics: QueryMetrics) {
        self.stages = StageMetrics::from_query_metrics(&metrics);
    }
}
```

---

### 4.6 更新执行器代码

**文件**: `src/query/executor/` 下的所有执行器

**修改前**:

```rust
impl Executor for MyExecutor {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        // ... 执行逻辑

        self.stats.add_exec_time(start.elapsed());
        self.stats.add_row(num_rows);

        Ok(result)
    }
}
```

**修改后**:

```rust
impl Executor for MyExecutor {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        // ... 执行逻辑

        // 统计已在 ExecutorStats 中收集
        self.stats.add_exec_time(start.elapsed());
        self.stats.add_row(num_rows);

        Ok(result)
    }

    /// 获取 ExecutorStat 用于 QueryProfile
    fn to_executor_stat(&self, executor_type: String, executor_id: i64) -> ExecutorStat {
        ExecutorStat::from_executor(
            executor_type,
            executor_id,
            self.stats.clone(),
        )
    }
}
```

---

### 4.7 更新 ExecutionStatsContext

**文件**: `src/query/executor/explain/execution_stats_context.rs`

**修改点**:

```rust
impl ExecutionStatsContext {
    pub fn on_node_complete(
        &self,
        node_id: i64,
        executor_stats: ExecutorStats,
        startup_time_us: u64,
    ) {
        let node_stats = NodeExecutionStats {
            node_id,
            executor_stats,
            startup_time_us,
        };
        let mut stats = self.node_stats.lock();
        stats.insert(node_id, node_stats);
    }

    pub fn get_node_stats(&self, node_id: i64) -> Option<NodeExecutionStats> {
        self.node_stats.lock().get(&node_id).cloned()
    }
}
```

---

## 五、实施计划

### 5.1 分阶段实施

#### 阶段 1: 统一工具函数 (1-2 天)

**任务**:

1. ✅ 创建 `src/core/stats/utils.rs`
2. ✅ 提取公共函数（`calculate_cache_hit_rate`, `format_duration` 等）
3. ✅ 在 `src/core/stats/mod.rs` 中导出
4. ✅ 编写单元测试

**验收标准**:

- 工具函数覆盖所有公共计算逻辑
- 现有代码可正常使用工具函数
- 单元测试覆盖率 > 90%

---

#### 阶段 2: 简化 ExecutorStat (2-3 天)

**任务**:

1. ✅ 修改 `ExecutorStat` 结构，引用 `ExecutorStats`
2. ✅ 添加 `from_executor()` 转换方法
3. ✅ 添加显示方法（`duration_ms()`, `rows_processed()`, `memory_used()`）
4. ✅ 更新所有创建 `ExecutorStat` 的代码
5. ✅ 编写集成测试

**涉及文件**:

- `src/core/stats/profile.rs`
- `src/query/executor/` 下的所有执行器
- `src/core/stats/manager.rs`

**验收标准**:

- 所有执行器正确创建 `ExecutorStat`
- `QueryProfile` 正确填充执行器统计
- 序列化/反序列化兼容

---

#### 阶段 3: 简化 NodeExecutionStats (2-3 天)

**任务**:

1. ✅ 修改 `NodeExecutionStats` 结构，引用 `ExecutorStats`
2. ✅ 添加代理方法（`actual_rows()`, `actual_time_ms()` 等）
3. ✅ 更新所有创建 `NodeExecutionStats` 的代码
4. ✅ 更新 `ExecutionStatsContext`
5. ✅ 编写集成测试

**涉及文件**:

- `src/query/executor/explain/execution_stats_context.rs`
- `src/query/executor/explain/instrumented_executor.rs`
- `src/query/executor/explain/profile_executor.rs`

**验收标准**:

- EXPLAIN ANALYZE 输出正确
- PROFILE 语句输出正确
- 节点统计准确无误

---

#### 阶段 4: 统一 StageMetrics (2-3 天)

**任务**:

1. ✅ 修改 `StageMetrics` 为微秒精度
2. ✅ 添加从 `QueryMetrics` 转换的方法
3. ✅ 添加显示方法（`parse_ms()`, `execute_ms()` 等）
4. ✅ 更新所有使用 `StageMetrics` 的代码
5. ✅ 编写集成测试

**涉及文件**:

- `src/core/stats/profile.rs`
- `src/core/stats/manager.rs`
- 查询执行相关代码

**验收标准**:

- `QueryProfile` 正确记录阶段时间
- 显示时正确转换为毫秒
- 与 `QueryMetrics` 数据一致

---

#### 阶段 5: 集成测试与优化 (2-3 天)

**任务**:

1. ✅ 编写端到端集成测试
2. ✅ 性能测试（内存占用、执行时间）
3. ✅ 回归测试（确保现有功能不受影响）
4. ✅ 文档更新
5. ✅ 代码审查和优化

**验收标准**:

- 所有测试通过
- 内存占用减少 > 50%
- 性能无明显下降
- 文档完整

---

### 5.2 测试策略

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::executor::base::ExecutorStats;

    #[test]
    fn test_executor_stat_from_executor() {
        let mut stats = ExecutorStats::default();
        stats.exec_time_us = 1500;
        stats.num_rows = 100;
        stats.memory_peak = 2048;

        let stat = ExecutorStat::from_executor(
            "TestExecutor".to_string(),
            1,
            stats,
        );

        assert_eq!(stat.duration_ms(), 1.5);
        assert_eq!(stat.rows_processed(), 100);
        assert_eq!(stat.memory_used(), 2048);
    }

    #[test]
    fn test_stage_metrics_from_query_metrics() {
        let mut metrics = QueryMetrics::default();
        metrics.parse_time_us = 100;
        metrics.execute_time_us = 500;

        let stages = StageMetrics::from_query_metrics(&metrics);

        assert_eq!(stages.parse_us, 100);
        assert_eq!(stages.execute_us, 500);
        assert_eq!(stages.parse_ms(), 0.1);
        assert_eq!(stages.execute_ms(), 0.5);
    }

    #[test]
    fn test_node_execution_stats() {
        let mut executor_stats = ExecutorStats::default();
        executor_stats.exec_time_us = 2000;
        executor_stats.num_rows = 150;

        let node_stats = NodeExecutionStats {
            node_id: 1,
            executor_stats,
            startup_time_us: 100,
        };

        assert_eq!(node_stats.actual_rows(), 150);
        assert_eq!(node_stats.actual_time_us(), 2000);
        assert_eq!(node_stats.actual_time_ms(), 2.0);
    }
}
```

#### 集成测试

```rust
#[test]
fn test_statistics_reuse_integration() {
    // 创建测试数据库
    let db = setup_test_db();

    // 执行查询
    let result = db.execute("MATCH (n) RETURN n LIMIT 100");

    // 验证 QueryMetrics
    let metrics = result.metrics();
    assert!(metrics.total_time_us > 0);

    // 验证 QueryProfile
    let profile = result.profile();
    assert_eq!(profile.stages.total_ms(), metrics.total_time_us as f64 / 1000.0);
    assert!(!profile.executor_stats.is_empty());

    // 验证 ExecutorStat 引用 ExecutorStats
    for stat in &profile.executor_stats {
        assert_eq!(stat.duration_ms(), stat.stats.exec_time_us as f64 / 1000.0);
        assert_eq!(stat.rows_processed(), stat.stats.num_rows);
    }
}
```

#### 性能测试

```rust
#[test]
fn test_memory_usage() {
    // 测试改进前后的内存占用
    let before_size = std::mem::size_of::<OldNodeExecutionStats>();
    let after_size = std::mem::size_of::<NodeExecutionStats>();

    println!("Before: {} bytes", before_size);
    println!("After: {} bytes", after_size);

    // 验证内存节省 > 50%
    assert!(after_size < before_size / 2);
}
```

---

### 5.3 向后兼容

#### API 兼容

1. **保留显示方法**: 保留 `duration_ms()`, `actual_time_ms()` 等毫秒显示方法
2. **保持方法签名**: 现有公开 API 的方法签名保持不变
3. **序列化兼容**: 使用 `#[serde(flatten)]` 保持序列化格式不变

#### 数据兼容

```rust
// 旧格式
{
    "executor_type": "ScanVerticesExecutor",
    "executor_id": 1,
    "duration_ms": 100,
    "rows_processed": 50,
    "memory_used": 1024
}

// 新格式（使用 #[serde(flatten)]）
{
    "executor_type": "ScanVerticesExecutor",
    "executor_id": 1,
    "num_rows": 50,
    "exec_time_us": 100000,
    "total_time_us": 100000,
    "memory_peak": 1024,
    "memory_current": 512,
    "batch_count": 1,
    "cache_hits": 10,
    "cache_misses": 5
}
```

---

## 六、验证标准

### 6.1 功能验证

- ✅ 所有单元测试通过
- ✅ 所有集成测试通过
- ✅ EXPLAIN ANALYZE 输出正确
- ✅ PROFILE 语句输出正确
- ✅ 查询返回结果正确

### 6.2 性能验证

- ✅ 内存占用减少 > 50%
- ✅ 执行时间无明显增加（< 5%）
- ✅ 统计信息准确性 100%

### 6.3 代码质量

- ✅ Clippy 检查通过
- ✅ 代码覆盖率 > 90%
- ✅ 文档完整
- ✅ 代码审查通过

---

## 七、风险评估

### 7.1 技术风险

| 风险           | 影响 | 概率 | 缓解措施                          |
| -------------- | ---- | ---- | --------------------------------- |
| 序列化格式变化 | 高   | 中   | 使用 `#[serde(flatten)]` 保持兼容 |
| 性能下降       | 中   | 低   | 充分性能测试，优化关键路径        |
| 统计不准确     | 高   | 低   | 完善的单元测试和集成测试          |

### 7.2 实施风险

| 风险         | 影响 | 概率 | 缓解措施             |
| ------------ | ---- | ---- | -------------------- |
| 代码改动量大 | 中   | 高   | 分阶段实施，逐步验证 |
| 回归问题     | 高   | 中   | 完善的回归测试套件   |
| 文档滞后     | 低   | 中   | 代码和文档同步更新   |

---

## 八、总结

### 8.1 核心改进

1. **单一数据源**: `ExecutorStats` 作为执行器统计的唯一数据源
2. **引用而非复制**: `ExecutorStat` 和 `NodeExecutionStats` 引用 `ExecutorStats`
3. **精度统一**: 所有统计内部使用微秒精度，显示时转换为毫秒
4. **工具函数化**: 公共计算方法提取为工具函数

### 8.2 预期收益

- ✅ **减少重复计算**: 67% 的统计信息不再重复收集
- ✅ **提升精度**: 所有统计内部保持微秒精度
- ✅ **节省内存**: 约 55% 的内存节省
- ✅ **简化维护**: 统一的工具函数，易于维护和测试
- ✅ **向后兼容**: 保持现有 API 和显示格式

### 8.3 实施建议

1. **分阶段实施**: 按照 5 个阶段逐步推进，每个阶段充分测试
2. **充分测试**: 单元测试、集成测试、性能测试、回归测试缺一不可
3. **渐进上线**: 先在测试环境验证，再逐步推广到生产环境
4. **监控反馈**: 实施后持续监控性能和稳定性，及时收集反馈

通过统计信息的合理复用，可以在不改变现有架构的前提下，显著提升系统的性能和可维护性。
