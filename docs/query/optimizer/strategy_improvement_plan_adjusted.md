# 查询优化策略改进方案（调整后）

## 概述

基于阶段1的修复经验和对实际需求的重新评估，本文档对原改进方案进行了调整，去除过度设计，保留核心改进。

---

## 一、阶段1修复总结（已完成）

### 1.1 traversal_direction.rs - 成本计算逻辑修复 ✅
**问题：** `select_by_cost` 中 `forward_cost` 和 `backward_cost` 使用相同参数计算

**修复：**
- 新增 `calculate_cost_with_degree` 方法，根据度数计算成本
- 前向和后向使用各自的度数（out_degree/in_degree）计算成本
- 添加超级节点检测逻辑

### 1.2 memory_budget.rs - NodeID修复 ✅
**问题：** 使用指针地址作为NodeID，节点移动后失效

**修复：**
- 使用 `plan.id()` 方法获取节点的唯一ID（i64转u64）
- 这是轻量级修复，不需要修改PlanNodeEnum结构

### 1.3 expression_precomputation.rs - 确定性检查修复 ✅
**问题：** 对未知表达式类型默认返回true（过于乐观）

**修复：**
- 对未知类型返回false（保守策略）
- 添加更多表达式类型的处理（Property、TypeCast、Subscript等）
- 添加debug日志记录未知类型

---

## 二、剩余阶段调整说明

### 2.1 原方案的过度设计分析

| 原方案内容 | 问题 | 调整决策 |
|-----------|------|---------|
| 完整的表达式统计系统 | 实现复杂，维护成本高，收益有限 | 简化为轻量级执行计数 |
| 复杂的查询反馈系统 | 需要大量基础设施支持 | 延迟到后续版本 |
| 数据相关性统计 | 计算开销大，实时性要求高 | 仅保留基础的相关性提示 |
| 图结构统计（社区发现等） | 算法复杂，与当前优化器集成度低 | 暂不实现 |
| 完整的工作负载分析 | 需要长期数据收集 | 暂不实现 |

### 2.2 调整原则

1. **YAGNI原则**：只实现当前确实需要的功能
2. **KISS原则**：保持简单，避免过度抽象
3. **渐进式改进**：小步快跑，快速验证
4. **成本效益**：投入产出比要合理

---

## 三、调整后阶段2：轻量级配置化（1周）

### 3.1 目标
将关键的硬编码阈值提取为可配置参数，但不引入复杂的配置系统。

### 3.2 实现方案

```rust
// 在CostModelConfig中添加策略相关配置
#[derive(Debug, Clone)]
pub struct CostModelConfig {
    // 已有字段...
    
    // 新增：策略阈值配置
    pub strategy_thresholds: StrategyThresholds,
}

#[derive(Debug, Clone)]
pub struct StrategyThresholds {
    // 聚合策略
    pub small_dataset_threshold: u64,      // 默认：1000
    pub low_cardinality_threshold: u64,    // 默认：100
    
    // 遍历策略
    pub super_node_threshold: f64,         // 默认：1000.0
    pub bidirectional_savings_threshold: f64, // 默认：0.3
    
    // TopN策略
    pub topn_threshold: f64,               // 默认：0.1
    
    // 物化策略
    pub max_result_rows: u64,              // 默认：10000
    pub min_reference_count: usize,        // 默认：2
}

impl Default for StrategyThresholds {
    fn default() -> Self {
        Self {
            small_dataset_threshold: 1000,
            low_cardinality_threshold: 100,
            super_node_threshold: 1000.0,
            bidirectional_savings_threshold: 0.3,
            topn_threshold: 0.1,
            max_result_rows: 10000,
            min_reference_count: 2,
        }
    }
}
```

### 3.3 修改范围

只需修改以下文件：
1. `src/query/optimizer/cost/config.rs` - 添加配置结构
2. `aggregate_strategy.rs` - 使用配置替换硬编码值
3. `traversal_direction.rs` - 使用配置替换硬编码值
4. `topn_optimization.rs` - 使用配置替换硬编码值
5. `materialization.rs` - 使用配置替换硬编码值

---

## 四、调整后阶段3：轻量级统计扩展（1周）

### 4.1 目标
添加最基础的统计信息扩展，支持更准确的基数估计。

### 4.2 实现方案

#### 4.2.1 属性组合统计（简化版）

```rust
/// 轻量级属性组合统计
#[derive(Debug, Clone)]
pub struct PropertyCombinationStats {
    /// 属性组合键（如 "tag.prop1.prop2"）
    pub key: String,
    /// 联合不同值数量
    pub combined_distinct_values: u64,
    /// 样本数量
    pub sample_count: u64,
    /// 最后更新时间
    pub last_updated: Instant,
}

impl StatisticsManager {
    /// 获取属性组合的联合基数
    pub fn get_combined_cardinality(
        &self,
        tag_name: &str,
        properties: &[String],
    ) -> Option<u64> {
        let key = format!("{}.{}", tag_name, properties.join("."));
        self.property_combo_stats
            .get(&key)
            .map(|s| s.combined_distinct_values)
    }
}
```

#### 4.2.2 简单的执行反馈

