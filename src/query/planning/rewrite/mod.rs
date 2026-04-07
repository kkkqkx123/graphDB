//! Plan to rewrite the module
//!
//! This module contains all the heuristic optimization rules, which are applied directly during the plan generation phase.
//! These rules do not rely on cost calculations; they always generate better or equivalent plans.
//!
//! # Module Structure
//!
//! “Rewrite the context definition” means to create a new or revised version of the definition of a particular term, concept, or situation, taking into account any changes, updates, or new information that may have become available. This could involve modifying the wording, the structure of the definition, or the overall approach used to explain the concept. The goal of rewriting the context definition is to ensure that it remains accurate, clear, and relevant in the current context.
//! `pattern`: Definition of pattern matching
//! “result”: Rewrite the definition of the “result” concept.
//! “Rule”: Definition of the rewrite rule trait
//! “macros”: Macro definitions for rewriting rules
//! `rewrite_rule`: The definition of the `rewrite_rule` trait and the corresponding adapter (compatibility layer)
//! `plan_rewriter`: Implementation of the plan rewriter
//! `predicate_pushdown`: The predicate pushdown rule
//! “merge”: The operation involves merging rules.
//! `projection_pushdown`: The rule for projecting data downwards (i.e., applying transformations or calculations to the lower levels of a data structure)
//! “Elimination”: elimination rule
//! `limit_pushdown`: The rule for LIMIT pushdown operations
//! “Aggregate”: Aggregation optimization rules.
//!
//! # Rule Classification
//!
//! ## Predicate Pushdown Rule
//! Push the filtering conditions to the lowest level of the planning tree to reduce the amount of data processing.
//!
//! ## Operation merge rules
//! Merge multiple consecutive operations of the same type to reduce the number of intermediate results.
//!
//! ## Projection Pushdown Rule
//! Push the projection operation down to the lowest level of the planning tree.
//!
//! ## Elimination rules
//! Operations to eliminate redundancy, including:
//! - Permanent filter rule (`EliminateFilterRule`)
//! - RemoveNoopProjectRule: This rule removes any unnecessary or redundant project-related operations.
//! - Unnecessary deduplication (`DedupEliminationRule`)
//! - Redundant sorting (`EliminateSortRule`) – When the input is already sorted.
//!
//! ## LIMIT Pushdown Rule (limit_pushdown)
//! Push down the LIMIT/TOPN operations.
//!
//! ## Aggregate Optimization Rules
//! Optimize aggregate operations.
//!
//! # The relationship with cost-based optimization
//!
//! The rules of this module are **heuristic rules**; they do not rely on cost calculations and are always executed.
//! Cost-based optimization is implemented in the `strategy` module, which includes the following aspects:
//! Selection of sorting strategy (`SortEliminationOptimizer`) –决定是否 to convert the result to a TopN list based on the cost associated with the sorting process.
//! Selection of the aggregation strategy (`AggregateStrategySelector`)
//! Optimization of the connection order (`JoinOrderOptimizer`)
//! Optimization of the traversal direction (`TraversalDirectionOptimizer`)
//! - SubqueryUnnestingOptimizer: A transformation based on analysis
//!
//! Heuristic rules are executed first, followed by cost-based optimization.
//!
//! # Usage Examples
//!
//! ```rust
//! use crate::query::planning::rewrite::{PlanRewriter, create_default_rewriter, rewrite_plan};
//! use crate::query::planning::plan::ExecutionPlan;
//!
// Use the default writer.
//! let plan = ExecutionPlan::new(...);
//! let optimized_plan = rewrite_plan(plan)?;
//!
// Custom rewrite器
//! let mut rewriter = PlanRewriter::new();
//! rewriter.add_rule(MyCustomRule);
//! let optimized_plan = rewriter.rewrite(plan)?;
//! ```

// Core Type Modules (New)
pub mod context;
pub mod expression_utils;
pub mod pattern;
pub mod result;
pub mod rule;
pub mod visitor;

// Macro module
pub mod macros;

// Core trait and implementation
pub mod plan_rewriter;
pub mod rewrite_rule;

// Enumeration of static distribution rules
pub mod rule_enum;

// Specific Rules Module
pub mod aggregate;
pub mod elimination;
pub mod join_optimization;
pub mod limit_pushdown;
pub mod merge;
pub mod predicate_pushdown;
pub mod projection_pushdown;

// ==================== Exporting Core Types =====================

// Export from the new independent module.
pub use context::RewriteContext;
pub use pattern::{
    MatchNode, NodeVisitor, NodeVisitorFinder, NodeVisitorRecorder, Pattern, PlanNodeMatcher,
};
pub use result::{MatchedResult, RewriteError, RewriteResult, TransformResult};
pub use rule::{
    BaseRewriteRule, EliminationRule, IntoRuleWrapper, MergeRule, PushDownRule, RewriteRule,
    RuleWrapper,
};
pub use visitor::ChildRewriteVisitor;

// Export from the compatibility layer
pub use rewrite_rule::{HeuristicRule, HeuristicRuleAdapter, IntoOptRule};

pub use plan_rewriter::{create_default_rewriter, rewrite_plan, PlanRewriter};

// Export the enumeration of static distribution rules.
pub use rule_enum::{RewriteRule as RewriteRuleEnum, RuleRegistry};

// Export all rewriting rules in a unified manner.
pub use aggregate::*;
pub use elimination::*;
pub use join_optimization::*;
pub use limit_pushdown::*;
pub use merge::*;
pub use predicate_pushdown::*;
pub use projection_pushdown::*;
