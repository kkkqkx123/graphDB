//! Storage Identifier Types
//!
//! Provides fundamental type aliases and identifier structures shared across
//! storage and transaction modules. This eliminates bidirectional dependencies
//! by centralizing cross-module types.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Fundamental Type Aliases
// ============================================================================

/// Timestamp type for MVCC
pub type Timestamp = u32;

/// Label ID type for vertex and edge type identification
pub type LabelId = u32;

/// Edge ID type
pub type EdgeId = u64;

/// Column ID type for property columns
pub type ColumnId = i32;

// ============================================================================
// VertexId - Unified Byte Representation
// ============================================================================

/// Vertex identifier - unified byte representation
///
/// This type can represent both integer and string vertex IDs,
/// storing them as raw bytes for efficient storage and comparison.
/// The byte representation is directly compatible with RocksDB keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(Vec<u8>);

impl VertexId {
    pub const fn new() -> Self {
        VertexId(Vec::new())
    }

    pub fn from_int64(id: i64) -> Self {
        VertexId(id.to_be_bytes().to_vec())
    }

    pub fn from_u64(id: u64) -> Self {
        VertexId(id.to_be_bytes().to_vec())
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        VertexId(s.into().into_bytes())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        VertexId(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_int64(&self) -> Option<i64> {
        if self.0.len() == 8 {
            let arr: [u8; 8] = self.0[..].try_into().ok()?;
            Some(i64::from_be_bytes(arr))
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if self.0.len() == 8 {
            let arr: [u8; 8] = self.0[..].try_into().ok()?;
            Some(u64::from_be_bytes(arr))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    pub fn is_int64(&self) -> bool {
        self.0.len() == 8
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Display for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(i) = self.as_int64() {
            write!(f, "{}", i)
        } else if let Some(s) = self.as_str() {
            write!(f, "\"{}\"", s)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

impl Default for VertexId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for VertexId {
    fn from(id: i64) -> Self {
        Self::from_int64(id)
    }
}

impl From<u64> for VertexId {
    fn from(id: u64) -> Self {
        Self::from_u64(id)
    }
}

impl From<String> for VertexId {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for VertexId {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl Ord for VertexId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for VertexId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================================
// Edge Key and Identifier Types
// ============================================================================

/// Edge key for identifying an edge type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKey {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
}

impl EdgeKey {
    pub fn new(src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> Self {
        Self {
            src_label,
            dst_label,
            edge_label,
        }
    }
}

/// Edge location for identifying a specific edge instance with offsets
#[derive(Debug, Clone)]
pub struct EdgeLocation {
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
    pub oe_offset: i32,
    pub ie_offset: i32,
}

impl EdgeLocation {
    pub fn new(
        src_vid: VertexId,
        dst_vid: VertexId,
        edge_label: LabelId,
        oe_offset: i32,
        ie_offset: i32,
    ) -> Self {
        Self {
            src_vid,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
        }
    }
}

/// Edge identifier for fully identifying an edge instance
#[derive(Debug, Clone)]
pub struct EdgeIdentifier {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

impl EdgeIdentifier {
    pub fn new(
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
    ) -> Self {
        Self {
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
        }
    }
}

/// Edge operation context containing all necessary information for edge operations
#[derive(Debug, Clone)]
pub struct EdgeOperationContext {
    pub edge_key: EdgeKey,
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub timestamp: Timestamp,
}

impl EdgeOperationContext {
    pub fn new(
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
        src_vid: VertexId,
        dst_vid: VertexId,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            edge_key: EdgeKey::new(src_label, dst_label, edge_label),
            src_vid,
            dst_vid,
            timestamp,
        }
    }
}

/// Vertex identifier for identifying a vertex
#[derive(Debug, Clone)]
pub struct VertexIdentifier {
    pub label: LabelId,
    pub vid: VertexId,
}

impl VertexIdentifier {
    pub fn new(label: LabelId, vid: VertexId) -> Self {
        Self { label, vid }
    }
}

/// Edge property update context
#[derive(Debug, Clone)]
pub struct EdgePropertyUpdateContext {
    pub edge_id: EdgeIdentifier,
    pub property_name: String,
    pub timestamp: Timestamp,
}

impl EdgePropertyUpdateContext {
    pub fn new(
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
        property_name: String,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            edge_id: EdgeIdentifier::new(src_label, src_vid, dst_label, dst_vid, edge_label),
            property_name,
            timestamp,
        }
    }
}

/// Edge deletion context with offsets
#[derive(Debug, Clone)]
pub struct EdgeDeletionContext {
    pub edge_id: EdgeIdentifier,
    pub oe_offset: i32,
    pub ie_offset: i32,
    pub timestamp: Timestamp,
}

/// Parameters for creating EdgeDeletionContext
pub struct EdgeDeletionContextParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
    pub oe_offset: i32,
    pub ie_offset: i32,
    pub timestamp: Timestamp,
}

impl EdgeDeletionContext {
    pub fn new(params: EdgeDeletionContextParams) -> Self {
        Self {
            edge_id: EdgeIdentifier::new(
                params.src_label,
                params.src_vid,
                params.dst_label,
                params.dst_vid,
                params.edge_label,
            ),
            oe_offset: params.oe_offset,
            ie_offset: params.ie_offset,
            timestamp: params.timestamp,
        }
    }
}
