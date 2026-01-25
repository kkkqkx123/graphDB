//! Schema接口拆分模块
//!
//! 将原有的SchemaManager trait拆分为多个专门的接口：
//! - SchemaReader: 只读操作
//! - SchemaWriter: 写操作
//! - SchemaVersionControl: 版本控制操作
//! - SchemaChangeTracker: 变更追踪
//! - SchemaImportExport: 导入导出

use crate::core::error::ManagerResult;
use super::types::{
    EdgeTypeDefWithId, FieldDef, Schema, SchemaChange,
    SchemaExportConfig, SchemaHistory, SchemaImportResult, SchemaVersion, TagDefWithId,
};

/// Schema读取接口 - 提供Schema的只读访问
///
/// 包含所有查询场景需要的Schema信息获取方法。
/// 实现此接口的类型必须是Send + Sync以支持并发访问。
pub trait SchemaReader: Send + Sync + std::fmt::Debug {
    /// 获取指定名称的Schema
    fn get_schema(&self, name: &str) -> Option<Schema>;

    /// 列出所有Schema名称
    fn list_schemas(&self) -> Vec<String>;

    /// 检查Schema是否存在
    fn has_schema(&self, name: &str) -> bool;

    /// 获取Tag定义
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDefWithId>;

    /// 获取Tag定义（按名称）
    fn get_tag_by_name(&self, space_id: i32, tag_name: &str) -> Option<TagDefWithId>;

    /// 列出指定Space的所有Tag
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDefWithId>>;

    /// 检查Tag是否存在
    fn has_tag(&self, space_id: i32, tag_id: i32) -> bool;

    /// 检查Tag是否存在（按名称）
    fn has_tag_by_name(&self, space_id: i32, tag_name: &str) -> bool;

    /// 获取EdgeType定义
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDefWithId>;

    /// 获取EdgeType定义（按名称）
    fn get_edge_type_by_name(&self, space_id: i32, edge_type_name: &str) -> Option<EdgeTypeDefWithId>;

    /// 列出指定Space的所有EdgeType
    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDefWithId>>;

    /// 检查EdgeType是否存在
    fn has_edge_type(&self, space_id: i32, edge_type_id: i32) -> bool;

    /// 检查EdgeType是否存在（按名称）
    fn has_edge_type_by_name(&self, space_id: i32, edge_type_name: &str) -> bool;

    /// 获取指定版本的Schema
    fn get_schema_version(&self, space_id: i32, version: i32) -> Option<SchemaVersion>;

    /// 获取最新版本号
    fn get_latest_schema_version(&self, space_id: i32) -> Option<i32>;

    /// 获取当前版本号
    fn get_current_version(&self, space_id: i32) -> Option<i32>;

    /// 获取Schema历史版本列表
    fn get_schema_history(&self, space_id: i32) -> ManagerResult<SchemaHistory>;

    /// 获取Schema变更历史
    fn get_schema_changes(&self, space_id: i32) -> ManagerResult<Vec<SchemaChange>>;
}

/// Schema写入接口 - 提供Schema的修改操作
///
/// 包含所有需要修改Schema的方法。
/// 实现此接口的类型必须是Send + Sync以支持并发访问。
pub trait SchemaWriter: Send + Sync + std::fmt::Debug {
    /// 创建Tag
    fn create_tag(
        &self,
        space_id: i32,
        tag_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;

    /// 删除Tag
    fn drop_tag(&self, space_id: i32, tag_id: i32) -> ManagerResult<()>;

    /// 修改Tag（完整替换）
    fn alter_tag(&self, space_id: i32, tag_id: i32, new_fields: Vec<FieldDef>) -> ManagerResult<()>;

    /// 创建EdgeType
    fn create_edge_type(
        &self,
        space_id: i32,
        edge_type_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;

    /// 删除EdgeType
    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> ManagerResult<()>;

    /// 修改EdgeType（完整替换）
    fn alter_edge_type(
        &self,
        space_id: i32,
        edge_type_id: i32,
        new_fields: Vec<FieldDef>,
    ) -> ManagerResult<()>;

    /// 为Tag添加字段
    fn add_tag_field(&self, space_id: i32, tag_name: &str, field: FieldDef) -> ManagerResult<()>;

    /// 删除Tag的字段
    fn drop_tag_field(
        &self,
        space_id: i32,
        tag_name: &str,
        field_name: &str,
    ) -> ManagerResult<()>;

    /// 修改Tag的字段
    fn alter_tag_field(
        &self,
        space_id: i32,
        tag_name: &str,
        field_name: &str,
        new_field: FieldDef,
    ) -> ManagerResult<()>;

    /// 为EdgeType添加字段
    fn add_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field: FieldDef,
    ) -> ManagerResult<()>;

    /// 删除EdgeType的字段
    fn drop_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field_name: &str,
    ) -> ManagerResult<()>;

