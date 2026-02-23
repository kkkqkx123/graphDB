# GraphDB 优化器核心模块分阶段实施计划

本文档基于 [final_architecture_design.md](final_architecture_design.md) 设计，详细描述优化器核心模块（`src/query/optimizer/core/`）的分阶段实施计划。

## 当前状态概览

| 文件 | 状态 | 说明 |
|------|------|------|
| `statistics.rs` | ✅ 已实现 | 表级、列级、索引、图结构统计信息 |
| `selectivity.rs` | ✅ 已实现 | 选择性估计器（等值、范围条件） |
| `cost_model.rs` | ✅ 已实现 | 代价模型配置和上下文 |
| `cost.rs` | ✅ 已实现 | 基础代价类型（旧版） |
| `config.rs` | ✅ 已实现 | 优化配置（旧版） |
| `analyze.rs` | ❌ 未实现 | 统计信息收集器 |

---

## 第一阶段：整合现有模块

### 目标
统一 `mod.rs` 导出，消除重复定义，使新实现的模块可被外部使用。

### 当前问题
- `mod.rs` 仅导出旧的 `cost.rs` 和 `config.rs`
- 新实现的 `statistics.rs`、`selectivity.rs`、`cost_model.rs` 未在 `mod.rs` 中导出

### 修改内容

**文件**: `src/query/optimizer/core/mod.rs`

```rust
//! 核心类型模块
//! 提供优化器所需的核心数据类型，包括代价模型、统计信息、选择性估计和配置

// 基础模块
pub mod cost;
pub mod config;

// 新实现的核心模块
pub mod statistics;
pub mod selectivity;
pub mod cost_model;

// 统计信息收集器（第二阶段实现）
#[cfg(feature = "analyze")]
pub mod analyze;

// 从旧模块重新导出（保持向后兼容性）
pub use cost::{Cost, Statistics as LegacyStatistics, TableStats, ColumnStats, PlanNodeProperties};
pub use config::{OptimizationConfig, OptimizationStats};

// 从 statistics 模块重新导出
pub use statistics::{
    TableStatistics, 
    ColumnStatistics, 
    IndexStatistics,
    GraphStatistics,
    StatisticsProvider,
    InMemoryStatisticsProvider,
};

// 从 selectivity 模块重新导出
pub use selectivity::{
    SelectivityEstimator, 
    RangeOp, 
    BooleanOp,
    JoinType as SelectivityJoinType,
};

// 从 cost_model 模块重新导出
pub use cost_model::{
    CostModelConfig, 
    CostContext, 
    CostCalculator,
};

// 从 analyze 模块重新导出（第二阶段实现）
#[cfg(feature = "analyze")]
pub use analyze::{
    StatisticsCollector,
    AnalyzeConfig,
    AnalyzeError,
};

pub use crate::query::core::OptimizationPhase;
```

### 验收标准
- [ ] 所有模块正确导出
- [ ] 无编译错误
- [ ] 现有代码兼容性保持

---

## 第二阶段：实现 analyze.rs（统计信息收集器）

### 目标
实现 `ANALYZE` 命令功能，收集表和列的统计信息。

### 设计要点

#### 1. 核心结构

```rust
/// 统计信息收集器
/// 
/// 负责从存储层采样数据并计算统计信息
pub struct StatisticsCollector<S: StorageClient> {
    storage: S,
    config: AnalyzeConfig,
}

/// 分析配置
#[derive(Debug, Clone)]
pub struct AnalyzeConfig {
    /// 采样率 (0.0 - 1.0)，默认 0.1
    pub sample_ratio: f64,
    /// 最大采样行数，默认 10000
    pub max_sample_rows: u64,
    /// MCV 列表大小，默认 100
    pub mcv_target: usize,
    /// 直方图桶数，默认 100
    pub histogram_buckets: usize,
    /// 最小采样行数，默认 100
    pub min_sample_rows: u64,
}

/// 分析错误类型
#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("表不存在: {0}")]
    TableNotFound(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("采样错误: {0}")]
    SamplingError(String),
    #[error("统计计算错误: {0}")]
    CalculationError(String),
}
```

