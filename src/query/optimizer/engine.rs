//! Optimizer Engine Module
//!
//! This module provides a query optimization engine, which is responsible for coordinating and managing all components related to query optimization.
//!
//! ## Design Specifications
//!
//! `OptimizerEngine` is the core component of the query optimization layer and is shared and used wherever it is needed through dependency injection.
//! It integrates functions such as statistical information management, cost calculation, and selective estimation, providing a unified optimization service for the query pipeline.
//!
//! ## Explanation of Shared Instances
//!
//! The `OptimizerEngine` is designed to be a component that can be shared across multiple queries for the following reasons:
//!
//! 1. **Sharing of statistical information**: All queries share the same set of statistical information, ensuring consistency in cost estimates.
//! 2. **Resource Efficiency**: Avoid the repeated creation of optimizer components in each query pipeline.
//! 3. **Configuration Consistency**: A unified cost model configuration is applied to all queries.
//!
//! ## How to use it
//!
//! ```rust
// Created during the initialization of the database instance
//! let optimizer_engine = Arc::new(OptimizerEngine::new(CostModelConfig::default()));
//!
// Used in the query pipeline through dependency injection
//! let pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer_engine);
//! ```
//!
//! ## Thread Safety
//!
//! `OptimizerEngine` utilizes `Arc` as well as thread-safe data structures, which allow for safe sharing in a multi-threaded environment.
//!
//! ## Attention
//!
//! This is not a global singleton, but an instance that is shared between components through `Arc`. Each database instance can have its own optimizer engine configuration.

use std::sync::Arc;

use crate::query::optimizer::{
    AggregateStrategySelector, CostCalculator, CostModelConfig, CteCacheManager,
    ExpressionAnalyzer, MaterializationOptimizer, ReferenceCountAnalyzer, SelectivityEstimator,
    SelectivityFeedbackManager, SortEliminationOptimizer, StatisticsManager,
    SubqueryUnnestingOptimizer,
};
use crate::query::validator::context::ExpressionAnalysisContext;

/// Optimizer engine
///
/// A globally unique instance of the optimizer engine, responsible for coordinating and managing all components related to query optimization.
/// It has the same lifecycle as the database instance and provides unified optimization services for all queries.
#[derive(Debug)]
pub struct OptimizerEngine {
    /// Expression context, used for sharing expression information across different stages
    expression_context: Arc<ExpressionAnalysisContext>,
    /// Statistics Information Manager
    stats_manager: Arc<StatisticsManager>,
    /// Selective Feedback Manager
    selectivity_feedback_manager: Arc<SelectivityFeedbackManager>,
    /// CTE Cache Manager
    cte_cache_manager: Arc<CteCacheManager>,
    /// Cost Calculator
    cost_calculator: Arc<CostCalculator>,
    /// Selective Estimator
    selectivity_estimator: Arc<SelectivityEstimator>,
    /// Sorting Elimination Optimizer
    sort_elimination_optimizer: Arc<SortEliminationOptimizer>,
    /// Aggregation Policy Selector
    aggregate_strategy_selector: AggregateStrategySelector,
    /// Expression Analyzer
    expression_analyzer: ExpressionAnalyzer,
    /// Reference Count Analyzer
    reference_count_analyzer: ReferenceCountAnalyzer,
    /// Subquery de-correlating optimizer
    subquery_unnesting_optimizer: SubqueryUnnestingOptimizer,
    /// CTE (Common Table Expression) Materialization Optimizer
    materialization_optimizer: MaterializationOptimizer,
    /// Cost model configuration
    cost_config: CostModelConfig,
}

impl OptimizerEngine {
    /// Create a new optimizer engine.
    ///
    /// # Parameters
    /// `cost_config`: Configuration of the cost model
    pub fn new(cost_config: CostModelConfig) -> Self {
        Self::with_expression_context(Arc::new(ExpressionAnalysisContext::new()), cost_config)
    }

    /// Create an optimizer engine using the shared ExpressionContext.
    ///
    /// # 参数
    /// `expression_context`: A shared context for expressions (shared across different stages).
    /// - `cost_config`: 代价模型配置
    pub fn with_expression_context(
        expression_context: Arc<ExpressionAnalysisContext>,
        cost_config: CostModelConfig,
    ) -> Self {
        // Create a statistical information manager
        let stats_manager = Arc::new(StatisticsManager::new());

        // Create a selective feedback manager
        let selectivity_feedback_manager = Arc::new(SelectivityFeedbackManager::new());

        // Create a CTE (Common Table Expression) for cache manager management.
        let cte_cache_manager = Arc::new(CteCacheManager::new());

        // Create a cost calculator and a selective estimator.
        let cost_calculator = Arc::new(CostCalculator::with_config(
            stats_manager.clone(),
            cost_config,
        ));
        let selectivity_estimator = Arc::new(SelectivityEstimator::new(stats_manager.clone()));

        // Create a sorting elimination optimizer
        let sort_elimination_optimizer =
            Arc::new(SortEliminationOptimizer::new(cost_calculator.clone()));

        // Create an analyzer
        let expression_analyzer = ExpressionAnalyzer::new();
        let reference_count_analyzer = ReferenceCountAnalyzer::new();

        // Create an aggregate policy selector that uses an expression analyzer and a shared context.
        let aggregate_strategy_selector = AggregateStrategySelector::with_context(
            cost_calculator.clone(),
            expression_analyzer.clone(),
            expression_context.clone(),
        );

        // Create a subquery to de-associate the optimizer.
        let subquery_unnesting_optimizer =
            SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager);

