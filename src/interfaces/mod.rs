//! Cross-Module Interfaces
//!
//! This module contains all traits and types that cross module boundaries.
//! By centralizing these interfaces, we achieve:
//! - Clear dependency boundaries between modules
//! - Reduced coupling between transaction and storage layers
//! - Easier testing through interface mocking
//! - Better separation of concerns
//!
//! ## Design Principles
//!
//! 1. **Interface Location**: Cross-module traits are defined here, not in consuming modules
//! 2. **Implementation Location**: Concrete implementations remain in their respective modules
//! 3. **Re-export Pattern**: Some types are re-exported from their original location for convenience
//! 4. **Gradual Migration**: Existing code can continue to work while we migrate to the new structure

pub mod transaction_buffer;
pub mod transaction_context;

pub use transaction_buffer::TransactionBuffer;
pub use transaction_context::TransactionContextInfo;