    /// 修改EdgeType的字段
    fn alter_edge_type_field(
        &self,
        space_id: i32,
        edge_type_name: &str,
        field_name: &str,
        new_field: FieldDef,
    ) -> ManagerResult<()>;

    /// 记录Schema变更
    fn record_schema_change(&self, space_id: i32, change: SchemaChange) -> ManagerResult<()>;

    /// 清除Schema变更历史
    fn clear_schema_changes(&self, space_id: i32) -> ManagerResult<()>;
}

/// Schema版本控制接口
///
/// 提供Schema版本的管理能力，支持版本回滚和历史查询。
pub trait SchemaVersionControl: Send + Sync + std::fmt::Debug {
    /// 创建Schema快照（新版本）
    fn create_schema_version(&self, space_id: i32, comment: Option<String>) -> ManagerResult<i32>;

    /// 回滚Schema到指定版本
    fn rollback_schema(&self, space_id: i32, version: i32) -> ManagerResult<()>;
}

/// Schema持久化接口
///
/// 提供Schema的持久化操作。
pub trait SchemaPersistence: Send + Sync + std::fmt::Debug {
    /// 从磁盘加载Schema
    fn load_from_disk(&self) -> ManagerResult<()>;

    /// 保存Schema到磁盘
    fn save_to_disk(&self) -> ManagerResult<()>;
}

/// Schema导入导出接口
///
/// 提供Schema的导入导出功能。
pub trait SchemaImportExport: Send + Sync + std::fmt::Debug {
    /// 导出Schema
    fn export_schema(&self, space_id: i32, config: SchemaExportConfig) -> ManagerResult<String>;

    /// 导入Schema
    fn import_schema(
        &self,
        space_id: i32,
        schema_data: &str,
    ) -> ManagerResult<SchemaImportResult>;

    /// 验证Schema兼容性
    fn validate_schema_compatibility(
        &self,
        space_id: i32,
        target_version: i32,
    ) -> ManagerResult<bool>;
}

/// 统一的Schema管理器接口
///
/// 组合所有Schema接口，提供完整功能。
/// 这是原有SchemaManager trait的兼容包装。
pub trait SchemaManager:
    SchemaReader + SchemaWriter + SchemaVersionControl + SchemaPersistence + SchemaImportExport
{
}

impl<T> SchemaManager for T where
    T: SchemaReader
        + SchemaWriter
        + SchemaVersionControl
        + SchemaPersistence
        + SchemaImportExport
        + Send
        + Sync
        + std::fmt::Debug
{
}

/// Schema管理器构建器
///
/// 提供流畅的接口来构建SchemaManager实例。
#[derive(Debug, Default)]
pub struct SchemaManagerBuilder;

impl SchemaManagerBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self
    }

    /// 构建包含所有功能的SchemaManager
    ///
    /// 返回一个实现了所有接口的复合对象。
    /// 在实际实现中，可以将不同的接口实现组合在一起。
    #[cfg(feature = "schema-manager-default")]
    pub fn build<T: SchemaManager>(self, manager: T) -> T {
        manager
    }
}
