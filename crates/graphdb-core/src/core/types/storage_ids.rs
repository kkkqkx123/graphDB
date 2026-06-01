//! Storage Identifier Types
//!
//! Provides fundamental type aliases and identifier structures shared across
//! storage and transaction modules. This eliminates bidirectional dependencies
//! by centralizing cross-module types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};

use crate::core::Value;

// ============================================================================
// Fundamental Type Aliases
// ============================================================================

/// Timestamp type for MVCC
pub type Timestamp = u32;

/// Invalid timestamp sentinel value (u32::MAX indicates "deleted" or "not set")
pub const INVALID_TIMESTAMP: Timestamp = u32::MAX;
/// Maximum valid timestamp value (u32::MAX - 1 used for "latest" queries)
pub const MAX_TIMESTAMP: Timestamp = u32::MAX - 1;

/// Label ID type for vertex and edge type identification
pub type LabelId = u32;

/// Edge ID type
pub type EdgeId = u64;

/// Column ID type for property columns
pub type ColumnId = i32;

/// Transaction ID type
pub type TransactionId = u64;

// ============================================================================
// VertexId - Unified Byte Representation
// ============================================================================

/// Maximum size for VertexId in bytes
/// Supports int64 (8 bytes) and small strings (up to 32 bytes)
pub const VERTEX_ID_MAX_SIZE: usize = 32;

/// Vertex identifier - unified byte representation
///
/// This type can represent both integer and string vertex IDs,
/// storing them as raw bytes for efficient storage and comparison.
/// Uses a fixed-size array to enable Copy trait and stack allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId {
    data: [u8; VERTEX_ID_MAX_SIZE],
    len: u8,
}

impl VertexId {
    pub const fn new() -> Self {
        VertexId {
            data: [0; VERTEX_ID_MAX_SIZE],
            len: 0,
        }
    }

    pub fn from_int64(id: i64) -> Self {
        let bytes = id.to_be_bytes();
        let mut data = [0u8; VERTEX_ID_MAX_SIZE];
        data[..8].copy_from_slice(&bytes);
        VertexId { data, len: 8 }
    }

    pub fn from_u64(id: u64) -> Self {
        let bytes = id.to_be_bytes();
        let mut data = [0u8; VERTEX_ID_MAX_SIZE];
        data[..8].copy_from_slice(&bytes);
        VertexId { data, len: 8 }
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        let s = s.into();
        let bytes = s.as_bytes();
        let len = bytes.len().min(VERTEX_ID_MAX_SIZE);
        let mut data = [0u8; VERTEX_ID_MAX_SIZE];
        data[..len].copy_from_slice(&bytes[..len]);
        VertexId {
            data,
            len: len as u8,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let len = bytes.len().min(VERTEX_ID_MAX_SIZE);
        let mut data = [0u8; VERTEX_ID_MAX_SIZE];
        data[..len].copy_from_slice(&bytes[..len]);
        VertexId {
            data,
            len: len as u8,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub fn as_int64(&self) -> Option<i64> {
        if self.len == 8 {
            let arr: [u8; 8] = self.data[..8].try_into().ok()?;
            Some(i64::from_be_bytes(arr))
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if self.len == 8 {
            let arr: [u8; 8] = self.data[..8].try_into().ok()?;
            Some(u64::from_be_bytes(arr))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }

    pub fn is_int64(&self) -> bool {
        self.len == 8
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_int64().map(|v| v as usize)
    }

    pub fn zero() -> Self {
        Self::from_int64(0)
    }

    pub const fn const_default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(i) = self.as_int64() {
            write!(f, "{}", i)
        } else if let Some(s) = self.as_str() {
            write!(f, "\"{}\"", s)
        } else {
            write!(f, "{:?}", self.as_bytes())
        }
    }
}

impl Default for VertexId {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for VertexId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Add<u64> for VertexId {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        if let Some(id) = self.as_u64() {
            Self::from_u64(id + rhs)
        } else if let Some(id) = self.as_int64() {
            Self::from_int64(id + rhs as i64)
        } else {
            panic!("Cannot add to non-integer VertexId");
        }
    }
}

impl AddAssign<u64> for VertexId {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl TryFrom<&Value> for VertexId {
    type Error = crate::core::StorageError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(i) => Ok(Self::from_int64(*i as i64)),
            Value::BigInt(i) => Ok(Self::from_int64(*i)),
            Value::String(s) => Ok(Self::from_string(s)),
            Value::Vertex(v) => Ok(v.vid),
            _ => Err(crate::core::StorageError::invalid_input(
                "Cannot convert Value to VertexId",
            )),
        }
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

impl From<VertexId> for Value {
    fn from(vid: VertexId) -> Self {
        if let Some(i) = vid.as_int64() {
            Value::BigInt(i)
        } else if let Some(s) = vid.as_str() {
            Value::String(s.to_string())
        } else {
            Value::Blob(vid.into_inner())
        }
    }
}

impl Ord for VertexId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
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
