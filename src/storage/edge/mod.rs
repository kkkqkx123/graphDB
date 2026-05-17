//! Edge Storage Module
//!
//! Provides CSR (Compressed Sparse Row) based edge storage.
//!
//! ## Components
//!
//! - `MutableCsr`: Mutable CSR supporting dynamic edge operations
//! - `SingleMutableCsr`: Optimized mutable CSR for single-edge scenarios
//! - `MutableCsrVariant`: Enum wrapper for runtime CSR selection
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
pub mod edge_table;
pub mod mutable_csr;
pub mod mutable_csr_variant;
pub mod property_table;
pub mod single_mutable_csr;

pub use crate::storage::storage_types::StoragePropertyDef as PropertyDef;

pub use csr::Csr;
pub use csr_trait::{CsrBase, CsrType, ImmutableCsrTrait, MutableCsrTrait};
pub use edge_table::{
    EdgeTable, EdgeTableScanIterator, EdgeVertexIterator, UpdateEdgePropertyByOffsetParams,
};
pub use mutable_csr::{LoadFromPartsParams, MutableCsr, MutableCsrEdgeIterator, MutableCsrIterator};
pub use mutable_csr_variant::{CsrEdgeIterator, CsrIterator, MutableCsrVariant};
pub use property_table::PropertyTable;
pub use property_table::{PROP_OFFSET_NONE, prop_index_to_offset, prop_offset_to_index};
pub use single_mutable_csr::{SingleCsrEdgeIterator, SingleMutableCsr, SingleMutableCsrIterator};

pub use crate::core::types::{EdgeId, LabelId, Timestamp, VertexId};

pub const INVALID_TIMESTAMP: u32 = u32::MAX;
pub const MAX_TIMESTAMP: u32 = u32::MAX - 1;
pub const INVALID_EDGE_ID: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStrategy {
    None,
    Single,
    Multiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Out,
    In,
}

#[derive(Debug, Clone)]
pub struct EdgeRecord {
    pub edge_id: EdgeId,
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub ranking: i64,
    pub properties: Vec<(String, crate::core::Value)>,
}

#[derive(Debug, Clone)]
pub struct EdgeSchema {
    pub label_id: LabelId,
    pub label_name: String,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<PropertyDef>,
    pub oe_strategy: EdgeStrategy,
    pub ie_strategy: EdgeStrategy,
}

impl EdgeSchema {
    pub fn from_edge_type_info(
        edge_type: &crate::core::types::EdgeTypeInfo,
        label_id: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
    ) -> Self {
        let properties: Vec<PropertyDef> = edge_type.properties.iter().map(|p| p.into()).collect();
        Self {
            label_id,
            label_name: edge_type.edge_type_name.clone(),
            src_label,
            dst_label,
            properties,
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub timestamp: Timestamp,
}

impl Nbr {
    pub fn new(
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

    pub fn is_deleted(&self) -> bool {
        self.timestamp == INVALID_TIMESTAMP
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImmutableNbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
}

impl ImmutableNbr {
    pub fn new(neighbor: VertexId, edge_id: EdgeId, prop_offset: u32) -> Self {
        Self {
            neighbor,
            edge_id,
            prop_offset,
        }
    }
}