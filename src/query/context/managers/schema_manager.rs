//! Schema管理器接口 - 定义Schema管理的基本操作

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

/// Tag定义 - 用于Vertex的类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDef {
    pub tag_id: i32,
    pub tag_name: String,
    pub fields: Vec<FieldDef>,
    pub comment: Option<String>,
}

/// EdgeType定义 - 用于Edge的类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeDef {
    pub edge_type_id: i32,
    pub edge_type_name: String,
    pub fields: Vec<FieldDef>,
    pub comment: Option<String>,
}

/// Schema信息 - 表示数据库Schema（保留向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, String>,
    pub is_vertex: bool,
}

/// Schema管理器接口 - 定义Schema管理的基本操作
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    /// 获取指定名称的Schema
    fn get_schema(&self, name: &str) -> Option<Schema>;
    /// 列出所有Schema名称
    fn list_schemas(&self) -> Vec<String>;
    /// 检查Schema是否存在
    fn has_schema(&self, name: &str) -> bool;
    
    /// 创建Tag
    fn create_tag(&self, space_id: i32, tag_name: &str, fields: Vec<FieldDef>) -> Result<i32, String>;
    /// 删除Tag
    fn drop_tag(&self, space_id: i32, tag_id: i32) -> Result<(), String>;
    /// 获取Tag定义
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDef>;
    /// 列出指定Space的所有Tag
    fn list_tags(&self, space_id: i32) -> Result<Vec<TagDef>, String>;
    /// 检查Tag是否存在
    fn has_tag(&self, space_id: i32, tag_id: i32) -> bool;
    
    /// 创建EdgeType
    fn create_edge_type(&self, space_id: i32, edge_type_name: &str, fields: Vec<FieldDef>) -> Result<i32, String>;
    /// 删除EdgeType
    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> Result<(), String>;
    /// 获取EdgeType定义
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDef>;
    /// 列出指定Space的所有EdgeType
    fn list_edge_types(&self, space_id: i32) -> Result<Vec<EdgeTypeDef>, String>;
    /// 检查EdgeType是否存在
    fn has_edge_type(&self, space_id: i32, edge_type_id: i32) -> bool;
    
    /// 从磁盘加载Schema
    fn load_from_disk(&self) -> Result<(), String>;
    /// 保存Schema到磁盘
    fn save_to_disk(&self) -> Result<(), String>;
}

/// 字符集信息 - 管理字符集和排序规则
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
