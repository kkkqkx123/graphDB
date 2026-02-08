//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

// 基础设施模块
pub mod rule_patterns;
pub mod rule_traits;

// 表达式工具模块
pub mod expression_utils;

// 核心类型模块
pub mod core;

// 规则枚举和配置模块
pub mod rule_enum;
pub mod rule_config;
pub mod rule_registry;
pub mod rule_registrar;
pub mod optimizer_config;
pub mod plan_node_visitor;

// 执行计划表示模块
pub mod plan;

// 优化引擎模块
pub mod engine;

// 优化规则模块（新结构）
pub mod rules;

// Re-export core types
pub use core::{Cost, OptimizationConfig, OptimizationPhase, OptimizationStats, Statistics};

// Re-export rule enum and config
pub use rule_enum::OptimizationRule;
pub use rule_config::RuleConfig;
pub use rule_registry::RuleRegistry;
pub use optimizer_config::{load_optimizer_config, OptimizerConfigInfo};
pub use plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitable};

// Re-export plan types
pub use plan::{
    OptContext, OptGroup, OptGroupNode, MatchedResult, MatchNode, OptRule, Pattern,
    PlanCandidate, PlanNodeProperties, TransformResult, OptimizerError,
};
pub use crate::utils::ObjectPool;

// Re-export engine types
pub use engine::{ExplorationState, Optimizer, RuleSet};

// Re-export all rule structs for convenient access (from new rules module)
pub use rules::*;

// Re-export rule traits
pub use rule_traits::{BaseOptRule, EliminationRule, MergeRule, PushDownRule};
