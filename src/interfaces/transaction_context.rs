//! Transaction Context Provider Interface
//!
//! Defines the interface for accessing transaction context information.
//! This abstraction allows the storage layer to interact with transaction
//! contexts without depending on the concrete transaction module types.

/// Provider for transaction context information.
///
/// This trait abstracts the transaction context interface so that
/// the storage layer can access transaction metadata (like MVCC timestamps)
/// without depending on the concrete transaction module implementation.
pub trait TransactionContextProvider: Send + Sync {
    /// Get the transaction ID
    fn id(&self) -> u64;

    /// Get the MVCC timestamp associated with this transaction
    fn timestamp(&self) -> u32;

    /// Check if this is a read-only transaction
    fn is_read_only(&self) -> bool;
}