# 查询优化策略改进方案

## 概述

本文档基于策略分析和统计信息扩展方案，提供具体的代码修改方案，包括高优先级缺陷修复和中长期改进计划。

---

## 一、高优先级缺陷修复

### 1.1 修复 traversal_direction.rs 成本计算逻辑错误

**问题：** `select_by_cost` 方法中 `forward_cost` 和 `backward_cost` 计算使用相同参数

**修复方案：**

```rust
// 当前代码（有bug）
fn select_by_cost(&self, context: &DirectionContext, out_degree: f64, in_degree: f64) 
    -> TraversalDirectionDecision {
    let forward_cost = self.calculate_cost(context, false);
    let backward_cost = self.calculate_cost(context, false); // 错误：相同参数
    // ...
}

// 修复后代码
fn select_by_cost(&self, context: &DirectionContext, out_degree: f64, in_degree: f64) 
    -> TraversalDirectionDecision {
    // 创建前向和后向的上下文
    let mut forward_context = context.clone();
    forward_context.estimated_degree = out_degree;
    
    let mut backward_context = context.clone();
    backward_context.estimated_degree = in_degree;
    
    let forward_cost = self.calculate_cost_with_degree(&forward_context, out_degree);
    let backward_cost = self.calculate_cost_with_degree(&backward_context, in_degree);
    
    let (direction, avg_degree) = if forward_cost <= backward_cost {
        (TraversalDirection::Forward, out_degree)
    } else {
        (TraversalDirection::Backward, in_degree)
    };
    
    TraversalDirectionDecision {
        direction,
        estimated_output_rows: (context.start_nodes as f64 * avg_degree) as u64,
        estimated_cost: forward_cost.min(backward_cost),
        reason: DirectionSelectionReason::CostBased {
            forward_cost,
            backward_cost,
        },
        avg_degree,
        involves_super_node: out_degree > self.super_node_threshold || 
                            in_degree > self.super_node_threshold,
    }
}

// 新增方法
fn calculate_cost_with_degree(&self, context: &DirectionContext, degree: f64) -> f64 {
    let base_cost = self.cost_calculator
        .calculate_expand_cost(context.start_nodes, Some(&context.edge_type));
    
    // 根据度数调整成本
    let degree_factor = if degree > self.super_node_threshold {
        self.cost_calculator.config().super_node_penalty
    } else {
        1.0 + (degree / self.super_node_threshold).ln_1p()
    };
    
    base_cost * degree_factor
}
```

---

### 1.2 修复 memory_budget.rs NodeID 问题

**问题：** 使用指针地址作为NodeID，在计划节点被移动后会失效

**修复方案：**

```rust
// 新增：为PlanNode添加唯一ID生成
pub struct PlanNodeIdGenerator {
    counter: AtomicU64,
}

impl PlanNodeIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }
    
    pub fn next_id(&self) -> NodeId {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

// 修改 MemoryBudgetAllocator
pub struct MemoryBudgetAllocator {
    total_budget: usize,
    config: CostModelConfig,
    default_row_size: usize,
    // 新增：节点ID到内存需求的映射
    node_id_generator: Arc<PlanNodeIdGenerator>,
}

impl MemoryBudgetAllocator {
    /// 为计划节点分配唯一ID（应在计划构建时调用）
    pub fn assign_node_ids(&self, plan: &mut PlanNodeEnum) {
        self.assign_ids_recursive(plan);
    }
    
    fn assign_ids_recursive(&self, plan: &mut PlanNodeEnum) {
        // 为当前节点分配ID
        let id = self.node_id_generator.next_id();
        plan.set_node_id(id); // 需要在PlanNodeEnum中添加set_node_id方法
        
        // 递归处理子节点
        for child in self.get_children_mut(plan) {
            self.assign_ids_recursive(child);
        }
    }
    
    /// 使用节点ID而非指针地址
    fn get_node_id(&self, plan: &PlanNodeEnum) -> NodeId {
        plan.node_id().unwrap_or_else(|| {
            // 如果节点没有ID，生成一个临时ID（警告：这不是最佳实践）
            self.node_id_generator.next_id()
        })
    }
}

// 在 PlanNodeEnum 中添加方法（需要修改 plan 模块）
impl PlanNodeEnum {
    pub fn node_id(&self) -> Option<NodeId> {
        match self {
            PlanNodeEnum::Sort(n) => n.node_id(),
            PlanNodeEnum::Filter(n) => n.node_id(),
            // ... 其他变体
            _ => None,
        }
    }
    
    pub fn set_node_id(&mut self, id: NodeId) {
        match self {
            PlanNodeEnum::Sort(n) => n.set_node_id(id),
            PlanNodeEnum::Filter(n) => n.set_node_id(id),
            // ... 其他变体
            _ => {}
        }
    }
}
```

