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

use crate::storage::storage_types::StoragePropertyDef;

pub use column_store::{
    Column, ColumnStorage, ColumnStore, FixedWidthColumn, VariableWidthColumn, element_size,
    is_variable_length_type,
};
pub use encoding::{select_encoding, EncodingStats, EncodingType};
pub use id_indexer::{IdIndexer, IdKey};
pub use vertex_table::VertexTable;
pub use vertex_timestamp::VertexTimestamp;

use crate::core::Value;
use crate::core::vertex_edge_path::Tag;
use crate::core::types::TagInfo;
use crate::storage::utils::props_to_map;

pub use crate::core::types::{LabelId, Timestamp, VertexId, INVALID_TIMESTAMP, MAX_TIMESTAMP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexStatus {
    Active,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct VertexRecord {
    pub vid: VertexId,
    pub internal_id: u32,
    pub properties: Vec<(String, Value)>,
}

impl From<&VertexRecord> for crate::core::Vertex {
    fn from(record: &VertexRecord) -> Self {
        let properties = props_to_map(&record.properties);

        crate::core::Vertex {
            vid: record.vid,
            id: record.internal_id as i64,
            tags: vec![Tag {
                name: String::new(),
                properties: properties.clone(),
            }],
            properties,
        }
    }
}

impl VertexRecord {
    pub fn into_vertex_with_tag(self, tag_name: &str) -> crate::core::Vertex {
        let properties: std::collections::HashMap<String, Value> =
            self.properties.into_iter().collect();

        crate::core::Vertex {
            vid: self.vid,
            id: self.internal_id as i64,
            tags: vec![Tag {
                name: tag_name.to_string(),
                properties: properties.clone(),
            }],
            properties,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexSchema {
    pub label_id: LabelId,
    pub label_name: String,
    pub properties: Vec<StoragePropertyDef>,
    pub primary_key_index: usize,
}

impl VertexSchema {
    pub fn from_tag_info(tag: &TagInfo, label_id: LabelId) -> Self {
        let properties: Vec<StoragePropertyDef> = tag.properties.iter().map(StoragePropertyDef::from_core).collect();
        let primary_key_index = 0;
        Self {
            label_id,
            label_name: tag.tag_name.clone(),
            properties,
            primary_key_index,
        }
    }
}
