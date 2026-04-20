//! Vector Types
//!
//! Shared types for vector synchronization to avoid circular dependencies.

use serde::{Deserialize, Serialize};

/// Vector change type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VectorChangeType {
    Insert,
    Delete,
}

impl From<crate::sync::coordinator::ChangeType> for VectorChangeType {
    fn from(ct: crate::sync::coordinator::ChangeType) -> Self {
        match ct {
            crate::sync::coordinator::ChangeType::Insert => VectorChangeType::Insert,
            crate::sync::coordinator::ChangeType::Delete => VectorChangeType::Delete,
            _ => VectorChangeType::Delete,
        }
    }
}
