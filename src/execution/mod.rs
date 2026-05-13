//! Execution Module
//!
//! Provides execution engines for query processing.
//!
//! ## Arena-based Execution
//!
//! For high-performance scenarios with many temporary allocations, use
//! the arena-backed structures (`ArenaVectorBatch`, `ArenaSelectionVector`).
//! These leverage `ArenaAllocator` for efficient batch memory management.

pub mod vector;

pub use vector::{
    ArenaSelectionVector, ArenaVectorBatch, VectorBatch, VectorColumn, VectorOperation,
    VectorProcessor, VectorSelector, VECTOR_BATCH_SIZE,
};
