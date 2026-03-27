# 基于新代价估算的优化规则分析

## 概述

本文档基于新的代价估算功能（表达式计算成本、内存使用估算、数据类型成本系数），分析可以补充的优化规则。

## 现有优化策略回顾

### 1. 聚合策略选择器 (AggregateStrategySelector)
**当前功能**：
- 在 HashAggregate、SortAggregate、StreamingAggregate 之间选择
- 基于输入行数、分组键数量、内存限制做决策
- 已有 `estimated_memory_bytes` 字段

**可补充的优化规则**：

#### 1.1 内存压力感知聚合策略选择
```rust
// 基于新的 memory_pressure_threshold 和 memory_pressure_penalty
pub fn select_strategy_with_memory_pressure(&self, context: &AggregateContext) -> AggregateStrategyDecision {
    let base_decision = self.select_strategy(context);
    
    // 如果估计内存超过阈值，重新评估
    if base_decision.estimated_memory_bytes > self.cost_calculator.config().memory_pressure_threshold as u64 {
        // 优先选择 SortAggregate 或 StreamingAggregate
        // 因为 HashAggregate 需要更多内存
        let hash_cost = self.calculate_hash_aggregate_cost_with_penalty(context);
        let sort_cost = self.calculate_sort_aggregate_cost(context);
        
        if sort_cost < hash_cost * 1.5 { // 允许 50% 的性能损失换取内存安全
            return AggregateStrategyDecision {
                strategy: AggregateStrategy::SortAggregate,
                // ...
                reason: SelectionReason::MemoryConstrained,
            };
        }
    }
    base_decision
}
```

#### 1.2 表达式复杂度感知的聚合策略
```rust
// 利用 expression_cost 评估聚合函数的复杂度
pub fn analyze_aggregation_complexity(&self, agg_functions: &[ContextualExpression]) -> f64 {
    agg_functions.iter()
        .map(|f| self.cost_calculator.calculate_expression_cost(f))
        .sum()
}

// 复杂聚合函数倾向于 HashAggregate（减少重复计算）
// 简单聚合函数可以使用 SortAggregate
```

### 2. 连接顺序优化器 (JoinOrderOptimizer)
**当前功能**：
- 动态规划 (DP) 和贪心算法选择最优连接顺序
- 基于表大小、选择性、索引信息

**可补充的优化规则**：

#### 2.1 内存感知连接顺序
```rust
// 基于内存估算选择连接策略
pub fn optimize_join_order_with_memory(
    &self,
    tables: &[TableInfo],
    conditions: &[JoinCondition],
) -> JoinOrderResult {
    // 估算每种连接顺序的内存使用
    let memory_estimates: Vec<usize> = tables.iter()
        .map(|t| self.estimate_join_memory(t.estimated_rows))
        .collect();
    
    // 如果内存使用超过阈值，优先选择 NestedLoopJoin
    // 因为 HashJoin 需要构建哈希表
    let memory_threshold = self.cost_calculator.config().memory_pressure_threshold;
    
    // 修改代价计算，加入内存惩罚
    let adjusted_cost = |cost: f64, memory: usize| -> f64 {
        self.cost_calculator.calculate_memory_aware_cost(cost, memory)
    };
    
    // ... 使用调整后的代价进行 DP 或贪心算法
}
```

#### 2.2 表达式复杂度感知的连接条件排序
```rust
// 优先执行简单的连接条件（减少中间结果）
pub fn sort_join_conditions_by_complexity(&self, conditions: &mut [JoinCondition]) {
    conditions.sort_by(|a, b| {
        let cost_a = a.expression.as_ref()
            .map(|e| self.cost_calculator.calculate_expression_cost(e))
            .unwrap_or(0.0);
        let cost_b = b.expression.as_ref()
            .map(|e| self.cost_calculator.calculate_expression_cost(e))
            .unwrap_or(0.0);
        cost_a.partial_cmp(&cost_b).unwrap()
    });
}
```

### 3. CTE 物化优化器 (MaterializationOptimizer)
**当前功能**：
- 基于引用次数、结果集大小、表达式复杂度决策

**可补充的优化规则**：

