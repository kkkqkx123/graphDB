//! Optimization Pipeline Module
//!
//! This module defines the optimization pipeline that coordinates heuristic and cost-based optimization phases.
//!
//! # Optimization Pipeline Architecture
//!
//! The optimization process is divided into two distinct phases:
//!
//! ## Phase 1: Heuristic Optimization (Always Executed)
//! - Predicate Pushdown
//! - Projection Pushdown
//! - Elimination Rules
//! - Merge Operations
//! - Limit Pushdown
//!
//! ## Phase 2: Cost-Based Optimization (Optional, Configuration-Dependent)
//! - Join Order Optimization
//! - Index Selection
//! - Traversal Start Selection
//! - Traversal Direction Optimization
//! - Aggregate Strategy Selection
//! - Materialization Decision
//!
//! # Usage Examples
//!
//! ```rust
//! use crate::query::optimizer::pipeline::OptimizationPipeline;
//! use crate::query::optimizer::pipeline::PipelineConfig;
//!
//! let config = PipelineConfig::default();
//! let pipeline = OptimizationPipeline::new(config);
//! let optimized_plan = pipeline.optimize(initial_plan)?;
//! ```

pub mod config;
pub mod optimizer_pipeline;
pub mod phase;

// Re-export main types
pub use config::PipelineConfig;
pub use optimizer_pipeline::OptimizationPipeline;
pub use phase::OptimizationPhase;