```rust
/// 轻量级执行反馈
#[derive(Debug, Clone)]
pub struct SimpleExecutionFeedback {
    /// 查询模式指纹（简化）
    pub query_pattern: String,
    /// 估计行数
    pub estimated_rows: u64,
    /// 实际行数
    pub actual_rows: u64,
    /// 估计误差
    pub estimation_error: f64,
    /// 执行次数
    pub execution_count: u64,
}

impl StatisticsManager {
    /// 记录执行反馈
    pub fn record_feedback(&self, feedback: SimpleExecutionFeedback) {
        // 使用滑动窗口平均更新估计误差
        if let Some(mut existing) = self.feedback_map.get_mut(&feedback.query_pattern) {
            let total_execs = existing.execution_count + feedback.execution_count;
            existing.estimated_rows = (existing.estimated_rows * existing.execution_count 
                + feedback.estimated_rows * feedback.execution_count) / total_execs;
            existing.actual_rows = (existing.actual_rows * existing.execution_count 
                + feedback.actual_rows * feedback.execution_count) / total_execs;
            existing.estimation_error = (existing.estimation_error * existing.execution_count as f64
                + feedback.estimation_error * feedback.execution_count as f64) / total_execs as f64;
            existing.execution_count = total_execs;
        } else {
            self.feedback_map.insert(feedback.query_pattern.clone(), feedback);
        }
    }
    
    /// 获取平均估计误差
    pub fn get_avg_estimation_error(&self, query_pattern: &str) -> Option<f64> {
        self.feedback_map.get(query_pattern).map(|f| f.estimation_error)
    }
}
```

### 4.3 应用场景

```rust
// aggregate_strategy.rs - 改进基数估计
fn estimate_group_by_cardinality(&self, context: &AggregateContext) -> u64 {
    // 尝试使用属性组合统计
    if let Some(combined_card) = self.stats_manager.get_combined_cardinality(
        context.table_name.as_deref().unwrap_or(""),
        &context.group_keys,
    ) {
        return combined_card.min(context.input_rows).max(1);
    }
    
    // 回退到启发式估计
    self.heuristic_cardinality_estimate(context)
}
```

---

## 五、移除的内容及原因

### 5.1 移除：复杂的表达式统计系统

**原因：**
- 实现需要修改表达式执行引擎
- 维护成本高（需要跟踪每个表达式的执行）
- 当前优化器对表达式成本的估计已经足够准确

**替代方案：**
- 使用简单的执行反馈来校正估计误差

### 5.2 移除：完整的查询反馈系统

**原因：**
- 需要大量基础设施（计划指纹、执行跟踪等）
- 与当前架构集成复杂
- 收益与投入不成比例

**替代方案：**
- 仅保留简单的行数估计反馈

### 5.3 移除：图结构统计（社区发现等）

**原因：**
- 算法复杂（需要实现图算法）
- 与当前优化策略集成度低
- 对大多数查询影响不大

**替代方案：**
- 保留现有的热点顶点检测
- 后续根据实际需求再考虑添加

### 5.4 移除：自适应阈值系统

**原因：**
- 需要大量历史数据支持
- 算法复杂，调参困难
- 可配置阈值已经足够

**替代方案：**
- 提供可配置的阈值
- 文档说明推荐的配置值

---

## 六、调整后实施计划

### 阶段2：轻量级配置化（1周）

| 任务 | 文件 | 工作量 | 说明 |
|-----|------|--------|------|
| 添加配置结构 | cost/config.rs | 4小时 | 添加StrategyThresholds |
| 修改AggregateStrategy | aggregate_strategy.rs | 4小时 | 使用配置阈值 |
| 修改TraversalStrategy | traversal_direction.rs, bidirectional_traversal.rs | 4小时 | 使用配置阈值 |
| 修改其他策略 | topn_optimization.rs, materialization.rs | 4小时 | 使用配置阈值 |
| 测试 | - | 8小时 | 验证配置生效 |

### 阶段3：轻量级统计扩展（1周）

| 任务 | 文件 | 工作量 | 说明 |
|-----|------|--------|------|
| 添加属性组合统计 | stats/property.rs | 8小时 | 简化版实现 |
| 添加执行反馈 | stats/manager.rs | 8小时 | 轻量级反馈 |
| 集成到策略 | aggregate_strategy.rs | 4小时 | 改进基数估计 |
| 测试 | - | 8小时 | 验证统计生效 |

### 总计：2周（原方案6周）

---

## 七、预期收益对比

| 改进项 | 原方案预期 | 调整后预期 | 说明 |
|-------|-----------|-----------|------|
| 性能提升 | 10-40% | 5-15% | 保守估计，更实际 |
| 可维护性 | 高 | 中 | 简化设计，降低维护成本 |
| 实现风险 | 高 | 低 | 改动范围小，易于验证 |
| 投入时间 | 6周 | 2周 | 大幅减少 |

---

## 八、后续建议

### 8.1 监控指标

实施后需要监控以下指标：
1. 查询执行时间变化
2. 估计误差率
3. 内存使用稳定性
4. 配置参数的使用情况

### 8.2 后续优化方向

根据实际运行情况，考虑以下方向：
1. 如果估计误差仍然较大，再考虑更复杂的反馈系统
2. 如果特定查询模式性能不佳，针对性优化
3. 根据用户反馈，调整默认配置值

### 8.3 文档维护

- 记录配置参数的最佳实践
- 维护已知问题和解决方案
- 定期回顾优化效果

---

*调整日期：2026-03-28*
*调整原因：去除过度设计，聚焦核心改进*
