//! 模式定义结构
//!
//! 包含图数据库模式定义的相关数据结构

use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 表示图模式定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDef {
    pub vertex_types: HashMap<String, Vec<PropertyDef>>,
    pub edge_types: HashMap<String, Vec<PropertyDef>>,
    pub indexes: Vec<IndexDef>,
}

/// 表示模式中的属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub indexed: bool,
}

/// 表示属性的数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List(Box<DataType>),
    Map(String, Box<DataType>), // (键类型, 值类型)
    Custom(String),             // 自定义类型名称
}

/// 表示索引定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: String,
    pub entity_type: EntityType,
    pub property_name: String,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Vertex(String), // 顶点类型名称
    Edge(String),   // 边类型名称
}
