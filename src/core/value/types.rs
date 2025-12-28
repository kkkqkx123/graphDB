use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};

/// Value类型定义枚举
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

/// Null类型定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
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

/// 简单日期表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DateValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// 简单时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct TimeValue {
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

/// 简单日期时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DateTimeValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

/// 简单地理信息表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct GeographyValue {
    pub point: Option<(f64, f64)>,             // latitude, longitude
    pub linestring: Option<Vec<(f64, f64)>>,   // list of coordinates
    pub polygon: Option<Vec<Vec<(f64, f64)>>>, // list of rings (outer and holes)
}

// 手动实现Hash for GeographyValue
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

/// 简单持续时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DurationValue {
    pub seconds: i64,
    pub microseconds: i32,
    pub months: i32,
}

/// 简单列表表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct List {
    pub values: Vec<Value>,
}

// 手动实现Hash以处理Value的Hash
impl std::hash::Hash for List {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for value in &self.values {
            value.hash(state);
        }
    }
}

/// 简单数据集表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
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

impl List {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
}

/// 表示可以存储在节点/边属性中的值
/// 遵循Nebula的Value类型设计模式
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
    Vertex(Box<crate::core::vertex_edge_path::Vertex>),
    Edge(crate::core::vertex_edge_path::Edge),
    Path(crate::core::vertex_edge_path::Path),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    Geography(GeographyValue),
    Duration(DurationValue),
    DataSet(DataSet),
}

impl Value {
    /// 获取值的类型
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

    /// 检查值是否为null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    /// 检查值是否为空
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// 获取布尔值
    pub fn bool_value(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 计算值的哈希值
    pub fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}