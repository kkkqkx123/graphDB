//! Edge Storage Module
//!
//! Provides CSR (Compressed Sparse Row) based edge storage.
//!
//! ## Components
//!
//! - `MutableCsr`: Mutable CSR supporting dynamic edge operations
//! - `SingleMutableCsr`: Optimized mutable CSR for single-edge scenarios
//! - `CsrVariant`: Enum wrapper for runtime CSR selection (supports both mutable and immutable)
//! - `EdgeTable`: Edge table combining out/in CSRs and property storage
//! - `PropertyTable`: Edge property storage
//!
//! ## CSR Type Selection
//!
//! The `EdgeStrategy` enum determines which CSR type to use:
//! - `Multiple`: Use `MutableCsr` (supports multiple edges per vertex)
//! - `Single`: Use `SingleMutableCsr` (one edge per vertex, O(1) access)
//! - `None`: No edges stored
//!
//! ## Use Cases
//!
//! | Strategy | CSR Type | Use Case | Time Complexity |
//! |----------|----------|----------|-----------------|
//! | `Multiple` | `MutableCsr` | General multi-edge relationships | O(degree) |
//! | `Single` | `SingleMutableCsr` | One-to-one relationships (spouse, current_employer) | O(1) |
//! | `None` | - | No edges stored | - |

pub mod csr;
pub mod csr_trait;
pub mod csr_variant;
pub mod edge_table;
pub mod fragmentation_stats;
pub mod immutable_csr;
pub mod labeled_mutable_csr;
pub mod mutable_csr;
pub mod multi_single_mutable_csr;
pub mod property_table;
pub mod single_mutable_csr;
pub mod bloom_filter;

use crate::core::types::{EdgeId, LabelId, Timestamp, VertexId, INVALID_TIMESTAMP};
use crate::core::{Edge, Value};
use crate::storage::types::StoragePropertyDef;

pub use crate::core::types::EdgeStrategy;
pub use csr::Csr;
pub use csr_trait::{CsrBase, MutableCsrTrait};
pub use csr_variant::CsrVariant;
pub use edge_table::{EdgeTable, ExportedEdgeSnapshot, UpdateEdgePropertyByOffsetParams};
pub use fragmentation_stats::FragmentationStats;
pub use immutable_csr::ImmutableCsr;
pub use labeled_mutable_csr::{LabeledMutableCsr, LabeledMutableCsrIterator};
pub use mutable_csr::{MutableCsr, MutableCsrIterator};
pub use multi_single_mutable_csr::{MultiSingleMutableCsr, MultiSingleMutableCsrIterator};
pub use property_table::PropertyTable;
pub use single_mutable_csr::{SingleMutableCsr, SingleMutableCsrIterator};

pub use crate::core::types::INVALID_EDGE_ID;

#[derive(Debug, Clone, Copy)]
pub struct CompactionReport {
    /// Number of deleted edges that were removed
    pub removed_edges: usize,
    /// Number of bytes reclaimed
    pub reclaimed_bytes: usize,
    /// Fragmentation ratio before compaction
    pub old_fragmentation_ratio: f32,
    /// Fragmentation ratio after compaction
    pub new_fragmentation_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct EdgeRecord {
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub rank: i64,
    pub properties: Vec<(String, Value)>,
}

impl From<&EdgeRecord> for Edge {
    fn from(record: &EdgeRecord) -> Self {
        let props: std::collections::HashMap<String, Value> =
            record.properties.iter().cloned().collect();

        Edge {
            src: record.src_vid,
            dst: record.dst_vid,
            edge_type: String::new(),
            ranking: record.rank,
            props,
        }
    }
}

impl EdgeRecord {
    pub fn into_edge_with_type(self, edge_type: &str) -> Edge {
        let props: std::collections::HashMap<String, Value> = self.properties.into_iter().collect();

        Edge {
            src: self.src_vid,
            dst: self.dst_vid,
            edge_type: edge_type.to_string(),
            ranking: self.rank,
            props,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeSchema {
    pub label_id: LabelId,
    pub label_name: String,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<StoragePropertyDef>,
    pub oe_strategy: EdgeStrategy,
    pub ie_strategy: EdgeStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub create_ts: Timestamp,
    pub delete_ts: Timestamp,
}

impl Nbr {
    pub fn new(
        neighbor: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        create_ts: Timestamp,
    ) -> Self {
        Self {
            neighbor,
            edge_id,
            prop_offset,
            create_ts,
            delete_ts: u32::MAX,
        }
    }

    pub fn with_delete_ts(
        neighbor: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        create_ts: Timestamp,
        delete_ts: Timestamp,
    ) -> Self {
        Self {
            neighbor,
            edge_id,
            prop_offset,
            create_ts,
            delete_ts,
        }
    }

    pub fn is_valid_at(&self, ts: Timestamp) -> bool {
        self.create_ts <= ts && ts < self.delete_ts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImmutableNbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub timestamp: Timestamp,
}

impl ImmutableNbr {
    pub fn new(neighbor: VertexId, edge_id: EdgeId, prop_offset: u32) -> Self {
        Self::with_timestamp(neighbor, edge_id, prop_offset, 0)
    }

    pub fn with_timestamp(
        neighbor: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            neighbor,
            edge_id,
            prop_offset,
            timestamp,
        }
    }
}