        // Creating a CTE (Common Table Expression) materialization optimizer
        let materialization_optimizer = MaterializationOptimizer::with_thresholds(
            &reference_count_analyzer,
            &expression_analyzer,
            &stats_manager,
            &cost_config.strategy_thresholds,
        );

        Self {
            expression_context,
            stats_manager,
            selectivity_feedback_manager,
            cte_cache_manager,
            cost_calculator,
            selectivity_estimator,
            sort_elimination_optimizer,
            aggregate_strategy_selector,
            expression_analyzer,
            reference_count_analyzer,
            subquery_unnesting_optimizer,
            materialization_optimizer,
            cost_config,
        }
    }

    /// Create an optimized configuration using an SSD.
    pub fn for_ssd() -> Self {
        Self::new(CostModelConfig::for_ssd())
    }

    /// Create an optimized configuration using a memory-based database.
    pub fn for_in_memory() -> Self {
        Self::new(CostModelConfig::for_in_memory())
    }

    /// Obtaining the Cost Model Configuration
    pub fn cost_config(&self) -> &CostModelConfig {
        &self.cost_config
    }

    /// Obtain the Cost Calculator
    pub fn cost_calculator(&self) -> &CostCalculator {
        &self.cost_calculator
    }

    /// Statistics Information Manager
    pub fn stats_manager(&self) -> &StatisticsManager {
        &self.stats_manager
    }

    /// Obtaining a selective estimator
    pub fn selectivity_estimator(&self) -> &SelectivityEstimator {
        &self.selectivity_estimator
    }

    /// Obtaining the sorting elimination optimizer
    pub fn sort_elimination_optimizer(&self) -> &SortEliminationOptimizer {
        &self.sort_elimination_optimizer
    }

    /// Obtain an expression analyzer.
    pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
        &self.expression_analyzer
    }

    /// Obtain the context of the expression.
    pub fn expression_context(&self) -> &Arc<ExpressionAnalysisContext> {
        &self.expression_context
    }

    /// Obtain a reference count analyzer
    pub fn reference_count_analyzer(&self) -> &ReferenceCountAnalyzer {
        &self.reference_count_analyzer
    }

    /// Obtain the Aggregation Policy Selector
    pub fn aggregate_strategy_selector(&self) -> &AggregateStrategySelector {
        &self.aggregate_strategy_selector
    }

    /// Obtaining the subquery to de-associate the optimizer
    pub fn subquery_unnesting_optimizer(&self) -> &SubqueryUnnestingOptimizer {
        &self.subquery_unnesting_optimizer
    }

    /// Obtaining the CTE (Common Table Expression) materialization optimizer
    pub fn materialization_optimizer(&self) -> &MaterializationOptimizer {
        &self.materialization_optimizer
    }

    /// Obtaining the Selective Feedback Manager
    pub fn selectivity_feedback_manager(&self) -> &SelectivityFeedbackManager {
        &self.selectivity_feedback_manager
    }

    /// Obtaining the CTE Cache Manager
    pub fn cte_cache_manager(&self) -> &CteCacheManager {
        &self.cte_cache_manager
    }

    /// Update the Cost Model Configuration
    ///
    /// Updating the configuration will recreate the cost calculator, but it will not affect the existing decision cache.
    pub fn set_cost_config(&mut self, config: CostModelConfig) {
        self.cost_config = config;
        self.cost_calculator = Arc::new(CostCalculator::with_config(
            self.stats_manager.clone(),
            self.cost_config,
        ));
        // Re-create the sorting elimination optimizer, using a new cost calculator.
        self.sort_elimination_optimizer =
            Arc::new(SortEliminationOptimizer::new(self.cost_calculator.clone()));
        // Re-create the analyzer
        self.expression_analyzer = ExpressionAnalyzer::new();
        self.reference_count_analyzer = ReferenceCountAnalyzer::new();
        // Recreate the Aggregation Policy Selector
        self.aggregate_strategy_selector = AggregateStrategySelector::with_analyzer(
            self.cost_calculator.clone(),
            self.expression_analyzer.clone(),
        );
        // Re-create the subquery to de-associate the optimizer.
        self.subquery_unnesting_optimizer =
            SubqueryUnnestingOptimizer::new(&self.expression_analyzer, &self.stats_manager);
        // Re-create the CTE (Common Table Expression) materialization optimizer
        self.materialization_optimizer = MaterializationOptimizer::with_thresholds(
            &self.reference_count_analyzer,
            &self.expression_analyzer,
            &self.stats_manager,
            &self.cost_config.strategy_thresholds,
        );
        log::info!("优化器代价模型配置已更新: {:?}", self.cost_config);
    }
}

impl Default for OptimizerEngine {
    fn default() -> Self {
        Self::new(CostModelConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_engine_creation() {
        let _engine = OptimizerEngine::default();
    }

    #[test]
    fn test_optimizer_engine_with_config() {
        let config = CostModelConfig::for_ssd();
        let _engine = OptimizerEngine::new(config);
    }
}
