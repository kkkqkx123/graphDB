//! Execution Module
//!
//! Provides execution engines for query processing.

pub mod vector;

pub use vector::{
    VectorBatch, VectorColumn, VectorOperation, VectorProcessor, VectorSelector,
    VECTOR_BATCH_SIZE,
};
