//! 优化器引擎模块
//!
//! 本模块提供全局唯一的优化器引擎实例，负责协调和管理所有查询优化相关的组件。
//!
//! ## 设计说明
//!
//! `OptimizerEngine` 是查询优化层的核心组件，以全局实例的形式存在，与数据库实例同生命周期。
//! 它整合了统计信息管理、代价计算、选择性估计和决策缓存等功能，为查询流水线提供统一的优化服务。
//!
//! ## 全局实例说明
//!
//! 本文件中的 `OptimizerEngine` 设计为全局单例模式，原因如下：
//!
//! 1. **统计信息共享**：所有查询共享同一套统计信息，确保代价估算的一致性
//! 2. **决策缓存共享**：缓存的优化决策可以被不同查询复用，提高性能
//! 3. **资源效率**：避免每个查询管道重复创建优化器组件
//! 4. **配置一致性**：统一的代价模型配置应用于所有查询
//!
//! ## 使用方式
//!
//! ```rust
//! // 在数据库实例初始化时创建
//! let optimizer_engine = Arc::new(OptimizerEngine::new(CostModelConfig::default()));
//!
//! // 在查询流水线中使用
//! let decision = optimizer_engine.compute_decision(&stmt);
//! ```
//!
//! ## 线程安全
//!
//! `OptimizerEngine` 内部使用 `Arc` 和线程安全的数据结构，可以安全地在多线程环境中共享。

use std::sync::Arc;

