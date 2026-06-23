//! Schema version tracking and migration framework
//!
//! This module provides comprehensive schema versioning, change tracking,
//! compatibility analysis, and migration support for the storage engine.
//!
//! ## Components
//!
//! - `change`: Schema change events and history logs
//! - `version_history`: Version snapshots and compatibility tracking
//! - `compatibility`: Compatibility analysis and migration strategies
//! - `migration_engine`: Schema migration planning and execution (Phase 2)

pub mod change;
pub mod compatibility;
pub mod version_history;

pub use change::{ChangeDetails, ChangeLog, PropertyChange, SchemaObjectType};
pub use compatibility::{
    CompatibilityAnalysis, CompatibilityAnalyzer, BreakingChange, NonBreakingChange,
    MigrationStrategy,
};
pub use version_history::{
    LabelVersionHistory, SchemaVersionHistory,
};
