//! 值类型系统
//!
//! 本模块定义了图数据库查询引擎中使用的所有值类型。
//!
//! ## 类型层次
//!
//! - **基础类型**: NullType, DateValue, TimeValue, DateTimeValue
//! - **空间/时间类型**: GeographyValue, DurationValue (预留)
//! - **复合类型**: List, Map, Set, DataSet
//! - **图类型**: Vertex, Edge, Path
//!
//! ## 预留类型说明
//!
//! 以下类型当前版本可能未完全使用，但为支持高级查询预留：
//!
//! - [`GeographyValue`] - 地理空间坐标，用于位置相关查询
//! - [`DurationValue`] - 时间间隔，用于时间范围查询
//!
//! 未来版本计划支持：
//! - 空间索引和地理查询
//! - 时间序列分析
//! - 时空联合查询
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

/// 简单日期表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DateValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Default for DateValue {
    fn default() -> Self {
        DateValue {
            year: 1970,
            month: 1,
            day: 1,
        }
    }
}

/// 简单时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct TimeValue {
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

impl Default for TimeValue {
    fn default() -> Self {
        TimeValue {
            hour: 0,
            minute: 0,
            sec: 0,
            microsec: 0,
        }
    }
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

impl Default for DateTimeValue {
    fn default() -> Self {
        DateTimeValue {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            sec: 0,
            microsec: 0,
        }
    }
}

/// 地理信息表示 - 仅支持基础坐标点
///
/// ## 用途
/// 用于表示地理位置坐标，支持基础的空间数据查询。
/// 
/// ## 示例
/// ```rust
/// use graphdb::core::value::types::GeographyValue;
/// 
/// let location = GeographyValue {
///     latitude: 39.9042,   // 北京纬度
///     longitude: 116.4074, // 北京经度
/// };
/// ```
///
/// ## 支持的查询（预留）
/// - 距离计算：`st_distance(point1, point2)`
/// - 区域查询：基于坐标的范围筛选
/// - 附近搜索：查找特定距离内的点
///
/// ## 注意事项
/// 当前版本仅支持基础坐标点，完整的地理空间查询需要扩展：
/// - 多边形支持
/// - 空间索引
/// - 投影坐标系转换
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct GeographyValue {
    pub latitude: f64,
    pub longitude: f64,
}

// 手动实现Hash以处理f64字段
impl std::hash::Hash for GeographyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // 将f64转换为位表示进行哈希
        self.latitude.to_bits().hash(state);
        self.longitude.to_bits().hash(state);
    }
}

impl Default for GeographyValue {
    fn default() -> Self {
        GeographyValue {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

impl GeographyValue {
    /// 估算地理值的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// 简单持续时间表示
///
/// ## 用途
/// 用于表示时间间隔，支持时间相关的查询和计算。
///
/// ## 字段说明
/// - `seconds`: 秒数（可为负数）
/// - `microseconds`: 微秒数（-999999 到 999999）
/// - `months`: 月数（用于日历相关的持续时间）
///
/// ## 示例
/// ```rust
/// use graphdb::core::value::types::DurationValue;
/// 
/// // 表示 2小时30分45.5秒
/// let duration = DurationValue {
///     seconds: 9045,
///     microseconds: 500000,
///     months: 0,
/// };
///
/// // 表示 3个月
/// let month_duration = DurationValue {
///     seconds: 0,
///     microseconds: 0,
///     months: 3,
/// };
/// ```
///
/// ## 支持的查询（预留）
/// - 时间间隔算术：`date + duration`、`date - duration`
/// - 持续时间比较：`duration1 < duration2`
/// - 提取组件：`duration.seconds`、`duration.months`
///
/// ## 与 Nebula-Graph 兼容性
/// 参考 Nebula-Graph 的 DURATION 类型设计，支持：
/// - 秒和微秒精度
/// - 月份计算（考虑月份天数差异）
/// - 负时间间隔
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DurationValue {
    pub seconds: i64,
    pub microseconds: i32,
    pub months: i32,
}

impl DurationValue {
    /// 估算持续时间的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// 简单列表表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
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

    /// 估算数据集的内存使用大小
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        
        // 计算 col_names 的容量开销
        size += self.col_names.capacity() * std::mem::size_of::<String>();
        for col_name in &self.col_names {
            size += col_name.capacity();
        }
        
        // 计算 rows 的容量开销
        size += self.rows.capacity() * std::mem::size_of::<Vec<Value>>();
        for row in &self.rows {
            size += row.capacity() * std::mem::size_of::<Value>();
            for value in row {
                size += value.estimated_size();
            }
        }
        
        size
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
            Value::List(list) => Ok(Value::Int(list.len() as i64)),
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
            Value::Date(_) => base_size,
            Value::Time(_) => base_size,
            Value::DateTime(_) => base_size,
            Value::Vertex(v) => base_size + std::mem::size_of::<Box<crate::core::vertex_edge_path::Vertex>>() + v.estimated_size(),
            Value::Edge(e) => base_size + std::mem::size_of::<crate::core::vertex_edge_path::Edge>() + e.estimated_size(),
            Value::Path(p) => base_size + std::mem::size_of::<crate::core::vertex_edge_path::Path>() + p.estimated_size(),
            Value::List(vec) => {
                let mut size = base_size + std::mem::size_of::<Vec<Value>>();
                size += vec.capacity() * std::mem::size_of::<Value>();
                for v in vec {
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
            Value::DataSet(ds) => write!(f, "DataSet({:?})", ds),
        }
    }
}
