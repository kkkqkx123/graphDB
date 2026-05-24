//! Metadata Type Definitions
//!
//! This module defines the core metadata types used throughout the query planning
//! and execution process.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub index_type: IndexType,
    pub properties: HashMap<String, Value>,
}

impl IndexMetadata {
    pub fn new(
        index_name: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        index_type: IndexType,
    ) -> Self {
        Self {
            index_name,
            space_id,
            tag_name,
            field_name,
            index_type,
            properties: HashMap::new(),
        }
    }
}

/// Index type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    Vector,
    Fulltext,
    Property,
    Composite,
    Native,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::Vector => write!(f, "VECTOR"),
            IndexType::Fulltext => write!(f, "FULLTEXT"),
            IndexType::Property => write!(f, "PROPERTY"),
            IndexType::Composite => write!(f, "COMPOSITE"),
            IndexType::Native => write!(f, "NATIVE"),
        }
    }
}

/// Tag metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadata {
    pub tag_name: String,
    pub space_id: u64,
    pub properties: Vec<PropertyDefinition>,
    pub indexes: Vec<String>,
}

impl TagMetadata {
    pub fn new(tag_name: String, space_id: u64) -> Self {
        Self {
            tag_name,
            space_id,
            properties: Vec::new(),
            indexes: Vec::new(),
        }
    }
}

/// Edge type metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeMetadata {
    pub edge_type: String,
    pub space_id: u64,
    pub properties: Vec<PropertyDefinition>,
    pub indexes: Vec<String>,
}

impl EdgeTypeMetadata {
    pub fn new(edge_type: String, space_id: u64) -> Self {
        Self {
            edge_type,
            space_id,
            properties: Vec::new(),
            indexes: Vec::new(),
        }
    }
}

/// Property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub name: String,
    pub data_type: PropertyType,
    pub nullable: bool,
    pub default_value: Option<Value>,
}

impl PropertyDefinition {
    pub fn new(name: String, data_type: PropertyType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            default_value: None,
        }
    }
}

/// Property type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Vector,
    Date,
    DateTime,
    Timestamp,
    List,
    Map,
    Vertex,
    Edge,
    Path,
    Geography,
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::Bool => write!(f, "BOOL"),
            PropertyType::Int => write!(f, "INT"),
            PropertyType::Float => write!(f, "FLOAT"),
            PropertyType::String => write!(f, "STRING"),
            PropertyType::Vector => write!(f, "VECTOR"),
            PropertyType::Date => write!(f, "DATE"),
            PropertyType::DateTime => write!(f, "DATETIME"),
            PropertyType::Timestamp => write!(f, "TIMESTAMP"),
            PropertyType::List => write!(f, "LIST"),
            PropertyType::Map => write!(f, "MAP"),
            PropertyType::Vertex => write!(f, "VERTEX"),
            PropertyType::Edge => write!(f, "EDGE"),
            PropertyType::Path => write!(f, "PATH"),
            PropertyType::Geography => write!(f, "GEOGRAPHY"),
        }
    }
}

/// Value type (simplified for metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Null => write!(f, "NULL"),
        }
    }
}

/// Versioned metadata wrapper
#[derive(Debug, Clone)]
pub struct VersionedMetadata<T> {
    pub metadata: T,
    pub version: u64,
    pub timestamp: u64,
}

impl<T> VersionedMetadata<T> {
    pub fn new(metadata: T, version: u64) -> Self {
        Self {
            metadata,
            version,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn is_valid(&self, current_version: u64) -> bool {
        self.version == current_version
    }
}