#### 2. 核心方法

```rust
impl<S: StorageClient> StatisticsCollector<S> {
    /// 创建新的统计信息收集器
    pub fn new(storage: S) -> Self;
    
    /// 使用自定义配置创建
    pub fn with_config(storage: S, config: AnalyzeConfig) -> Self;
    
    /// 分析单个表
    pub async fn analyze_table(&mut self, table_name: &str) -> Result<TableStatistics, AnalyzeError>;
    
    /// 分析多个表
    pub async fn analyze_tables(&mut self, table_names: &[String]) -> Vec<Result<TableStatistics, AnalyzeError>>;
    
    /// 分析所有表
    pub async fn analyze_all(&mut self) -> Vec<Result<TableStatistics, AnalyzeError>>;
    
    /// 增量分析（仅分析变更的数据）
    pub async fn analyze_incremental(&mut self, table_name: &str, since: SystemTime) -> Result<TableStatistics, AnalyzeError>;
}
```

#### 3. 列分析算法

```rust
/// 列分析器
struct ColumnAnalyzer;

impl ColumnAnalyzer {
    /// 分析列统计信息
    fn analyze(
        column_name: &str,
        values: Vec<Value>,
        config: &AnalyzeConfig,
    ) -> ColumnStatistics {
        let total_count = values.len();
        
        // 1. 计算空值比例
        let null_count = values.iter().filter(|v| v.is_null()).count();
        let null_fraction = null_count as f64 / total_count as f64;
        
        // 2. 计算不同值
        let distinct_values = Self::compute_distinct_values(&values);
        let distinct_count = distinct_values.len() as u64;
        
        // 3. 识别 MCV（最常见值）
        let most_common_values = Self::compute_mcv(&distinct_values, config.mcv_target);
        
        // 4. 构建直方图（对非 MCV 值）
        let histogram_bounds = Self::compute_histogram(
            &values, 
            &most_common_values,
            config.histogram_buckets
        );
        
        // 5. 计算最小/最大值
        let (min_value, max_value) = Self::compute_min_max(&values);
        
        ColumnStatistics {
            column_name: column_name.to_string(),
            null_fraction,
            distinct_count,
            avg_width: Self::compute_avg_width(&values),
            most_common_values,
            histogram_bounds,
            min_value,
            max_value,
        }
    }
    
    /// 计算不同值及其频率
    fn compute_distinct_values(values: &[Value]) -> HashMap<Value, u64>;
    
    /// 计算最常见值
    fn compute_mcv(distinct: &HashMap<Value, u64>, target: usize) -> Vec<(Value, f64)>;
    
    /// 计算直方图边界
    fn compute_histogram(
        values: &[Value], 
        mcv: &[(Value, f64)],
        buckets: usize
    ) -> Vec<Value>;
    
    /// 计算最小最大值
    fn compute_min_max(values: &[Value]) -> (Option<Value>, Option<Value>);
    
    /// 计算平均宽度
    fn compute_avg_width(values: &[Value]) -> u64;
}
```

#### 4. 采样策略

```rust
/// 采样器
trait Sampler {
    /// 随机采样
    fn sample_random(&self, data: &[Value], ratio: f64) -> Vec<Value>;
    
    /// 系统采样（每隔 n 个取一个）
    fn sample_systematic(&self, data: &[Value], interval: usize) -> Vec<Value>;
    
    /// 分层采样（按某个属性分层）
    fn sample_stratified(&self, data: &[Value], strata: &[String]) -> Vec<Value>;
}

/// 默认采样实现
struct DefaultSampler;

impl Sampler for DefaultSampler {
    fn sample_random(&self, data: &[Value], ratio: f64) -> Vec<Value> {
        let mut rng = rand::thread_rng();
        data.iter()
            .filter(|_| rng.gen::<f64>() < ratio)
            .cloned()
            .collect()
    }
    // ...
}
```

