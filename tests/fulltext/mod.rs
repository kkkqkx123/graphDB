//! Fulltext Search Module Integration Tests
//!
//! Test coverage:
//! - Basic CRUD - create index, drop index, insert, update, delete, search
//! - BM25 engine - BM25 specific features, scoring, parameter tuning
//! - Inversearch engine - inverted index specific features, boolean queries, phrase queries
//! - Engine comparison - compare BM25 and Inversearch results
//! - Concurrent operations - concurrent inserts, searches, mixed operations
//! - Sync mechanism - vertex change auto-sync, transaction buffering
//! - Edge cases - empty content, unicode, special characters, very long content
//! - Error handling - index not found, duplicate creation, invalid queries
//! - Multi-space isolation - space isolation for indexes
//! - Performance - basic performance tests for both engines
//! - Transaction support - transaction buffer, commit, rollback
//! - Dead letter queue - failed operation handling and recovery
//! - Advanced queries - boolean queries, phrase queries, prefix search
//! - Persistence - index and document persistence across restarts
//! - Highlight - search result highlighting and matched fields

mod common;
mod basic;
mod bm25;
mod inversearch;
mod engine_comparison;
mod concurrent;
mod sync;
mod edge_cases;
mod performance;
mod transaction;
mod dead_letter_queue;
mod advanced_queries;
mod persistence;
mod highlight;
