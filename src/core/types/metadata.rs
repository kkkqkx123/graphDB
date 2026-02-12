//! 元数据类型定义（统一版本）
//!
//! 用于存储引擎管理层操作的元数据结构
//! 此模块定义了所有 Schema 相关的核心类型，作为系统统一的类型定义来源

use crate::core::{DataType, Value};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct MetadataVersion {
    pub version: i32,
    pub timestamp: i64,
    pub description: String,
}

impl Default for MetadataVersion {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: chrono::Utc::now().timestamp_millis(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaVersion {
    pub version: i32,
    pub space_id: i32,
    pub tags: Vec<TagInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub created_at: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaHistory {
    pub space_id: i32,
    pub versions: Vec<SchemaVersion>,
    pub current_version: i64,
    pub timestamp: i64,
}

impl Default for SchemaHistory {
    fn default() -> Self {
        Self {
            space_id: 0,
            versions: Vec::new(),
            current_version: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum SchemaChangeType {
    AddProperty,
    DropProperty,
    ModifyProperty,
    AddIndex,
    DropIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaChange {
    pub change_type: SchemaChangeType,
    pub target: String,
    pub property: Option<PropertyDef>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
    pub comment: Option<String>,
}

impl PropertyDef {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
            default: None,
            comment: None,
        }
    }

    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn with_default(mut self, default: Option<Value>) -> Self {
        self.default = default;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertVertexInfo {
    pub space_id: i32,
    pub vertex_id: Value,
    pub tag_name: String,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertEdgeInfo {
    pub space_id: i32,
    pub src_vertex_id: Value,
    pub dst_vertex_id: Value,
    pub edge_name: String,
    pub rank: i64,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateTarget {
    pub space_name: String,
    pub label: String,
    pub id: Value,
    pub prop: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum UpdateOp {
    Set,
    Add,
    Subtract,
    Append,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateInfo {
    pub update_target: UpdateTarget,
    pub update_op: UpdateOp,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct PasswordInfo {
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub role: String,
    pub is_locked: bool,
    pub roles: std::collections::HashMap<i32, String>,
}

impl UserInfo {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            role: "user".to_string(),
            is_locked: false,
            roles: std::collections::HashMap::new(),
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = is_locked;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserAlterInfo {
    pub username: String,
    pub new_role: Option<String>,
    pub is_locked: Option<bool>,
}

impl UserAlterInfo {
    pub fn new(username: String) -> Self {
        Self {
            username,
            new_role: None,
            is_locked: None,
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.new_role = Some(role);
        self
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct ClusterInfo {
    pub cluster_id: i32,
    pub nodes: Vec<String>,
    pub total_space: i64,
    pub used_space: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaExportConfig {
    pub space_id: Option<i32>,
    pub format: ExportFormat,
    pub include_comments: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum ExportFormat {
    JSON,
    YAML,
    Rust,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaImportResult {
    pub success: bool,
    pub space_name: String,
    pub imported_items: i32,
    pub imported_tags: Vec<String>,
    pub imported_edge_types: Vec<String>,
    pub skipped_items: Vec<String>,
    pub errors: Vec<String>,
}

impl Default for SchemaImportResult {
    fn default() -> Self {
        Self {
            success: false,
            space_name: String::new(),
            imported_items: 0,
            imported_tags: Vec::new(),
            imported_edge_types: Vec::new(),
            skipped_items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl SchemaImportResult {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceInfo {
    pub space_id: i32,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub vid_type: DataType,
    pub tags: Vec<TagInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub version: MetadataVersion,
    pub comment: Option<String>,
}

impl SpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_id: 0,
            space_name,
            partition_num: 1,
            replica_factor: 1,
            vid_type: DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: MetadataVersion::default(),
            comment: None,
        }
    }

    pub fn with_partition_num(mut self, partition_num: i32) -> Self {
        self.partition_num = partition_num;
        self
    }

    pub fn with_replica_factor(mut self, replica_factor: i32) -> Self {
        self.replica_factor = replica_factor;
        self
    }

    pub fn with_vid_type(mut self, vid_type: DataType) -> Self {
        self.vid_type = vid_type;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TagInfo {
    pub tag_id: i32,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl TagInfo {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_id: 0,
            tag_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct EdgeTypeInfo {
    pub edge_type_id: i32,
    pub edge_type_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl EdgeTypeInfo {
    pub fn new(edge_type_name: String) -> Self {
        Self {
            edge_type_id: 0,
            edge_type_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for SpaceInfo {
    fn default() -> Self {
        SpaceInfo::new("default".to_string())
    }
}

impl Default for TagInfo {
    fn default() -> Self {
        TagInfo::new("default".to_string())
    }
}

impl Default for EdgeTypeInfo {
    fn default() -> Self {
        EdgeTypeInfo::new("default".to_string())
    }
}
