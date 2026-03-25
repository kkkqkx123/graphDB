//! Rights Management Module
//!
//! Provide user rights checking and validation function

pub mod permission_checker;
pub mod permission_manager;

// Re-exporting permission types from the core layer
pub use crate::core::{Permission, RoleType};

pub use permission_checker::{OperationType, PermissionChecker};
pub use permission_manager::{PermissionManager, GOD_SPACE_ID};
