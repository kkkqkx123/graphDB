//! Value Type Definition - Core Enum and Basic Methods

use crate::core::{
    types::DataType,
    value::{
        date_time::{DateTimeValue, DateValue, DurationValue, TimeValue},
        decimal128::Decimal128Value,
        geography::GeographyValue,
        json::{Json, JsonB, JsonError},
        list::List,
        null::NullType,
        vector::VectorValue,
    },
    vertex_edge_path::{Edge, Path, Vertex},
};
use crate::query::DataSet;
use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
    Decimal128(Decimal128Value),
    String(String),
    /// Fixed-length strings for optimized storage of short strings
    FixedString {
        len: usize,
        data: String,
    },
    /// Binary data
    Blob(Vec<u8>),
    Date(DateValue),
    Time(TimeValue),
    DateTime(DateTimeValue),
    Vertex(Box<Vertex>),
    Edge(Box<Edge>),
    Path(Box<Path>),
    List(Box<List>),
    Map(Box<HashMap<String, Value>>),
    Set(Box<HashSet<Value>>),
    Geography(GeographyValue),
    Duration(DurationValue),
    Vector(VectorValue),
    DataSet(Box<DataSet>),

    /// JSON type (text format)
    Json(Box<Json>),
    /// JSONB type (binary format)
    JsonB(Box<JsonB>),
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
            Value::Vector(_) => DataType::Vector,
            Value::DataSet(_) => DataType::DataSet,
            Value::Json(_) => DataType::Json,
            Value::JsonB(_) => DataType::JsonB,
        }
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    /// Check if the value is a numeric type
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

    /// Check if the value is BadNull
    pub fn is_bad_null(&self) -> bool {
        use super::null::NullType;
        matches!(
            self,
            Value::Null(NullType::BadData) | Value::Null(NullType::BadType)
        )
    }

    /// Check if the value is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// Get Boolean value
    pub fn bool_value(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get String value
    pub fn string_value(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::FixedString { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Get vector value as Vec<f32> from List of Float values or Vector type
    pub fn as_vector(&self) -> Option<Vec<f32>> {
        match self {
            Value::Vector(vec) => Some(vec.to_dense()),
            Value::List(list) => {
                let vector: Option<Vec<f32>> = list
                    .iter()
                    .map(|v| match v {
                        Value::Float(f) => Some(*f as f32),
                        Value::Int(i) => Some(*i as f32),
                        _ => None,
                    })
                    .collect();
                vector
            }
            Value::Blob(blob) => {
                if blob.len() % std::mem::size_of::<f32>() == 0 {
                    let len = blob.len() / std::mem::size_of::<f32>();
                    let mut vector = Vec::with_capacity(len);
                    let ptr = blob.as_ptr() as *const f32;
                    for i in 0..len {
                        unsafe {
                            vector.push(*ptr.add(i));
                        }
                    }
                    Some(vector)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get reference to vector data (more efficient than as_vector)
    pub fn as_vector_ref(&self) -> Option<&[f32]> {
        match self {
            Value::Vector(vec) => vec.as_dense(),
            _ => None,
        }
    }

    /// Create a new vector value
    pub fn vector(data: Vec<f32>) -> Self {
        Value::Vector(super::vector::VectorValue::dense(data))
    }

    /// Create a new sparse vector value
    pub fn sparse_vector(indices: Vec<u32>, values: Vec<f32>) -> Self {
        Value::Vector(super::vector::VectorValue::sparse(indices, values))
    }

    /// Create fixed-length string value
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

    /// Estimate the memory usage of the value
    pub fn estimated_size(&self) -> usize {
        match self {
            Value::Empty => std::mem::size_of::<Self>(),
            Value::Null(_) => std::mem::size_of::<Self>(),
            Value::Bool(_) => std::mem::size_of::<Self>(),
            Value::Int(_) => std::mem::size_of::<Self>(),
            Value::Int8(_) => std::mem::size_of::<Self>(),
            Value::Int16(_) => std::mem::size_of::<Self>(),
            Value::Int32(_) => std::mem::size_of::<Self>(),
            Value::Int64(_) => std::mem::size_of::<Self>(),
            Value::UInt8(_) => std::mem::size_of::<Self>(),
            Value::UInt16(_) => std::mem::size_of::<Self>(),
            Value::UInt32(_) => std::mem::size_of::<Self>(),
            Value::UInt64(_) => std::mem::size_of::<Self>(),
            Value::Float(_) => std::mem::size_of::<Self>(),
            Value::Decimal128(_) => std::mem::size_of::<Self>(),
            Value::String(s) => std::mem::size_of::<Self>() + s.capacity(),
            Value::FixedString { data, .. } => std::mem::size_of::<Self>() + data.capacity(),
            Value::Blob(b) => std::mem::size_of::<Self>() + b.capacity(),
            Value::Date(_) => std::mem::size_of::<Self>(),
            Value::Time(_) => std::mem::size_of::<Self>(),
            Value::DateTime(_) => std::mem::size_of::<Self>(),
            Value::Vertex(v) => std::mem::size_of::<Self>() + v.estimated_size(),
            Value::Edge(e) => std::mem::size_of::<Self>() + e.estimated_size(),
            Value::Path(p) => std::mem::size_of::<Self>() + p.estimated_size(),
            Value::List(l) => std::mem::size_of::<Self>() + l.estimated_size(),
            Value::Map(m) => {
                let mut size = std::mem::size_of::<Self>();
                size +=
                    m.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
                for (k, v) in m.as_ref() {
                    size += k.capacity();
                    size += v.estimated_size();
                }
                size
            }
            Value::Set(s) => {
                let mut size = std::mem::size_of::<Self>();
                size += s.capacity() * std::mem::size_of::<Value>();
                for v in s.as_ref() {
                    size += v.estimated_size();
                }
                size
            }
            Value::Geography(g) => std::mem::size_of::<Self>() + g.estimated_size(),
            Value::Duration(d) => std::mem::size_of::<Self>() + d.estimated_size(),
            Value::Vector(v) => std::mem::size_of::<Self>() + v.estimated_size(),
            Value::DataSet(ds) => std::mem::size_of::<Self>() + ds.estimated_size(),
            Value::Json(j) => std::mem::size_of::<Self>() + j.estimated_size(),
            Value::JsonB(j) => std::mem::size_of::<Self>() + j.estimated_size(),
        }
    }

    /// Create JSON value
    pub fn json(text: &str) -> Result<Self, JsonError> {
        Ok(Value::Json(Box::new(Json::from_str(text)?)))
    }

    /// Create JSONB value
    pub fn jsonb(text: &str) -> Result<Self, JsonError> {
        Ok(Value::JsonB(Box::new(JsonB::from_str(text)?)))
    }

    /// Create JSON value from serde_json::Value
    pub fn from_json_value(value: serde_json::Value) -> Self {
        Value::JsonB(Box::new(JsonB::from_value(value)))
    }
}

impl Value {
    /// Create a new List value (wraps in Box)
    pub fn list(list: List) -> Self {
        Value::List(Box::new(list))
    }

    /// Create a new Map value (wraps in Box)
    pub fn map(map: HashMap<String, Value>) -> Self {
        Value::Map(Box::new(map))
    }

    /// Create a new Set value (wraps in Box)
    pub fn set(set: HashSet<Value>) -> Self {
        Value::Set(Box::new(set))
    }

    /// Create a new Edge value (wraps in Box)
    pub fn edge(edge: Edge) -> Self {
        Value::Edge(Box::new(edge))
    }

    /// Create a new Path value (wraps in Box)
    pub fn path(path: Path) -> Self {
        Value::Path(Box::new(path))
    }

    /// Create a new DataSet value (wraps in Box)
    pub fn dataset(dataset: DataSet) -> Self {
        Value::DataSet(Box::new(dataset))
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
                for (i, item) in list.iter().enumerate() {
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
            Value::Geography(g) => write!(f, "Geography(lat: {}, lon: {})", g.latitude, g.longitude),
            Value::Duration(d) => write!(f, "Duration({:?})", d),
            Value::Vector(v) => write!(f, "{}", v),
            Value::DataSet(ds) => write!(f, "DataSet({} rows)", ds.row_count()),
            Value::Json(j) => write!(f, "Json({})", j.as_str()),
            Value::JsonB(j) => write!(f, "JsonB({})", j.to_json_string()),
        }
    }
}
