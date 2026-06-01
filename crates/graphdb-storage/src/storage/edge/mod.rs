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

use crate::core::types::EdgeTypeInfo;
use crate::core::{Edge, Value};
use crate::core::types::{
    EdgeId, LabelId, Timestamp, VertexId, INVALID_TIMESTAMP,
};
use crate::storage::storage_types::StoragePropertyDef;
use crate::storage::utils::props_to_map;

pub use csr_trait::{CsrBase, CsrType, ImmutableCsrTrait, MutableCsrTrait};
pub use edge_table::{EdgeTable, UpdateEdgePropertyByOffsetParams};
pub use mutable_csr::{MutableCsr, MutableCsrEdgeIterator, MutableCsrIterator};
pub use mutable_csr_variant::{CsrEdgeIterator, CsrIterator, MutableCsrVariant};
pub use property_table::PropertyTable;
pub use single_mutable_csr::{
    SingleCsrEdgeIterator, SingleMutableCsr, SingleMutableCsrIterator,
};

pub const INVALID_EDGE_ID: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub properties: Vec<(String, Value)>,
}

impl From<&EdgeRecord> for Edge {
    fn from(record: &EdgeRecord) -> Self {
        let props = props_to_map(&record.properties);

        Edge {
            src: record.src_vid,
            dst: record.dst_vid,
            edge_type: String::new(),
            ranking: 0,
            id: record.edge_id as i64,
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
            ranking: 0,
            id: self.edge_id as i64,
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

impl EdgeSchema {
    pub fn from_edge_type_info(
        edge_type: &EdgeTypeInfo,
        label_id: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
    ) -> Self {
        let properties: Vec<StoragePropertyDef> = edge_type
            .properties
            .iter()
            .map(StoragePropertyDef::from_core)
            .collect();
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
