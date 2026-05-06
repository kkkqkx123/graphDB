//! Cache Types
//!
//! Core types for cache keys, values, and eviction handling.

use std::sync::Arc;

use moka::notification::RemovalCause;

use crate::core::Value;

/// Eviction cause for cache entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionCause {
    /// Entry was evicted due to capacity constraints
    Capacity,
    /// Entry expired due to TTL or TTI
    Expired,
    /// Entry was explicitly removed
    Explicit,
    /// Entry was replaced by a new value
    Replaced,
}

impl From<RemovalCause> for EvictionCause {
    fn from(cause: RemovalCause) -> Self {
        match cause {
            RemovalCause::Size => EvictionCause::Capacity,
            RemovalCause::Expired => EvictionCause::Expired,
            RemovalCause::Explicit => EvictionCause::Explicit,
            RemovalCause::Replaced => EvictionCause::Replaced,
        }
    }
}

/// Callback type for eviction notifications
pub type EvictionCallback = Arc<dyn Fn(&str, EvictionCause) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,
}

impl VertexCacheKey {
    pub fn new(label_id: u16, internal_id: u32, timestamp: u64) -> Self {
        Self {
            label_id,
            internal_id,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeCacheKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub edge_id: u64,
    pub timestamp: u64,
}

impl EdgeCacheKey {
    pub fn new(
        edge_label_id: u16,
        src_vid: u64,
        dst_vid: u64,
        edge_id: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            edge_label_id,
            src_vid,
            dst_vid,
            edge_id,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeQueryKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub timestamp: u64,
}

impl EdgeQueryKey {
    pub fn new(edge_label_id: u16, src_vid: u64, dst_vid: u64, timestamp: u64) -> Self {
        Self {
            edge_label_id,
            src_vid,
            dst_vid,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdIndexCacheKey {
    pub label_id: u16,
    pub external_id: String,
}

impl IdIndexCacheKey {
    pub fn new(label_id: u16, external_id: String) -> Self {
        Self {
            label_id,
            external_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedVertex {
    pub internal_id: u32,
    pub external_id: String,
    pub properties: Vec<(String, Value)>,
}

impl CachedVertex {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<u32>() * 2;
        size += self.external_id.len();
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size as u32
    }
}

#[derive(Debug, Clone)]
pub struct CachedEdge {
    pub edge_id: u64,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub properties: Vec<(String, Value)>,
}

impl CachedEdge {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<u64>() * 3;
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size as u32
    }
}
