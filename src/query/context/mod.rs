//! Query execution context module
//!
//! Manages contextual information during query execution, including execution manager, resource management, and spatial information.

pub mod execution_manager;
pub mod resource_context;
pub mod space_context;

pub use execution_manager::QueryExecutionManager;
pub use resource_context::QueryResourceContext;
pub use space_context::QuerySpaceContext;
