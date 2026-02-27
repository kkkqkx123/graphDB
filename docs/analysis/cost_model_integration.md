# 代价模型集成方案

## 1. 概述

本文档描述如何将代价模型功能正式集成到优化器层，实现基于代价的查询优化决策。

## 2. 当前架构分析

### 2.1 优化器层组件

```
┌─────────────────────────────────────────────────────────────┐
│                    QueryPipelineManager                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Parser      │→ │ Validator   │→ │ Planner             │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                          ↓                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              DecisionCache (可选)                    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      Optimizer Layer                         │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ StatisticsManager│  │ CostCalculator   │                 │
│  └────────┬─────────┘  └────────┬─────────┘                 │
│           │                     │                            │
│           ↓                     ↓                            │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ SelectivityEst.  │  │ TraversalStart   │                 │
│  │                  │  │ Selector         │                 │
│  └──────────────────┘  └────────┬─────────┘                 │
│                                 │                            │
│                                 ↓                            │
│                        ┌──────────────────┐                  │
│                        │ IndexSelector    │                  │
│                        └──────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 现有集成点

| 组件 | 已集成 | 说明 |
|------|--------|------|
| TraversalStartSelector | ✓ | 使用 CostCalculator 选择最优起点 |
| IndexSelector | ✓ | 使用 CostCalculator 选择最优索引 |
| CostAssigner | ✓ | 为计划节点赋值代价 |
| DecisionCache | 部分 | 缓存决策，但未使用代价模型计算 |
| Planner | 部分 | 有 compute_decision 接口，但返回默认决策 |

### 2.3 缺失的集成

1. **QueryPipelineManager** 未持有 StatisticsManager 和 CostCalculator
2. **Planner.compute_decision** 返回默认决策，未使用代价模型
3. **CostModelConfig** 未在系统级别配置

## 3. 集成方案设计

### 3.1 核心数据流

```
查询文本
    ↓
QueryPipelineManager
    ├── 创建/获取 StatisticsManager
    ├── 创建 CostCalculator (使用 CostModelConfig)
    ├── 检查 DecisionCache
    │       ↓ (缓存未命中)
    │   Planner.compute_decision()
    │       ├── TraversalStartSelector.select_start_node()
    │       │       └── CostCalculator.calculate_*_cost()
    │       └── IndexSelector.select_index()
    │               └── CostCalculator.calculate_index_scan_cost()
    │       ↓
    │   OptimizationDecision
    │       ↓
    │   存入 DecisionCache
    ↓
Planner.transform_with_decision()
    ↓
ExecutionPlan
```

### 3.2 组件职责划分

| 组件 | 职责 | 依赖 |
|------|------|------|
| QueryPipelineManager | 协调查询流程，管理统计信息和代价配置 | StatisticsManager, CostModelConfig |
| StatisticsManager | 管理表/属性统计信息 | 无 |
| CostCalculator | 计算操作代价 | StatisticsManager, CostModelConfig |
| SelectivityEstimator | 估计条件选择性 | StatisticsManager |
| TraversalStartSelector | 选择遍历起点 | CostCalculator, SelectivityEstimator |
| IndexSelector | 选择最优索引 | CostCalculator, SelectivityEstimator |
| DecisionCache | 缓存优化决策 | 无 |
| Planner | 生成执行计划，计算优化决策 | TraversalStartSelector, IndexSelector |

### 3.3 配置传递链

```
系统配置 (config.toml)
    ↓
CostModelConfig
    ↓
QueryPipelineManager
    ↓
CostCalculator
    ↓
TraversalStartSelector / IndexSelector
```

## 4. 接口设计

### 4.1 QueryPipelineManager 扩展

```rust
pub struct QueryPipelineManager<S: StorageClient> {
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
    decision_cache: Option<DecisionCache>,
    
    // 新增字段
    statistics_manager: Arc<StatisticsManager>,
    cost_config: CostModelConfig,
    cost_calculator: Arc<CostCalculator>,
}
```

### 4.2 Planner trait 扩展

```rust
pub trait Planner {
    // 现有方法...
    
    /// 设置代价计算器（新增）
    fn set_cost_calculator(&mut self, calculator: Arc<CostCalculator>);
    
    /// 设置选择性估计器（新增）
    fn set_selectivity_estimator(&mut self, estimator: Arc<SelectivityEstimator>);
}
```

### 4.3 OptimizationDecision 增强

```rust
pub struct OptimizationDecision {
    // 现有字段...
    
