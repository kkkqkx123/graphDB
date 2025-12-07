use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::hash::Hash;

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
                16u8.hash(state);
                d.hash(state);
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
        }
    }
}
