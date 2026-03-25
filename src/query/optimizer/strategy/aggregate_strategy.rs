//! Aggregation Policy Selector Module
//!
//! Cost-based selection of aggregation strategies: determining the optimal approach between hash aggregation and sort aggregation
//!
//! ## Usage Examples
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

use crate::core::types::ContextualExpression;
use crate::query::optimizer::analysis::ExpressionAnalyzer;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::decision::OptimizationDecision;
use crate::query::validator::context::ExpressionAnalysisContext;

/// Aggregation policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateStrategy {
    /// Hash Aggregation – Storing Group Status Using a HashMap
    /// Applicable to: cases where the number of group keys is high and there is sufficient memory available.
    HashAggregate,
    /// Sorting and Aggregation: First sort the data, then perform the aggregation.
    /// Applicable to: inputs that are already sorted or nearly sorted, and in cases where memory is limited.
    SortAggregate,
    /// Stream Aggregation – An optimized aggregation method when the input data is already sorted.
    /// Applicable to: The input data has already been sorted according to the grouping key.
    StreamingAggregate,
}

impl AggregateStrategy {
    /// Obtain the policy name
    pub fn name(&self) -> &'static str {
        match self {
            AggregateStrategy::HashAggregate => "HashAggregate",
            AggregateStrategy::SortAggregate => "SortAggregate",
            AggregateStrategy::StreamingAggregate => "StreamingAggregate",
        }
    }
}

/// Aggregation policy decision-making
#[derive(Debug, Clone)]
pub struct AggregateStrategyDecision {
    /// Selected Aggregation Strategy
    pub strategy: AggregateStrategy,
    /// Estimated number of output lines
    pub estimated_output_rows: u64,
    /// Estimated cost
    pub estimated_cost: f64,
    /// Estimated memory usage (in bytes)
    pub estimated_memory_bytes: u64,
    /// Reason for the choice
    pub reason: SelectionReason,
}

/// Reasons for the choice of strategy
#[derive(Debug, Clone)]
pub enum SelectionReason {
    /// The input is already sorted, and stream aggregation is being used.
    InputAlreadySorted,
    /// A high cardinality of the grouping key results in better performance for hash aggregation.
    HighCardinality,
    /// The base value of the grouping key is low, which makes sorting and aggregation more efficient.
    LowCardinality,
    /// Memory is limited; choose the sorting algorithm for aggregation.
    MemoryConstrained,
    /// With small amounts of data, hash aggregation is much simpler.
    SmallDataSet,
    /// Large volumes of data: Sorting and aggregation should be performed to avoid memory overflow.
    LargeDataSet,
    /// Cost-based decision-making
    CostBased { hash_cost: f64, sort_cost: f64 },
}

/// Aggregation Policy Selector
///
/// Selecting the optimal aggregation execution strategy based on a cost model
#[derive(Debug)]
pub struct AggregateStrategySelector {
    cost_calculator: Arc<CostCalculator>,
    /// Expression analyzer, used to analyze the characteristics of aggregate expressions
    expression_analyzer: ExpressionAnalyzer,
    /// Expression context, used for caching analysis results
    expression_context: Arc<ExpressionAnalysisContext>,
}

/// Contextual information for the selection of aggregation strategies
#[derive(Debug, Clone)]
pub struct AggregateContext {
    /// Number of input lines
    pub input_rows: u64,
    /// List of grouping keys
    pub group_keys: Vec<String>,
    /// Number of aggregate functions
    pub agg_function_count: usize,
    /// Memory limit (in bytes; 0 indicates no limit)
    pub memory_limit: u64,
    /// Are the input data already sorted?
    pub input_is_sorted: bool,
    /// Does the sorting key match the grouping key?
    pub sort_keys_match_group_keys: bool,
    /// Are aggregate expressions deterministic?
    pub is_deterministic: bool,
    /// Aggregation expression complexity score
    pub complexity_score: u32,
}

