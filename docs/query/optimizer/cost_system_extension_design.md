# Cost体系扩展设计方案

## 概述

本文档详细描述GraphDB查询优化器Cost体系的扩展设计方案，包括各优化策略的价值评估、开销分析和详细实现设计。

## 一、优化策略价值与开销评估矩阵

| 优化策略 | 价值等级 | 实现复杂度 | 运行时开销 | 存储开销 | 必要性评分 | 推荐阶段 |
|---------|---------|-----------|-----------|---------|-----------|---------|
| **P0 - 核心优化** |
| 直方图统计 | ⭐⭐⭐⭐⭐ | 中 | 低 | 中 | 95 | 第一阶段 |
| 双向BFS遍历 | ⭐⭐⭐⭐⭐ | 中 | 低 | 无 | 90 | 第一阶段 |
| 运行时统计反馈 | ⭐⭐⭐⭐ | 中 | 低 | 低 | 85 | 第二阶段 |
| **P1 - 重要优化** |
| 数据倾斜检测 | ⭐⭐⭐⭐ | 低 | 低 | 低 | 80 | 第一阶段 |
| CTE结果缓存 | ⭐⭐⭐⭐ | 中 | 中 | 高 | 75 | 第二阶段 |
| 自适应连接算法 | ⭐⭐⭐⭐ | 高 | 中 | 中 | 75 | 第三阶段 |
| **P2 - 增强优化** |
| 相关性统计 | ⭐⭐⭐ | 高 | 中 | 高 | 65 | 第三阶段 |
| 表达式常量折叠 | ⭐⭐⭐ | 低 | 极低 | 无 | 60 | 第一阶段 |
| 遍历方向智能选择 | ⭐⭐⭐ | 低 | 极低 | 无 | 60 | 第一阶段 |
| **P3 - 未来优化** |
| ML代价模型 | ⭐⭐⭐⭐ | 极高 | 高 | 高 | 50 | 第四阶段 |
| 向量化执行决策 | ⭐⭐⭐ | 高 | 中 | 中 | 45 | 第四阶段 |

## 二、第一阶段：基础增强

### 2.1 直方图统计系统

#### 价值分析
- 当前选择性估计使用固定默认值（等值0.1，范围0.333）
- 对于倾斜数据，误差可能达到10-100倍
- 直方图可将选择性估计误差控制在2倍以内

#### 存储开销评估
```
假设：
- 每个属性维护100个桶的等深直方图
- 每个桶存储：(边界值8字节 + 频率8字节) = 16字节
- 每个属性直方图 = 100 * 16 = 1.6KB

对于一个中等规模数据库：
- 100个标签 × 平均10个属性 = 1000个属性
- 总存储 = 1000 × 1.6KB = 1.6MB
- 内存缓存开销 ≈ 3-5MB（可接受）
```

#### 实现设计

```rust
// src/query/optimizer/stats/histogram.rs
//! 直方图统计模块
//! 
//! 使用等深直方图（equi-depth histogram）记录属性值分布
//! 每个直方图包含固定数量的桶，每个桶记录相同数量的元组

use crate::core::Value;

/// 直方图桶
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    /// 桶上界（包含）
    pub upper_bound: Value,
    /// 桶内元组数量
    pub count: u64,
    /// 不同值数量（NDV）
    pub distinct_values: u64,
}

/// 等深直方图
#[derive(Debug, Clone)]
pub struct Histogram {
    /// 桶列表（按上界排序）
    buckets: Vec<HistogramBucket>,
    /// 空值比例
    null_fraction: f64,
    /// 总不同值数量
    total_distinct_values: u64,
    /// 最后更新时间
    last_updated: std::time::Instant,
}

impl Histogram {
    /// 创建直方图（从采样数据构建）
    pub fn from_samples(samples: Vec<Value>, num_buckets: usize) -> Self {
        // 实现等深直方图构建算法
        // 每个桶包含大致相同数量的样本
        todo!()
    }
    
    /// 估计等值查询选择性
    pub fn estimate_equality_selectivity(&self, value: &Value) -> f64 {
        // 找到包含该值的桶
        // 使用桶内均匀分布假设：1 / 桶内NDV
        todo!()
    }
    
    /// 估计范围查询选择性
    pub fn estimate_range_selectivity(&self, range: &RangeCondition) -> f64 {
        // 计算覆盖的完整桶 + 部分桶的估计
        todo!()
    }
}

// 在PropertyStatistics中集成直方图
pub struct PropertyStatistics {
    pub property_name: String,
    pub tag_name: Option<String>,
    pub distinct_values: u64,
    /// 可选的直方图（高基数属性启用）
    pub histogram: Option<Histogram>,
    /// 是否适合使用直方图（低基数属性不需要）
    pub use_histogram: bool,
}
```

