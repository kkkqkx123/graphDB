//! 字段类型定义
//!
//! 定义了存储层支持的各种字段类型

use crate::core::Value;

/// 字段类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Bool,
    Int,
    Float,
    Double,
    String,
    FixedString(usize), // 固定长度字符串
    Timestamp,
    Date,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Set,
    Map,
    Blob,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub fixed_length: Option<usize>, // 用于FIXED_STRING类型
}

impl FieldDef {
    pub fn new(name: String, field_type: FieldType) -> Self {
        Self {
            name,
            field_type,
            nullable: false,
            default_value: None,
            fixed_length: None,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn fixed_length(mut self, length: usize) -> Self {
        self.fixed_length = Some(length);
        self
    }
}

/// 列定义
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: FieldType,
    pub nullable: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, data_type: FieldType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable,
        }
    }
}

impl From<FieldDef> for ColumnDef {
    fn from(field: FieldDef) -> Self {
        Self {
            name: field.name,
            data_type: field.field_type,
            nullable: field.nullable,
        }
    }
}

/// 编码格式
#[derive(Debug, Clone)]
pub enum EncodingFormat {
    /// Nebula的默认编码格式
    Nebula,
    /// 简化的编码格式（用于测试）
    Simple,
}