impl AggregateContext {
    /// Create a new aggregation context.
    pub fn new(input_rows: u64, group_keys: Vec<String>, agg_function_count: usize) -> Self {
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

    /// Setting memory limits
    pub fn with_memory_limit(mut self, memory_limit: u64) -> Self {
        self.memory_limit = memory_limit;
        self
    }

    /// Set the input to be sorted.
    pub fn with_sorted_input(mut self, sort_keys_match: bool) -> Self {
        self.input_is_sorted = true;
        self.sort_keys_match_group_keys = sort_keys_match;
        self
    }

    /// Setting expression properties
    pub fn with_expression_analysis(
        mut self,
        is_deterministic: bool,
        complexity_score: u32,
    ) -> Self {
        self.is_deterministic = is_deterministic;
        self.complexity_score = complexity_score;
        self
    }
}

impl AggregateStrategySelector {
    /// Create a new selector for aggregating policies.
    pub fn new(cost_calculator: Arc<CostCalculator>) -> Self {
        Self {
            cost_calculator,
            expression_analyzer: ExpressionAnalyzer::new(),
            expression_context: Arc::new(ExpressionAnalysisContext::new()),
        }
    }

    /// Create an aggregate policy selector with an expression analyzer
    pub fn with_analyzer(
        cost_calculator: Arc<CostCalculator>,
        expression_analyzer: ExpressionAnalyzer,
    ) -> Self {
        Self {
            cost_calculator,
            expression_analyzer,
            expression_context: Arc::new(ExpressionAnalysisContext::new()),
        }
    }

    /// Create an aggregate policy selector that includes the context of the expressions.
    pub fn with_context(
        cost_calculator: Arc<CostCalculator>,
        expression_analyzer: ExpressionAnalyzer,
        expression_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            cost_calculator,
            expression_analyzer,
            expression_context,
        }
    }

    /// Analyze aggregate expressions and create context.
    ///
    /// Use an expression analyzer to analyze the characteristics of aggregate expressions and create a complete aggregate context.
    /// The analysis results will be cached in the ExpressionContext to avoid duplicate analyses.
    pub fn analyze_and_create_context(
        &self,
        input_rows: u64,
        group_keys: Vec<String>,
        agg_function_count: usize,
        ctx_expressions: &[ContextualExpression],
    ) -> AggregateContext {
        let mut context = AggregateContext::new(input_rows, group_keys, agg_function_count);

        // Analyze all aggregate expressions.
        for ctx_expr in ctx_expressions {
            let expr_id = ctx_expr.id();
            if let Some(analysis) = self.expression_context.get_analysis(expr_id) {
                // Using the cached analysis results
                if !analysis.is_deterministic {
                    context.is_deterministic = false;
                }
                context.complexity_score += analysis.complexity_score;
            } else {
                // Analyze the expression and cache the result.
                let analysis = self.expression_analyzer.analyze(ctx_expr);
                self.expression_context
                    .set_analysis(expr_id, analysis.clone());

                if !analysis.is_deterministic {
                    context.is_deterministic = false;
                }
                context.complexity_score += analysis.complexity_score;
            }
        }

        context
    }

