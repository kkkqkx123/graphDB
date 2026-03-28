# 统计信息扩展方案

## 概述

本文档基于策略分析结果，识别当前统计信息体系的不足，提出需要扩展的统计信息类型和具体实施方案。

---

## 一、现有统计信息体系分析

### 1.1 现有统计类型

| 统计类型 | 数据结构 | 主要字段 | 用途 |
|---------|---------|---------|------|
| TagStatistics | 标签统计 | vertex_count, avg_out_degree, avg_in_degree | 标签级别成本估计 |
| EdgeTypeStatistics | 边类型统计 | edge_count, avg/max degree, std_dev, gini, hot_vertices | 遍历成本估计、倾斜检测 |
| PropertyStatistics | 属性统计 | distinct_values, histogram | 选择性估计 |
| Histogram | 直方图 | buckets, null_fraction, total_distinct_values | 范围查询选择性 |

### 1.2 现有能力评估

**优势：**
- 支持基本的标签、边类型、属性级别统计
- 直方图支持等深分桶，适合范围查询
- 边统计包含倾斜度指标（基尼系数、热点顶点）
- 使用DashMap实现线程安全

**不足：**
- 缺乏表达式级别统计
- 缺少查询执行反馈统计
- 无数据分布相关性统计
- 缺少历史统计信息（无法检测数据变化趋势）

---

## 二、需要扩展的统计信息

### 2.1 高优先级扩展

#### 2.1.1 表达式统计信息 (ExpressionStatistics)

**需求来源：**
- `expression_precomputation.rs` 需要了解表达式的实际执行成本
- `aggregate_strategy.rs` 需要聚合表达式的复杂度统计
- `subquery_unnesting.rs` 需要子查询执行统计

**数据结构：**

```rust
/// 表达式执行统计
#[derive(Debug, Clone)]
pub struct ExpressionStatistics {
    /// 表达式指纹（用于识别相同表达式）
    pub fingerprint: String,
    /// 执行次数
    pub execution_count: u64,
    /// 平均执行时间（微秒）
    pub avg_execution_time_us: f64,
    /// 执行时间标准差
    pub execution_time_std_dev: f64,
    /// 平均结果大小（字节）
    pub avg_result_size: usize,
    /// 结果大小标准差
    pub result_size_std_dev: f64,
    /// 缓存命中率（如果支持缓存）
    pub cache_hit_rate: f64,
    /// 最后更新时间
    pub last_updated: Instant,
}

/// 表达式统计管理器
#[derive(Debug)]
pub struct ExpressionStatisticsManager {
    /// 表达式指纹 -> 统计信息
    stats: Arc<DashMap<String, ExpressionStatistics>>,
    /// 配置
    config: ExpressionStatsConfig,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatsConfig {
    /// 是否收集详细执行时间
    pub collect_execution_time: bool,
    /// 是否收集结果大小
    pub collect_result_size: bool,
    /// 统计信息过期时间
    pub stats_ttl: Duration,
    /// 最大统计条目数
    pub max_entries: usize,
}
```

**应用场景：**

```rust
// expression_precomputation.rs
pub fn should_precompute(&self, expression: &ContextualExpression) -> PrecomputationDecision {
    // 获取表达式的历史执行统计
    if let Some(stats) = self.stats_manager.get_expression_stats(&expression.fingerprint()) {
        // 基于实际执行成本而非估计成本决策
        let actual_benefit = stats.avg_execution_time_us * expression.reference_count as f64;
        let actual_cost = stats.avg_result_size as f64 * 0.001; // 存储成本
        
        if actual_benefit / actual_cost > self.precompute_threshold {
            return PrecomputationDecision::Precompute { ... };
        }
    }
    // 回退到基于成本的估计
    self.should_precompute_based_on_cost(expression)
}
```

---

#### 2.1.2 查询执行反馈统计 (QueryFeedbackStatistics)

**需求来源：**
- 所有策略都需要执行后的实际成本反馈
- 需要检测估计误差并调整模型
- 需要识别频繁执行的查询模式

**数据结构：**

