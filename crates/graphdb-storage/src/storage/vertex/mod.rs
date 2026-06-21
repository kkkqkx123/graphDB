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

pub mod column_store;
pub mod id_indexer;
pub mod vertex_table;
pub mod vertex_timestamp;

use crate::storage::types::StoragePropertyDef;

pub use column_store::ColumnStore;
pub use id_indexer::{IdIndexer, IdKey};
pub use vertex_table::VertexTable;
pub use vertex_timestamp::VertexTimestamp;

use crate::core::vertex_edge_path::Tag;
use crate::core::Value;

pub use crate::core::types::{LabelId, Timestamp, VertexId, INVALID_TIMESTAMP, MAX_TIMESTAMP};

#[derive(Debug, Clone)]
pub struct VertexRecord {
    pub vid: VertexId,
    pub internal_id: u32,
    pub properties: Vec<(String, Value)>,
}

impl From<&VertexRecord> for crate::core::Vertex {
    fn from(record: &VertexRecord) -> Self {
        let properties: std::collections::HashMap<String, Value> =
            record.properties.iter().cloned().collect();

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
    /// Schema version for migration tracking
    #[serde(default)]
    pub schema_version: u32,
    /// SHA256 hash of the schema definition for integrity checking
    #[serde(default)]
    pub schema_digest: String,
}

impl VertexSchema {
    /// Validate that the loaded schema matches the expected version.
    /// Returns Ok(()) if valid, Err with description if there are issues.
    ///
    /// Note: This method must be called explicitly by callers to enforce version checking.
    pub fn validate(&self, expected_version: u32) -> Result<(), String> {
        if self.schema_version != expected_version {
            return Err(format!(
                "Schema version mismatch: expected {}, got {}",
                expected_version, self.schema_version
            ));
        }

        // Digest validation reserved for future use
        Ok(())
    }

    /// Increment schema version when schema changes.
    pub fn increment_version(&mut self) {
        self.schema_version += 1;
    }
}