#### 更新策略
- 异步更新：后台任务定期采样更新
- 触发条件：数据变化超过10%或超过1小时未更新
- 采样率：大表使用1-5%采样，小表全量统计

### 2.2 数据倾斜检测

#### 价值分析
- 超级节点（如"明星用户"）会导致遍历性能急剧下降
- 当前仅有`super_node_threshold`阈值，缺乏动态检测

#### 实现设计

```rust
// 扩展 EdgeTypeStatistics
pub struct EdgeTypeStatistics {
    pub edge_type: String,
    pub edge_count: u64,
    pub avg_out_degree: f64,
    pub avg_in_degree: f64,
    pub max_out_degree: u64,
    pub max_in_degree: u64,
    pub unique_src_vertices: u64,
    
    // 新增：倾斜度指标
    /// 度数标准差（衡量分布离散程度）
    pub out_degree_std_dev: f64,
    /// 基尼系数（0-1，越大越倾斜）
    pub degree_gini_coefficient: f64,
    /// 热点顶点列表（Top-K高度数顶点）
    pub hot_vertices: Vec<HotVertexInfo>,
}

pub struct HotVertexInfo {
    pub vertex_id: i64,
    pub out_degree: u64,
    pub in_degree: u64,
}

/// 倾斜度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkewnessLevel {
    None,       // 无倾斜
    Mild,       // 轻度倾斜
    Moderate,   // 中度倾斜
    Severe,     // 严重倾斜
}

/// 倾斜检测算法
impl EdgeTypeStatistics {
    /// 判断是否存在严重倾斜
    pub fn is_heavily_skewed(&self) -> bool {
        // 基尼系数 > 0.5 认为存在严重倾斜
        self.degree_gini_coefficient > 0.5 
            || self.max_out_degree as f64 > self.avg_out_degree * 10.0
    }
    
    /// 获取倾斜度等级
    pub fn skewness_level(&self) -> SkewnessLevel {
        match self.degree_gini_coefficient {
            g if g > 0.7 => SkewnessLevel::Severe,
            g if g > 0.5 => SkewnessLevel::Moderate,
            g if g > 0.3 => SkewnessLevel::Mild,
            _ => SkewnessLevel::None,
        }
    }
}
```

#### 优化策略应用
```rust
// 在 GraphTraversalEstimator 中
fn estimate_expand_cost(&self, start_nodes: u64, edge_type: Option<&str>) -> f64 {
    let stats = edge_type.and_then(|et| self.stats_manager.get_edge_stats(et));
    
    match stats {
        Some(s) if s.is_heavily_skewed() => {
            // 倾斜数据使用更保守的估计
            // 考虑使用采样或限制遍历深度
            self.calculate_skewed_expand_cost(start_nodes, s)
        }
        Some(s) => self.calculate_normal_expand_cost(start_nodes, s),
        None => self.calculate_default_expand_cost(start_nodes),
    }
}
```

### 2.3 双向BFS遍历优化

#### 价值分析
- 传统BFS从单点出发，搜索空间随深度指数增长
- 双向BFS同时从起点和终点搜索，可将复杂度从O(b^d)降到O(b^(d/2))
- 对于最短路径查询，性能提升可达10-100倍

