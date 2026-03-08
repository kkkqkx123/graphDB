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
    Float,
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
            Value::Float(_) => ValueType::Float,
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
        matches!(self, ValueType::Int | ValueType::Float)
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
            ValueType::Float => write!(f, "FLOAT"),
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
