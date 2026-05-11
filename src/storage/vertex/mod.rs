//! Vertex Storage Module
//!
//! Provides columnar storage for vertex data with MVCC timestamp support.
//!
//! ## Components
//!
//! - `VertexTable`: Main vertex storage with columnar layout
//! - `IdIndexer`: External ID to internal ID mapping
//! - `ColumnStore`: Columnar property storage
//! - `VertexTimestamp`: MVCC timestamp tracking for vertices
//! - `encoding`: Compression encodings (Dictionary, RLE)

pub mod column_store;
pub mod encoding;
pub mod id_indexer;
pub mod vertex_table;
pub mod vertex_timestamp;

pub use column_store::{Column, ColumnStore};
pub use encoding::{EncodingStats, EncodingType, select_encoding};
pub use id_indexer::IdIndexer;
pub use vertex_table::VertexTable;
pub use vertex_timestamp::VertexTimestamp;

pub type LabelId = u32;
pub type VertexId = u64;
pub type Timestamp = u32;

pub const INVALID_TIMESTAMP: Timestamp = u32::MAX;
pub const MAX_TIMESTAMP: Timestamp = u32::MAX - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexStatus {
    Active,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct VertexRecord {
    pub vid: VertexId,
    pub internal_id: u32,
    pub properties: Vec<(String, crate::core::Value)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexSchema {
    pub label_id: LabelId,
    pub label_name: String,
    pub properties: Vec<PropertyDef>,
    pub primary_key_index: usize,
}

impl VertexSchema {
    pub fn from_tag_info(tag: &crate::core::types::TagInfo, label_id: LabelId) -> Self {
        let properties: Vec<PropertyDef> = tag.properties.iter().map(|p| p.into()).collect();
        let primary_key_index = 0;
        Self {
            label_id,
            label_name: tag.tag_name.clone(),
            properties,
            primary_key_index,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn default(mut self, value: crate::core::Value) -> Self {
        self.default_value = Some(value);
        self
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
