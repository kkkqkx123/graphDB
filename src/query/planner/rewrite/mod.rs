//! 计划重写模块
//!
//! 该模块包含所有启发式优化规则，在计划生成阶段直接应用。
//! 这些规则不依赖代价计算，总是产生更优或等价的计划。
//!
//! # 模块结构
//!
//! - `rewrite_rule`: 重写规则 trait 定义
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
//! 消除冗余的操作。
//!
//! ## LIMIT 下推规则 (limit_pushdown)
//! 将 LIMIT/TOPN 操作下推。
//!
//! ## 聚合优化规则 (aggregate)
//! 优化聚合操作。

// 核心 trait 和实现
pub mod rewrite_rule;
pub mod plan_rewriter;

// 具体规则模块
pub mod predicate_pushdown;
pub mod merge;
pub mod projection_pushdown;
pub mod elimination;
pub mod limit_pushdown;
pub mod aggregate;

// 统一导出核心类型
pub use rewrite_rule::{RewriteRule, RewriteRuleMut, RewriteError};
pub use plan_rewriter::{PlanRewriter, create_default_rewriter};

// 统一导出所有重写规则
pub use predicate_pushdown::*;
pub use merge::*;
pub use projection_pushdown::*;
pub use elimination::*;
pub use limit_pushdown::*;
pub use aggregate::*;
