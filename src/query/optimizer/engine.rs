//! 优化器引擎模块
//!
//! 本模块提供查询优化引擎，负责协调和管理所有查询优化相关的组件。
//!
//! ## 设计说明
//!
//! `OptimizerEngine` 是查询优化层的核心组件，通过依赖注入在需要的地方共享使用。
//! 它整合了统计信息管理、代价计算和选择性估计等功能，为查询流水线提供统一的优化服务。
//!
//! ## 共享实例说明
//!
//! `OptimizerEngine` 设计为可在多个查询间共享的组件，原因如下：
//!
//! 1. **统计信息共享**：所有查询共享同一套统计信息，确保代价估算的一致性
//! 2. **资源效率**：避免每个查询管道重复创建优化器组件
//! 3. **配置一致性**：统一的代价模型配置应用于所有查询
//!
//! ## 使用方式
//!
//! ```rust
//! // 在数据库实例初始化时创建
//! let optimizer_engine = Arc::new(OptimizerEngine::new(CostModelConfig::default()));
//!
//! // 在查询流水线中通过依赖注入使用
//! let pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer_engine);
//! ```
//!
//! ## 线程安全
//!
//! `OptimizerEngine` 内部使用 `Arc` 和线程安全的数据结构，可以安全地在多线程环境中共享。
//!
//! ## 注意
//!
//! 这不是全局单例，而是通过 `Arc` 在组件间共享的实例。每个数据库实例可以有自己的优化器引擎配置。

use std::sync::Arc;

use crate::query::optimizer::{
    CostCalculator, CostModelConfig, SelectivityEstimator, StatisticsManager,
    SortEliminationOptimizer, ExpressionAnalyzer, ReferenceCountAnalyzer,
    AggregateStrategySelector,
};

/// 优化器引擎
///
/// 全局唯一的优化器引擎实例，负责协调和管理所有查询优化相关的组件。
/// 与数据库实例同生命周期，为所有查询提供统一的优化服务。
#[derive(Debug)]
pub struct OptimizerEngine {
    /// 统计信息管理器
    stats_manager: Arc<StatisticsManager>,
    /// 代价计算器
    cost_calculator: Arc<CostCalculator>,
    /// 选择性估计器
    selectivity_estimator: Arc<SelectivityEstimator>,
    /// 排序消除优化器
    sort_elimination_optimizer: Arc<SortEliminationOptimizer>,
    /// 聚合策略选择器
    aggregate_strategy_selector: AggregateStrategySelector,
    /// 表达式分析器
    expression_analyzer: ExpressionAnalyzer,
    /// 引用计数分析器
    reference_count_analyzer: ReferenceCountAnalyzer,
    /// 代价模型配置
    cost_config: CostModelConfig,
}

impl OptimizerEngine {
    /// 创建新的优化器引擎
    ///
    /// # 参数
    /// - `cost_config`: 代价模型配置
    pub fn new(cost_config: CostModelConfig) -> Self {
        // 创建统计信息管理器
        let stats_manager = Arc::new(StatisticsManager::new());

        // 创建代价计算器和选择性估计器
        let cost_calculator = Arc::new(CostCalculator::with_config(
            stats_manager.clone(),
            cost_config,
        ));
        let selectivity_estimator = Arc::new(SelectivityEstimator::new(stats_manager.clone()));

        // 创建排序消除优化器
        let sort_elimination_optimizer = Arc::new(SortEliminationOptimizer::new(
            cost_calculator.clone(),
        ));

        // 创建分析器
        let expression_analyzer = ExpressionAnalyzer::new();
        let reference_count_analyzer = ReferenceCountAnalyzer::new();

        // 创建聚合策略选择器，使用表达式分析器
        let aggregate_strategy_selector = AggregateStrategySelector::with_analyzer(
            cost_calculator.clone(),
            expression_analyzer.clone(),
        );

        Self {
            stats_manager,
            cost_calculator,
            selectivity_estimator,
            sort_elimination_optimizer,
            aggregate_strategy_selector,
            expression_analyzer,
            reference_count_analyzer,
            cost_config,
        }
    }

    /// 使用默认配置创建优化器引擎
    pub fn default() -> Self {
        Self::new(CostModelConfig::default())
    }

    /// 使用 SSD 优化配置创建
    pub fn for_ssd() -> Self {
        Self::new(CostModelConfig::for_ssd())
    }

    /// 使用内存数据库优化配置创建
    pub fn for_in_memory() -> Self {
        Self::new(CostModelConfig::for_in_memory())
    }

    /// 获取代价模型配置
    pub fn cost_config(&self) -> &CostModelConfig {
        &self.cost_config
    }

    /// 获取代价计算器
    pub fn cost_calculator(&self) -> &CostCalculator {
        &self.cost_calculator
    }

    /// 获取统计信息管理器
    pub fn stats_manager(&self) -> &StatisticsManager {
        &self.stats_manager
    }

    /// 获取选择性估计器
    pub fn selectivity_estimator(&self) -> &SelectivityEstimator {
        &self.selectivity_estimator
    }

    /// 获取排序消除优化器
    pub fn sort_elimination_optimizer(&self) -> &SortEliminationOptimizer {
        &self.sort_elimination_optimizer
    }

    /// 获取表达式分析器
    pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
        &self.expression_analyzer
    }

    /// 获取引用计数分析器
    pub fn reference_count_analyzer(&self) -> &ReferenceCountAnalyzer {
        &self.reference_count_analyzer
    }

    /// 获取聚合策略选择器
    pub fn aggregate_strategy_selector(&self) -> &AggregateStrategySelector {
        &self.aggregate_strategy_selector
    }

    /// 更新代价模型配置
    ///
    /// 注意：更新配置会重新创建代价计算器，但不会影响已有的决策缓存
    pub fn set_cost_config(&mut self, config: CostModelConfig) {
        self.cost_config = config;
        self.cost_calculator = Arc::new(CostCalculator::with_config(
            self.stats_manager.clone(),
            self.cost_config,
        ));
        // 重新创建排序消除优化器，使用新的代价计算器
        self.sort_elimination_optimizer = Arc::new(SortEliminationOptimizer::new(
            self.cost_calculator.clone(),
        ));
        // 重新创建分析器
        self.expression_analyzer = ExpressionAnalyzer::new();
        self.reference_count_analyzer = ReferenceCountAnalyzer::new();
        // 重新创建聚合策略选择器
        self.aggregate_strategy_selector = AggregateStrategySelector::with_analyzer(
            self.cost_calculator.clone(),
            self.expression_analyzer.clone(),
        );
        log::info!("优化器代价模型配置已更新: {:?}", self.cost_config);
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
