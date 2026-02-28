//! 计划重写模块
//!
//! 该模块包含所有启发式优化规则，在计划生成阶段直接应用。
//! 这些规则不依赖代价计算，总是产生更优或等价的计划。
//!
//! # 模块结构
//!
//! - `context`: 重写上下文定义
//! - `pattern`: 模式匹配定义
//! - `result`: 重写结果定义
//! - `rule`: 重写规则 trait 定义
//! - `macros`: 重写规则宏定义
//! - `rewrite_rule`: 重写规则 trait 定义和适配器（兼容层）
//! - `plan_rewriter`: 计划重写器实现
//! - `predicate_pushdown`: 谓词下推规则
//! - `merge`: 操作合并规则
//! - `projection_pushdown`: 投影下推规则
//! - `elimination`: 消除规则
//! - `limit_pushdown`: LIMIT 下推规则
//! - `aggregate`: 聚合优化规则
//!
//! # 规则分类
//!
//! ## 谓词下推规则 (predicate_pushdown)
//! 将过滤条件下推到计划树的底层，减少数据处理量。
//!
//! ## 操作合并规则 (merge)
//! 合并多个连续的相同类型操作，减少中间结果。
//!
//! ## 投影下推规则 (projection_pushdown)
//! 将投影操作下推到计划树底层。
//!
//! ## 消除规则 (elimination)
//! 消除冗余的操作，包括：
//! - 永假式过滤 (`EliminateFilterRule`)
//! - 无操作投影 (`RemoveNoopProjectRule`)
//! - 不必要的去重 (`DedupEliminationRule`)
//! - 冗余的排序 (`EliminateSortRule`) - 当输入已有序时
//!
//! ## LIMIT 下推规则 (limit_pushdown)
//! 将 LIMIT/TOPN 操作下推。
//!
//! ## 聚合优化规则 (aggregate)
//! 优化聚合操作。
//!
//! # 与基于代价的优化的关系
//!
//! 本模块的规则是**启发式规则**，不依赖代价计算，总是执行。
//! 基于代价的优化在 `strategy` 模块中实现，包括：
//! - 排序策略选择 (`SortEliminationOptimizer`) - 基于代价决定是否转换为 TopN
//! - 聚合策略选择 (`AggregateStrategySelector`)
//! - 连接顺序优化 (`JoinOrderOptimizer`)
//! - 遍历方向优化 (`TraversalDirectionOptimizer`)
//!
//! 启发式规则优先执行，基于代价的优化在之后执行。
//!
//! # 使用示例
//!
//! ```rust
//! use crate::query::planner::rewrite::{PlanRewriter, create_default_rewriter, rewrite_plan};
//! use crate::query::planner::plan::ExecutionPlan;
//!
//! // 使用默认重写器
//! let plan = ExecutionPlan::new(...);
//! let optimized_plan = rewrite_plan(plan)?;
//!
//! // 自定义重写器
//! let mut rewriter = PlanRewriter::new();
//! rewriter.add_rule(MyCustomRule);
//! let optimized_plan = rewriter.rewrite(plan)?;
//! ```

// 核心类型模块（新）
pub mod context;
pub mod pattern;
pub mod result;
pub mod rule;
pub mod expression_utils;

// 宏模块
pub mod macros;

// 核心 trait 和实现
pub mod rewrite_rule;
pub mod plan_rewriter;

// 静态分发规则枚举
pub mod rule_enum;

// 具体规则模块
pub mod predicate_pushdown;
pub mod merge;
pub mod projection_pushdown;
pub mod elimination;
pub mod limit_pushdown;
pub mod aggregate;

// ==================== 导出核心类型 ====================

// 从新的独立模块导出
pub use context::RewriteContext;
pub use pattern::{Pattern, MatchNode, PlanNodeMatcher, NodeVisitor, NodeVisitorRecorder, NodeVisitorFinder};
pub use result::{RewriteError, RewriteResult, TransformResult, MatchedResult};
pub use rule::{
    RewriteRule, 
    BaseRewriteRule, 
    MergeRule, 
    PushDownRule, 
    EliminationRule,
    RuleWrapper,
    IntoRuleWrapper,
};

// 从兼容层导出
pub use rewrite_rule::{
    HeuristicRule,
    HeuristicRuleAdapter,
    IntoOptRule,
};

pub use plan_rewriter::{
    PlanRewriter,
    create_default_rewriter,
    rewrite_plan,
};

// 导出静态分发规则枚举
pub use rule_enum::{
    RewriteRule as RewriteRuleEnum,
    RuleRegistry,
};

// 统一导出所有重写规则
pub use predicate_pushdown::*;
pub use merge::*;
pub use projection_pushdown::*;
pub use elimination::*;
pub use limit_pushdown::*;
pub use aggregate::*;


