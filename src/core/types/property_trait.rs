//! 属性类型基础 trait 定义
//!
//! 本模块定义属性相关的通用 trait，用于抽象 PropertyDef、PropertyType、IndexField 的共同属性

use crate::core::{DataType, Value};

/// 属性类型 trait
///
/// 定义 PropertyDef、PropertyType、IndexField 的共同接口
pub trait PropertyTypeTrait: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync {
    /// 获取属性名称
    fn name(&self) -> &str;

    /// 获取数据类型
    fn data_type(&self) -> &DataType;

    /// 是否可为空
    fn is_nullable(&self) -> bool;

    /// 获取默认值（如果有）
    fn default_value(&self) -> Option<&Value>;

    /// 获取注释（如果有）
    fn comment(&self) -> Option<&str>;

    /// 设置属性名称
    fn set_name(&mut self, name: String);

    /// 设置数据类型
    fn set_data_type(&mut self, data_type: DataType);

    /// 设置是否可为空
    fn set_nullable(&mut self, nullable: bool);

    /// 设置默认值
    fn set_default_value(&mut self, default: Option<Value>);

    /// 设置注释
    fn set_comment(&mut self, comment: Option<String>);

    /// 获取属性类型名称（用于区分不同类型）
    fn property_type_name(&self) -> &'static str;
}
