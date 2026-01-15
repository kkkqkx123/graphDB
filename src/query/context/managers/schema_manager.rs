//! Schema管理器接口 - 定义Schema管理的基本操作

use crate::core::error::ManagerResult;
use super::types::{
    CharsetInfo, EdgeTypeDefWithId, FieldDef, Schema, SchemaChange, SchemaChangeType,
    SchemaExportConfig, SchemaHistory, SchemaImportResult, SchemaVersion, TagDefWithId,
};

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
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDefWithId>;
    /// 列出指定Space的所有Tag
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDefWithId>>;
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
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDefWithId>;
    /// 列出指定Space的所有EdgeType
    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDefWithId>>;
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