### 文件结构

```
src/query/optimizer/core/
├── mod.rs                    # 模块导出（第一阶段修改）
├── statistics.rs             # 统计信息结构体（已实现）
├── selectivity.rs            # 选择性估计器（已实现）
├── cost_model.rs             # 代价模型（已实现）
├── cost.rs                   # 基础代价类型（旧版）
├── config.rs                 # 优化配置（旧版）
└── analyze.rs                # 统计信息收集器（第二阶段实现）
```

### 验收标准
- [ ] `StatisticsCollector` 结构体定义完整
- [ ] `AnalyzeConfig` 配置项齐全
- [ ] `AnalyzeError` 错误类型定义
- [ ] 基础方法骨架实现（可返回 todo!()）
- [ ] 模块在 `mod.rs` 中正确导出
- [ ] 编译通过

---

## 第三阶段：统一统计信息接口（后续）

### 目标
定义 `StatisticsProvider` trait，解耦优化器与存储层。

### 关键设计

```rust
/// 统计信息提供者 trait
/// 
/// 由存储层实现，为优化器提供统计信息查询接口
/// 参考 PostgreSQL 的统计信息系统
pub trait StatisticsProvider: Send + Sync {
    /// 获取表统计信息
    fn get_table_stats(&self, table_name: &str) -> Option<TableStatistics>;
    
    /// 获取列统计信息
    fn get_column_stats(&self, table_name: &str, column_name: &str) -> Option<ColumnStatistics>;
    
    /// 获取索引统计信息
    fn get_index_stats(&self, index_name: &str) -> Option<IndexStatistics>;
    
    /// 获取图结构统计信息
    fn get_graph_stats(&self) -> Option<GraphStatistics>;
    
    /// 获取最后分析时间
    fn get_last_analyzed(&self, table_name: &str) -> Option<SystemTime>;
}
```

---

## 第四阶段：完善选择性估计器（后续）

### 待补充方法

| 方法 | 说明 |
|------|------|
| `estimate_in` | IN 条件选择性 |
| `estimate_pattern` | LIKE/模式匹配选择性 |
| `estimate_boolean` | AND/OR/NOT 复合条件 |
| `estimate_join` | 连接选择性 |
| `estimate_graph_traversal` | 图遍历选择性 |

---

## 第五阶段：完善代价计算器（后续）

### 待实现方法

| 方法 | 说明 |
|------|------|
| `calculate_seq_scan` | 顺序扫描代价 |
| `calculate_index_scan` | 索引扫描代价 |
| `calculate_nested_loop_join` | 嵌套循环连接代价 |
| `calculate_hash_join` | 哈希连接代价 |
| `calculate_merge_join` | 归并连接代价 |
| `calculate_graph_traversal` | 图遍历代价 |

---

## 实施检查清单

### 第一阶段检查项
- [ ] 修改 `src/query/optimizer/core/mod.rs`
- [ ] 添加新模块的 pub mod 声明
- [ ] 添加 pub use 重新导出
- [ ] 运行 `cargo check` 验证编译

### 第二阶段检查项
- [ ] 创建 `src/query/optimizer/core/analyze.rs`
- [ ] 定义 `StatisticsCollector` 结构体
- [ ] 定义 `AnalyzeConfig` 配置结构体
- [ ] 定义 `AnalyzeError` 错误枚举
- [ ] 实现基础方法骨架
- [ ] 在 `mod.rs` 中添加条件编译导出
- [ ] 运行 `cargo check` 验证编译

---

## 参考文档

- [final_architecture_design.md](final_architecture_design.md) - 架构设计文档
- [postgresql_optimizer_design.md](postgresql_optimizer_design.md) - PostgreSQL 优化器参考
- [postgresql_cost_model_reference.md](postgresql_cost_model_reference.md) - PostgreSQL 代价模型参考
