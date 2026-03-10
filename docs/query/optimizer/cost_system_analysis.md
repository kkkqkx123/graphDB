# Cost体系分析报告

## 概述

本文档详细分析GraphDB查询优化器中代价计算（Cost）体系的结构、实现机制及实际使用情况。

## 一、目录结构

```
src/query/optimizer/cost/
├── mod.rs                    # 模块入口，统一导出
├── config.rs                 # 代价模型配置（CostModelConfig）
├── calculator.rs             # 代价计算器（CostCalculator）
├── estimate.rs               # 节点代价估算结果（NodeCostEstimate）
├── selectivity.rs            # 选择性估计器（SelectivityEstimator）
├── assigner.rs               # 代价赋值器（CostAssigner）
├── child_accessor.rs         # 子节点访问器（ChildAccessor）
├── expression_parser.rs      # 表达式解析器（ExpressionParser）
└── node_estimators/          # 各类节点估算器
    ├── mod.rs                # 节点估算器trait定义
    ├── scan.rs               # 扫描操作估算器
    ├── graph_traversal.rs    # 图遍历操作估算器
    ├── join.rs               # 连接操作估算器
    ├── sort_limit.rs         # 排序限制操作估算器
    ├── data_processing.rs    # 数据处理节点估算器
    ├── control_flow.rs       # 控制流节点估算器
    └── graph_algorithm.rs    # 图算法节点估算器
```

## 二、核心组件详解

### 2.1 代价模型配置（CostModelConfig）

参考PostgreSQL代价模型设计，针对图数据库特性进行扩展。

#### 基础I/O参数
| 参数名 | 默认值 | 说明 |
|-------|-------|------|
| seq_page_cost | 1.0 | 顺序页读取代价 |
| random_page_cost | 4.0 | 随机页读取代价 |
| cpu_tuple_cost | 0.01 | 行处理CPU代价 |
| cpu_index_tuple_cost | 0.005 | 索引行处理代价 |
| cpu_operator_cost | 0.0025 | 操作符计算代价 |

#### 算法相关参数
| 参数名 | 默认值 | 说明 |
|-------|-------|------|
| hash_build_overhead | 0.1 | 哈希构建开销系数 |
| sort_comparison_cost | 1.0 | 排序比较代价系数 |
| memory_sort_threshold | 10000 | 内存排序阈值（行数） |
| external_sort_page_cost | 2.0 | 外部排序页代价 |

#### 图数据库特有参数
| 参数名 | 默认值 | 说明 |
|-------|-------|------|
| edge_traversal_cost | 0.02 | 边遍历代价 |
| multi_hop_penalty | 1.2 | 多跳遍历每步递增系数 |
| neighbor_lookup_cost | 0.015 | 邻居节点查找代价 |
| effective_cache_pages | 10000 | 有效缓存大小（页数） |
| cache_hit_cost_factor | 0.1 | 缓存命中代价系数 |
| shortest_path_base_cost | 10.0 | 最短路径算法基础代价 |
| path_enumeration_factor | 2.0 | 路径枚举指数系数 |
| super_node_threshold | 10000 | 超级节点阈值（度数） |
| super_node_penalty | 2.0 | 超级节点额外代价系数 |

### 2.2 代价计算器（CostCalculator）

提供各类操作的代价计算方法，主要功能包括：

#### 扫描操作
- `calculate_scan_vertices_cost` - 全表扫描顶点代价
- `calculate_scan_edges_cost` - 全表扫描边代价
- `calculate_index_scan_cost` - 索引扫描代价
- `calculate_edge_index_scan_cost` - 边索引扫描代价

#### 图遍历操作
- `calculate_expand_cost` - 单步扩展代价
- `calculate_expand_all_cost` - 全扩展代价
- `calculate_traverse_cost` - 多步遍历代价
- `calculate_get_neighbors_cost` - 获取邻居节点代价

#### 连接操作
- `calculate_hash_join_cost` - 哈希内连接代价
- `calculate_hash_left_join_cost` - 哈希左连接代价
- `calculate_nested_loop_join_cost` - 嵌套循环连接代价
- `calculate_cross_join_cost` - 交叉连接代价

#### 排序和聚合
- `calculate_sort_cost` - 排序代价（支持Top-N优化）
- `calculate_limit_cost` - Limit代价
- `calculate_topn_cost` - TopN代价
- `calculate_aggregate_cost` - 聚合代价

### 2.3 选择性估计器（SelectivityEstimator）

基于统计信息和启发式规则进行选择性估计。

#### 默认选择性常量
```rust
pub mod defaults {
    pub const EQUALITY: f64 = 0.1;        // 等值查询
    pub const RANGE: f64 = 0.333;         // 范围查询
    pub const COMPARISON: f64 = 0.333;    // 比较查询
    pub const NOT_EQUAL: f64 = 0.9;       // 不等查询
    pub const IS_NULL: f64 = 0.05;        // IS NULL
    pub const IS_NOT_NULL: f64 = 0.95;    // IS NOT NULL
    pub const IN_LIST: f64 = 0.3;         // IN列表
    pub const EXISTS: f64 = 0.5;          // EXISTS
}
```

#### 主要功能
- `estimate_equality_selectivity` - 基于不同值数量估计等值选择性
- `estimate_range_selectivity` - 范围查询选择性
- `estimate_like_selectivity` - 基于LIKE模式的选择性
- `estimate_from_expression` - 从表达式解析选择性

