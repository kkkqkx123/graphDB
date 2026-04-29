//! Transaction Module Integration Tests
//!
//! Test coverage:
//! - Basic lifecycle - begin, commit, rollback
//! - Vertex operations - insert, update, delete
//! - Edge operations - create, delete, properties
//! - Complex operations - multiple operations, cascading
//! - Concurrent transactions - read-only concurrency, write exclusivity
//! - Timeout handling - transaction timeout, query timeout, statement timeout, idle timeout
//! - Savepoints - create, rollback, multiple, find by name
//! - Durability levels - immediate, none
//! - Statistics - transaction stats, cleanup
//! - Retry mechanism - execute_with_retry, retryable vs non-retryable errors
//! - Batch commit - commit multiple transactions
//! - Metrics - transaction metrics collection
//! - Max concurrent - transaction limit enforcement
//! - Cleanup - expired transaction cleanup
//! - Shutdown - graceful shutdown with active transactions
//! - Transaction info - list active, get info by id
//! - HTTP API - BEGIN/COMMIT/ROLLBACK via HTTP API, concurrent HTTP requests, async/await pattern
//! - Deadlock prevention - verifies fix for spawn_blocking + block_on deadlock issue

mod common;
mod basic;
mod vertex;
mod edge;
mod complex;
mod concurrent;
mod timeout;
mod advanced;
mod http_api;
mod deadlock_prevention;