```rust
/// 查询执行反馈
#[derive(Debug, Clone)]
pub struct QueryExecutionFeedback {
    /// 查询指纹
    pub query_fingerprint: String,
    /// 执行计划指纹
    pub plan_fingerprint: String,
    /// 估计成本
    pub estimated_cost: f64,
    /// 实际执行时间（毫秒）
    pub actual_execution_time_ms: f64,
    /// 实际返回行数
    pub actual_rows: u64,
    /// 实际内存使用（字节）
    pub actual_memory_bytes: u64,
    /// 使用的优化策略
    pub applied_strategies: Vec<StrategyFeedback>,
    /// 执行时间戳
    pub executed_at: Instant,
}

/// 策略执行反馈
#[derive(Debug, Clone)]
pub struct StrategyFeedback {
    /// 策略名称
    pub strategy_name: String,
    /// 策略决策
    pub decision: String,
    /// 估计成本
    pub estimated_cost: f64,
    /// 实际成本贡献
    pub actual_cost_contribution: f64,
    /// 是否最优
    pub was_optimal: bool,
}

/// 反馈驱动的选择性估计
#[derive(Debug)]
pub struct FeedbackDrivenSelectivity {
    /// 谓词模式 -> 实际选择性历史
    selectivity_history: Arc<DashMap<String, SelectivityHistory>>,
    /// 反馈权重（新反馈的权重）
    feedback_weight: f64,
}

#[derive(Debug, Clone)]
pub struct SelectivityHistory {
    /// 估计选择性
    pub estimated_selectivity: f64,
    /// 实际选择性历史（滑动窗口）
    pub actual_selectivities: Vec<f64>,
    /// 平均实际选择性
    pub avg_actual_selectivity: f64,
    /// 估计误差
    pub estimation_error: f64,
    /// 样本数量
    pub sample_count: u64,
}
```

**应用场景：**

```rust
// index.rs - 基于反馈的选择性估计
pub fn select_index(&self, predicates: &[PropertyPredicate]) -> IndexSelection {
    for predicate in predicates {
        // 检查是否有反馈统计
        if let Some(feedback) = self.feedback_manager.get_selectivity_feedback(&predicate) {
            // 如果估计误差过大，使用反馈调整
            if feedback.estimation_error > 0.3 {
                let adjusted_selectivity = feedback.avg_actual_selectivity;
                // 使用调整后的选择性计算成本
                let cost = self.calculate_cost_with_selectivity(adjusted_selectivity);
            }
        }
    }
}

// aggregate_strategy.rs - 基于反馈的策略调整
pub fn select_strategy(&self, context: &AggregateContext) -> AggregateStrategyDecision {
    let base_decision = self.select_strategy_based_on_model(context);
    
    // 检查历史反馈
    if let Some(feedback) = self.feedback_manager.get_strategy_feedback("aggregate", context) {
        // 如果历史决策经常不是最优，调整策略
        if feedback.suboptimal_decision_rate > 0.3 {
            return self.select_conservative_strategy(context);
        }
    }
    
    base_decision
}
```

---

#### 2.1.3 数据相关性统计 (CorrelationStatistics)

**需求来源：**
- `join_order.rs` 需要了解表之间的相关性
- `index.rs` 需要了解多列索引的选择性
- `traversal_start.rs` 需要了解标签组合的选择性

**数据结构：**

```rust
/// 属性相关性统计
#[derive(Debug, Clone)]
pub struct PropertyCorrelation {
    /// 第一个属性
    pub property1: String,
    /// 第二个属性
    pub property2: String,
    /// 标签（可选）
    pub tag_name: Option<String>,
    /// 相关系数 (-1.0 到 1.0)
    pub correlation_coefficient: f64,
    /// 联合选择性
    pub combined_selectivity: f64,
    /// 独立性假设下的选择性
    pub independent_selectivity: f64,
    /// 样本数量
    pub sample_count: u64,
}

/// 多属性统计信息
#[derive(Debug, Clone)]
pub struct MultiPropertyStatistics {
    /// 属性组合键
    pub property_key: String,
    /// 标签名
    pub tag_name: String,
    /// 联合不同值数量
    pub combined_distinct_values: u64,
    /// 属性相关性矩阵
    pub correlations: Vec<PropertyCorrelation>,
    /// 多维直方图（可选）
    pub multi_dim_histogram: Option<MultiDimHistogram>,
}

/// 标签组合统计
#[derive(Debug, Clone)]
pub struct TagCombinationStatistics {
    /// 标签组合
    pub tags: Vec<String>,
    /// 具有这些标签的顶点数
    pub vertex_count: u64,
    /// 相对于独立假设的比率
    pub correlation_ratio: f64,
}
```

