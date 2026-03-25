//! Query Optimization Module
//!
//! Provide query optimization capabilities, including the management of statistical information, cost calculation, and optimization strategies.
//!
//! ## Module Structure
//!
//! “Engine” refers to the optimizer engine, which is a globally unique instance of the optimizer that integrates all optimization components.
//! The “stats” module is responsible for managing statistical data related to tags, edge types, and their attributes.
//! `cost` – A module for calculating costs, which determines the cost of query operations.
//! “Analysis” module: This module is responsible for plan analysis, providing information on reference counts as well as an analysis of expressions.
//! “Strategy” module: an optimization strategy module that provides options for selecting the starting point and the index for traversal.
//! Optimization of the decision-making module: Implementation of a cache mechanism based on the decision-making process.
//!
//! ## Usage Examples
//!
//! ```rust
//! use graphdb::query::optimizer::OptimizerEngine;
//!
// Create the optimizer engine (global instance)
//! let optimizer = OptimizerEngine::default();
//!
// Calculate the optimization decision
//! let decision = optimizer.compute_decision(&stmt, sentence_kind);
//! ```

pub mod analysis;
pub mod cost;
pub mod decision;
pub mod engine;
pub mod stats;
pub mod strategy;

// Re-export the main types
pub use engine::OptimizerEngine;

pub use stats::{
    EdgeTypeStatistics, ExecutionFeedbackCollector, FeedbackDrivenSelectivity, OperatorFeedback,
    PropertyStatistics, QueryExecutionFeedback, QueryFeedbackHistory, SelectivityFeedbackManager,
    StatisticsManager, TagStatistics,
};

pub use crate::core::error::optimize::CostError;
pub use cost::{CostAssigner, CostCalculator, CostModelConfig, SelectivityEstimator};

// Reexport the analysis module type.
pub use analysis::{
    AnalysisOptions, ExpressionAnalysis, ExpressionAnalyzer, ReferenceCountAnalysis,
    ReferenceCountAnalyzer,
};

pub use strategy::{
    AggregateContext, AggregateSelectionReason, AggregateStrategy, AggregateStrategyDecision,
    AggregateStrategySelector, CandidateStart, CteCacheConfig, CteCacheDecision,
    CteCacheDecisionMaker, CteCacheEntry, CteCacheManager, CteCacheStats, DegreeInfo,
    DirectionContext, DirectionSelectionReason, IndexSelection, IndexSelector, JoinCondition,
    JoinOrderOptimizer, JoinOrderResult, KeepReason, MaterializationDecision,
    MaterializationOptimizer, MaterializeReason, NoMaterializeReason, OptimizationMethod,
    PredicateOperator, PropertyPredicate, SortContext, SortEliminationDecision,
    SortEliminationOptimizer, SortKeepReason, SubqueryUnnestingOptimizer, TableInfo,
    TopNConversionReason, TraversalDirection, TraversalDirectionDecision,
    TraversalDirectionOptimizer, TraversalSelectionReason, TraversalStartSelector, UnnestDecision,
    UnnestReason,
};

pub use decision::{
    AccessPath, EntityIndexChoice, EntityType, IndexChoice, IndexSelectionDecision, JoinAlgorithm,
    JoinOrderDecision, OptimizationDecision, RewriteRuleId, TraversalStartDecision,
};
