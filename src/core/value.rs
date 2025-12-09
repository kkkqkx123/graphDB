use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueTypeDef {
    Empty,
    Null,
    Bool,
    Int,
    Float,
    String,
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
    IntRange,
    FloatRange,
    StringRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NullType {
    Null,
    NaN,
    BadData,
    BadType,
    Overflow,
    UnknownProp,
    DivByZero,
    OutOfRange,
}

impl Default for NullType {
    fn default() -> Self {
        NullType::Null
    }
}

/// Simple Date representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct DateValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Simple Time representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct TimeValue {
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

/// Simple DateTime representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct DateTimeValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

/// Simple Geography representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeographyValue {
    pub point: Option<(f64, f64)>,             // latitude, longitude
    pub linestring: Option<Vec<(f64, f64)>>,   // list of coordinates
    pub polygon: Option<Vec<Vec<(f64, f64)>>>, // list of rings (outer and holes)
}

// Manual implementation of Hash for GeographyValue
impl std::hash::Hash for GeographyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Convert f64 values to bits for hashing
        if let Some((lat, lon)) = self.point {
            (lat.to_bits()).hash(state);
            (lon.to_bits()).hash(state);
        } else {
            (0u64).hash(state); // For None case
        }

        if let Some(ref line) = self.linestring {
            for (lat, lon) in line {
                (lat.to_bits()).hash(state);
                (lon.to_bits()).hash(state);
            }
        } else {
            (0u64).hash(state); // For None case
        }

        if let Some(ref poly) = self.polygon {
            for ring in poly {
                for (lat, lon) in ring {
                    (lat.to_bits()).hash(state);
                    (lon.to_bits()).hash(state);
                }
            }
        } else {
            (0u64).hash(state); // For None case
        }
    }
}

/// Simple Duration representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct DurationValue {
    pub seconds: i64,
    pub microseconds: i32,
    pub months: i32,
}

/// Simple DataSet representation similar to Nebula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct DataSet {
    pub col_names: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

impl DataSet {
    pub fn new() -> Self {
        Self {
            col_names: Vec::new(),
            rows: Vec::new(),
        }
    }
}

/// Represents a value that can be stored in node/edge properties
/// This follows the design pattern of Nebula's Value type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(DateValue),
    Time(TimeValue),
    DateTime(DateTimeValue),
    Vertex(Box<Vertex>),
    Edge(Edge),
    Path(Path),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    Geography(GeographyValue),
    Duration(DurationValue),
    DataSet(DataSet),
}

// Implement PartialEq manually to handle f64 comparison properly
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Null(a), Value::Null(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a == b) || (a.is_nan() && b.is_nan()), // Handle NaN properly
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::Vertex(a), Value::Vertex(b)) => a == b,
            (Value::Edge(a), Value::Edge(b)) => a == b,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Geography(a), Value::Geography(b)) => a == b,
            (Value::Duration(a), Value::Duration(b)) => a == b,
            _ => false,
        }
    }
}

// Implement Eq manually since f64 doesn't implement Eq
impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Use the hash values to create a consistent ordering
        self.hash_value().cmp(&other.hash_value())
    }
}

// 手动实现Hash以处理f64哈希
impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Empty => 0u8.hash(state),
            Value::Null(n) => {
                1u8.hash(state);
                n.hash(state);
            }
            Value::Bool(b) => {
                2u8.hash(state);
                b.hash(state);
            }
            Value::Int(i) => {
                3u8.hash(state);
                i.hash(state);
            }
            Value::Float(f) => {
                4u8.hash(state);
                // Create a hash from the bit representation of the float
                if f.is_nan() {
                    // All NaN values should hash to the same value
                    (0x7ff80000u32 as u64).hash(state);
                } else if *f == 0.0 {
                    // Ensure +0.0 and -0.0 hash to the same value
                    0.0_f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::String(s) => {
                5u8.hash(state);
                s.hash(state);
            }
            Value::Date(d) => {
                6u8.hash(state);
                d.hash(state);
            }
            Value::Time(t) => {
                7u8.hash(state);
                t.hash(state);
            }
            Value::DateTime(dt) => {
                8u8.hash(state);
                dt.hash(state);
            }
            Value::Vertex(v) => {
                9u8.hash(state);
                v.hash(state);
            }
            Value::Edge(e) => {
                10u8.hash(state);
                e.hash(state);
            }
            Value::Path(p) => {
                11u8.hash(state);
                p.hash(state);
            }
            Value::List(l) => {
                12u8.hash(state);
                l.hash(state);
            }
            Value::Map(m) => {
                13u8.hash(state);
                // Hash a map by hashing key-value pairs in sorted order
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by_key(|&(k, _)| k);
                pairs.hash(state);
            }
            Value::Set(s) => {
                14u8.hash(state);
                // For set, we'll hash all values in sorted order to ensure consistency
                let mut values: Vec<_> = s.iter().collect();
                values.sort();
                values.hash(state);
            }
            Value::Geography(g) => {
                15u8.hash(state);
                g.hash(state);
            }
            Value::Duration(d) => {
                17u8.hash(state);
                d.hash(state);
            }
            Value::DataSet(ds) => {
                18u8.hash(state);
                ds.hash(state);
            }
        }
    }
}