    /// Selecting the optimal aggregation strategy
    ///
    /// # Parameters
    /// Context: Information about the context in which the aggregation operation is taking place.
    ///
    /// # Return
    /// Aggregation policy decision-making, including the selected policy and the estimated cost.
    pub fn select_strategy(&self, context: &AggregateContext) -> AggregateStrategyDecision {
        // If the input is already sorted and the sorting key matches the grouping key, stream aggregation should be used preferentially.
        if context.input_is_sorted && context.sort_keys_match_group_keys {
            return self.create_streaming_aggregate_decision(context);
        }

        // If the expression is non-deterministic, hash aggregation should be used preferentially (to avoid the uncertainties associated with sorting).
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
                    sort_cost: hash_cost * 1.5, // Assume that the cost of sorting is higher.
                },
            };
        }

        // Estimating the cardinality of the group key
        let group_by_cardinality = self.estimate_group_by_cardinality(context);

        // Calculate the cost of each strategy.
        let hash_cost = self.calculate_hash_aggregate_cost(context, group_by_cardinality);
        let sort_cost = self.calculate_sort_aggregate_cost(context);

        // Check the memory limitations.
        let hash_memory = self.estimate_hash_memory_usage(context, group_by_cardinality);
        let memory_constrained = context.memory_limit > 0 && hash_memory > context.memory_limit;

        // Decision-making logic
        let (strategy, reason) = if memory_constrained {
            // Memory is limited; priority should be given to sorting and aggregation operations.
            (
                AggregateStrategy::SortAggregate,
                SelectionReason::MemoryConstrained,
            )
        } else if context.input_rows < 1000 {
            // With small amounts of data, hash aggregation is simpler and more efficient.
            (
                AggregateStrategy::HashAggregate,
                SelectionReason::SmallDataSet,
            )
        } else if group_by_cardinality < 100 {
            // With a small base size, sorting and aggregation may be more advantageous (as the data becomes more localized after sorting).
            if sort_cost < hash_cost * 1.2 {
                (
                    AggregateStrategy::SortAggregate,
                    SelectionReason::LowCardinality,
                )
            } else {
                (
                    AggregateStrategy::HashAggregate,
                    SelectionReason::CostBased {
                        hash_cost,
                        sort_cost,
                    },
                )
            }
        } else if group_by_cardinality > context.input_rows / 10 {
            // For high cardinalities (close to unique values), hash aggregation is more advantageous.
            (
                AggregateStrategy::HashAggregate,
                SelectionReason::HighCardinality,
            )
        } else {
            // Based on cost comparison
            if hash_cost <= sort_cost {
                (
                    AggregateStrategy::HashAggregate,
                    SelectionReason::CostBased {
                        hash_cost,
                        sort_cost,
                    },
                )
            } else {
                (
                    AggregateStrategy::SortAggregate,
                    SelectionReason::CostBased {
                        hash_cost,
                        sort_cost,
                    },
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
                AggregateStrategy::SortAggregate => self.estimate_sort_memory_usage(context),
                AggregateStrategy::StreamingAggregate => {
                    self.estimate_streaming_memory_usage(context)
                }
            },
            reason,
        }
    }

    /// Rapid selection strategy (simplified version, for use in decision-making caching)
    pub fn select_strategy_quick(
        &self,
        input_rows: u64,
        group_key_count: usize,
        _agg_function_count: usize,
    ) -> AggregateStrategy {
        if input_rows < 1000 {
            return AggregateStrategy::HashAggregate;
        }

        // Estimating the cardinality of the grouping key
        let cardinality = self.estimate_cardinality_quick(input_rows, group_key_count);

        if cardinality < 100 {
            AggregateStrategy::SortAggregate
        } else {
            AggregateStrategy::HashAggregate
        }
    }

    /// Estimating the cardinality of the grouping key
    fn estimate_group_by_cardinality(&self, context: &AggregateContext) -> u64 {
        self.estimate_cardinality_quick(context.input_rows, context.group_keys.len())
    }

    /// Quick estimation of the base number
    fn estimate_cardinality_quick(&self, input_rows: u64, key_count: usize) -> u64 {
        if key_count == 0 {
            return 1;
        }

        // Heuristic formula: The base number decreases as the number of keys increases.
        // Assume that for each additional key added, the base value is divided by 2.
        let divisor = 2_u64.saturating_pow(key_count as u32).max(1);
        let estimated = (input_rows / divisor).max(10);

        estimated.min(input_rows).max(1)
    }

    /// Calculating the cost of hash aggregation
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

    /// Calculating the cost of sorting and aggregation operations
    fn calculate_sort_aggregate_cost(&self, context: &AggregateContext) -> f64 {
        // Sorting cost + Aggregation cost
        let sort_cost = self.cost_calculator.calculate_sort_cost(
            context.input_rows,
            context.group_keys.len(),
            None,
        );

        // The aggregated cost after sorting is lower (the data has been grouped).
        let agg_cost = context.input_rows as f64
            * context.agg_function_count as f64
            * self.cost_calculator.config().cpu_operator_cost
            * 0.5; // The cost of aggregation is reduced by half after sorting.

        sort_cost + agg_cost
    }

    /// Calculating the cost of stream aggregation
    fn calculate_streaming_aggregate_cost(&self, context: &AggregateContext) -> f64 {
        // Stream aggregation only requires the cost of aggregation; sorting is not necessary.
        context.input_rows as f64
            * context.agg_function_count as f64
            * self.cost_calculator.config().cpu_operator_cost
    }

    /// Estimating the memory usage of hash aggregation
    fn estimate_hash_memory_usage(
        &self,
        context: &AggregateContext,
        group_by_cardinality: u64,
    ) -> u64 {
        // Estimation of the size of a hash table entry (key + aggregation state)
        let key_size = context.group_keys.len() as u64 * 16; // Assume that each key is 16 bytes in size.
        let agg_state_size = context.agg_function_count as u64 * 24; // Assume that each aggregated state occupies 24 bytes.
        let entry_overhead = 16; // Hash table overhead

        let entry_size = key_size + agg_state_size + entry_overhead;
        group_by_cardinality * entry_size.max(64)
    }

    /// Estimating the memory usage for sorting and aggregation operations
    fn estimate_sort_memory_usage(&self, context: &AggregateContext) -> u64 {
        // Sorting may require caching all the data.
        let row_size = 64; // Assume that each line contains 64 bytes.
        context.input_rows * row_size
    }

    /// Estimating the memory usage of stream aggregation
    fn estimate_streaming_memory_usage(&self, context: &AggregateContext) -> u64 {
        // Stream aggregation only requires maintaining the state of the current group.
        let key_size = context.group_keys.len() as u64 * 16;
        let agg_state_size = context.agg_function_count as u64 * 24;
        (key_size + agg_state_size) * 2 // Double buffering
    }

    /// Creating streaming aggregation decisions
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

    /// Update and optimize the information on the aggregation strategies used in decision-making processes.
    pub fn update_decision(&self, decision: &mut OptimizationDecision, context: &AggregateContext) {
        let strategy_decision = self.select_strategy(context);

        // Encode the aggregation policy information into the `rewrite_rules` used for decision-making.
        // Use a specific rule ID to indicate the selection of the aggregation policy.
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
            memory_limit: 1024, // Memory limit of 1 KB (very small)
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

        // For small amounts of data, hash aggregation should be chosen.
        let strategy = selector.select_strategy_quick(500, 1, 1);
        assert_eq!(strategy, AggregateStrategy::HashAggregate);

        // For large amounts of data with multiple keys (and a low cardinality), sorting and aggregation should be chosen as the appropriate methods for processing the data.
        let strategy = selector.select_strategy_quick(100000, 10, 1);
        assert_eq!(strategy, AggregateStrategy::SortAggregate);

        // For large volumes of data with a high number of unique keys (high cardinality), hash aggregation should be chosen.
        let strategy = selector.select_strategy_quick(100000, 1, 1);
        assert_eq!(strategy, AggregateStrategy::HashAggregate);
    }

    #[test]
    fn test_cardinality_estimation() {
        let selector = create_test_selector();

        // If there is no grouping key, the result should be 1.
        let cardinality = selector.estimate_cardinality_quick(1000, 0);
        assert_eq!(cardinality, 1);

        // The cardinality of a single key should be high.
        let cardinality = selector.estimate_cardinality_quick(10000, 1);
        assert!(cardinality > 100);

        // The cardinality of multiple keys should be relatively low.
        let cardinality = selector.estimate_cardinality_quick(10000, 5);
        assert!(cardinality < 1000);
    }
}
