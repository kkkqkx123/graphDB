//! 聚合策略选择器模块
//!
//! 基于代价的聚合策略选择，在哈希聚合和排序聚合之间选择最优策略
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::AggregateStrategySelector;
//! use graphdb::query::optimizer::cost::CostCalculator;
//! use std::sync::Arc;
//!
//! let selector = AggregateStrategySelector::new(cost_calculator);
//! let decision = selector.select_strategy(
//!     input_rows,
//!     group_keys,
//!     agg_functions,
//!     memory_limit,
//! );
//! ```

use std::sync::Arc;

use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::decision::OptimizationDecision;
use crate::query::optimizer::analysis::ExpressionAnalyzer;

/// 聚合策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateStrategy {
    /// 哈希聚合 - 使用 HashMap 存储分组状态
    /// 适用于：分组键基数高、内存充足
    HashAggregate,
    /// 排序聚合 - 先排序再聚合
    /// 适用于：输入已排序或接近排序、内存受限
    SortAggregate,
    /// 流式聚合 - 输入已排序时的优化聚合
    /// 适用于：输入数据已经按分组键排序
    StreamingAggregate,
}

impl AggregateStrategy {
    /// 获取策略名称
    pub fn name(&self) -> &'static str {
        match self {
            AggregateStrategy::HashAggregate => "HashAggregate",
            AggregateStrategy::SortAggregate => "SortAggregate",
            AggregateStrategy::StreamingAggregate => "StreamingAggregate",
        }
    }
}

/// 聚合策略决策
#[derive(Debug, Clone)]
pub struct AggregateStrategyDecision {
    /// 选择的聚合策略
    pub strategy: AggregateStrategy,
    /// 估计的输出行数
    pub estimated_output_rows: u64,
    /// 估计的代价
    pub estimated_cost: f64,
    /// 估计的内存使用（字节）
    pub estimated_memory_bytes: u64,
    /// 选择原因
    pub reason: SelectionReason,
}

/// 策略选择原因
#[derive(Debug, Clone)]
pub enum SelectionReason {
    /// 输入已排序，使用流式聚合
    InputAlreadySorted,
    /// 分组键基数高，哈希聚合更优
    HighCardinality,
    /// 分组键基数低，排序聚合更优
    LowCardinality,
    /// 内存受限，选择排序聚合
    MemoryConstrained,
    /// 小数据量，哈希聚合更简单
    SmallDataSet,
    /// 大数据量，排序聚合避免内存溢出
    LargeDataSet,
    /// 基于代价计算的选择
    CostBased {
        hash_cost: f64,
        sort_cost: f64,
    },
}

/// 聚合策略选择器
///
/// 基于代价模型选择最优的聚合执行策略
#[derive(Debug)]
pub struct AggregateStrategySelector {
    cost_calculator: Arc<CostCalculator>,
    /// 表达式分析器，用于分析聚合表达式的特性
    expression_analyzer: ExpressionAnalyzer,
}

/// 聚合策略选择的上下文信息
#[derive(Debug, Clone)]
pub struct AggregateContext {
    /// 输入行数
    pub input_rows: u64,
    /// 分组键列表
    pub group_keys: Vec<String>,
    /// 聚合函数数量
    pub agg_function_count: usize,
    /// 内存限制（字节，0表示无限制）
    pub memory_limit: u64,
    /// 输入数据是否已排序
    pub input_is_sorted: bool,
    /// 排序键是否与分组键匹配
    pub sort_keys_match_group_keys: bool,
    /// 聚合表达式是否确定性
    pub is_deterministic: bool,
    /// 聚合表达式复杂度评分
    pub complexity_score: u32,
}

impl AggregateContext {
    /// 创建新的聚合上下文
    pub fn new(
        input_rows: u64,
        group_keys: Vec<String>,
        agg_function_count: usize,
    ) -> Self {
        Self {
            input_rows,
            group_keys,
            agg_function_count,
            memory_limit: 0,
            input_is_sorted: false,
            sort_keys_match_group_keys: false,
            is_deterministic: true,
            complexity_score: 0,
        }
    }

    /// 设置内存限制
    pub fn with_memory_limit(mut self, memory_limit: u64) -> Self {
        self.memory_limit = memory_limit;
        self
    }

    /// 设置输入已排序
    pub fn with_sorted_input(mut self, sort_keys_match: bool) -> Self {
        self.input_is_sorted = true;
        self.sort_keys_match_group_keys = sort_keys_match;
        self
    }

    /// 设置表达式特性
    pub fn with_expression_analysis(mut self, is_deterministic: bool, complexity_score: u32) -> Self {
        self.is_deterministic = is_deterministic;
        self.complexity_score = complexity_score;
        self
    }
}