---

### 1.3 修复 expression_precomputation.rs 对未知类型的处理

**问题：** `check_expression_deterministic` 对未知类型默认返回true，过于乐观

**修复方案：**

```rust
// 当前代码
_ => true, // Conservative: assume deterministic for unknown types

// 修复后代码
_ => {
    // 对于未知类型，保守地假设非确定性
    // 记录警告以便后续完善
    log::warn!(
        "Unknown expression type encountered in determinism check: {:?}", 
        expr
    );
    false // 保守：假设非确定性
}
```

同时添加更完整的表达式类型处理：

```rust
// 添加更多表达式类型的处理
fn check_expression_deterministic(&self, expr: &Expression) -> bool {
    match expr {
        // 已有处理...
        
        // 新增：处理参数表达式
        Expression::Parameter(_) => {
            // 参数在单次查询中是确定的
            true
        }
        
        // 新增：处理标签表达式
        Expression::Label(_) => true,
        
        // 新增：处理路径构建
        Expression::PathBuild(exprs) => {
            exprs.iter().all(|e| self.check_expression_deterministic(e))
        }
        
        // 新增：处理类型转换
        Expression::TypeCast { expression, .. } => {
            self.check_expression_deterministic(expression)
        }
        
        // 新增：处理下标访问
        Expression::Subscript { collection, index } => {
            self.check_expression_deterministic(collection) &&
            self.check_expression_deterministic(index)
        }
        
        // 新增：处理范围表达式
        Expression::Range { collection, start, end } => {
            self.check_expression_deterministic(collection) &&
            start.as_ref().map_or(true, |e| self.check_expression_deterministic(e)) &&
            end.as_ref().map_or(true, |e| self.check_expression_deterministic(e))
        }
        
        // 新增：处理列表推导
        Expression::ListComprehension { source, filter, map, .. } => {
            self.check_expression_deterministic(source) &&
            filter.as_ref().map_or(true, |e| self.check_expression_deterministic(e)) &&
            map.as_ref().map_or(true, |e| self.check_expression_deterministic(e))
        }
        
        // 新增：处理Reduce表达式
        Expression::Reduce { initial, source, mapping, .. } => {
            self.check_expression_deterministic(initial) &&
            self.check_expression_deterministic(source) &&
            self.check_expression_deterministic(mapping)
        }
        
        // 保守处理未知类型
        _ => {
            log::warn!(
                "Unknown expression type in determinism check, assuming non-deterministic: {:?}",
                std::mem::discriminant(expr)
            );
            false
        }
    }
}
```

---

## 二、中优先级改进

### 2.1 聚合策略基数估计改进

**问题：** `estimate_cardinality_quick` 使用固定除数，未考虑实际数据分布

**改进方案：**

```rust
// 新增：基于统计信息的基数估计
pub struct CardinalityEstimator {
    stats_manager: Arc<StatisticsManager>,
}

impl CardinalityEstimator {
    /// 改进的基数估计
    pub fn estimate_group_by_cardinality(
        &self,
        input_rows: u64,
        group_keys: &[String],
        table_name: Option<&str>,
    ) -> u64 {
        if group_keys.is_empty() {
            return 1;
        }
        
        // 尝试获取属性统计
        let mut total_distinct = 1u64;
        let mut has_stats = false;
        
        for key in group_keys {
            if let Some(stats) = self.stats_manager.get_property_stats(table_name, key) {
                has_stats = true;
                // 使用实际的不同值数量
                total_distinct = total_distinct.saturating_mul(stats.distinct_values.max(1));
            }
        }
        
        if has_stats {
            // 使用统计信息，但不超过输入行数
            total_distinct.min(input_rows).max(1)
        } else {
            // 回退到启发式估计，但使用更智能的公式
            self.heuristic_estimate(input_rows, group_keys.len())
        }
    }
    
    /// 启发式估计（改进版）
    fn heuristic_estimate(&self, input_rows: u64, key_count: usize) -> u64 {
        // 使用对数缩放而非指数缩放
        // 假设：每个键将基数减少到原来的 1/sqrt(2)
        let reduction_factor = (key_count as f64).sqrt();
        let estimated = (input_rows as f64 / reduction_factor) as u64;
        
        // 确保在合理范围内
        estimated.clamp(1, input_rows)
    }
}

// 在 AggregateStrategySelector 中使用
impl AggregateStrategySelector {
    pub fn select_strategy(&self, context: &AggregateContext) -> AggregateStrategyDecision {
        // 使用改进的基数估计
        let group_by_cardinality = if let Some(table_name) = &context.table_name {
            self.cardinality_estimator.estimate_group_by_cardinality(
                context.input_rows,
                &context.group_keys,
                Some(table_name),
            )
        } else {
            self.estimate_group_by_cardinality(context) // 回退到原方法
        };
        
        // ... 后续逻辑
    }
}
```

