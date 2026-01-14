use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

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
    Any,
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

    /// 检查值是否为BadNull（BadData或BadType）
    pub fn is_bad_null(&self) -> bool {
        matches!(self, Value::Null(NullType::BadData) | Value::Null(NullType::BadType))
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

    /// 转换为布尔值
    pub fn cast_to_bool(&self) -> Result<Value, String> {
        match self {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::Int(i) => Ok(Value::Bool(*i != 0)),
            Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
            Value::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
                    "false" | "0" | "no" | "off" => Ok(Value::Bool(false)),
                    _ => Err(format!("无法将字符串 '{}' 转换为布尔值", s)),
                }
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            Value::Empty => Ok(Value::Null(NullType::Null)),
            _ => Err(format!("无法将 {:?} 转换为布尔值", self.get_type())),
        }
    }

    /// 转换为整数
    pub fn cast_to_int(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Int(*f as i64)),
            Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
            Value::String(s) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| format!("无法将字符串 '{}' 转换为整数", s)),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            Value::Empty => Ok(Value::Null(NullType::Null)),
            _ => Err(format!("无法将 {:?} 转换为整数", self.get_type())),
        }
    }

    /// 转换为浮点数
    pub fn cast_to_float(&self) -> Result<Value, String> {
        match self {
            Value::Float(f) => Ok(Value::Float(*f)),
            Value::Int(i) => Ok(Value::Float(*i as f64)),
            Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
            Value::String(s) => s
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| format!("无法将字符串 '{}' 转换为浮点数", s)),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            Value::Empty => Ok(Value::Null(NullType::Null)),
            _ => Err(format!("无法将 {:?} 转换为浮点数", self.get_type())),
        }
    }

    /// 转换为字符串
    pub fn cast_to_string(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.clone())),
            Value::Bool(b) => Ok(Value::String(if *b {
                "true".to_string()
            } else {
                "false".to_string()
            })),
            Value::Int(i) => Ok(Value::String(i.to_string())),
            Value::Float(f) => Ok(Value::String(f.to_string())),
            Value::Null(_) => Ok(Value::String("null".to_string())),
            Value::Empty => Ok(Value::String("empty".to_string())),
            Value::List(list) => {
                let items: Vec<String> = list
                    .iter()
                    .map(|v| {
                        v.cast_to_string().map(|s| {
                            if let Value::String(s) = s {
                                s
                            } else {
                                "?".to_string()
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::String(format!("[{}]", items.join(", "))))
            }
            Value::Map(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        v.cast_to_string().map(|s| {
                            if let Value::String(s) = s {
                                format!("{}: {}", k, s)
                            } else {
                                format!("{}: ?", k)
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::String(format!("{{{}}}", items.join(", "))))
            }
            _ => Err(format!("无法将 {:?} 转换为字符串", self.get_type())),
        }
    }

    /// 转换为列表
    pub fn cast_to_list(&self) -> Result<Value, String> {
        match self {
            Value::List(list) => Ok(Value::List(list.clone())),
            Value::Set(set) => Ok(Value::List(set.iter().cloned().collect())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            Value::Empty => Ok(Value::List(vec![])),
            Value::String(s) => {
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                Ok(Value::List(chars))
            }
            _ => Err(format!("无法将 {:?} 转换为列表", self.get_type())),
        }
    }

    /// 转换为映射
    pub fn cast_to_map(&self) -> Result<Value, String> {
        match self {
            Value::Map(map) => Ok(Value::Map(map.clone())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            Value::Empty => Ok(Value::Map(std::collections::HashMap::new())),
            Value::List(list) => {
                let mut map = std::collections::HashMap::new();
                for (i, v) in list.iter().enumerate() {
                    map.insert(i.to_string(), v.clone());
                }
                Ok(Value::Map(map))
            }
            _ => Err(format!("无法将 {:?} 转换为映射", self.get_type())),
        }
    }

    /// 取反操作
    pub fn negate(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(format!("无法对 {:?} 进行取反操作", self.get_type())),
        }
    }

    /// 绝对值操作
    pub fn abs(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err(format!("无法计算 {:?} 的绝对值", self.get_type())),
        }
    }

    /// 长度操作
    pub fn length(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(list) => Ok(Value::Int(list.len() as i64)),
            Value::Map(map) => Ok(Value::Int(map.len() as i64)),
            Value::Set(set) => Ok(Value::Int(set.len() as i64)),
            Value::Path(p) => Ok(Value::Int(p.length() as i64)),
            _ => Err(format!("无法计算 {:?} 的长度", self.get_type())),
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
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
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
            Value::Geography(g) => write!(f, "Geography({:?})", g),
            Value::Duration(d) => write!(f, "Duration({:?})", d),
            Value::DataSet(ds) => write!(f, "DataSet({:?})", ds),
        }
    }
}