impl AggregateStrategySelector {
    /// 创建新的聚合策略选择器
    pub fn new(cost_calculator: Arc<CostCalculator>) -> Self {
        Self {
            cost_calculator,
            expression_analyzer: ExpressionAnalyzer::new(),
        }
    }

    /// 创建带表达式分析器的聚合策略选择器
    pub fn with_analyzer(
        cost_calculator: Arc<CostCalculator>,
        expression_analyzer: ExpressionAnalyzer,
    ) -> Self {
        Self {
            cost_calculator,
            expression_analyzer,
        }
    }

    /// 分析聚合表达式并创建上下文
    ///
    /// 使用表达式分析器分析聚合表达式的特性，创建完整的聚合上下文
    pub fn analyze_and_create_context(
        &self,
        input_rows: u64,
        group_keys: Vec<String>,
        agg_function_count: usize,
        expressions: &[crate::core::Expression],
    ) -> AggregateContext {
        let mut context = AggregateContext::new(input_rows, group_keys, agg_function_count);

        // 分析所有聚合表达式
        for expr in expressions {
            let analysis = self.expression_analyzer.analyze(expr);
            if !analysis.is_deterministic {
                context.is_deterministic = false;
            }
            context.complexity_score += analysis.complexity_score;
        }

        context
    }

    /// 选择最优聚合策略
    ///
    /// # 参数
    /// - `context`: 聚合操作的上下文信息
    ///
    /// # 返回
    /// 聚合策略决策，包含选择的策略和估计代价
    pub fn select_strategy(&self, context: &AggregateContext) -> AggregateStrategyDecision {
        // 如果输入已排序且排序键匹配分组键，优先使用流式聚合
        if context.input_is_sorted && context.sort_keys_match_group_keys {
            return self.create_streaming_aggregate_decision(context);
        }

        // 如果表达式非确定性，优先使用哈希聚合（避免排序带来的不确定性）
        if !context.is_deterministic {
            let group_by_cardinality = self.estimate_group_by_cardinality(context);
            let hash_cost = self.calculate_hash_aggregate_cost(context, group_by_cardinality);
            let hash_memory = self.estimate_hash_memory_usage(context, group_by_cardinality);
            return AggregateStrategyDecision {
                strategy: AggregateStrategy::HashAggregate,
                estimated_output_rows: group_by_cardinality.max(1),
                estimated_cost: hash_cost,
                estimated_memory_bytes: hash_memory,
                reason: SelectionReason::CostBased {
                    hash_cost,
                    sort_cost: hash_cost * 1.5, // 假设排序代价更高
                },
            };
        }

        // 估算分组键基数
        let group_by_cardinality = self.estimate_group_by_cardinality(context);

        // 计算各策略的代价
        let hash_cost = self.calculate_hash_aggregate_cost(context, group_by_cardinality);
        let sort_cost = self.calculate_sort_aggregate_cost(context);

        // 检查内存限制
        let hash_memory = self.estimate_hash_memory_usage(context, group_by_cardinality);
        let memory_constrained =
            context.memory_limit > 0 && hash_memory > context.memory_limit;

        // 决策逻辑
        let (strategy, reason) = if memory_constrained {
            // 内存受限，优先选择排序聚合
            (
                AggregateStrategy::SortAggregate,
                SelectionReason::MemoryConstrained,
            )
        } else if context.input_rows < 1000 {
            // 小数据量，哈希聚合更简单高效
            (
                AggregateStrategy::HashAggregate,
                SelectionReason::SmallDataSet,
            )
        } else if group_by_cardinality < 100 {
            // 低基数，排序聚合可能更优（排序后数据局部性好）
            if sort_cost < hash_cost * 1.2 {
                (
                    AggregateStrategy::SortAggregate,
                    SelectionReason::LowCardinality,
                )
            } else {
                (
                    AggregateStrategy::HashAggregate,
                    SelectionReason::CostBased { hash_cost, sort_cost },
                )
            }
        } else if group_by_cardinality > context.input_rows / 10 {
            // 高基数（接近唯一值），哈希聚合更优
            (
                AggregateStrategy::HashAggregate,
                SelectionReason::HighCardinality,
            )
        } else {
            // 基于代价比较
            if hash_cost <= sort_cost {
                (
                    AggregateStrategy::HashAggregate,
                    SelectionReason::CostBased { hash_cost, sort_cost },
                )
            } else {
                (
                    AggregateStrategy::SortAggregate,
                    SelectionReason::CostBased { hash_cost, sort_cost },
                )
            }
        };

        let estimated_cost = match strategy {
            AggregateStrategy::HashAggregate => hash_cost,
            AggregateStrategy::SortAggregate => sort_cost,
            AggregateStrategy::StreamingAggregate => {
                self.calculate_streaming_aggregate_cost(context)
            }
        };

        AggregateStrategyDecision {
            strategy,
            estimated_output_rows: group_by_cardinality.max(1),
            estimated_cost,
            estimated_memory_bytes: match strategy {
                AggregateStrategy::HashAggregate => hash_memory,
                AggregateStrategy::SortAggregate => {
                    self.estimate_sort_memory_usage(context)
                }
                AggregateStrategy::StreamingAggregate => {
                    self.estimate_streaming_memory_usage(context)
                }
            },
            reason,
        }
    }