impl Value {
    fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_type(&self) -> ValueTypeDef {
        match self {
            Value::Empty => ValueTypeDef::Empty,
            Value::Null(_) => ValueTypeDef::Null,
            Value::Bool(_) => ValueTypeDef::Bool,
            Value::Int(_) => ValueTypeDef::Int,
            Value::Float(_) => ValueTypeDef::Float,
            Value::String(_) => ValueTypeDef::String,
            Value::Date(_) => ValueTypeDef::Date,
            Value::Time(_) => ValueTypeDef::Time,
            Value::DateTime(_) => ValueTypeDef::DateTime,
            Value::Vertex(_) => ValueTypeDef::Vertex,
            Value::Edge(_) => ValueTypeDef::Edge,
            Value::Path(_) => ValueTypeDef::Path,
            Value::List(_) => ValueTypeDef::List,
            Value::Map(_) => ValueTypeDef::Map,
            Value::Set(_) => ValueTypeDef::Set,
            Value::Geography(_) => ValueTypeDef::Geography,
            Value::Duration(_) => ValueTypeDef::Duration,
            Value::DataSet(_) => ValueTypeDef::DataSet,
        }
    }

    // Helper method to check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    // Helper method to check if value is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    // Helper method to get boolean value
    pub fn bool_value(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    // Arithmetic operations
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            _ => Err("Cannot add values of these types".to_string()),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - *b as f64)),
            _ => Err("Cannot subtract these values".to_string()),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * *b as f64)),
            _ => Err("Cannot multiply these values".to_string()),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(*a as f64 / *b as f64))
            }
            (Float(a), Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(a / b))
            }
            (Int(a), Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(*a as f64 / b))
            }
            (Float(a), Int(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(a / *b as f64))
            }
            _ => Err("Cannot divide these values".to_string()),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Int(a % b))
            }
            (Float(a), Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(a % b))
            }
            (Int(a), Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(*a as f64 % b))
            }
            (Float(a), Int(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Float(a % *b as f64))
            }
            _ => Err("Cannot take modulo of these values".to_string()),
        }
    }

    // Unary operations
    pub fn negate(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(i) => Ok(Int(-i)),
            Float(f) => Ok(Float(-f)),
            _ => Err("Cannot negate this value".to_string()),
        }
    }

    // Comparison operations
    pub fn equals(&self, other: &Value) -> bool {
        self == other
    }

    pub fn less_than(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a < b,
            (Float(a), Float(b)) => a < b,
            (String(a), String(b)) => a < b,
            _ => false,
        }
    }

    pub fn less_than_equal(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a <= b,
            (Float(a), Float(b)) => a <= b,
            (String(a), String(b)) => a <= b,
            _ => false,
        }
    }

    pub fn greater_than(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a > b,
            (Float(a), Float(b)) => a > b,
            (String(a), String(b)) => a > b,
            _ => false,
        }
    }

    pub fn greater_than_equal(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a >= b,
            (Float(a), Float(b)) => a >= b,
            (String(a), String(b)) => a >= b,
            _ => false,
        }
    }

    // Container operations
    pub fn contains(&self, item: &Value) -> bool {
        match self {
            Value::List(items) => items.contains(item),
            Value::Set(items) => items.contains(item),
            Value::Map(m) => m.contains_key(&item.to_string()),
            _ => false,
        }
    }

    // Type casting methods
    pub fn cast_to_bool(&self) -> Result<Value, String> {
        match self {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::Int(i) => Ok(Value::Bool(*i != 0)),
            Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
            Value::String(s) => {
                if s.to_lowercase() == "true" {
                    Ok(Value::Bool(true))
                } else if s.to_lowercase() == "false" {
                    Ok(Value::Bool(false))
                } else {
                    Err("Cannot cast string to bool".to_string())
                }
            },
            Value::Empty => Ok(Value::Bool(false)),
            Value::Null(_) => Ok(Value::Bool(false)),
            _ => Err("Cannot cast to bool".to_string()),
        }
    }

    pub fn cast_to_int(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Int(*f as i64)),
            Value::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(Value::Int(i)),
                    Err(_) => Err("Cannot cast string to int".to_string()),
                }
            },
            Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
            Value::Empty => Ok(Value::Int(0)),
            Value::Null(_) => Ok(Value::Int(0)),
            _ => Err("Cannot cast to int".to_string()),
        }
    }

    pub fn cast_to_float(&self) -> Result<Value, String> {
        match self {
            Value::Float(f) => Ok(Value::Float(*f)),
            Value::Int(i) => Ok(Value::Float(*i as f64)),
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(f) => Ok(Value::Float(f)),
                    Err(_) => Err("Cannot cast string to float".to_string()),
                }
            },
            Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
            Value::Empty => Ok(Value::Float(0.0)),
            Value::Null(_) => Ok(Value::Float(0.0)),
            _ => Err("Cannot cast to float".to_string()),
        }
    }

    pub fn cast_to_string(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.clone())),
            Value::Int(i) => Ok(Value::String(i.to_string())),
            Value::Float(f) => Ok(Value::String(f.to_string())),
            Value::Bool(b) => Ok(Value::String(if *b { "true".to_string() } else { "false".to_string() })),
            Value::Empty => Ok(Value::String("".to_string())),
            Value::Null(_) => Ok(Value::String("null".to_string())),
            _ => Err("Cannot cast to string".to_string()),
        }
    }

    pub fn cast_to_date(&self) -> Result<Value, String> {
        match self {
            Value::Date(d) => Ok(Value::Date(d.clone())),
            Value::String(s) => {
                // Parse date string in format "YYYY-MM-DD"
                let parts: Vec<&str> = s.split('-').collect();
                if parts.len() == 3 {
                    match (parts[0].parse::<i32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                        (Ok(year), Ok(month), Ok(day)) => {
                            Ok(Value::Date(DateValue { year, month, day }))
                        },
                        _ => Err("Invalid date format".to_string()),
                    }
                } else {
                    Err("Date string must be in YYYY-MM-DD format".to_string())
                }
            },
            Value::Empty => Ok(Value::Date(DateValue { year: 1, month: 1, day: 1 })),
            _ => Err("Cannot cast to date".to_string()),
        }
    }

    pub fn cast_to_time(&self) -> Result<Value, String> {
        match self {
            Value::Time(t) => Ok(Value::Time(t.clone())),
            Value::String(s) => {
                // Parse time string in format "HH:MM:SS"
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() == 3 {
                    match (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                        (Ok(hour), Ok(minute), Ok(sec)) => {
                            Ok(Value::Time(TimeValue { hour, minute, sec, microsec: 0 }))
                        },
                        _ => Err("Invalid time format".to_string()),
                    }
                } else {
                    Err("Time string must be in HH:MM:SS format".to_string())
                }
            },
            Value::Empty => Ok(Value::Time(TimeValue { hour: 0, minute: 0, sec: 0, microsec: 0 })),
            _ => Err("Cannot cast to time".to_string()),
        }
    }

    pub fn cast_to_datetime(&self) -> Result<Value, String> {
        match self {
            Value::DateTime(dt) => Ok(Value::DateTime(dt.clone())),
            Value::String(s) => {
                // Parse datetime string in format "YYYY-MM-DD HH:MM:SS"
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() == 2 {
                    let date_part = parts[0];
                    let time_part = parts[1];

                    let date_parts: Vec<&str> = date_part.split('-').collect();
                    let time_parts: Vec<&str> = time_part.split(':').collect();

                    if date_parts.len() == 3 && time_parts.len() == 3 {
                        match (
                            date_parts[0].parse::<i32>(),
                            date_parts[1].parse::<u32>(),
                            date_parts[2].parse::<u32>(),
                            time_parts[0].parse::<u32>(),
                            time_parts[1].parse::<u32>(),
                            time_parts[2].parse::<u32>()
                        ) {
                            (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute), Ok(sec)) => {
                                Ok(Value::DateTime(DateTimeValue {
                                    year, month, day, hour, minute, sec, microsec: 0
                                }))
                            },
                            _ => Err("Invalid datetime format".to_string()),
                        }
                    } else {
                        Err("Invalid datetime format".to_string())
                    }
                } else {
                    Err("Datetime string must be in YYYY-MM-DD HH:MM:SS format".to_string())
                }
            },
            Value::Empty => Ok(Value::DateTime(DateTimeValue {
                year: 1, month: 1, day: 1, hour: 0, minute: 0, sec: 0, microsec: 0
            })),
            _ => Err("Cannot cast to datetime".to_string()),
        }
    }

    pub fn cast_to_vertex(&self) -> Result<Value, String> {
        match self {
            Value::Vertex(v) => Ok(Value::Vertex(v.clone())),
            _ => Err("Cannot cast to vertex".to_string()),
        }
    }

    pub fn cast_to_edge(&self) -> Result<Value, String> {
        match self {
            Value::Edge(e) => Ok(Value::Edge(e.clone())),
            _ => Err("Cannot cast to edge".to_string()),
        }
    }

    pub fn cast_to_path(&self) -> Result<Value, String> {
        match self {
            Value::Path(p) => Ok(Value::Path(p.clone())),
            _ => Err("Cannot cast to path".to_string()),
        }
    }

    pub fn cast_to_list(&self) -> Result<Value, String> {
        match self {
            Value::List(l) => Ok(Value::List(l.clone())),
            Value::Empty => Ok(Value::List(vec![])),
            _ => Err("Cannot cast to list".to_string()),
        }
    }

    pub fn cast_to_map(&self) -> Result<Value, String> {
        match self {
            Value::Map(m) => Ok(Value::Map(m.clone())),
            Value::Empty => Ok(Value::Map(HashMap::new())),
            _ => Err("Cannot cast to map".to_string()),
        }
    }

    pub fn cast_to_set(&self) -> Result<Value, String> {
        match self {
            Value::Set(s) => Ok(Value::Set(s.clone())),
            Value::Empty => Ok(Value::Set(std::collections::HashSet::new())),
            _ => Err("Cannot cast to set".to_string()),
        }
    }

    pub fn cast_to_duration(&self) -> Result<Value, String> {
        match self {
            Value::Duration(d) => Ok(Value::Duration(d.clone())),
            _ => Err("Cannot cast to duration".to_string()),
        }
    }

    pub fn cast_to_geography(&self) -> Result<Value, String> {
        match self {
            Value::Geography(g) => Ok(Value::Geography(g.clone())),
            _ => Err("Cannot cast to geography".to_string()),
        }
    }

    pub fn cast_to_dataset(&self) -> Result<Value, String> {
        match self {
            Value::DataSet(ds) => Ok(Value::DataSet(ds.clone())),
            _ => Err("Cannot cast to dataset".to_string()),
        }
    }

    // Methods for expression evaluation
    pub fn abs(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(i64::abs(*i))),
            Value::Float(f) => Ok(Value::Float(f64::abs(*f))),
            _ => Err("Cannot get absolute value".to_string()),
        }
    }

    pub fn ceil(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Float(f64::ceil(*f))),
            _ => Err("Cannot get ceiling value".to_string()),
        }
    }

    pub fn floor(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Float(f64::floor(*f))),
            _ => Err("Cannot get floor value".to_string()),
        }
    }

    pub fn round(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Float(f64::round(*f))),
            _ => Err("Cannot round value".to_string()),
        }
    }

    pub fn lower(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err("Cannot convert to lowercase".to_string()),
        }
    }

    pub fn upper(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err("Cannot convert to uppercase".to_string()),
        }
    }

    pub fn trim(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err("Cannot trim value".to_string()),
        }
    }

    pub fn length(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(l) => Ok(Value::Int(l.len() as i64)),
            Value::Map(m) => Ok(Value::Int(m.len() as i64)),
            Value::Set(s) => Ok(Value::Int(s.len() as i64)),
            _ => Err("Cannot get length".to_string()),
        }
    }
}