#### 实现设计

```rust
// src/query/optimizer/strategy/bidirectional_traversal.rs
//! 双向遍历优化器

use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::stats::StatisticsManager;

/// 双向遍历决策
#[derive(Debug, Clone)]
pub struct BidirectionalDecision {
    /// 是否使用双向遍历
    pub use_bidirectional: bool,
    /// 正向搜索起点
    pub forward_start: String,
    /// 反向搜索起点
    pub backward_start: String,
    /// 预计减少的搜索空间比例
    pub estimated_savings: f64,
}

pub struct BidirectionalTraversalOptimizer {
    cost_calculator: Arc<CostCalculator>,
}

impl BidirectionalTraversalOptimizer {
    /// 评估是否适合双向遍历
    pub fn evaluate(
        &self,
        start_node: &str,
        end_node: &str,
        edge_types: &[String],
        max_depth: u32,
    ) -> BidirectionalDecision {
        // 获取起点和终点的度数估计
        let start_cardinality = self.estimate_reachable_nodes(start_node, max_depth / 2);
        let end_cardinality = self.estimate_reachable_nodes(end_node, max_depth / 2);
        
        // 双向搜索空间 = 2 * b^(d/2)
        // 单向搜索空间 = b^d
        let bidirectional_cost = 2.0 * (start_cardinality + end_cardinality);
        let unidirectional_cost = self.estimate_reachable_nodes(start_node, max_depth);
        
        let savings = 1.0 - (bidirectional_cost / unidirectional_cost);
        
        BidirectionalDecision {
            use_bidirectional: savings > 0.3 && max_depth >= 2, // 节省30%以上且深度>=2
            forward_start: start_node.to_string(),
            backward_start: end_node.to_string(),
            estimated_savings: savings,
        }
    }
}
```

### 2.4 表达式常量折叠

#### 价值分析
- 编译期优化，零运行时开销
- 简化复杂表达式，减少执行时计算

#### 实现设计

```rust
// 在 expression_parser.rs 中扩展
impl ExpressionParser {
    /// 尝试折叠常量表达式
    pub fn fold_constants(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { op, left, right } => {
                let folded_left = self.fold_constants(left);
                let folded_right = self.fold_constants(right);
                
                // 如果两边都是常量，直接计算结果
                if let (Expression::Literal(l), Expression::Literal(r)) = (&folded_left, &folded_right) {
                    return self.evaluate_binary_op(op, l, r);
                }
                
                Expression::Binary {
                    op: op.clone(),
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                }
            }
            Expression::Function { name, args } => {
                let folded_args: Vec<_> = args.iter()
                    .map(|a| self.fold_constants(a))
                    .collect();
                
                // 如果所有参数都是常量，尝试计算
                if folded_args.iter().all(|a| matches!(a, Expression::Literal(_))) {
                    if let Some(result) = self.evaluate_function(name, &folded_args) {
                        return Expression::Literal(result);
                    }
                }
                
                Expression::Function {
                    name: name.clone(),
                    args: folded_args,
                }
            }
            _ => expr.clone(),
        }
    }
}
```

### 2.5 遍历方向智能选择

#### 价值分析
- 图遍历可以从出边或入边方向进行
- 选择度数较小的方向可以显著减少搜索空间

#### 实现设计

```rust
// src/query/optimizer/strategy/traversal_direction.rs
//! 遍历方向优化器

pub struct TraversalDirectionOptimizer {
    stats_manager: Arc<StatisticsManager>,
}

impl TraversalDirectionOptimizer {
    /// 选择最优遍历方向
    pub fn select_direction(
        &self,
        edge_type: &str,
        preferred_direction: EdgeDirection,
    ) -> EdgeDirection {
        let stats = self.stats_manager.get_edge_stats(edge_type);
        
        match stats {
            Some(s) => {
                let out_cost = s.avg_out_degree;
                let in_cost = s.avg_in_degree;
                
                match preferred_direction {
                    EdgeDirection::Out if out_cost > in_cost * 2.0 => {
                        // 出度远大于入度，考虑反向遍历
                        if in_cost < 10.0 {
                            EdgeDirection::In
                        } else {
                            EdgeDirection::Out
                        }
                    }
                    EdgeDirection::In if in_cost > out_cost * 2.0 => {
                        // 入度远大于出度，考虑正向遍历
                        if out_cost < 10.0 {
                            EdgeDirection::Out
                        } else {
                            EdgeDirection::In
                        }
                    }
                    _ => preferred_direction,
                }
            }
            None => preferred_direction,
        }
    }
}
```

