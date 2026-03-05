//! 查询执行上下文模块
//!
//! 管理查询执行过程中的上下文信息，包括执行状态、资源管理和空间信息。

pub mod execution_state;
pub mod resource_context;
pub mod space_context;

pub use execution_state::QueryExecutionState;
pub use resource_context::QueryResourceContext;
pub use space_context::QuerySpaceContext;
