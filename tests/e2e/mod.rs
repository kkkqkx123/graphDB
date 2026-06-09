//! E2E Test Suite for GraphDB
//!
//! End-to-end tests migrated from Python to Rust.
//! These tests verify the complete system functionality including:
//! - Social network scenario tests
//! - Query optimizer tests
//! - Extended type tests
//! - Schema manager tests

pub mod extended_types;
pub mod optimizer;
pub mod schema_manager;
pub mod social_network;