    /// 代价模型配置快照（新增）
    pub cost_config_snapshot: CostModelConfig,
    /// 估计的总代价（新增）
    pub estimated_total_cost: f64,
}
```

## 5. 实现步骤

### 5.1 Phase 1: 基础设施集成

1. 在 QueryPipelineManager 中集成 StatisticsManager 和 CostCalculator
2. 添加 CostModelConfig 配置支持
3. 实现配置传递机制

### 5.2 Phase 2: Planner 集成

1. 为 PlannerEnum 添加代价计算器字段
2. 实现 Planner.compute_decision 的实际计算逻辑
3. 在 MatchStatementPlanner 中实现决策计算

### 5.3 Phase 3: 决策缓存集成

1. 在决策缓存中存储代价信息
2. 实现基于代价的缓存淘汰策略
3. 添加版本感知的缓存失效

### 5.4 Phase 4: 测试和验证

1. 添加集成测试
2. 验证代价计算正确性
3. 性能基准测试

## 6. 配置示例

### 6.1 config.toml

```toml
[cost_model]
# 基础 I/O 代价
seq_page_cost = 1.0
random_page_cost = 4.0

# CPU 代价
cpu_tuple_cost = 0.01
cpu_index_tuple_cost = 0.005
cpu_operator_cost = 0.0025

# 图数据库特有
edge_traversal_cost = 0.02
multi_hop_penalty = 1.2
effective_cache_pages = 10000
super_node_threshold = 10000

# 硬件环境: "hdd", "ssd", "memory"
hardware_profile = "ssd"

[decision_cache]
enabled = true
max_entries = 1000
ttl_seconds = 3600
```

### 6.2 代码使用

```rust
// 创建代价模型配置
let cost_config = CostModelConfig::for_ssd();

// 创建统计信息管理器
let stats_manager = Arc::new(StatisticsManager::new());

// 创建代价计算器
let cost_calculator = Arc::new(CostCalculator::with_config(
    stats_manager.clone(),
    cost_config.clone(),
));

// 创建查询管道管理器
let pipeline = QueryPipelineManager::with_cost_config(
    storage,
    stats_manager,
    cost_config,
);
```

## 7. 决策计算流程

### 7.1 遍历起点选择

```rust
fn compute_traversal_start_decision(
    &self,
    pattern: &Pattern,
) -> TraversalStartDecision {
    // 1. 评估所有候选起点
    let candidates = self.traversal_selector.evaluate_pattern(pattern);
    
    // 2. 基于代价选择最优起点
    let best = candidates.into_iter()
        .min_by(|a, b| a.estimated_cost.partial_cmp(&b.estimated_cost).unwrap());
    
    // 3. 构建决策
    match best {
        Some(candidate) => TraversalStartDecision::new(
            candidate.node_pattern.variable.clone(),
            candidate.access_path,
            candidate.estimated_selectivity,
            candidate.estimated_cost,
        ),
        None => default_decision(),
    }
}
```

### 7.2 索引选择

```rust
fn compute_index_decision(
    &self,
    tag_name: &str,
    predicates: &[PropertyPredicate],
) -> IndexSelectionDecision {
    // 1. 获取可用索引
    let indexes = self.get_available_indexes(tag_name);
    
    // 2. 使用 IndexSelector 选择最优索引
    let selection = self.index_selector.select_index(
        tag_name,
        predicates,
        &indexes,
    );
    
    // 3. 构建决策
    IndexSelectionDecision::from(selection)
}
```

## 8. 缓存策略

### 8.1 缓存键构成

```rust
DecisionCacheKey {
    query_template_hash,  // 查询模板哈希
    space_id,             // 图空间 ID
    statement_type,       // 语句类型
    pattern_fingerprint,  // 模式指纹
}
```

### 8.2 缓存失效条件

1. **统计信息变更**：stats_version 不匹配
2. **索引变更**：index_version 不匹配
3. **TTL 过期**：超过配置的存活时间
4. **代价配置变更**：cost_config_version 不匹配

### 8.3 淘汰策略

使用 LRU + 价值分数的混合策略：

```rust
fn value_score(&self) -> f64 {
    let recency = 1.0 / (1.0 + age_secs / 3600.0);
    let frequency = access_count.ln_1p();
    let cost_savings = estimated_total_cost;  // 代价越大的决策越有价值
    
    recency * frequency * (1.0 + cost_savings / 1000.0)
}
```

## 9. 监控和诊断

### 9.1 日志输出

```
[INFO] 代价模型配置: SSD 模式, effective_cache_pages=10000
[INFO] 遍历起点选择: 变量 'n', 代价=15.2, 选择性=0.01
[INFO] 索引选择: Person.name_idx, 代价=5.3, 选择性=0.001
[INFO] 决策缓存命中: key=abc123, 节省计算时间=2.5ms
```

### 9.2 指标收集

```rust
pub struct CostModelMetrics {
    pub decisions_computed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_decision_time_ms: f64,
    pub avg_estimated_cost: f64,
}
```

## 10. 未来扩展

1. **自适应优化**：根据实际执行反馈调整代价参数
2. **机器学习**：使用 ML 模型预测复杂查询代价
3. **并行优化**：支持并行代价计算
4. **分布式**：添加网络传输代价
