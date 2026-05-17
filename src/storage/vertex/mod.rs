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

pub use crate::storage::storage_types::StoragePropertyDef as PropertyDef;

pub use column_store::{Column, ColumnStore};
pub use encoding::{EncodingStats, EncodingType, select_encoding};
pub use id_indexer::IdIndexer;
pub use vertex_table::VertexTable;
pub use vertex_timestamp::VertexTimestamp;

pub use crate::core::types::{
    INVALID_TIMESTAMP, LabelId, MAX_TIMESTAMP, Timestamp, VertexId,
};

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
