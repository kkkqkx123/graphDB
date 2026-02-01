//! 字段类型定义
//!
//! 定义了存储层支持的各种字段类型，与 nebula-graph 的 PropertyType 对应

use crate::core::Value;

/// 字段类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// 布尔类型，1字节
    Bool,
    /// 8位整数
    Int8,
    /// 16位整数
    Int16,
    /// 32位整数
    Int32,
    /// 64位整数
    Int64,
    /// 单精度浮点数，4字节
    Float,
    /// 双精度浮点数，8字节
    Double,
    /// 变长字符串：8字节（4字节偏移 + 4字节长度）
    String,
    /// 固定长度字符串
    FixedString(usize),
    /// 时间戳，8字节毫秒级 Unix 时间戳
    Timestamp,
    /// 日期类型，4字节（2字节 year + 1字节 month + 1字节 day）
    Date,
    /// 时间类型，8字节（1字节 hour + 1字节 minute + 1字节 sec + 4字节 microsec）
    Time,
    /// 日期时间类型，10字节（2字节 year + 1字节 month + 1字节 day + 1字节 hour + 1字节 minute + 1字节 sec + 4字节 microsec）
    DateTime,
    /// 顶点ID，8字节字符串
    VID,
    /// 二进制数据，8字节（4字节偏移 + 4字节长度）
    Blob,
    /// 顶点类型
    Vertex,
    /// 边类型
    Edge,
    /// 路径类型
    Path,
    /// 列表类型
    List,
    /// 集合类型
    Set,
    /// 映射类型
    Map,
    /// 地理空间类型，8字节（4字节偏移 + 4字节长度），存储 WKB 格式
    Geography,
    /// 持续时间类型，16字节（8字节 seconds + 4字节 microseconds + 4字节 months）
    Duration,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub fixed_length: Option<usize>,
    pub offset: usize,
    pub null_flag_pos: Option<usize>,
    pub geo_shape: Option<GeoShape>,
}

impl FieldDef {
    pub fn new(name: String, field_type: FieldType) -> Self {
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
}

/// 地理空间形状类型
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
    /// Nebula的默认编码格式（V2版本）
    Nebula,
    /// 简化的编码格式（用于测试）
    Simple,
}
