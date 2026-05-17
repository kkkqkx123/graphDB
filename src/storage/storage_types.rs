//! Types of data operations at the storage level

use crate::core::types::PropertyDef as CorePropertyDef;
use crate::core::DataType;
use crate::core::Value;

/// Storage-level property definition.
/// Combines features from both vertex and edge property definitions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoragePropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
}

impl StoragePropertyDef {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
            default_value: None,
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
}

impl From<CorePropertyDef> for StoragePropertyDef {
    fn from(prop: CorePropertyDef) -> Self {
        Self {
            name: prop.name,
            data_type: prop.data_type,
            nullable: prop.nullable,
            default_value: prop.default,
        }
    }
}

impl From<&CorePropertyDef> for StoragePropertyDef {
    fn from(prop: &CorePropertyDef) -> Self {
        Self {
            name: prop.name.clone(),
            data_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
        }
    }
}

/// Field Definitions
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
            DataType::SmallInt => 2,
            DataType::Int => 4,
            DataType::BigInt => 8,
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
            DataType::Uuid => 16,
            DataType::Interval => 16,
            _ => 8,
        }
    }
}

impl From<CorePropertyDef> for FieldDef {
    fn from(prop: CorePropertyDef) -> Self {
        Self {
            name: prop.name,
            field_type: prop.data_type,
            nullable: prop.nullable,
            default_value: prop.default,
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
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

/// column definition
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