### 2.4 节点估算器架构

采用策略模式，每类节点有专门的估算器，统一实现 `NodeEstimator` trait。

```rust
pub trait NodeEstimator {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError>;
}
```

#### 估算器分类

| 估算器 | 负责节点类型 | 关键功能 |
|-------|-------------|---------|
| ScanEstimator | ScanVertices, ScanEdges, IndexScan, EdgeIndexScan | 基于统计信息估算扫描代价 |
| GraphTraversalEstimator | Expand, ExpandAll, Traverse, GetNeighbors | 基于边度数统计估算遍历代价 |
| JoinEstimator | HashInnerJoin, HashLeftJoin, InnerJoin, LeftJoin, CrossJoin | 基于输入行数估算连接代价 |
| SortLimitEstimator | Sort, Limit, TopN, Aggregate, Dedup, Sample | 基于算法复杂度估算代价 |
| DataProcessingEstimator | Filter, Project, Unwind, DataCollect, Start | 基于操作类型估算处理代价 |
| ControlFlowEstimator | Loop, Select, PassThrough, Argument | 基于控制流逻辑估算代价 |
| GraphAlgorithmEstimator | ShortestPath, AllPaths, MultiShortestPath, BFSShortest | 基于算法复杂度估算代价 |

## 三、Cost体系在优化器中的使用

### 3.1 优化器引擎（OptimizerEngine）

优化器引擎整合所有优化组件，作为全局唯一的优化器实例：

```rust
pub struct OptimizerEngine {
    stats_manager: Arc<StatisticsManager>,
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
    sort_elimination_optimizer: Arc<SortEliminationOptimizer>,
    aggregate_strategy_selector: AggregateStrategySelector,
    // ... 其他组件
}
```

### 3.2 索引选择（IndexSelector）

基于代价选择最优索引：
- 评估不同索引的扫描代价
- 比较索引扫描 vs 全表扫描
- 支持复合索引策略选择

### 3.3 连接顺序优化（JoinOrderOptimizer）

- 动态规划（DP）算法：≤8个表时使用
- 贪心算法：>8个表时使用
- 基于代价选择连接算法（哈希/嵌套循环/索引）

### 3.4 聚合策略选择（AggregateStrategySelector）

基于代价选择聚合策略：
- 哈希聚合（HashAggregate）
- 排序聚合（SortAggregate）
- 流式聚合（StreamingAggregate）

## 四、统计信息体系

### 4.1 统计信息管理器（StatisticsManager）

统一管理所有统计信息，提供线程安全的访问：

```rust
pub struct StatisticsManager {
    tag_stats: Arc<RwLock<HashMap<String, TagStatistics>>>,
    tag_id_to_name: Arc<RwLock<HashMap<i32, String>>>,
    edge_stats: Arc<RwLock<HashMap<String, EdgeTypeStatistics>>>,
    property_stats: Arc<RwLock<HashMap<String, PropertyStatistics>>>,
}
```

### 4.2 统计信息类型

#### 标签统计（TagStatistics）
```rust
pub struct TagStatistics {
    pub tag_name: String,
    pub vertex_count: u64,        // 顶点数量
    pub avg_out_degree: f64,      // 平均出度
    pub avg_in_degree: f64,       // 平均入度
}
```

#### 边类型统计（EdgeTypeStatistics）
```rust
pub struct EdgeTypeStatistics {
    pub edge_type: String,
    pub edge_count: u64,          // 边总数
    pub avg_out_degree: f64,      // 平均出度
    pub avg_in_degree: f64,       // 平均入度
    pub max_out_degree: u64,      // 最大出度
    pub max_in_degree: u64,       // 最大入度
    pub unique_src_vertices: u64, // 唯一源顶点数
}
```

#### 属性统计（PropertyStatistics）
```rust
pub struct PropertyStatistics {
    pub property_name: String,
    pub tag_name: Option<String>,
    pub distinct_values: u64,     // 不同值数量
}
```

## 五、当前局限性

### 5.1 统计信息局限性
1. **无直方图统计** - 只能使用平均选择性，对倾斜数据估计不准
2. **无多列相关性** - 复合条件选择性估计使用独立性假设
3. **无运行时反馈** - 无法根据实际执行调整估计模型

### 5.2 代价模型局限性
1. **固定参数** - 代价参数需要手动调优，无法自适应
2. **无机器学习** - 无法从历史执行中学习更准确的代价模型
3. **简化的IO模型** - 未充分考虑存储引擎的具体特性

### 5.3 优化策略局限性
1. **无双向遍历** - 最短路径查询未使用双向BFS优化
2. **无CTE缓存** - 重复CTE计算无法复用
3. **无自适应算法** - 执行过程中无法动态调整算法

## 六、总结

GraphDB的Cost体系已经具备了良好的基础架构：

1. **模块化设计** - 各组件职责清晰，易于扩展
2. **统计信息支持** - Tag、EdgeType、Property三级统计
3. **多策略支持** - DP/贪心算法、多种连接算法、聚合策略
4. **图数据库特性** - 超级节点处理、多跳惩罚、遍历方向

主要改进方向包括：
- 增强统计信息（直方图、相关性、运行时反馈）
- 图遍历优化（双向BFS、遍历方向智能选择）
- 运行时优化（CTE缓存、自适应算法）
- 高级特性（ML代价模型、向量化执行）
