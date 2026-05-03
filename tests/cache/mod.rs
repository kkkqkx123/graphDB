//! Cache Module Tests
//!
//! Test coverage:
//! - basic: CacheManager and basic functionality
//! - config: Configuration validation and presets
//! - stats: Statistics collection and reporting
//! - plan_cache: Query plan cache functionality
//! - cte_cache: CTE result cache functionality
//! - invalidation: Cache invalidation strategies
//! - warmup: Cache warmup functionality
//! - concurrent: Concurrency and thread safety

pub mod basic;
pub mod config;
pub mod stats;
pub mod plan_cache;
pub mod cte_cache;
pub mod invalidation;
pub mod warmup;
pub mod concurrent;
