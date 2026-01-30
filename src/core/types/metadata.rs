//! 元数据类型定义（统一版本）
//!
//! 用于存储引擎管理层操作的元数据结构
//! 此模块定义了所有 Schema 相关的核心类型，作为系统统一的类型定义来源

use crate::core::{DataType, Value};
use serde::{Deserialize, Serialize};

/// 统一的 SpaceInfo 结构
///
/// 代表图数据库中的一个图空间（Graph Space），
/// 包含空间的基本信息和关联的 Schema 定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    pub fn with_tags(mut self, tags: Vec<TagInfo>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_edge_types(mut self, edge_types: Vec<EdgeTypeInfo>) -> Self {
        self.edge_types = edge_types;
        self
    }
}

/// 元数据版本信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataVersion {
    pub version: i32,
    pub timestamp: i64,
    pub description: String,
}

impl Default for MetadataVersion {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: 0,
            description: String::new(),
        }
    }
}

impl MetadataVersion {
    pub fn new(description: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;
        Self {
            version: 1,
            timestamp,
            description,
        }
    }

    pub fn increment(&self, description: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;
        Self {
            version: self.version + 1,
            timestamp,
            description,
        }
    }
}

/// 属性定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            nullable: true,
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

/// 标签信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 边类型信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 索引信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexInfo {
    pub index_id: i32,
    pub index_name: String,
    pub space_id: i32,
    pub target_type: IndexTargetType,
    pub target_name: String,
    pub properties: Vec<String>,
    pub is_unique: bool,
    pub status: IndexStatus,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexTargetType {
    Tag,
    EdgeType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexStatus {
    Creating,
    Active,
    Dropped,
    Failed,
}

impl IndexInfo {
    pub fn new(index_name: String, target_type: IndexTargetType, target_name: String) -> Self {
        Self {
            index_id: 0,
            index_name,
            space_id: 0,
            target_type,
            target_name,
            properties: Vec::new(),
            is_unique: false,
            status: IndexStatus::Creating,
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_unique(mut self, is_unique: bool) -> Self {
        self.is_unique = is_unique;
        self
    }
}

/// 集群信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_id: String,
    pub meta_servers: Vec<String>,
    pub storage_servers: Vec<String>,
    pub version: MetadataVersion,
}

impl ClusterInfo {
    pub fn new(cluster_id: String) -> Self {
        Self {
            cluster_id,
            meta_servers: Vec::new(),
            storage_servers: Vec::new(),
            version: MetadataVersion::default(),
        }
    }

    pub fn with_meta_servers(mut self, servers: Vec<String>) -> Self {
        self.meta_servers = servers;
        self
    }

    pub fn with_storage_servers(mut self, servers: Vec<String>) -> Self {
        self.storage_servers = servers;
        self
    }
}

/// Schema 版本快照
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: i32,
    pub space_id: i32,
    pub tags: Vec<TagInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub created_at: i64,
    pub comment: Option<String>,
}

/// Schema 历史记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaHistory {
    pub space_id: i32,
    pub versions: Vec<SchemaVersion>,
    pub current_version: i32,
}

/// Schema 变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaChangeType {
    CreateTag,
    DropTag,
    AlterTag,
    CreateEdgeType,
    DropEdgeType,
    AlterEdgeType,
    CreateIndex,
    DropIndex,
}

/// Schema 变更记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaChange {
    pub change_type: SchemaChangeType,
    pub target_name: String,
    pub description: String,
    pub timestamp: i64,
}

impl SchemaChange {
    pub fn new(change_type: SchemaChangeType, target_name: String, description: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;
        Self {
            change_type,
            target_name,
            description,
            timestamp,
        }
    }
}

/// Schema 导出配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaExportConfig {
    pub include_versions: bool,
    pub include_comments: bool,
    pub format: ExportFormat,
    pub space_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
   Yaml,
    Sql,
}

impl Default for SchemaExportConfig {
    fn default() -> Self {
        Self {
            include_versions: false,
            include_comments: true,
            format: ExportFormat::Json,
            space_id: None,
        }
    }
}

/// Schema 导入结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaImportResult {
    pub imported_tags: Vec<String>,
    pub imported_edge_types: Vec<String>,
    pub imported_indexes: Vec<String>,
    pub skipped_items: Vec<String>,
    pub errors: Vec<String>,
}

impl SchemaImportResult {
    pub fn new() -> Self {
        Self {
            imported_tags: Vec::new(),
            imported_edge_types: Vec::new(),
            imported_indexes: Vec::new(),
            skipped_items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// 字符集信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 插入顶点信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertVertexInfo {
    pub space_name: String,
    pub tag_name: String,
    pub vertex_id: Value,
    pub properties: Vec<(String, Value)>,
}

/// 插入边信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertEdgeInfo {
    pub space_name: String,
    pub edge_name: String,
    pub src_vertex_id: Value,
    pub dst_vertex_id: Value,
    pub rank: i64,
    pub properties: Vec<(String, Value)>,
}

/// 更新目标
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTarget {
    Vertex {
        vertex_id: Value,
        tag_name: String,
    },
    Edge {
        src_vertex_id: Value,
        dst_vertex_id: Value,
        rank: i64,
        edge_name: String,
    },
}

/// 更新操作
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateOp {
    Set {
        property: String,
        value: Value,
    },
    Delete {
        property: String,
    },
}

/// 更新信息
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub space_name: String,
    pub target: UpdateTarget,
    pub operations: Vec<UpdateOp>,
}

/// 密码信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordInfo {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}