    /// 快速选择策略（简化版本，用于决策缓存）
    pub fn select_strategy_quick(
        &self,
        input_rows: u64,
        group_key_count: usize,
        _agg_function_count: usize,
    ) -> AggregateStrategy {
        if input_rows < 1000 {
            return AggregateStrategy::HashAggregate;
        }

        // 估算分组键基数
        let cardinality = self.estimate_cardinality_quick(input_rows, group_key_count);

        if cardinality < 100 {
            AggregateStrategy::SortAggregate
        } else {
            AggregateStrategy::HashAggregate
        }
    }

    /// 估算分组键基数
    fn estimate_group_by_cardinality(&self, context: &AggregateContext) -> u64 {
        self.estimate_cardinality_quick(context.input_rows, context.group_keys.len())
    }

    /// 快速估算基数
    fn estimate_cardinality_quick(&self, input_rows: u64, key_count: usize) -> u64 {
        if key_count == 0 {
            return 1;
        }

        // 启发式公式：基数随键数量增加而减少
        // 假设每增加一个键，基数除以2
        let divisor = 2_u64.saturating_pow(key_count as u32).max(1);
        let estimated = (input_rows / divisor).max(10);

        estimated.min(input_rows).max(1)
    }

    /// 计算哈希聚合代价
    fn calculate_hash_aggregate_cost(
        &self,
        context: &AggregateContext,
        _group_by_cardinality: u64,
    ) -> f64 {
        self.cost_calculator.calculate_aggregate_cost(
            context.input_rows,
            context.agg_function_count,
            context.group_keys.len(),
        )
    }

    /// 计算排序聚合代价
    fn calculate_sort_aggregate_cost(&self, context: &AggregateContext) -> f64 {
        // 排序代价 + 聚合代价
        let sort_cost = self
            .cost_calculator
            .calculate_sort_cost(context.input_rows, context.group_keys.len(), None);

        // 排序后的聚合代价较低（数据已分组）
        let agg_cost = context.input_rows as f64
            * context.agg_function_count as f64
            * self.cost_calculator.config().cpu_operator_cost
            * 0.5; // 排序后聚合代价减半

        sort_cost + agg_cost
    }

    /// 计算流式聚合代价
    fn calculate_streaming_aggregate_cost(&self, context: &AggregateContext) -> f64 {
        // 流式聚合只需要聚合代价，无需排序
        context.input_rows as f64
            * context.agg_function_count as f64
            * self.cost_calculator.config().cpu_operator_cost
    }

    /// 估算哈希聚合内存使用
    fn estimate_hash_memory_usage(
        &self,
        context: &AggregateContext,
        group_by_cardinality: u64,
    ) -> u64 {
        // 哈希表条目大小估算（键 + 聚合状态）
        let key_size = context.group_keys.len() as u64 * 16; // 假设每个键16字节
        let agg_state_size = context.agg_function_count as u64 * 24; // 假设每个聚合状态24字节
        let entry_overhead = 16; // 哈希表开销

        let entry_size = key_size + agg_state_size + entry_overhead;
        group_by_cardinality * entry_size.max(64)
    }

    /// 估算排序聚合内存使用
    fn estimate_sort_memory_usage(&self, context: &AggregateContext) -> u64 {
        // 排序可能需要缓存所有数据
        let row_size = 64; // 假设每行64字节
        context.input_rows * row_size
    }

    /// 估算流式聚合内存使用
    fn estimate_streaming_memory_usage(&self, context: &AggregateContext) -> u64 {
        // 流式聚合只需要维护当前分组的状态
        let key_size = context.group_keys.len() as u64 * 16;
        let agg_state_size = context.agg_function_count as u64 * 24;
        (key_size + agg_state_size) * 2 // 双缓冲
    }

