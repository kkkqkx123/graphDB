//! Query execution context module
//!
//! Manages contextual information during query execution, including execution manager.
//!
//! # Optimization Note (2024-03-27)
//! Previously exported `QueryResourceContext` and `QuerySpaceContext` have been inlined into
//! `QueryContext` to reduce indirection. These modules are kept for backward compatibility
//! but are no longer exported from this module.

pub mod execution_manager;

// Note: resource_context and space_context modules have been inlined into QueryContext
// They are kept in the codebase for backward compatibility but are deprecated.
// pub mod resource_context;  // Deprecated: inlined into QueryContext
// pub mod space_context;     // Deprecated: inlined into QueryContext

pub use execution_manager::QueryExecutionManager;
