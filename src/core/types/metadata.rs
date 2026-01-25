//! 元数据类型定义
//!
//! 用于存储引擎管理层操作的元数据结构

use crate::core::{DataType, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub vid_type: DataType,
    pub comment: Option<String>,
}

impl SpaceInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            partition_num: 1,
            replica_factor: 1,
            vid_type: DataType::String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
}

impl PropertyDef {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            default: None,
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagInfo {
    pub space_name: String,
    pub name: String,
    pub properties: Vec<super::graph_schema::PropertyType>,
    pub comment: Option<String>,
}

impl TagInfo {
    pub fn new(space_name: String, name: String) -> Self {
        Self {
            space_name,
            name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<super::graph_schema::PropertyType>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagTypeDesc {
    pub tag_name: String,
    pub fields: Vec<super::graph_schema::PropertyType>,
    pub comment: Option<String>,
}

impl TagTypeDesc {
    pub fn from_schema(schema: &TagInfo) -> Self {
        Self {
            tag_name: schema.name.clone(),
            fields: schema.properties.clone(),
            comment: schema.comment.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeTypeDesc {
    pub edge_type_name: String,
    pub fields: Vec<super::graph_schema::PropertyType>,
    pub comment: Option<String>,
}

impl EdgeTypeDesc {
    pub fn from_schema(schema: &EdgeTypeSchema) -> Self {
        Self {
            edge_type_name: schema.name.clone(),
            fields: schema.properties.clone(),
            comment: schema.comment.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeTypeSchema {
    pub space_name: String,
    pub name: String,
    pub properties: Vec<super::graph_schema::PropertyType>,
    pub comment: Option<String>,
}

impl EdgeTypeSchema {
    pub fn new(space_name: String, name: String) -> Self {
        Self {
            space_name,
            name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<super::graph_schema::PropertyType>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexInfo {
    pub space_name: String,
    pub name: String,
    pub target_type: String,
    pub target_name: String,
    pub properties: Vec<String>,
    pub comment: Option<String>,
}

impl IndexInfo {
    pub fn new(space_name: String, name: String, target_type: String, target_name: String) -> Self {
        Self {
            space_name,
            name,
            target_type,
            target_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertVertexInfo {
    pub space_name: String,
    pub tag_name: String,
    pub vertex_id: String,
    pub properties: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertEdgeInfo {
    pub space_name: String,
    pub edge_name: String,
    pub src_vertex_id: String,
    pub dst_vertex_id: String,
    pub rank: i64,
    pub properties: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTarget {
    Vertex {
        vertex_id: String,
        tag_name: String,
    },
    Edge {
        src_vertex_id: String,
        dst_vertex_id: String,
        rank: i64,
        edge_name: String,
    },
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub space_name: String,
    pub target: UpdateTarget,
    pub operations: Vec<UpdateOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordInfo {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}