#### 3.1 内存成本感知的物化决策
```rust
// 考虑物化的内存成本 vs 重新计算的 CPU 成本
pub fn should_materialize_with_memory_cost(
    &self,
    cte_node: &PlanNodeEnum,
    plan_root: &PlanNodeEnum,
) -> MaterializationDecision {
    let base_decision = self.should_materialize(cte_node, plan_root);
    
    if let MaterializationDecision::Materialize { estimated_rows, .. } = &base_decision {
        // 估算物化所需的内存
        let materialize_memory = estimated_rows * 64; // 假设每行 64 字节
        
        // 如果内存成本过高，可能选择不物化
        let memory_cost = materialize_memory as f64 * self.config.memory_byte_cost;
        let recompute_cost = self.estimate_recompute_cost(cte_node);
        
        if memory_cost > recompute_cost * 0.5 { // 内存成本超过重算成本的 50%
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::MemoryCostTooHigh,
            };
        }
    }
    base_decision
}
```

### 4. TopN 优化器 (SortEliminationOptimizer)
**当前功能**：
- Sort + Limit 转换为 TopN 的决策

**可补充的优化规则**：

#### 4.1 内存感知的 TopN 阈值调整
```rust
// 根据可用内存动态调整 TopN 转换阈值
pub fn optimize_with_memory_context(&self, context: &SortContext, available_memory: usize) -> SortEliminationDecision {
    let sort_memory = self.cost_calculator.estimate_sort_memory(context.input_rows, 
        context.sort_node.sort_items.len());
    
    // 如果排序内存需求超过可用内存，强制使用 TopN（外部排序）
    if sort_memory > available_memory {
        return SortEliminationDecision::ConvertToTopN {
            reason: TopNConversionReason::MemoryConstrained,
            // ...
        };
    }
    
    self.optimize(context)
}
```

### 5. 遍历方向优化器 (TraversalDirectionOptimizer)
**当前功能**：
- 基于出度/入度选择遍历方向
- 考虑超级节点

**可补充的优化规则**：

#### 5.1 内存感知的双向遍历深度分配
```rust
// 基于内存限制优化深度分配
pub fn calculate_depth_allocation_with_memory(
    &self,
    context: &DepthAllocationContext,
    memory_limit: usize,
) -> (u32, u32) {
    let (forward_depth, backward_depth) = self.calculate_depth_allocation(context);
    
    // 估算双向遍历的内存使用
    let memory_estimate = self.estimate_bidirectional_memory(
        context.start_degree, 
        context.end_degree,
        forward_depth,
        backward_depth,
    );
    
    // 如果超过内存限制，减少深度
    if memory_estimate > memory_limit {
        let ratio = (memory_limit as f64 / memory_estimate as f64).sqrt();
        (
            (forward_depth as f64 * ratio) as u32,
            (backward_depth as f64 * ratio) as u32,
        )
    } else {
        (forward_depth, backward_depth)
    }
}
```

## 新增优化策略建议

### 6. 表达式预计算优化器 (ExpressionPrecomputationOptimizer)
**新策略**：基于表达式成本系数，决定哪些表达式应该预计算

```rust
pub struct ExpressionPrecomputationOptimizer {
    cost_calculator: Arc<CostCalculator>,
    // 预计算阈值：表达式成本超过此值时考虑预计算
    precompute_threshold: f64,
}

impl ExpressionPrecomputationOptimizer {
    /// 分析表达式是否应该预计算
    pub fn should_precompute(&self, expr: &Expression, reference_count: usize) -> bool {
        let expression_cost = self.cost_calculator.calculate_expression_cost(expr);
        
        // 如果表达式复杂且被多次引用，预计算更划算
        let precompute_benefit = expression_cost * reference_count as f64;
        let precompute_cost = expression_cost + self.config.precompute_overhead;
        
        precompute_benefit > precompute_cost * 2.0 // 收益是成本的 2 倍以上
    }
}
```

### 7. 数据类型优化器 (DataTypeOptimizer)
**新策略**：基于数据类型成本系数优化执行策略

```rust
pub struct DataTypeOptimizer {
    cost_calculator: Arc<CostCalculator>,
}

impl DataTypeOptimizer {
    /// 分析计划中数据类型的复杂度
    pub fn analyze_type_complexity(&self, plan: &PlanNodeEnum) -> TypeComplexityAnalysis {
        // 遍历计划树，统计各类型使用频率
        // 返回复杂度评分和建议
    }
    
    /// 为复杂类型（如图类型）选择专用执行策略
    pub fn select_execution_strategy_for_types(&self, plan: &PlanNodeEnum) -> ExecutionStrategy {
        let complexity = self.analyze_type_complexity(plan);
        
        if complexity.graph_type_ratio > 0.5 {
            // 主要使用图类型，选择图专用算子
            ExecutionStrategy::GraphOptimized
        } else if complexity.variable_type_ratio > 0.3 {
            // 大量变长类型，优化内存分配策略
            ExecutionStrategy::VariableLengthOptimized
        } else {
            ExecutionStrategy::Standard
        }
    }
}
```

