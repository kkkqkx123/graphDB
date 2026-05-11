//! Edge Storage Module
//!
//! Provides CSR (Compressed Sparse Row) based edge storage.
//!
//! ## Components
//!
//! - `Csr`: Immutable CSR for read-optimized edge storage
//! - `MutableCsr`: Mutable CSR supporting dynamic edge operations
//! - `EdgeTable`: Edge table combining out/in CSRs and property storage
//! - `PropertyTable`: Edge property storage
//! - `CsrPersistence`: CSR persistence support

pub mod csr;
pub mod csr_persistence;
pub mod edge_table;
pub mod mutable_csr;
pub mod property_table;

pub use csr::Csr;
pub use csr_persistence::CsrPersistence;
pub use edge_table::{EdgeTable, EdgeTableScanIterator, EdgeVertexIterator, UpdateEdgePropertyByOffsetParams};
pub use mutable_csr::{MutableCsr, MutableCsrEdgeIterator, MutableCsrIterator, LoadFromPartsParams};
pub use property_table::PropertyTable;

pub type EdgeId = u64;
pub type LabelId = u32;
pub type VertexId = u64;
pub type Timestamp = u32;

pub const INVALID_TIMESTAMP: Timestamp = u32::MAX;
pub const MAX_TIMESTAMP: Timestamp = u32::MAX - 1;
pub const INVALID_EDGE_ID: EdgeId = u64::MAX;

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

#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: crate::core::DataType,
    pub nullable: bool,
    pub default_value: Option<crate::core::Value>,
}

impl PropertyDef {
    pub fn new(name: String, data_type: crate::core::DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
            default_value: None,
        }
    }
}

impl From<crate::core::types::PropertyDef> for PropertyDef {
    fn from(prop: crate::core::types::PropertyDef) -> Self {
        Self {
            name: prop.name,
            data_type: prop.data_type,
            nullable: prop.nullable,
            default_value: prop.default,
        }
    }
}

impl From<&crate::core::types::PropertyDef> for PropertyDef {
    fn from(prop: &crate::core::types::PropertyDef) -> Self {
        Self {
            name: prop.name.clone(),
            data_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
