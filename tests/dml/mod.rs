//! Data Manipulation Language (DML) Integration Tests
//!
//! Test coverage:
//! - INSERT VERTEX - Insert vertex data
//! - INSERT EDGE - Insert edge data
//! - UPDATE - Update properties
//! - DELETE - Delete vertices and edges
//! - UPSERT - Insert or update
//! - MERGE - Merge operation

mod common;
mod insert_vertex;
mod insert_edge;
mod update;
mod delete;
mod upsert;
mod batch_operations;
