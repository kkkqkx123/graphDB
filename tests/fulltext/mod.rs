//! Fulltext Search Module Integration Tests
//!
//! Test coverage:
//! - Basic CRUD - create index, drop index, insert, update, delete, search
//! - Engine comparison - compare BM25 and Inversearch results
//! - Concurrent operations - concurrent inserts, searches, mixed operations
//! - Sync mechanism - vertex change auto-sync, transaction buffering
//! - Edge cases - empty content, unicode, special characters, very long content
//! - Error handling - index not found, duplicate creation, invalid queries
//! - Multi-space isolation - space isolation for indexes
//! - Performance - basic performance tests for both engines
//! - Transaction support - transaction buffer, commit, rollback
//! - Advanced queries - boolean queries, phrase queries, prefix search
//! - Persistence - index and document persistence across restarts
//!
//! Note: Dead letter queue tests have been moved to unit tests in src/sync/dead_letter_queue.rs

mod common;
mod basic;
mod engine_comparison;
mod concurrent;
mod sync;
mod edge_cases;
mod performance;
mod transaction;
mod advanced_queries;
mod persistence;
