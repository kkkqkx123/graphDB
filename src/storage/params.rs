//! Storage Parameter Types
//!
//! Provides parameter structures for storage operations to reduce function argument count
//! and improve code maintainability.

use super::edge::{LabelId, VertexId};
use super::vertex::Timestamp;

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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