### 8. 内存预算分配器 (MemoryBudgetAllocator)
**新策略**：基于内存压力阈值，为查询的不同阶段分配内存预算

```rust
pub struct MemoryBudgetAllocator {
    config: CostModelConfig,
    total_budget: usize,
}

impl MemoryBudgetAllocator {
    /// 为计划树的每个节点分配内存预算
    pub fn allocate_budget(&self, plan: &PlanNodeEnum) -> HashMap<NodeId, usize> {
        let mut allocations = HashMap::new();
        
        // 1. 估算每个节点的内存需求
        let requirements = self.estimate_memory_requirements(plan);
        
        // 2. 如果总需求超过预算，按比例缩减
        let total_required: usize = requirements.values().sum();
        if total_required > self.total_budget {
            let scale = self.total_budget as f64 / total_required as f64;
            for (node_id, req) in requirements {
                allocations.insert(node_id, (req as f64 * scale) as usize);
            }
        } else {
            allocations = requirements;
        }
        
        allocations
    }
    
    /// 根据内存预算选择算子实现
    pub fn select_operator_implementation(
        &self,
        node: &PlanNodeEnum,
        budget: usize,
    ) -> OperatorImplementation {
        match node {
            PlanNodeEnum::Sort(_) => {
                if budget < 1024 * 1024 { // 小于 1MB
                    OperatorImplementation::ExternalSort
                } else {
                    OperatorImplementation::InMemorySort
                }
            }
            PlanNodeEnum::HashJoin(_) => {
                if budget < 10 * 1024 * 1024 { // 小于 10MB
                    OperatorImplementation::NestedLoopJoin
                } else {
                    OperatorImplementation::HashJoin
                }
            }
            // ... 其他算子
        }
    }
}
```

### 9. 自适应代价优化器 (AdaptiveCostOptimizer)
**新策略**：根据运行时统计动态调整代价参数

```rust
pub struct AdaptiveCostOptimizer {
    cost_calculator: Arc<CostCalculator>,
    feedback_collector: Arc<FeedbackCollector>,
}

impl AdaptiveCostOptimizer {
    /// 根据反馈调整代价参数
    pub fn adapt_cost_parameters(&mut self) {
        let feedback = self.feedback_collector.get_recent_feedback();
        
        for record in feedback {
            let estimated_cost = record.estimated_cost;
            let actual_cost = record.actual_cost;
            let ratio = actual_cost / estimated_cost;
            
            // 如果估计误差超过阈值，调整相应参数
            if ratio > 2.0 || ratio < 0.5 {
                self.adjust_parameters_for_node(&record.node_type, ratio);
            }
        }
    }
    
    /// 调整特定类型节点的代价参数
    fn adjust_parameters_for_node(&mut self, node_type: &str, error_ratio: f64) {
        match node_type {
            "Filter" => {
                // 调整 expression_cost 相关参数
                self.config.simple_expression_cost *= error_ratio.sqrt();
            }
            "HashJoin" => {
                // 调整内存相关参数
                self.config.memory_byte_cost *= error_ratio.sqrt();
            }
            // ...
        }
    }
}
```

## 实施优先级建议

### 高优先级（立即实施）
1. **内存压力感知聚合策略选择** - 直接影响大查询的稳定性
2. **内存感知的 TopN 阈值调整** - 防止排序操作内存溢出
3. **内存预算分配器** - 整体内存管理的基础

### 中优先级（近期实施）
4. **表达式预计算优化器** - 提升复杂查询性能
5. **内存成本感知的物化决策** - 优化 CTE 使用
6. **自适应代价优化器** - 长期性能优化

### 低优先级（远期考虑）
7. **数据类型优化器** - 针对特定数据类型的优化
8. **表达式复杂度感知的连接条件排序** - 细粒度优化

## 总结

新的代价估算功能为优化器提供了更丰富的决策信息：

- **表达式成本** 使优化器能够评估计算复杂度，支持预计算决策和条件排序
- **内存使用估算** 使优化器能够避免内存压力，支持内存感知策略选择
- **数据类型成本系数** 使优化器能够针对不同类型的数据选择专用策略

这些新能力可以显著提升优化器在处理复杂查询、大数据量和资源受限场景下的表现。
