//! GraphDB - A lightweight single-node graph database implemented in Rust
//!
//! This crate provides the core functionality for a graph database that runs
//! as a single executable for personal and small-scale applications.

pub mod api;
pub mod common;
pub mod config;
pub mod core;
pub mod query;
pub mod search;
pub mod storage;
pub mod transaction;
pub mod utils;

#[cfg(feature = "c-api")]
pub mod c_api;
