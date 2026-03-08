//! Schema 类型基础 trait 定义
//!
//! 本模块定义 Schema 相关的通用 trait，用于抽象 TagInfo 和 EdgeTypeInfo 的共同属性

use super::property::PropertyDef;

/// Schema 信息 trait
///
/// 定义 TagInfo 和 EdgeTypeInfo 的共同接口
pub trait SchemaInfo: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync {
    /// 获取 Schema ID
    fn schema_id(&self) -> i32;

    /// 获取 Schema 名称
    fn schema_name(&self) -> &str;

    /// 获取属性列表
    fn properties(&self) -> &[PropertyDef];

    /// 获取注释
    fn comment(&self) -> Option<&str>;

    /// 获取 TTL 持续时间
    fn ttl_duration(&self) -> Option<i64>;

    /// 获取 TTL 列名
    fn ttl_col(&self) -> Option<&str>;

    /// 设置 Schema ID
    fn set_schema_id(&mut self, id: i32);

    /// 设置属性列表
    fn set_properties(&mut self, properties: Vec<PropertyDef>);

    /// 设置注释
    fn set_comment(&mut self, comment: Option<String>);

    /// 设置 TTL
    fn set_ttl(&mut self, duration: Option<i64>, col: Option<String>);

    /// 获取 Schema 类型名称（用于区分 Tag 或 Edge）
    fn schema_type_name(&self) -> &'static str;

    /// 是否为 Tag 类型
    fn is_tag(&self) -> bool;

    /// 是否为 Edge 类型
    fn is_edge(&self) -> bool;
}
