//! 索引类型定义模块
//!
//! 提供统一的索引类型定义，包括索引状态、类型、结构等

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
    #[serde(rename = "fulltext")]
    FulltextIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct IndexField {
    pub name: String,
    pub value_type: Value,
    pub is_nullable: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub space_id: i32,
    pub schema_name: String,
    pub fields: Vec<IndexField>,
    pub index_type: IndexType,
    pub status: IndexStatus,
    pub is_unique: bool,
    pub comment: Option<String>,
}

impl Index {
    pub fn new(
        id: i32,
        name: String,
        space_id: i32,
        schema_name: String,
        fields: Vec<IndexField>,
        index_type: IndexType,
        is_unique: bool,
    ) -> Self {
        Self {
            id,
            name,
            space_id,
            schema_name,
            fields,
            index_type,
            status: IndexStatus::Active,
            is_unique,
            comment: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexInfo {
    pub index_id: i32,
    pub index_name: String,
    pub total_entries: usize,
    pub unique_entries: usize,
    pub last_updated: i64,
    pub memory_usage_bytes: usize,
    pub query_count: u64,
    pub avg_query_time_ms: f64,
}

impl IndexInfo {
    pub fn new(index_id: i32, index_name: String) -> Self {
        Self {
            index_id,
            index_name,
            total_entries: 0,
            unique_entries: 0,
            last_updated: 0,
            memory_usage_bytes: 0,
            query_count: 0,
            avg_query_time_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexOptimization {
    pub index_id: i32,
    pub index_name: String,
    pub suggestions: Vec<String>,
    pub priority: String,
}

impl IndexOptimization {
    pub fn new(index_id: i32, index_name: String) -> Self {
        Self {
            index_id,
            index_name,
            suggestions: Vec::new(),
            priority: String::new(),
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

        let active_json = serde_json::to_string(&active).expect("Failed to serialize IndexStatus::Active in test");
        let failed_json = serde_json::to_string(&failed).expect("Failed to serialize IndexStatus::Failed in test");

        assert!(active_json.contains("active"));
        assert!(failed_json.contains("failed"));
        assert!(failed_json.contains("error"));
    }

    #[test]
    fn test_index_type_serialization() {
        let tag = IndexType::TagIndex;
        let edge = IndexType::EdgeIndex;

        let tag_json = serde_json::to_string(&tag).expect("Failed to serialize IndexType::TagIndex in test");
        let edge_json = serde_json::to_string(&edge).expect("Failed to serialize IndexType::EdgeIndex in test");

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

        let index = Index::new(
            1,
            "person_name_idx".to_string(),
            1,
            "person".to_string(),
            fields,
            IndexType::TagIndex,
            false,
        );

        assert_eq!(index.id, 1);
        assert_eq!(index.name, "person_name_idx");
        assert_eq!(index.schema_name, "person");
        assert_eq!(index.fields.len(), 1);
        assert_eq!(index.status, IndexStatus::Active);
    }

    #[test]
    fn test_index_info_creation() {
        let info = IndexInfo::new(1, "test_index".to_string());

        assert_eq!(info.index_id, 1);
        assert_eq!(info.index_name, "test_index");
        assert_eq!(info.total_entries, 0);
        assert_eq!(info.query_count, 0);
    }
}