**应用场景：**

```rust
// index.rs - 复合索引选择性估计
pub fn evaluate_composite_index(&self, index: &Index, predicates: &[PropertyPredicate]) 
    -> Option<IndexSelection> {
    let mut total_selectivity = 1.0;
    
    for i in 0..predicates.len() {
        for j in (i+1)..predicates.len() {
            // 检查属性相关性
            if let Some(corr) = self.stats_manager.get_property_correlation(
                &predicates[i].property_name,
                &predicates[j].property_name
            ) {
                // 如果属性相关，不使用独立性假设
                if corr.correlation_coefficient.abs() > 0.5 {
                    total_selectivity *= corr.combined_selectivity / corr.independent_selectivity;
                }
            }
        }
    }
    
    // 使用调整后的选择性计算成本
    self.calculate_index_cost(index, total_selectivity)
}
```

---

### 2.2 中优先级扩展

#### 2.2.1 数据变化趋势统计 (DataTrendStatistics)

**需求来源：**
- 需要检测数据分布的变化
- 需要决定何时更新统计信息
- 需要预测未来的数据分布

**数据结构：**

```rust
/// 数据变化趋势
#[derive(Debug, Clone)]
pub struct DataTrend {
    /// 统计项名称
    pub stat_name: String,
    /// 历史值（时间序列）
    pub history: Vec<TimedValue>,
    /// 变化率（每秒）
    pub change_rate: f64,
    /// 趋势方向
    pub trend_direction: TrendDirection,
    /// 预测值（下一个周期）
    pub predicted_next: f64,
}

#[derive(Debug, Clone)]
pub struct TimedValue {
    pub timestamp: Instant,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Fluctuating,
}

/// 统计信息新鲜度
#[derive(Debug, Clone)]
pub struct StatisticsFreshness {
    /// 标签名
    pub tag_name: String,
    /// 最后更新时间
    pub last_updated: Instant,
    /// 数据变化率
    pub data_change_rate: f64,
    /// 估计的统计误差
    pub estimated_error: f64,
    /// 是否需要更新
    pub needs_update: bool,
}
```

---

#### 2.2.2 缓存效率统计 (CacheEfficiencyStatistics)

**需求来源：**
- `materialization.rs` 需要了解CTE缓存效率
- `expression_precomputation.rs` 需要了解预计算缓存效率
- 需要决定缓存策略

**数据结构：**

```rust
/// 缓存效率统计
#[derive(Debug, Clone)]
pub struct CacheEfficiencyStats {
    /// 缓存项标识
    pub cache_key: String,
    /// 缓存类型
    pub cache_type: CacheType,
    /// 创建时间
    pub created_at: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 命中次数
    pub hit_count: u64,
    /// 未命中次数
    pub miss_count: u64,
    /// 平均命中时间
    pub avg_hit_time_ms: f64,
    /// 平均未命中时间（重新计算）
    pub avg_miss_time_ms: f64,
    /// 缓存大小（字节）
    pub cache_size_bytes: usize,
    /// 命中率
    pub hit_rate: f64,
    /// 效率评分（越高越好）
    pub efficiency_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    CteResult,
    PrecomputedExpression,
    IndexScan,
    ExecutionPlan,
}
```

---

#### 2.2.3 图结构统计 (GraphStructureStatistics)

**需求来源：**
- `traversal_start.rs` 需要了解图拓扑
- `bidirectional_traversal.rs` 需要了解连通性
- 需要识别图的社区结构

**数据结构：**

```rust
/// 图结构统计
#[derive(Debug, Clone)]
pub struct GraphStructureStats {
    /// 连通分量数量
    pub connected_components: u64,
    /// 最大连通分量大小
    pub largest_component_size: u64,
    /// 平均路径长度
    pub avg_path_length: f64,
    /// 聚类系数
    pub clustering_coefficient: f64,
    /// 度分布幂律指数
    pub degree_power_law_exponent: f64,
    /// 社区结构（标签传播结果）
    pub communities: Vec<CommunityInfo>,
}

#[derive(Debug, Clone)]
pub struct CommunityInfo {
    /// 社区ID
    pub community_id: u64,
    /// 社区大小
    pub size: u64,
    /// 内部边数
    pub internal_edges: u64,
    /// 外部边数
    pub external_edges: u64,
    /// 主要标签
    pub dominant_tags: Vec<String>,
}
```