use crate::query::optimizer::{
    CostCalculator, CostModelConfig, DecisionCache, SelectivityEstimator, StatisticsManager,
    TraversalStartSelector,
};
use crate::query::optimizer::decision::{
    AccessPath, EntityType, IndexSelectionDecision, JoinOrderDecision, OptimizationDecision,
    TraversalStartDecision,
};
use crate::query::parser::ast::Stmt;
use crate::query::planner::planner::SentenceKind;

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
    /// 决策缓存
    decision_cache: Option<DecisionCache>,
    /// 代价模型配置
    cost_config: CostModelConfig,
    /// 统计信息版本（用于决策缓存失效）
    stats_version: u64,
    /// 索引版本（用于决策缓存失效）
    index_version: u64,
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

        // 尝试创建决策缓存
        let decision_cache = match DecisionCache::with_default_config() {
            Ok(cache) => {
                log::info!("优化器决策缓存已启用");
                Some(cache)
            }
            Err(e) => {
                log::warn!("无法创建优化器决策缓存: {}", e);
                None
            }
        };

        Self {
            stats_manager,
            cost_calculator,
            selectivity_estimator,
            decision_cache,
            cost_config,
            stats_version: 1,
            index_version: 1,
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

    /// 更新代价模型配置
    ///
    /// 注意：更新配置会重新创建代价计算器，但不会影响已有的决策缓存
    pub fn set_cost_config(&mut self, config: CostModelConfig) {
        self.cost_config = config;
        self.cost_calculator = Arc::new(CostCalculator::with_config(
            self.stats_manager.clone(),
            self.cost_config,
        ));
        log::info!("优化器代价模型配置已更新: {:?}", self.cost_config);
    }

    /// 计算优化决策
    ///
    /// 根据语句类型计算最优的优化决策，包括遍历起点选择、索引选择等。
    ///
    /// # 参数
    /// - `stmt`: 解析后的语句
    /// - `kind`: 语句类型
    pub fn compute_decision(
        &self,
        stmt: &Stmt,
        kind: SentenceKind,
    ) -> Result<OptimizationDecision, crate::query::optimizer::decision::DecisionCacheError> {
        match kind {
            SentenceKind::Match => self.compute_match_decision(stmt),
            _ => self.compute_default_decision(),
        }
    }

    /// 计算 MATCH 语句的优化决策
    fn compute_match_decision(
        &self,
        stmt: &Stmt,
    ) -> Result<OptimizationDecision, crate::query::optimizer::decision::DecisionCacheError> {
        // 创建遍历起点选择器
        let selector = TraversalStartSelector::new(
            self.cost_calculator.clone(),
            self.selectivity_estimator.clone(),
        );

        // 提取模式并选择起点
        if let Stmt::Match(match_stmt) = stmt {
            if let Some(pattern) = match_stmt.patterns.first() {
                if let Some(candidate) = selector.select_start_node(pattern) {
                    let access_path = self.convert_selection_reason_to_access_path(&candidate.reason);

                    let variable_name = candidate
                        .node_pattern
                        .variable
                        .clone()
                        .unwrap_or_else(|| "n".to_string());

                    let traversal_decision = TraversalStartDecision::new(
                        variable_name,
                        access_path,
                        candidate.estimated_start_nodes as f64 / 10000.0,
                        candidate.estimated_cost,
                    );

                    log::debug!(
                        "遍历起点决策: 变量={}, 代价={:.2}, 选择性={:.4}",
                        traversal_decision.start_variable,
                        traversal_decision.estimated_cost(),
                        traversal_decision.estimated_selectivity()
                    );

                    return Ok(OptimizationDecision::new(
                        traversal_decision,
                        IndexSelectionDecision::empty(),
                        JoinOrderDecision::empty(),
                        self.stats_version,
                        self.index_version,
                    ));
                }
            }
        }

        // 返回默认决策
        self.compute_default_decision()
    }

    /// 计算默认优化决策
    fn compute_default_decision(
        &self,
    ) -> Result<OptimizationDecision, crate::query::optimizer::decision::DecisionCacheError> {
        Ok(OptimizationDecision::new(
            TraversalStartDecision::new(
                "default".to_string(),
                AccessPath::FullScan {
                    entity_type: EntityType::Vertex { tag_name: None },
                },
                1.0,
                1000.0,
            ),
            IndexSelectionDecision::empty(),
            JoinOrderDecision::empty(),
            self.stats_version,
            self.index_version,
        ))
    }

    /// 将选择原因转换为访问路径
    fn convert_selection_reason_to_access_path(
        &self,
        reason: &crate::query::optimizer::strategy::SelectionReason,
    ) -> AccessPath {
        use crate::query::optimizer::strategy::SelectionReason;

        match reason {
            SelectionReason::ExplicitVid => AccessPath::ExplicitVid {
                vid_description: "explicit".to_string(),
            },
            SelectionReason::HighSelectivityIndex { .. } => AccessPath::IndexScan {
                index_name: "auto".to_string(),
                property_name: "unknown".to_string(),
                predicate_description: "high_selectivity".to_string(),
            },
            SelectionReason::TagIndex { .. } => AccessPath::TagIndex {
                tag_name: "default".to_string(),
            },
            SelectionReason::FullScan { .. } => AccessPath::FullScan {
                entity_type: EntityType::Vertex { tag_name: None },
            },
            SelectionReason::VariableBinding { variable_name } => AccessPath::VariableBinding {
                source_variable: variable_name.clone(),
            },
        }
    }

    /// 增加统计信息版本（当统计信息更新时调用）
    pub fn bump_stats_version(&mut self) {
        self.stats_version += 1;
        log::debug!("统计信息版本已更新: {}", self.stats_version);
    }

    /// 增加索引版本（当索引变更时调用）
    pub fn bump_index_version(&mut self) {
        self.index_version += 1;
        log::debug!("索引版本已更新: {}", self.index_version);
    }

    /// 获取当前统计信息版本
    pub fn stats_version(&self) -> u64 {
        self.stats_version
    }

    /// 获取当前索引版本
    pub fn index_version(&self) -> u64 {
        self.index_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_engine_creation() {
        let engine = OptimizerEngine::default();
        assert_eq!(engine.stats_version(), 1);
        assert_eq!(engine.index_version(), 1);
    }

    #[test]
    fn test_optimizer_engine_with_config() {
        let config = CostModelConfig::for_ssd();
        let engine = OptimizerEngine::new(config);
        assert_eq!(engine.cost_config().random_page_cost, 1.1);
    }

    #[test]
    fn test_version_bump() {
        let mut engine = OptimizerEngine::default();
        engine.bump_stats_version();
        assert_eq!(engine.stats_version(), 2);
        engine.bump_index_version();
        assert_eq!(engine.index_version(), 2);
    }
}