---

### 2.2 索引选择性估计改进

**问题：** LIKE操作符使用固定选择性0.3，未考虑模式特征

**改进方案：**

```rust
// 新增：LIKE模式分析器
pub struct LikePatternAnalyzer;

impl LikePatternAnalyzer {
    /// 分析LIKE模式并估计选择性
    pub fn analyze_pattern(pattern: &str) -> LikeSelectivity {
        // 前缀匹配（如 'abc%'）通常有较低选择性
        if pattern.ends_with('%') && !pattern[..pattern.len()-1].contains('%') {
            let prefix_len = pattern.len() - 1;
            return LikeSelectivity::Prefix {
                selectivity: 0.1 / prefix_len as f64,
                prefix: pattern[..prefix_len].to_string(),
            };
        }
        
        // 后缀匹配（如 '%abc'）
        if pattern.starts_with('%') && !pattern[1..].contains('%') {
            return LikeSelectivity::Suffix {
                selectivity: 0.3,
            };
        }
        
        // 包含匹配（如 '%abc%'）
        if pattern.starts_with('%') && pattern.ends_with('%') {
            let inner = &pattern[1..pattern.len()-1];
            if !inner.contains('%') {
                return LikeSelectivity::Contains {
                    selectivity: 0.5,
                    substring: inner.to_string(),
                };
            }
        }
        
        // 复杂模式
        let wildcard_count = pattern.matches('%').count() + pattern.matches('_').count();
        LikeSelectivity::Complex {
            selectivity: (0.3 + wildcard_count as f64 * 0.1).min(0.9),
            wildcard_count,
        }
    }
}

pub enum LikeSelectivity {
    Prefix { selectivity: f64, prefix: String },
    Suffix { selectivity: f64 },
    Contains { selectivity: f64, substring: String },
    Complex { selectivity: f64, wildcard_count: usize },
}

// 在 IndexSelector 中使用
impl IndexSelector {
    fn evaluate_index(&self, index: &Index, predicates: &[PropertyPredicate]) 
        -> Option<IndexSelection> {
        // ...
        
        for predicate in &covered_predicates {
            let selectivity = match predicate.operator {
                PredicateOperator::Like => {
                    if let Expression::Literal(Value::String(pattern)) = &predicate.value {
                        let analysis = LikePatternAnalyzer::analyze_pattern(pattern);
                        analysis.selectivity()
                    } else {
                        0.3 // 回退到默认值
                    }
                }
                // ... 其他操作符
            };
            total_selectivity *= selectivity;
        }
        
        // ...
    }
}
```

---

### 2.3 子查询解关联改进

**问题：** `is_simple_subquery` 仅支持Scan和Filter，限制过严

**改进方案：**

```rust
// 扩展简单子查询的定义
fn is_simple_subquery(&self, node: &PlanNodeEnum) -> SimplicityScore {
    match node {
        PlanNodeEnum::ScanVertices(_) => SimplicityScore::simple(1.0),
        
        PlanNodeEnum::Filter(n) => {
            let condition = n.condition();
            let analysis = self.expression_analyzer.analyze(condition);
            
            if !analysis.is_deterministic {
                return SimplicityScore::not_simple();
            }
            
            // 递归检查输入
            let input_score = self.is_simple_subquery(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)
            );
            
            // 根据复杂度调整分数
            input_score.with_penalty(analysis.complexity_score as f64 * 0.1)
        }
        
        // 新增：支持简单投影
        PlanNodeEnum::Project(n) => {
            let input_score = self.is_simple_subquery(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)
            );
            input_score.with_penalty(0.2) // 投影增加少量复杂度
        }
        
        // 新增：支持简单Limit
        PlanNodeEnum::Limit(n) => {
            let limit = n.count();
            if limit <= 1000 { // 小limit仍然简单
                let input_score = self.is_simple_subquery(
                    crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)
                );
                input_score.with_penalty(0.1)
            } else {
                SimplicityScore::not_simple()
            }
        }
        
        // 新增：支持简单排序（小数据量）
        PlanNodeEnum::Sort(n) => {
            let input_score = self.is_simple_subquery(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)
            );
            // 排序增加较多复杂度
            input_score.with_penalty(0.5)
        }
        
        // 不支持的其他情况
        _ => SimplicityScore::not_simple(),
    }
}

/// 简单性评分
pub struct SimplicityScore {
    is_simple: bool,
    score: f64, // 0.0 - 1.0, 越高越简单
}

impl SimplicityScore {
    pub fn simple(score: f64) -> Self {
        Self { is_simple: true, score: score.clamp(0.0, 1.0) }
    }
    
    pub fn not_simple() -> Self {
        Self { is_simple: false, score: 0.0 }
    }
    
    pub fn with_penalty(self, penalty: f64) -> Self {
        if self.is_simple {
            let new_score = (self.score - penalty).max(0.0);
            Self {
                is_simple: new_score > 0.3, // 阈值
                score: new_score,
            }
        } else {
            self
        }
    }
}
```

