//! Recovery Operation Interface
//!
//! Defines the interface for WAL recovery operations.
//! This trait abstracts storage-specific recovery details from the WAL layer.
//!
//! NOTE: The actual RecoveryApplier trait is defined in crate::transaction::wal::recovery.
//! This module serves as a centralized access point for cross-module usage.

pub use crate::transaction::wal::recovery::RecoveryApplier;