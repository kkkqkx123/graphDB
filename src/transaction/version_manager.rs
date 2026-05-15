//! MVCC Version Manager
//!
//! Re-exports the version manager from core::mvcc for backward compatibility.

pub use crate::core::mvcc::{
    InsertTimestampGuard, ReadTimestampGuard, UpdateTimestampGuard, VersionManager,
    VersionManagerConfig, VersionManagerError, VersionManagerResult,
};