//! C API Module
//!
//! Provides a C language interface for GraphDB

#[cfg(feature = "embedded")]
pub use crate::api::embedded::c_api::*;