    /// 创建流式聚合决策
    fn create_streaming_aggregate_decision(
        &self,
        context: &AggregateContext,
    ) -> AggregateStrategyDecision {
        let estimated_cost = self.calculate_streaming_aggregate_cost(context);
        let group_by_cardinality = self.estimate_group_by_cardinality(context);

        AggregateStrategyDecision {
            strategy: AggregateStrategy::StreamingAggregate,
            estimated_output_rows: group_by_cardinality.max(1),
            estimated_cost,
            estimated_memory_bytes: self.estimate_streaming_memory_usage(context),
            reason: SelectionReason::InputAlreadySorted,
        }
    }

    /// 更新优化决策中的聚合策略信息
    pub fn update_decision(
        &self,
        decision: &mut OptimizationDecision,
        context: &AggregateContext,
    ) {
        let strategy_decision = self.select_strategy(context);

        // 将聚合策略信息编码到决策的 rewrite_rules 中
        // 使用特定的规则ID表示聚合策略选择
        let rule_id = match strategy_decision.strategy {
            AggregateStrategy::HashAggregate => {
                crate::query::optimizer::decision::RewriteRuleId::AggregateOptimization
            }
            AggregateStrategy::SortAggregate => {
                crate::query::optimizer::decision::RewriteRuleId::AggregateOptimization
            }
            AggregateStrategy::StreamingAggregate => {
                crate::query::optimizer::decision::RewriteRuleId::AggregateOptimization
            }
        };

        if !decision.rewrite_rules.contains(&rule_id) {
            decision.rewrite_rules.push(rule_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::stats::StatisticsManager;

    fn create_test_selector() -> AggregateStrategySelector {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        AggregateStrategySelector::new(cost_calculator)
    }

    #[test]
    fn test_streaming_aggregate_when_sorted() {
        let selector = create_test_selector();
        let context = AggregateContext {
            input_rows: 10000,
            group_keys: vec!["category".to_string()],
            agg_function_count: 2,
            memory_limit: 0,
            input_is_sorted: true,
            sort_keys_match_group_keys: true,
            is_deterministic: true,
            complexity_score: 0,
        };

        let decision = selector.select_strategy(&context);
        assert_eq!(decision.strategy, AggregateStrategy::StreamingAggregate);
        matches!(decision.reason, SelectionReason::InputAlreadySorted);
    }

    #[test]
    fn test_hash_aggregate_for_small_data() {
        let selector = create_test_selector();
        let context = AggregateContext {
            input_rows: 500,
            group_keys: vec!["category".to_string()],
            agg_function_count: 1,
            memory_limit: 0,
            input_is_sorted: false,
            sort_keys_match_group_keys: false,
            is_deterministic: true,
            complexity_score: 0,
        };

        let decision = selector.select_strategy(&context);
        assert_eq!(decision.strategy, AggregateStrategy::HashAggregate);
        matches!(decision.reason, SelectionReason::SmallDataSet);
    }

    #[test]
    fn test_memory_constrained_fallback() {
        let selector = create_test_selector();
        let context = AggregateContext {
            input_rows: 100000,
            group_keys: vec!["category".to_string(), "subcategory".to_string()],
            agg_function_count: 3,
            memory_limit: 1024, // 1KB 内存限制（非常小）
            input_is_sorted: false,
            sort_keys_match_group_keys: false,
            is_deterministic: true,
            complexity_score: 0,
        };

        let decision = selector.select_strategy(&context);
        assert_eq!(decision.strategy, AggregateStrategy::SortAggregate);
        matches!(decision.reason, SelectionReason::MemoryConstrained);
    }

    #[test]
    fn test_quick_selection() {
        let selector = create_test_selector();

        // 小数据量应该选哈希聚合
        let strategy = selector.select_strategy_quick(500, 1, 1);
        assert_eq!(strategy, AggregateStrategy::HashAggregate);

        // 大数据量且多键（低基数）应该选排序聚合
        let strategy = selector.select_strategy_quick(100000, 10, 1);
        assert_eq!(strategy, AggregateStrategy::SortAggregate);

        // 大数据量且单键（高基数）应该选哈希聚合
        let strategy = selector.select_strategy_quick(100000, 1, 1);
        assert_eq!(strategy, AggregateStrategy::HashAggregate);
    }

    #[test]
    fn test_cardinality_estimation() {
        let selector = create_test_selector();

        // 无分组键应该返回1
        let cardinality = selector.estimate_cardinality_quick(1000, 0);
        assert_eq!(cardinality, 1);

        // 单个键的基数应该较高
        let cardinality = selector.estimate_cardinality_quick(10000, 1);
        assert!(cardinality > 100);

        // 多个键的基数应该较低
        let cardinality = selector.estimate_cardinality_quick(10000, 5);
        assert!(cardinality < 1000);
    }
}
