//! 排序和限制操作估算器
//!
//! 为排序限制节点提供代价估算：
//! - Sort
//! - Limit
//! - TopN
//! - Aggregate
//! - Dedup
//! - Sample
//!
//! 基于实际执行器实现（参考 aggregation.rs, sort.rs, limit.rs）：
//! - Aggregate: 使用 HashMap 存储分组状态，代价包括聚合函数处理和哈希操作
//! - Sort: 支持 Top-N 优化（当数据量 > limit * 10 时使用堆排序）
//! - Limit: 简单的内存操作，代价与 offset + limit 成正比

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 排序和限制操作估算器
pub struct SortLimitEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> SortLimitEstimator<'a> {
    /// 创建新的排序限制估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }

    /// 估算 GROUP BY 键的基数
    ///
    /// 基于实际 AggregateExecutor 实现（使用 HashMap）：
    /// - 如果没有 GROUP BY，返回 1（全局聚合）
    /// - 否则基于键的数量和输入行数进行估算
    fn estimate_group_by_cardinality(&self, group_keys: &[String], input_rows: u64) -> u64 {
        if group_keys.is_empty() {
            // 全局聚合，只返回一行（如 COUNT(*)）
            return 1;
        }

        // 基于 GROUP BY 键的数量估算基数
        // 键越多，分组越细，输出行数越多
        // 使用启发式公式：min(input_rows, max(10, input_rows / (2 ^ key_count)))
        let key_count = group_keys.len() as u32;
        let divisor = 2_u64.saturating_pow(key_count).max(1);
        let estimated = (input_rows / divisor).max(10);

        estimated.min(input_rows).max(1)
    }
}

impl<'a> NodeEstimator for SortLimitEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::Sort(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let sort_keys = n.sort_items().len();
                // Sort 节点本身没有 limit，但如果有子 Limit 节点，可以传递 limit 进行优化
                let cost = self.cost_calculator.calculate_sort_cost(input_rows_val, sort_keys, None);
                // Sort 不改变行数
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::Limit(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let limit = n.count();
                // 基于实际 LimitExecutor 实现：代价与 offset + limit 成正比
                let offset = n.offset();
                let rows_to_process = ((limit.max(0) + offset.max(0)) as u64).min(input_rows_val);
                let cost = self.cost_calculator.calculate_limit_cost(input_rows_val, limit)
                    + rows_to_process as f64 * self.cost_calculator.config().cpu_tuple_cost * 0.1;
                let output_rows = (limit.max(0) as u64).min(input_rows_val);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::TopN(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let limit = n.limit();
                // TopN 使用堆实现，复杂度 O(n log k)
                let cost = self.cost_calculator.calculate_topn_cost(input_rows_val, limit);
                let output_rows = (limit.max(0) as u64).min(input_rows_val);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Aggregate(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let agg_funcs = n.aggregation_functions().len();
                let group_keys = n.group_keys().len();

                // 基于实际 AggregateExecutor 实现计算代价
                // 包括聚合函数处理和哈希表操作
                let cost = self.cost_calculator.calculate_aggregate_cost(
                    input_rows_val,
                    agg_funcs,
                    group_keys,
                );

                // 聚合输出行数基于 GROUP BY 键的基数（HashMap 键的数量）
                let output_rows = self.estimate_group_by_cardinality(n.group_keys(), input_rows_val);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Dedup(_) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let cost = self.cost_calculator.calculate_dedup_cost(input_rows_val);
                // 去重后行数减少（假设为输入的 70%）
                let output_rows = (input_rows_val as f64 * 0.7).max(1.0) as u64;
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Sample(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                // SampleNode 使用 count 指定采样数量
                let sample_count = n.count().max(0) as u64;
                let cost = self.cost_calculator.calculate_sample_cost(input_rows_val);
                // 输出行为采样数量（不超过输入行数）
                let output_rows = sample_count.min(input_rows_val);
                Ok((cost, output_rows.max(1)))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("排序限制估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
