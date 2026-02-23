//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

// 基础设施模块
pub mod rule_traits;

// 表达式工具模块
pub mod expression_utils;

// 核心类型模块
pub mod core;

// 规则枚举和配置模块
pub mod rule_enum;
pub mod rule_config;
pub mod optimizer_config;

// 执行计划表示模块
pub mod plan;

// 优化器实现
pub mod optimizer_impl;

// 优化规则模块（新结构）
pub mod rules;

// 索引选择器模块
pub mod index_selector;

// Re-export core types
pub use core::{Cost, OptimizationConfig, OptimizationPhase, OptimizationStats};
pub use core::LegacyStatistics as Statistics;

// Re-export rule enum and config
pub use rule_enum::OptimizationRule;
pub use rule_config::RuleConfig;
pub use optimizer_config::{load_optimizer_config, OptimizerConfigInfo};

// Re-export plan types
pub use plan::{
    OptContext, OptGroup, OptGroupNode, MatchedResult, MatchNode, OptRule, Pattern,
    PlanCandidate, PlanNodeProperties, TransformResult, OptimizerError,
};
pub use crate::utils::ObjectPool;

// Re-export optimizer types
pub use optimizer_impl::{Optimizer, RuleSet};

// Re-export all rule structs for convenient access (from new rules module)
pub use rules::*;

// Re-export rule traits
pub use rule_traits::{BaseOptRule, EliminationRule, MergeRule, PushDownRule};

// Re-export index selector
pub use index_selector::{IndexCandidate, IndexColumnHint, IndexScore, IndexSelector};
