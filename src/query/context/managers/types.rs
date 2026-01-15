//! 管理器公共类型定义
//!
//! 统一管理所有管理器使用的公共类型，避免类型冲突

use serde::{Deserialize, Serialize};

/// 属性类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
}

/// 属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub type_: PropertyType,
    pub nullable: bool,
    pub default: Option<String>,
}

/// 字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

/// 标签定义（元数据客户端使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDef {
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
}

/// 带ID的标签定义（Schema管理器使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDefWithId {
    pub tag_id: i32,
    pub tag_name: String,
    pub fields: Vec<FieldDef>,
    pub comment: Option<String>,
}

/// 边类型定义（元数据客户端使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeDef {
    pub edge_name: String,
    pub properties: Vec<PropertyDef>,
}

/// 带ID的边类型定义（Schema管理器使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeDefWithId {
    pub edge_type_id: i32,
    pub edge_type_name: String,
    pub fields: Vec<FieldDef>,
    pub comment: Option<String>,
}

/// 元数据版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataVersion {
    pub version: i32,
    pub timestamp: i64,
    pub description: String,
}

/// 集群信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_id: String,
    pub meta_servers: Vec<String>,
    pub storage_servers: Vec<String>,
    pub version: MetadataVersion,
}

/// 空间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub space_id: i32,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub tags: Vec<TagDef>,
    pub edge_types: Vec<EdgeTypeDef>,
    pub version: MetadataVersion,
}

/// Schema信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: std::collections::HashMap<String, String>,
    pub is_vertex: bool,
}

/// 字符集信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

impl Default for CharsetInfo {
    fn default() -> Self {
        Self {
            charset: "utf8mb4".to_string(),
            collation: "utf8mb4_general_ci".to_string(),
        }
    }
}

/// Schema版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: i32,
    pub space_id: i32,
    pub tags: Vec<TagDefWithId>,
    pub edge_types: Vec<EdgeTypeDefWithId>,
    pub created_at: i64,
    pub comment: Option<String>,
}

/// Schema历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaHistory {
    pub space_id: i32,
    pub versions: Vec<SchemaVersion>,
    pub current_version: i32,
}

/// Schema变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChangeType {
    CreateTag,
    DropTag,
    AlterTag,
    CreateEdgeType,
    DropEdgeType,
    AlterEdgeType,
}

/// Schema变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub change_type: SchemaChangeType,
    pub target_name: String,
    pub description: String,
    pub timestamp: i64,
}

/// Schema导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaExportConfig {
    pub include_versions: bool,
    pub include_comments: bool,
    pub format: String,
}

/// Schema导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaImportResult {
    pub imported_tags: Vec<String>,
    pub imported_edge_types: Vec<String>,
    pub skipped_items: Vec<String>,
    pub errors: Vec<String>,
}
