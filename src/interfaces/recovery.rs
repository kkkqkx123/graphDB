//! Recovery Operation Interface
//!
//! Defines the interface for WAL recovery operations.
//! This module re-exports the recovery types from core for cross-module access.

pub use crate::core::wal::traits::RecoveryApplier;