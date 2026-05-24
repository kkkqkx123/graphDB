//! General Infrastructure Module
//!
//! This module contains all general-purpose infrastructure code, including:
//! - Basic utilities and ID generation
//! - Memory management
//! - Thread management

pub mod id;

// Re-export commonly used types and functions for easy use by other modules
pub use id::*;