## 三、第二阶段：运行时优化

### 3.1 运行时统计反馈

#### 价值分析
- 静态统计可能过时，导致优化决策失误
- 运行时反馈可动态调整估计模型

#### 低开销设计方案

```rust
// src/query/optimizer/stats/feedback.rs
//! 运行时统计反馈模块

use std::sync::atomic::{AtomicU64, Ordering};

/// 轻量级执行反馈收集器
pub struct ExecutionFeedbackCollector {
    /// 实际输出行数（原子计数器）
    actual_rows: AtomicU64,
    /// 执行时间（微秒）
    execution_time_us: AtomicU64,
}

/// 反馈驱动的选择性校正
pub struct FeedbackDrivenSelectivity {
    /// 原始估计选择性
    estimated_selectivity: f64,
    /// 历史实际选择性（滑动窗口平均）
    actual_selectivity_ewma: f64,
    /// 校正因子
    correction_factor: f64,
}

impl FeedbackDrivenSelectivity {
    /// 获取校正后的选择性
    pub fn corrected_selectivity(&self) -> f64 {
        self.estimated_selectivity * self.correction_factor
    }
    
    /// 更新校正因子（根据新反馈）
    pub fn update_with_feedback(&mut self, actual_selectivity: f64) {
        // 使用指数加权移动平均
        let alpha = 0.3; // 新反馈权重
        self.correction_factor = (1.0 - alpha) * self.correction_factor 
            + alpha * (actual_selectivity / self.estimated_selectivity);
        
        // 限制校正因子范围，避免过度校正
        self.correction_factor = self.correction_factor.clamp(0.1, 10.0);
    }
}
```

### 3.2 CTE结果缓存

#### 价值分析
- 递归CTE和复杂子查询可被多次引用
- 缓存可避免重复计算

#### 存储与淘汰策略

```rust
// src/query/optimizer/strategy/cte_cache.rs
//! CTE结果缓存管理器

pub struct CteCacheManager {
    /// 缓存条目（LRU淘汰）
    cache: LruCache<String, CteCacheEntry>,
    /// 最大缓存大小（字节）
    max_size: usize,
    /// 当前使用大小
    current_size: usize,
}

pub struct CteCacheEntry {
    /// 结果数据
    data: Arc<DataSet>,
    /// 估计重用概率
    reuse_probability: f64,
    /// 创建时间
    created_at: Instant,
    /// 访问次数
    access_count: u64,
}

impl CteCacheManager {
    /// 决定是否缓存CTE结果
    pub fn should_cache(&self, cte_definition: &str, estimated_rows: u64) -> bool {
        // 小结果集不值得缓存（缓存开销 > 计算开销）
        if estimated_rows < 100 {
            return false;
        }
        
        // 大结果集可能超出内存限制
        let estimated_size = estimated_rows * 64; // 假设每行64字节
        if estimated_size > self.max_size / 10 {
            return false;
        }
        
        // 检查历史重用模式
        self.predict_reuse_probability(cte_definition) > 0.5
    }
}
```

## 四、第三阶段：高级优化

### 4.1 自适应连接算法

#### 价值分析
- 运行时数据特征可能与估计不符
- 动态切换算法可应对数据倾斜

#### 实现复杂度
高（需要执行引擎支持）

