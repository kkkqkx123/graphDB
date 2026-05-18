//! Edge Table Cache Trait
//!
//! Defines the cache interface for edge property caching.
//! This trait allows for pluggable cache implementations,
//! decoupling EdgeTable from specific cache implementations.

use crate::core::Value;
use crate::storage::storage_types::PropertyId;

/// Trait for edge table property caching.
///
/// This trait abstracts the cache operations, allowing EdgeTable
/// to work with any cache implementation that satisfies this interface.
/// The default implementation (NoOpEdgeTableCache) does nothing,
/// effectively disabling caching.
pub trait EdgeTableCache: Send + Sync + std::fmt::Debug {
    /// Get a cached property value by offset and property name.
    fn get_by_offset(&self, prop_offset: u32, prop_name: &str) -> Option<Value>;

    /// Put a property value into cache by offset and property name.
    fn put_by_offset(&self, prop_offset: u32, prop_name: &str, value: Value);

    /// Get a cached property value by offset and property ID.
    fn get_by_offset_id(&self, prop_offset: u32, prop_id: PropertyId) -> Option<Value>;

    /// Put a property value into cache by offset and property ID.
    fn put_by_offset_id(&self, prop_offset: u32, prop_id: PropertyId, value: Value);

    /// Invalidate all cached properties for a given offset.
    fn invalidate_by_offset(&self, prop_offset: u32);

    /// Check if caching is enabled.
    fn is_enabled(&self) -> bool {
        true
    }
}

/// No-operation cache implementation.
///
/// This is the default cache that does nothing,
/// used when caching is disabled.
#[derive(Debug, Default)]
pub struct NoOpEdgeTableCache;

impl EdgeTableCache for NoOpEdgeTableCache {
    fn get_by_offset(&self, _prop_offset: u32, _prop_name: &str) -> Option<Value> {
        None
    }

    fn put_by_offset(&self, _prop_offset: u32, _prop_name: &str, _value: Value) {}

    fn get_by_offset_id(&self, _prop_offset: u32, _prop_id: PropertyId) -> Option<Value> {
        None
    }

    fn put_by_offset_id(&self, _prop_offset: u32, _prop_id: PropertyId, _value: Value) {}

    fn invalidate_by_offset(&self, _prop_offset: u32) {}

    fn is_enabled(&self) -> bool {
        false
    }
}
