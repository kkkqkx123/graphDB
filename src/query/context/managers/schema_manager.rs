//! Schema管理器接口 - 定义Schema管理的基本操作

use crate::core::error::ManagerResult;
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
    fn create_tag(
        &self,
        space_id: i32,
        tag_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;
    /// 删除Tag
    fn drop_tag(&self, space_id: i32, tag_id: i32) -> ManagerResult<()>;
    /// 获取Tag定义
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDef>;
    /// 列出指定Space的所有Tag
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDef>>;
    /// 检查Tag是否存在
    fn has_tag(&self, space_id: i32, tag_id: i32) -> bool;

    /// 创建EdgeType
    fn create_edge_type(
        &self,
        space_id: i32,
        edge_type_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;
    /// 删除EdgeType
    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> ManagerResult<()>;
    /// 获取EdgeType定义
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDef>;
    /// 列出指定Space的所有EdgeType
    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDef>>;
    /// 检查EdgeType是否存在
    fn has_edge_type(&self, space_id: i32, edge_type_id: i32) -> bool;

    /// 从磁盘加载Schema
    fn load_from_disk(&self) -> ManagerResult<()>;
    /// 保存Schema到磁盘
    fn save_to_disk(&self) -> ManagerResult<()>;

    /// Schema版本控制
    /// 创建Schema快照（新版本）
    fn create_schema_version(&self, space_id: i32, comment: Option<String>) -> ManagerResult<i32>;
    /// 获取指定版本的Schema
    fn get_schema_version(&self, space_id: i32, version: i32) -> Option<SchemaVersion>;
    /// 获取最新版本号
    fn get_latest_schema_version(&self, space_id: i32) -> Option<i32>;
    /// 获取Schema历史版本列表
    fn get_schema_history(&self, space_id: i32) -> ManagerResult<SchemaHistory>;
    /// 回滚Schema到指定版本
    fn rollback_schema(&self, space_id: i32, version: i32) -> ManagerResult<()>;
    /// 获取当前版本号
    fn get_current_version(&self, space_id: i32) -> Option<i32>;

    /// 字段级别的操作
    /// 为Tag添加字段
    fn add_tag_field(&self, space_id: i32, tag_name: &str, field: FieldDef) -> ManagerResult<()>;
    /// 删除Tag的字段
    fn drop_tag_field(&self, space_id: i32, tag_name: &str, field_name: &str) -> ManagerResult<()>;
    /// 修改Tag的字段
    fn alter_tag_field(&self, space_id: i32, tag_name: &str, field_name: &str, new_field: FieldDef) -> ManagerResult<()>;
    /// 为EdgeType添加字段
    fn add_edge_type_field(&self, space_id: i32, edge_type_name: &str, field: FieldDef) -> ManagerResult<()>;
    /// 删除EdgeType的字段
    fn drop_edge_type_field(&self, space_id: i32, edge_type_name: &str, field_name: &str) -> ManagerResult<()>;
    /// 修改EdgeType的字段
    fn alter_edge_type_field(&self, space_id: i32, edge_type_name: &str, field_name: &str, new_field: FieldDef) -> ManagerResult<()>;

    /// Schema变更历史
    /// 记录Schema变更
    fn record_schema_change(&self, space_id: i32, change: SchemaChange) -> ManagerResult<()>;
    /// 获取Schema变更历史
    fn get_schema_changes(&self, space_id: i32) -> ManagerResult<Vec<SchemaChange>>;
    /// 清除Schema变更历史
    fn clear_schema_changes(&self, space_id: i32) -> ManagerResult<()>;

    /// Schema导出/导入
    /// 导出Schema
    fn export_schema(&self, space_id: i32, config: SchemaExportConfig) -> ManagerResult<String>;
    /// 导入Schema
    fn import_schema(&self, space_id: i32, schema_data: &str) -> ManagerResult<SchemaImportResult>;
    /// 验证Schema兼容性
    fn validate_schema_compatibility(&self, space_id: i32, target_version: i32) -> ManagerResult<bool>;
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

/// Schema版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: i32,
    pub space_id: i32,
    pub tags: Vec<TagDef>,
    pub edge_types: Vec<EdgeTypeDef>,
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
