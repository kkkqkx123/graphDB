//! value type system
//!
//! This module defines all the value types used in the graph database query engine.
//!
//! ## Type hierarchy
//!
//! - **Base type**: NullType
//! - **Composite types**: List, Map, Set, DataSet
//! - **Map types**: Vertex, Edge, Path
//! - **Date-time type**: see date_time module
//! - **Geospatial type**: see geography module
//! - **Dataset type**: see dataset module
//!
//! ## Module organization
//!
//! - `types.rs` - 核心类型定义（NullType、Value、DataType）
//! - `date_time.rs` - 日期时间类型和操作
//! - `geography.rs` - 地理空间类型和操作
//! - `dataset.rs` - 数据集和列表类型及操作
//! - `operations.rs` - 值运算
//! - `conversion.rs` - 类型转换
//! - `comparison.rs` - 值比较
//!
//! ## Compatibility with Nebula-Graph
//!
//! This implementation references Nebula-Graph's type system design to ensure compatibility where necessary.

use crate::core::types::DataType;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Null Type Definition
///
/// Nebula-Graph-compatible null value type definition with the following variants:
/// - **Null**: standard null value
/// - **NaN**: Non-numeric results
/// - **BadData**: Bad data (e.g., wrong date format)
/// - **BadType**: type mismatch error
/// - **ErrOverflow**: Numeric overflow error
/// - **UnknownProp**: Unknown property
/// - **DivByZero**: divide by zero error
/// - **OutOfRange**: value out of range
///
/// ## vs. Nebula-Graph
///
/// This implementation is fully compatible with Nebula-Graph's NullType enumerations, ensuring cross-platform data consistency.
///
/// ```rust
/// use graphdb::core::value::NullType;
///
/// let null_val = NullType::Null;
/// let nan_val = NullType::NaN;
/// let div_zero = NullType::DivByZero;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode, Default)]
pub enum NullType {
    #[default]
    Null, // Standard null values
    NaN,         // Non-numeric results
    BadData,     // Bad data (parsing failure)
    BadType,     // Type mismatch
    ErrOverflow, // numeric overflow
    UnknownProp, // unknown property
    DivByZero,   // division error
    OutOfRange,  // Value out of range
}

impl NullType {
    pub fn is_bad(&self) -> bool {
        matches!(
            self,
            NullType::BadData | NullType::BadType | NullType::ErrOverflow | NullType::OutOfRange
        )
    }

    pub fn is_computational_error(&self) -> bool {
        matches!(
            self,
            NullType::NaN | NullType::DivByZero | NullType::ErrOverflow
        )
    }

    pub fn to_string(&self) -> &str {
        match self {
            NullType::Null => "NULL",
            NullType::NaN => "NaN",
            NullType::BadData => "BAD_DATA",
            NullType::BadType => "BAD_TYPE",
            NullType::ErrOverflow => "ERR_OVERFLOW",
            NullType::UnknownProp => "UNKNOWN_PROP",
            NullType::DivByZero => "DIV_BY_ZERO",
            NullType::OutOfRange => "OUT_OF_RANGE",
        }
    }
}

impl std::fmt::Display for NullType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Indicates values that can be stored in node/edge attributes
/// Following Nebula's Value type design pattern
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float(f64),
    Decimal128(super::decimal128::Decimal128Value),
    String(String),
    /// Fixed-length strings for optimized storage of short strings
    FixedString {
        len: usize,
        data: String,
    },
    /// binary data
    Blob(Vec<u8>),
    Date(super::date_time::DateValue),
    Time(super::date_time::TimeValue),
    DateTime(super::date_time::DateTimeValue),
    Vertex(Box<crate::core::vertex_edge_path::Vertex>),
    Edge(crate::core::vertex_edge_path::Edge),
    Path(crate::core::vertex_edge_path::Path),
    List(super::dataset::List),
    Map(std::collections::HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    Geography(super::geography::GeographyValue),
    Duration(super::date_time::DurationValue),
    DataSet(super::dataset::DataSet),
}