---

### 2.3 低优先级扩展

#### 2.3.1 工作负载特征统计 (WorkloadCharacteristics)

**数据结构：**

```rust
/// 工作负载特征
#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    /// 查询类型分布
    pub query_type_distribution: HashMap<QueryType, f64>,
    /// 访问模式（热点数据）
    pub access_patterns: Vec<AccessPattern>,
    /// 时间模式（峰值时段）
    pub temporal_patterns: Vec<TemporalPattern>,
    /// 资源使用模式
    pub resource_usage_patterns: ResourceUsagePatterns,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    /// 访问的数据区域
    pub data_region: String,
    /// 访问频率
    pub access_frequency: f64,
    /// 关联访问（一起访问的数据）
    pub correlated_accesses: Vec<String>,
}
```

---

## 三、统计信息收集策略

### 3.1 同步收集 vs 异步收集

| 收集方式 | 适用场景 | 优点 | 缺点 |
|---------|---------|------|------|
| 同步收集 | 表达式执行统计 | 实时、准确 | 增加查询延迟 |
| 异步收集 | 大规模数据分析 | 不影响性能 | 有延迟、可能丢失 |
| 采样收集 | 高频操作 | 低开销 | 可能不准确 |

### 3.2 收集触发机制

```rust
/// 统计信息收集触发器
#[derive(Debug, Clone)]
pub enum CollectionTrigger {
    /// 定期收集
    Periodic(Duration),
    /// 数据变化阈值
    DataChangeThreshold { change_ratio: f64 },
    /// 查询次数阈值
    QueryCountThreshold { count: u64 },
    /// 手动触发
    Manual,
    /// 自适应（基于估计误差）
    Adaptive { error_threshold: f64 },
}
```

### 3.3 存储策略

```rust
/// 统计信息存储配置
#[derive(Debug, Clone)]
pub struct StatsStorageConfig {
    /// 内存中保留的统计条目数
    pub in_memory_entries: usize,
    /// 是否持久化到存储
    pub persist_to_storage: bool,
    /// 持久化间隔
    pub persist_interval: Duration,
    /// 压缩旧统计
    pub compress_old_stats: bool,
    /// 保留历史时长
    pub history_retention: Duration,
}
```

---

## 四、实施路线图

### 阶段一：基础扩展（2-3周）

1. **表达式统计信息**
   - 实现 ExpressionStatistics 结构
   - 添加表达式指纹生成
   - 集成到 ExpressionPrecomputationOptimizer

2. **查询执行反馈**
   - 实现 QueryExecutionFeedback 结构
   - 添加执行计划指纹
   - 实现反馈收集机制

### 阶段二：高级统计（3-4周）

1. **数据相关性统计**
   - 实现 PropertyCorrelation 结构
   - 添加相关性计算算法
   - 集成到索引选择器

2. **数据变化趋势**
   - 实现 DataTrend 结构
   - 添加趋势预测算法
   - 实现统计信息新鲜度检查

### 阶段三：优化集成（2-3周）

1. **反馈驱动优化**
   - 实现 FeedbackDrivenSelectivity
   - 添加估计误差修正
   - 集成到所有策略

2. **自适应阈值**
   - 基于反馈调整硬编码阈值
   - 实现阈值学习算法

### 阶段四：高级功能（可选，2-3周）

1. **图结构统计**
2. **缓存效率统计**
3. **工作负载特征分析**

---

## 五、预期收益

### 5.1 性能提升

| 优化项 | 预期提升 | 说明 |
|-------|---------|------|
| 表达式预计算 | 10-30% | 基于实际执行成本决策 |
| 索引选择 | 15-25% | 基于反馈调整选择性估计 |
| 聚合策略 | 10-20% | 基于历史反馈选择策略 |
| 连接顺序 | 20-40% | 基于相关性统计优化 |

### 5.2 可维护性提升

- 减少硬编码阈值
- 提供执行可视化和调试能力
- 支持自适应优化

### 5.3 准确性提升

- 估计误差降低30-50%
- 更好地处理数据倾斜
- 适应数据分布变化

---

*文档生成时间：2026-03-28*
*版本：v1.0*
