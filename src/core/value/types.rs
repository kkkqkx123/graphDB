//! 值类型系统
//!
//! 本模块定义了图数据库查询引擎中使用的所有值类型。
//!
//! ## 类型层次
//!
//! - **基础类型**: NullType
//! - **复合类型**: List, Map, Set, DataSet
//! - **图类型**: Vertex, Edge, Path
//! - **日期时间类型**: 见 date_time 模块
//! - **地理空间类型**: 见 geography 模块
//! - **数据集类型**: 见 dataset 模块
//!
//! ## 模块组织
//!
//! - `types.rs` - 核心类型定义（NullType、Value、DataType）
//! - `date_time.rs` - 日期时间类型和操作
//! - `geography.rs` - 地理空间类型和操作
//! - `dataset.rs` - 数据集和列表类型及操作
//! - `operations.rs` - 值运算
//! - `conversion.rs` - 类型转换
//! - `comparison.rs` - 值比较
//!
//! ## 与 Nebula-Graph 兼容性
//!
//! 本实现参考 Nebula-Graph 的类型系统设计，确保在必要时可以兼容。

use crate::core::types::DataType;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Null类型定义
///
/// 与 Nebula-Graph 兼容的空值类型定义，包含以下变体：
/// - **Null**: 标准 null 值
/// - **NaN**: 非数字结果
/// - **BadData**: 坏数据（如日期格式错误）
/// - **BadType**: 类型不匹配错误
/// - **ErrOverflow**: 数值溢出错误
/// - **UnknownProp**: 未知属性
/// - **DivByZero**: 除零错误
/// - **OutOfRange**: 值超出范围
///
/// ## 与 Nebula-Graph 对比
///
/// 此实现完全兼容 Nebula-Graph 的 NullType 枚举，确保跨平台数据一致性。
///
/// ```rust
/// use graphdb::core::value::NullType;
///
/// let null_val = NullType::Null;
/// let nan_val = NullType::NaN;
/// let div_zero = NullType::DivByZero;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub enum NullType {
    Null,          // 标准null值
    NaN,           // 非数字结果
    BadData,       // 坏数据（解析失败）
    BadType,       // 类型不匹配
    ErrOverflow,   // 数值溢出
    UnknownProp,   // 未知属性
    DivByZero,     // 除零错误
    OutOfRange,    // 值超出范围
}

impl NullType {
    pub fn is_bad(&self) -> bool {
        matches!(
            self,
            NullType::BadData | NullType::BadType | NullType::ErrOverflow | NullType::OutOfRange
        )
    }

    pub fn is_computational_error(&self) -> bool {
        matches!(self, NullType::NaN | NullType::DivByZero | NullType::ErrOverflow)
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

impl Default for NullType {
    fn default() -> Self {
        NullType::Null
    }
}

impl std::fmt::Display for NullType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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
    /// 获取值的类型
    pub fn get_type(&self) -> DataType {
        match self {
            Value::Empty => DataType::Empty,
            Value::Null(_) => DataType::Null,
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
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

    /// 检查值是否为null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    /// 检查值是否为数值类型（Int 或 Float）
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    /// 检查值是否为BadNull（BadData 或 BadType）
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
            Value::List(list) => Ok(Value::Int(list.values.len() as i64)),
            Value::Map(map) => Ok(Value::Int(map.len() as i64)),
            Value::Set(set) => Ok(Value::Int(set.len() as i64)),
            Value::Path(p) => Ok(Value::Int(p.length() as i64)),
            _ => Err(format!("无法计算 {:?} 的长度", self.get_type())),
        }
    }

    /// 估算值的内存使用大小
    pub fn estimated_size(&self) -> usize {
        let base_size = std::mem::size_of::<Value>();
        match self {
            Value::Empty => base_size,
            Value::Null(_) => base_size,
            Value::Bool(_) => base_size,
            Value::Int(_) => base_size,
            Value::Float(_) => base_size,
            Value::String(s) => base_size + std::mem::size_of::<String>() + s.capacity(),
            Value::Date(d) => base_size + d.estimated_size(),
            Value::Time(t) => base_size + t.estimated_size(),
            Value::DateTime(dt) => base_size + dt.estimated_size(),
            Value::Vertex(v) => base_size + std::mem::size_of::<Box<crate::core::vertex_edge_path::Vertex>>() + v.estimated_size(),
            Value::Edge(e) => base_size + std::mem::size_of::<crate::core::vertex_edge_path::Edge>() + e.estimated_size(),
            Value::Path(p) => base_size + std::mem::size_of::<crate::core::vertex_edge_path::Path>() + p.estimated_size(),
            Value::List(list) => {
                let mut size = base_size + std::mem::size_of::<super::dataset::List>();
                size += list.values.capacity() * std::mem::size_of::<Value>();
                for v in &list.values {
                    size += v.estimated_size();
                }
                size
            }
            Value::Map(map) => {
                let mut size = base_size + std::mem::size_of::<std::collections::HashMap<String, Value>>();
                size += map.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
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
            Value::Geography(g) => write!(f, "Geography(lat: {}, lon: {})", g.latitude, g.longitude),
            Value::Duration(d) => write!(f, "Duration({:?})", d),
            Value::DataSet(ds) => write!(f, "DataSet({:?})", ds),
        }
    }
}
