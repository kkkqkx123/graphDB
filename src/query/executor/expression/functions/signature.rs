//! 类型签名系统
//!
//! 定义函数签名中使用的值类型枚举，用于类型检查和函数重载解析

use crate::core::Value;
use std::fmt;

/// 值类型枚举（用于函数签名）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Decimal128,
    String,
    Blob,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    Geography,
    Duration,
    DataSet,
    Empty,
}

impl ValueType {
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Null(_) => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Int8(_) => ValueType::Int8,
            Value::Int16(_) => ValueType::Int16,
            Value::Int32(_) => ValueType::Int32,
            Value::Int64(_) => ValueType::Int64,
            Value::UInt8(_) => ValueType::UInt8,
            Value::UInt16(_) => ValueType::UInt16,
            Value::UInt32(_) => ValueType::UInt32,
            Value::UInt64(_) => ValueType::UInt64,
            Value::Float(_) => ValueType::Float,
            Value::Decimal128(_) => ValueType::Decimal128,
            Value::String(_) => ValueType::String,
            Value::Blob(_) => ValueType::Blob,
            Value::Date(_) => ValueType::Date,
            Value::Time(_) => ValueType::Time,
            Value::DateTime(_) => ValueType::DateTime,
            Value::Vertex(_) => ValueType::Vertex,
            Value::Edge(_) => ValueType::Edge,
            Value::Path(_) => ValueType::Path,
            Value::List(_) => ValueType::List,
            Value::Map(_) => ValueType::Map,
            Value::Set(_) => ValueType::Set,
            Value::Geography(_) => ValueType::Geography,
            Value::Duration(_) => ValueType::Duration,
            Value::DataSet(_) => ValueType::DataSet,
            Value::Empty => ValueType::Empty,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ValueType::Int
                | ValueType::Int8
                | ValueType::Int16
                | ValueType::Int32
                | ValueType::Int64
                | ValueType::UInt8
                | ValueType::UInt16
                | ValueType::UInt32
                | ValueType::UInt64
                | ValueType::Float
                | ValueType::Decimal128
        )
    }

    pub fn is_string(&self) -> bool {
        matches!(self, ValueType::String)
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, ValueType::List | ValueType::Map | ValueType::Set)
    }

    pub fn compatible_with(&self, other: &ValueType) -> bool {
        self == &ValueType::Empty || other == &ValueType::Empty || self == other
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Null => write!(f, "NULL"),
            ValueType::Bool => write!(f, "BOOL"),
            ValueType::Int => write!(f, "INT"),
            ValueType::Int8 => write!(f, "INT8"),
            ValueType::Int16 => write!(f, "INT16"),
            ValueType::Int32 => write!(f, "INT32"),
            ValueType::Int64 => write!(f, "INT64"),
            ValueType::UInt8 => write!(f, "UINT8"),
            ValueType::UInt16 => write!(f, "UINT16"),
            ValueType::UInt32 => write!(f, "UINT32"),
            ValueType::UInt64 => write!(f, "UINT64"),
            ValueType::Float => write!(f, "FLOAT"),
            ValueType::Decimal128 => write!(f, "DECIMAL128"),
            ValueType::String => write!(f, "STRING"),
            ValueType::Blob => write!(f, "BLOB"),
            ValueType::Date => write!(f, "DATE"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::DateTime => write!(f, "DATETIME"),
            ValueType::Vertex => write!(f, "VERTEX"),
            ValueType::Edge => write!(f, "EDGE"),
            ValueType::Path => write!(f, "PATH"),
            ValueType::List => write!(f, "LIST"),
            ValueType::Map => write!(f, "MAP"),
            ValueType::Set => write!(f, "SET"),
            ValueType::Geography => write!(f, "GEOGRAPHY"),
            ValueType::Duration => write!(f, "DURATION"),
            ValueType::DataSet => write!(f, "DATASET"),
            ValueType::Empty => write!(f, "EMPTY"),
        }
    }
}
