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

pub mod csr;
pub mod edge_table;
pub mod mutable_csr;
pub mod property_table;

pub use csr::Csr;
pub use edge_table::EdgeTable;
pub use mutable_csr::MutableCsr;
pub use property_table::PropertyTable;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub type EdgeId = u64;
pub type LabelId = u16;
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

#[derive(Debug, Clone, Copy)]
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub timestamp: Timestamp,
}

impl Nbr {
    pub fn new(neighbor: VertexId, edge_id: EdgeId, prop_offset: u32, timestamp: Timestamp) -> Self {
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