impl Value {
    /// Getting the type of value
    pub fn get_type(&self) -> DataType {
        match self {
            Value::Empty => DataType::Empty,
            Value::Null(_) => DataType::Null,
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Int8(_) => DataType::Int8,
            Value::Int16(_) => DataType::Int16,
            Value::Int32(_) => DataType::Int32,
            Value::Int64(_) => DataType::Int64,
            Value::UInt8(_) => DataType::UInt8,
            Value::UInt16(_) => DataType::UInt16,
            Value::UInt32(_) => DataType::UInt32,
            Value::UInt64(_) => DataType::UInt64,
            Value::Float(_) => DataType::Float,
            Value::Decimal128(_) => DataType::Decimal128,
            Value::String(_) => DataType::String,
            Value::FixedString { len, .. } => DataType::FixedString(*len),
            Value::Blob(_) => DataType::Blob,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Vertex(_) => DataType::Vertex,
            Value::Edge(_) => DataType::Edge,
            Value::Path(_) => DataType::Path,
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Set(_) => DataType::Set,
            Value::Geography(_) => DataType::Geography,
            Value::Duration(_) => DataType::Duration,
            Value::DataSet(_) => DataType::DataSet,
        }
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    /// Check if the value is a numeric type (Int, Float or Decimal128)
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Value::Int(_)
                | Value::Int8(_)
                | Value::Int16(_)
                | Value::Int32(_)
                | Value::Int64(_)
                | Value::UInt8(_)
                | Value::UInt16(_)
                | Value::UInt32(_)
                | Value::UInt64(_)
                | Value::Float(_)
                | Value::Decimal128(_)
        )
    }

    /// Check if the value is BadNull (BadData or BadType)
    pub fn is_bad_null(&self) -> bool {
        matches!(
            self,
            Value::Null(NullType::BadData) | Value::Null(NullType::BadType)
        )
    }

    /// Check if the value is null
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// Get Boolean
    pub fn bool_value(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Getting String Values
    pub fn string_value(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::FixedString { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Creating fixed-length string values
    pub fn fixed_string(len: usize, data: String) -> Self {
        let padded_data = if data.len() > len {
            data.chars().take(len).collect()
        } else {
            format!("{:<width$}", data, width = len)
        };
        Value::FixedString {
            len,
            data: padded_data,
        }
    }

    /// Get the length of a fixed-length string
    pub fn fixed_string_len(&self) -> Option<usize> {
        match self {
            Value::FixedString { len, .. } => Some(*len),
            _ => None,
        }
    }

    /// Compute the hash of the value
    pub fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// inverse operation
    pub fn negate(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Int8(i) => Ok(Value::Int8(-i)),
            Value::Int16(i) => Ok(Value::Int16(-i)),
            Value::Int32(i) => Ok(Value::Int32(-i)),
            Value::Int64(i) => Ok(Value::Int64(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            Value::Decimal128(d) => Ok(Value::Decimal128(d.neg())),
            _ => Err(format!("无法对 {:?} 进行取反操作", self.get_type())),
        }
    }

    /// absolute value operation
    pub fn abs(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(i.abs())),
            Value::Int8(i) => Ok(Value::Int8(i.abs())),
            Value::Int16(i) => Ok(Value::Int16(i.abs())),
            Value::Int32(i) => Ok(Value::Int32(i.abs())),
            Value::Int64(i) => Ok(Value::Int64(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            Value::Decimal128(d) => Ok(Value::Decimal128(d.abs())),
            _ => Err(format!("无法计算 {:?} 的绝对值", self.get_type())),
        }
    }

    /// length operation
    pub fn length(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::FixedString { data, .. } => Ok(Value::Int(data.len() as i64)),
            Value::Blob(b) => Ok(Value::Int(b.len() as i64)),
            Value::List(list) => Ok(Value::Int(list.values.len() as i64)),
            Value::Map(map) => Ok(Value::Int(map.len() as i64)),
            Value::Set(set) => Ok(Value::Int(set.len() as i64)),
            Value::Path(p) => Ok(Value::Int(p.length() as i64)),
            _ => Err(format!("无法计算 {:?} 的长度", self.get_type())),
        }
    }

    /// Memory Usage Size of Estimated Values
    pub fn estimated_size(&self) -> usize {
        let base_size = std::mem::size_of::<Value>();
        match self {
            Value::Empty => base_size,
            Value::Null(_) => base_size,
            Value::Bool(_) => base_size,
            Value::Int(_) => base_size,
            Value::Int8(_) => base_size,
            Value::Int16(_) => base_size,
            Value::Int32(_) => base_size,
            Value::Int64(_) => base_size,
            Value::UInt8(_) => base_size,
            Value::UInt16(_) => base_size,
            Value::UInt32(_) => base_size,
            Value::UInt64(_) => base_size,
            Value::Float(_) => base_size,
            Value::Decimal128(_) => {
                base_size + std::mem::size_of::<super::decimal128::Decimal128Value>()
            }
            Value::String(s) => base_size + std::mem::size_of::<String>() + s.capacity(),
            Value::FixedString { data, .. } => {
                base_size + std::mem::size_of::<String>() + data.capacity()
            }
            Value::Blob(b) => base_size + std::mem::size_of::<Vec<u8>>() + b.capacity(),
            Value::Date(d) => base_size + d.estimated_size(),
            Value::Time(t) => base_size + t.estimated_size(),
            Value::DateTime(dt) => base_size + dt.estimated_size(),
            Value::Vertex(v) => {
                base_size
                    + std::mem::size_of::<Box<crate::core::vertex_edge_path::Vertex>>()
                    + v.estimated_size()
            }
            Value::Edge(e) => {
                base_size
                    + std::mem::size_of::<crate::core::vertex_edge_path::Edge>()
                    + e.estimated_size()
            }
            Value::Path(p) => {
                base_size
                    + std::mem::size_of::<crate::core::vertex_edge_path::Path>()
                    + p.estimated_size()
            }
            Value::List(list) => {
                let mut size = base_size + std::mem::size_of::<super::dataset::List>();
                size += list.values.capacity() * std::mem::size_of::<Value>();
                for v in &list.values {
                    size += v.estimated_size();
                }
                size
            }
            Value::Map(map) => {
                let mut size =
                    base_size + std::mem::size_of::<std::collections::HashMap<String, Value>>();
                size +=
                    map.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
                for (k, v) in map {
                    size += k.capacity();
                    size += v.estimated_size();
                }
                size
            }
            Value::Set(set) => {
                let mut size = base_size + std::mem::size_of::<std::collections::HashSet<Value>>();
                size += set.capacity() * std::mem::size_of::<Value>();
                for v in set {
                    size += v.estimated_size();
                }
                size
            }
            Value::Geography(g) => base_size + g.estimated_size(),
            Value::Duration(d) => base_size + d.estimated_size(),
            Value::DataSet(ds) => base_size + ds.estimated_size(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Empty => write!(f, "EMPTY"),
            Value::Null(n) => write!(f, "NULL({:?})", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Int8(i) => write!(f, "{}", i),
            Value::Int16(i) => write!(f, "{}", i),
            Value::Int32(i) => write!(f, "{}", i),
            Value::Int64(i) => write!(f, "{}", i),
            Value::UInt8(i) => write!(f, "{}", i),
            Value::UInt16(i) => write!(f, "{}", i),
            Value::UInt32(i) => write!(f, "{}", i),
            Value::UInt64(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Decimal128(d) => write!(f, "{}", d),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::FixedString { len, data } => write!(f, "\"{}\"[fixed:{}]", data, len),
            Value::Blob(b) => write!(f, "Blob({} bytes)", b.len()),
            Value::Date(d) => write!(f, "{:04}-{:02}-{:02}", d.year, d.month, d.day),
            Value::Time(t) => write!(
                f,
                "{:02}:{:02}:{:02}.{:06}",
                t.hour, t.minute, t.sec, t.microsec
            ),
            Value::DateTime(dt) => write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.sec, dt.microsec
            ),
            Value::Vertex(v) => write!(f, "Vertex({:?})", v.id()),
            Value::Edge(e) => write!(f, "Edge({:?} -> {:?})", e.src(), e.dst()),
            Value::Path(p) => write!(f, "Path({:?})", p),
            Value::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(set) => {
                write!(f, "{{")?;
                for (i, item) in set.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Geography(g) => {
                write!(f, "Geography(lat: {}, lon: {})", g.latitude, g.longitude)
            }
            Value::Duration(d) => write!(f, "Duration({:?})", d),
            Value::DataSet(ds) => write!(f, "DataSet({:?})", ds),
        }
    }
}
