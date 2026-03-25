//! Index Type Definition Module
//!
//! Provide a unified index type definition, including index state, type, structure, etc.

use super::property_trait::PropertyTypeTrait;
use crate::core::Value;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum IndexStatus {
    #[serde(rename = "creating")]
    Creating,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "dropped")]
    Dropped,
    #[serde(rename = "failed")]
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum IndexType {
    #[serde(rename = "tag")]
    TagIndex,
    #[serde(rename = "edge")]
    EdgeIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct IndexField {
    pub name: String,
    pub value_type: Value,
    pub is_nullable: bool,
}

impl PropertyTypeTrait for IndexField {
    fn name(&self) -> &str {
        &self.name
    }

    fn data_type(&self) -> &crate::core::DataType {
        match &self.value_type {
            Value::Int(_) => &crate::core::DataType::Int,
            Value::Float(_) => &crate::core::DataType::Float,
            Value::Bool(_) => &crate::core::DataType::Bool,
            Value::String(_) => &crate::core::DataType::String,
            Value::Null(_) => &crate::core::DataType::Null,
            _ => &crate::core::DataType::String,
        }
    }

    fn is_nullable(&self) -> bool {
        self.is_nullable
    }

    fn default_value(&self) -> Option<&Value> {
        None
    }

    fn comment(&self) -> Option<&str> {
        None
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_data_type(&mut self, _data_type: crate::core::DataType) {}

    fn set_nullable(&mut self, nullable: bool) {
        self.is_nullable = nullable;
    }

    fn set_default_value(&mut self, _default: Option<Value>) {}

    fn set_comment(&mut self, _comment: Option<String>) {}

    fn property_type_name(&self) -> &'static str {
        "IndexField"
    }
}

impl IndexField {
    pub fn new(name: String, value_type: Value, is_nullable: bool) -> Self {
        Self {
            name,
            value_type,
            is_nullable,
        }
    }
}

/// Indexed Configuration Structures
#[derive(Debug, Clone)]
pub struct IndexConfig {
    pub id: i32,
    pub name: String,
    pub space_id: u64,
    pub schema_name: String,
    pub fields: Vec<IndexField>,
    pub properties: Vec<String>,
    pub index_type: IndexType,
    pub is_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub space_id: u64,
    pub schema_name: String,
    pub fields: Vec<IndexField>,
    pub properties: Vec<String>,
    pub index_type: IndexType,
    pub status: IndexStatus,
    pub is_unique: bool,
    pub comment: Option<String>,
}

impl Index {
    /// Creating an Index Using a Configuration Structure
    pub fn new(config: IndexConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            space_id: config.space_id,
            schema_name: config.schema_name,
            fields: config.fields,
            properties: config.properties,
            index_type: config.index_type,
            status: IndexStatus::Active,
            is_unique: config.is_unique,
            comment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_status_serialization() {
        let active = IndexStatus::Active;
        let failed = IndexStatus::Failed("error".to_string());

        let active_json = serde_json::to_string(&active)
            .expect("Failed to serialize IndexStatus::Active in test");
        let failed_json = serde_json::to_string(&failed)
            .expect("Failed to serialize IndexStatus::Failed in test");

        assert!(active_json.contains("active"));
        assert!(failed_json.contains("failed"));
        assert!(failed_json.contains("error"));
    }

    #[test]
    fn test_index_type_serialization() {
        let tag = IndexType::TagIndex;
        let edge = IndexType::EdgeIndex;

        let tag_json =
            serde_json::to_string(&tag).expect("Failed to serialize IndexType::TagIndex in test");
        let edge_json =
            serde_json::to_string(&edge).expect("Failed to serialize IndexType::EdgeIndex in test");

        assert!(tag_json.contains("tag"));
        assert!(edge_json.contains("edge"));
    }

    #[test]
    fn test_index_field_creation() {
        let field = IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        );

        assert_eq!(field.name, "name");
        assert!(matches!(field.value_type, Value::String(_)));
        assert!(!field.is_nullable);
    }

    #[test]
    fn test_index_creation() {
        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        let config = IndexConfig {
            id: 1,
            name: "person_name_idx".to_string(),
            space_id: 1,
            schema_name: "person".to_string(),
            fields,
            properties: vec![],
            index_type: IndexType::TagIndex,
            is_unique: false,
        };

        let index = Index::new(config);

        assert_eq!(index.id, 1);
        assert_eq!(index.name, "person_name_idx");
        assert_eq!(index.schema_name, "person");
        assert_eq!(index.fields.len(), 1);
        assert_eq!(index.status, IndexStatus::Active);
    }
}
