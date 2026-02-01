//! GraphDB - A lightweight single-node graph database implemented in Rust
//!
//! This crate provides the core functionality for a graph database that runs
//! as a single executable for personal and small-scale applications.

pub mod api;
pub mod common;
pub mod config;
pub mod core;
pub mod expression;
pub mod index;
pub mod query;
pub mod services;
pub mod storage;
pub mod utils;