---

## 三、配置化改进

### 3.1 提取硬编码阈值为配置

创建统一的策略配置结构：

```rust
/// 优化策略配置
#[derive(Debug, Clone)]
pub struct OptimizationStrategyConfig {
    /// 聚合策略配置
    pub aggregate: AggregateStrategyConfig,
    /// 索引选择配置
    pub index: IndexSelectionConfig,
    /// 连接顺序配置
    pub join_order: JoinOrderConfig,
    /// 遍历配置
    pub traversal: TraversalConfig,
    /// TopN配置
    pub topn: TopNConfig,
    /// 物化配置
    pub materialization: MaterializationConfig,
    /// 子查询配置
    pub subquery: SubqueryConfig,
    /// 内存预算配置
    pub memory_budget: MemoryBudgetConfig,
}

#[derive(Debug, Clone)]
pub struct AggregateStrategyConfig {
    /// 小数据集阈值（行数）
    pub small_dataset_threshold: u64,
    /// 低基数阈值
    pub low_cardinality_threshold: u64,
    /// 高基数比例（相对于输入行数）
    pub high_cardinality_ratio: f64,
    /// 内存受限时的性能损失容忍度
    pub memory_pressure_tolerance: f64,
}

impl Default for AggregateStrategyConfig {
    fn default() -> Self {
        Self {
            small_dataset_threshold: 1000,
            low_cardinality_threshold: 100,
            high_cardinality_ratio: 0.1,
            memory_pressure_tolerance: 1.5, // 允许50%性能损失
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraversalConfig {
    /// 超级节点阈值
    pub super_node_threshold: f64,
    /// 度数相等阈值（差异小于此值视为相等）
    pub degree_equality_threshold: f64,
    /// 双向遍历节省阈值
    pub bidirectional_savings_threshold: f64,
    /// 默认分支因子
    pub default_branching_factor: f64,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            super_node_threshold: 1000.0,
            degree_equality_threshold: 0.1,
            bidirectional_savings_threshold: 0.3,
            default_branching_factor: 2.0,
        }
    }
}

// 其他配置结构...

// 在策略中使用配置
pub struct AggregateStrategySelector {
    cost_calculator: Arc<CostCalculator>,
    expression_analyzer: ExpressionAnalyzer,
    expression_context: Arc<ExpressionAnalysisContext>,
    config: AggregateStrategyConfig, // 新增
}

impl AggregateStrategySelector {
    pub fn with_config(mut self, config: AggregateStrategyConfig) -> Self {
        self.config = config;
        self
    }
    
    fn select_strategy_internal(&self, context: &AggregateContext) -> AggregateStrategyDecision {
        // 使用配置而非硬编码值
        if context.input_rows < self.config.small_dataset_threshold {
            return self.create_hash_aggregate_decision(context, SelectionReason::SmallDataSet);
        }
        
        let group_by_cardinality = self.estimate_group_by_cardinality(context);
        
        if group_by_cardinality < self.config.low_cardinality_threshold {
            // ...
        }
        
        // ...
    }
}
```

---

## 四、反馈机制集成

### 4.1 策略决策反馈收集

