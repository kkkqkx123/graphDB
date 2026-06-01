//! Storage Module Integration Test Submodules
//!
//! Test coverage:
//! - Persistence recovery - flush, load, checkpoint round-trip integrity
//! - Batch data integrity - bulk insert, scan, and verify
//! - Cache coherence - cache behavior under load
//! - Config variants - storage config parameter validation

mod common;
mod persistence_recovery;
