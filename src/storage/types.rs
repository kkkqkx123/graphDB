//! 字段类型定义
//!
//! 定义了存储层支持的各种字段类型，与 nebula-graph 的 PropertyType 对应
//! 基于 core 层的 DataType 统一类型系统扩展

pub use crate::core::DataType;

pub type FieldType = DataType;

use crate::core::Value;

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub fixed_length: Option<usize>,
    pub offset: usize,
    pub null_flag_pos: Option<usize>,
    pub geo_shape: Option<GeoShape>,
}

impl FieldDef {
    pub fn new(name: String, field_type: DataType) -> Self {
        Self {
            name,
            field_type,
            nullable: false,
            default_value: None,
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
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

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn null_flag_pos(mut self, pos: usize) -> Self {
        self.null_flag_pos = Some(pos);
        self
    }

    pub fn geo_shape(mut self, shape: GeoShape) -> Self {
        self.geo_shape = Some(shape);
        self
    }

    pub fn estimated_size(&self) -> usize {
        match self.field_type {
            DataType::Bool => 1,
            DataType::Int8 => 1,
            DataType::Int16 => 2,
            DataType::Int32 => 4,
            DataType::Int64 => 8,
            DataType::Float => 4,
            DataType::Double => 8,
            DataType::String => 8,
            DataType::FixedString(len) => len,
            DataType::VID => 8,
            DataType::Timestamp => 8,
            DataType::Date => 4,
            DataType::Time => 8,
            DataType::DateTime => 10,
            DataType::Vertex => 16,
            DataType::Edge => 32,
            DataType::Path => 24,
            DataType::List => 8,
            DataType::Set => 8,
            DataType::Map => 8,
            DataType::Blob => 8,
            DataType::Geography => 8,
            DataType::Duration => 16,
            _ => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeoShape {
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
    Any,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, data_type: DataType, nullable: bool) -> Self {
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

#[derive(Debug, Clone)]
pub enum EncodingFormat {
    Nebula,
    Simple,
}