```rust
/// 策略决策记录
#[derive(Debug, Clone)]
pub struct StrategyDecisionRecord {
    /// 策略名称
    pub strategy_name: String,
    /// 决策上下文
    pub context: DecisionContext,
    /// 做出的决策
    pub decision: String,
    /// 估计成本
    pub estimated_cost: f64,
    /// 实际成本（执行后填充）
    pub actual_cost: Option<f64>,
    /// 决策时间戳
    pub decided_at: Instant,
}

/// 策略反馈收集器
trait StrategyFeedbackCollector {
    /// 记录决策
    fn record_decision(&self, record: StrategyDecisionRecord);
    
    /// 更新实际成本
    fn update_actual_cost(&self, decision_id: u64, actual_cost: f64);
    
    /// 获取历史决策统计
    fn get_decision_statistics(&self, strategy_name: &str) -> DecisionStatistics;
}

// 在策略中使用
impl AggregateStrategySelector {
    pub fn select_strategy(&self, context: &AggregateContext) -> AggregateStrategyDecision {
        let decision = self.select_strategy_internal(context);
        
        // 记录决策
        if let Some(feedback_collector) = &self.feedback_collector {
            feedback_collector.record_decision(StrategyDecisionRecord {
                strategy_name: "AggregateStrategy".to_string(),
                context: DecisionContext::Aggregate(context.clone()),
                decision: format!("{:?}", decision.strategy),
                estimated_cost: decision.estimated_cost,
                actual_cost: None,
                decided_at: Instant::now(),
            });
        }
        
        decision
    }
}
```

---

## 五、实施计划

### 阶段一：缺陷修复（1周）

| 任务 | 文件 | 工作量 | 优先级 |
|-----|------|--------|--------|
| 修复traversal_direction成本计算 | traversal_direction.rs | 4小时 | P0 |
| 修复memory_budget NodeID问题 | memory_budget.rs, plan nodes | 8小时 | P0 |
| 修复expression_precomputation确定性检查 | expression_precomputation.rs | 4小时 | P0 |

### 阶段二：配置化（1周）

| 任务 | 文件 | 工作量 | 优先级 |
|-----|------|--------|--------|
| 创建策略配置结构 | strategy/config.rs（新建） | 8小时 | P1 |
| 修改AggregateStrategy使用配置 | aggregate_strategy.rs | 4小时 | P1 |
| 修改TraversalStrategy使用配置 | traversal_direction.rs, bidirectional_traversal.rs | 4小时 | P1 |
| 修改其他策略使用配置 | 其他策略文件 | 8小时 | P1 |

### 阶段三：统计信息集成（2周）

| 任务 | 文件 | 工作量 | 优先级 |
|-----|------|--------|--------|
| 实现表达式统计 | stats/expression.rs（新建） | 16小时 | P1 |
| 实现查询反馈统计 | stats/feedback.rs（新建） | 16小时 | P1 |
| 集成反馈到策略 | 各策略文件 | 16小时 | P1 |

### 阶段四：高级改进（2周）

| 任务 | 文件 | 工作量 | 优先级 |
|-----|------|--------|--------|
| 改进基数估计 | aggregate_strategy.rs | 8小时 | P2 |
| 改进LIKE选择性 | index.rs | 8小时 | P2 |
| 扩展简单子查询定义 | subquery_unnesting.rs | 8小时 | P2 |
| 添加数据相关性统计 | stats/correlation.rs（新建） | 16小时 | P2 |

---

## 六、测试计划

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_traversal_direction_cost_calculation() {
        let optimizer = create_test_optimizer();
        let context = create_test_context();
        
        let decision = optimizer.select_by_cost(&context, 5.0, 50.0);
        
        // 验证：度数小的方向应该被选择
        assert_eq!(decision.direction, TraversalDirection::Forward);
        assert!(decision.estimated_cost > 0.0);
    }
    
    #[test]
    fn test_memory_budget_node_id_stability() {
        let allocator = MemoryBudgetAllocator::new(100 * 1024 * 1024);
        let mut plan = create_test_plan();
        
        // 分配ID
        allocator.assign_node_ids(&mut plan);
        
        // 验证ID稳定
        let id1 = allocator.get_node_id(&plan);
        let id2 = allocator.get_node_id(&plan);
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_expression_determinism_conservative() {
        let optimizer = ExpressionPrecomputationOptimizer::new(cost_calculator);
        
        // 未知表达式类型应该被视为非确定性
        let unknown_expr = Expression::UnknownVariant;
        let is_deterministic = optimizer.check_expression_deterministic(&unknown_expr);
        assert!(!is_deterministic);
    }
}
```

### 6.2 集成测试

```rust
#[test]
fn test_query_execution_with_feedback() {
    let mut optimizer = OptimizerEngine::new();
    let feedback_collector = Arc::new(FeedbackCollector::new());
    optimizer.set_feedback_collector(feedback_collector.clone());
    
    // 执行查询
    let query = "SELECT * FROM Person WHERE age > 25";
    let plan = optimizer.optimize(query);
    execute_plan(plan);
    
    // 验证反馈被收集
    let stats = feedback_collector.get_statistics();
    assert!(stats.decision_count > 0);
    assert!(stats.avg_estimation_error < 0.5); // 估计误差应该在合理范围内
}
```

---

*文档生成时间：2026-03-28*
*版本：v1.0*