```rust
/// 自适应连接执行器
pub enum AdaptiveJoinExecutor {
    /// 初始使用哈希连接
    HashJoin(HashJoinState),
    /// 检测到倾斜后切换到Graceful Degradation
    GraceHashJoin(GraceHashState),
    /// 小数据量回退到嵌套循环
    NestedLoopJoin(NLJoinState),
}

impl AdaptiveJoinExecutor {
    /// 执行过程中监控性能
    pub fn execute(&mut self, input: DataChunk) -> Result<DataChunk> {
        match self {
            Self::HashJoin(state) => {
                if state.detect_skew() {
                    // 动态切换到Grace Hash Join
                    *self = Self::GraceHashJoin(state.convert_to_grace());
                }
                state.process(input)
            }
            // ...
        }
    }
}
```

### 4.2 多列相关性统计

#### 存储开销
```
对于N列，两两相关性需要 N*(N-1)/2 个相关系数
100属性 × 100属性 = 约5K个系数
存储 ≈ 5K × 8字节 = 40KB（每对属性）
```

#### 适用场景
- 复合索引选择
- 多条件选择性估计

## 五、第四阶段：未来优化

### 5.1 ML代价模型

使用机器学习模型预测查询执行代价，替代固定的代价公式。

**特点**：
- 需要大量历史执行数据训练
- 实现复杂度高
- 需要持续维护和更新模型

### 5.2 向量化执行决策

根据数据特征选择行式或列式执行策略。

## 六、关键设计决策

### 6.1 统计信息更新策略

| 策略 | 优点 | 缺点 | 推荐场景 |
|-----|------|------|---------|
| 同步更新 | 数据一致性 | 影响写入性能 | 小表、关键表 |
| 异步后台更新 | 不影响写入 | 数据可能滞后 | 大表、非关键表 |
| 按需采样 | 精确度高 | 首次查询慢 | 复杂查询 |
| 增量更新 | 开销小 | 实现复杂 | 频繁更新的表 |

**推荐方案**：
- 小表（<10万行）：同步全量更新
- 中表（10万-1000万行）：异步采样更新（5%采样）
- 大表（>1000万行）：异步分区采样更新

### 6.2 内存管理策略

```rust
/// 统计信息内存预算管理
pub struct StatsMemoryBudget {
    /// 总预算（MB）
    total_budget_mb: usize,
    /// 直方图预算比例
    histogram_ratio: f64,
    /// 缓存预算比例
    cache_ratio: f64,
}

impl StatsMemoryBudget {
    /// 根据预算自动调整直方图精度
    pub fn adjust_histogram_buckets(&self, num_properties: usize) -> usize {
        let histogram_budget = (self.total_budget_mb as f64 * self.histogram_ratio) as usize;
        let bytes_per_histogram = 1024; // 1KB per histogram
        let max_histograms = histogram_budget * 1024 / bytes_per_histogram;
        
        // 动态调整桶数量
        if num_properties <= max_histograms {
            100 // 标准精度
        } else {
            // 减少桶数以适应预算
            (max_histograms * 100 / num_properties).max(10)
        }
    }
}
```

### 6.3 向后兼容性

所有扩展都遵循**可选增强**原则：
- 新统计信息字段使用 `Option<T>`
- 缺失统计时回退到现有启发式估计
- 配置项控制功能开关

```rust
pub struct OptimizerConfig {
    /// 启用直方图统计
    pub enable_histogram: bool,
    /// 启用运行时反馈
    pub enable_runtime_feedback: bool,
    /// 启用CTE缓存
    pub enable_cte_cache: bool,
    // ...
}
```

## 七、总结

本设计方案提供了完整的Cost体系扩展规划：

1. **第一阶段**（基础增强）：直方图统计、数据倾斜检测、双向BFS、表达式折叠、遍历方向选择
2. **第二阶段**（运行时优化）：运行时反馈、CTE缓存
3. **第三阶段**（高级优化）：自适应连接、相关性统计
4. **第四阶段**（未来优化）：ML代价模型、向量化执行

每个阶段都有明确的价值评估和开销分析，可根据实际需求和资源情况选择实施。