// Implement Display for Value enum
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Empty => write!(f, "Empty"),
            Value::Null(null_type) => write!(
                f,
                "Null({})",
                match null_type {
                    NullType::Null => "Null",
                    NullType::NaN => "NaN",
                    NullType::BadData => "BadData",
                    NullType::BadType => "BadType",
                    NullType::Overflow => "Overflow",
                    NullType::UnknownProp => "UnknownProp",
                    NullType::DivByZero => "DivByZero",
                    NullType::OutOfRange => "OutOfRange",
                }
            ),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Date(d) => write!(f, "Date({}-{}-{})", d.year, d.month, d.day),
            Value::Time(t) => write!(f, "Time({}:{})", t.hour, t.minute),
            Value::DateTime(dt) => write!(
                f,
                "DateTime({}-{}-{} {}:{})",
                dt.year, dt.month, dt.day, dt.hour, dt.minute
            ),
            Value::Vertex(v) => write!(f, "Vertex({:?})", v.vid),
            Value::Edge(e) => write!(f, "Edge({:?} -> {:?})", e.src, e.dst),
            Value::Path(p) => write!(f, "Path({:?})", p.src),
            Value::List(l) => {
                let items: Vec<String> = l.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Map(m) => {
                let pairs: Vec<String> = m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            Value::Set(s) => {
                let items: Vec<String> = s.iter().map(|v| format!("{}", v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Geography(g) => write!(f, "Geography({:?})", g.point),
            Value::Duration(d) => write!(f, "Duration({})", d.seconds),
            Value::DataSet(ds) => write!(
                f,
                "DataSet({} columns, {} rows)",
                ds.col_names.len(),
                ds.rows.len()
            ),
        }
    }
}
