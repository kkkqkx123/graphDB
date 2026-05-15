//! Transaction Context Information
//!
//! Defines a simple data struct for transaction context information.
//! This shared type allows the storage layer to access transaction metadata
//! without depending on the concrete transaction module types.

/// Transaction context information.
///
/// A simple data struct containing transaction metadata (MVCC timestamp, etc.).
/// This is used by the storage layer instead of a trait object to avoid
/// unnecessary dynamic dispatch, since there is only one implementation.
#[derive(Debug, Clone)]
pub struct TransactionContextInfo {
    /// Transaction ID
    pub id: u64,
    /// MVCC timestamp associated with this transaction
    pub timestamp: u32,
    /// Whether this is a read-only transaction
    pub is_read_only: bool,
}

impl TransactionContextInfo {
    /// Create a new TransactionContextInfo
    pub fn new(id: u64, timestamp: u32, is_read_only: bool) -> Self {
        Self {
            id,
            timestamp,
            is_read_only,
        }
    }
}