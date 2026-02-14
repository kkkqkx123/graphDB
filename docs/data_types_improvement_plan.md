# GraphDB 数据结构改进方案

## 概述

本文档基于 Nebula-Graph 与 GraphDB 的对比分析，针对 GraphDB 在功能实现和数据结构设计上的不足，制定详细的改进方案。

## 一、运算支持补充方案

### 1.1 当前状态分析

**已实现的运算：**
- ✅ 算术运算：add、sub、mul、div、rem、pow
- ✅ 一元运算：neg、abs
- ✅ 逻辑运算：and、or、not
- ✅ 比较运算：完整的 Ord、PartialOrd 实现

**问题分析：**
虽然基础运算已经实现，但存在以下问题：
1. 运算方法未被项目使用（operations.rs 文件注释说明）
2. 缺少位运算（&、|、^）
3. 缺少对日期时间类型的运算支持
4. 缺少对地理类型的运算支持
5. 运算错误处理不够完善

### 1.2 改进方案

#### 1.2.1 添加位运算支持

**目标文件：** `src/core/value/operations.rs`

**实现内容：**

```rust
impl Value {
    /// 按位与运算
    pub fn bit_and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a & b)),
            _ => Err("只能对整数类型进行按位与运算".to_string()),
        }
    }

    /// 按位或运算
    pub fn bit_or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a | b)),
            _ => Err("只能对整数类型进行按位或运算".to_string()),
        }
    }

    /// 按位异或运算
    pub fn bit_xor(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a ^ b)),
            _ => Err("只能对整数类型进行按位异或运算".to_string()),
        }
    }

    /// 按位取反运算
    pub fn bit_not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(!a)),
            _ => Err("只能对整数类型进行按位取反运算".to_string()),
        }
    }

    /// 左移运算
    pub fn shift_left(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 || *b > 63 {
                    Err("移位位数必须在 0-63 范围内".to_string())
                } else {
                    Ok(Int(a << *b))
                }
            }
            _ => Err("只能对整数类型进行左移运算".to_string()),
        }
    }

    /// 右移运算
    pub fn shift_right(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 || *b > 63 {
                    Err("移位位数必须在 0-63 范围内".to_string())
                } else {
                    Ok(Int(a >> *b))
                }
            }
            _ => Err("只能对整数类型进行右移运算".to_string()),
        }
    }
}
```

**实现步骤：**
1. 在 `src/core/value/operations.rs` 中添加上述方法
2. 添加单元测试验证功能
3. 更新文档说明新增的运算

**优先级：** 中

**预计工作量：** 2-3 小时

#### 1.2.2 扩展日期时间运算支持

**目标文件：** `src/core/value/operations.rs`

**实现内容：**

```rust
impl Value {
    /// 日期加持续时间
    pub fn date_add_duration(&self, duration: &DurationValue) -> Result<Value, String> {
        match self {
            Value::Date(date) => {
                let mut result = date.clone();
                result.add_duration(duration);
                Ok(Value::Date(result))
            }
            Value::DateTime(datetime) => {
                let mut result = datetime.clone();
                result.add_duration(duration);
                Ok(Value::DateTime(result))
            }
            Value::Time(time) => {
                let mut result = time.clone();
                result.add_duration(duration);
                Ok(Value::Time(result))
            }
            _ => Err("只能对日期时间类型进行持续时间运算".to_string()),
        }
    }

    /// 日期减持续时间
    pub fn date_sub_duration(&self, duration: &DurationValue) -> Result<Value, String> {
        match self {
            Value::Date(date) => {
                let mut result = date.clone();
                result.sub_duration(duration);
                Ok(Value::Date(result))
            }
            Value::DateTime(datetime) => {
                let mut result = datetime.clone();
                result.sub_duration(duration);
                Ok(Value::DateTime(result))
            }
            Value::Time(time) => {
                let mut result = time.clone();
                result.sub_duration(duration);
                Ok(Value::Time(result))
            }
            _ => Err("只能对日期时间类型进行持续时间运算".to_string()),
        }
    }

    /// 日期时间之间的差值
    pub fn date_diff(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Date(a), Value::Date(b)) => {
                let diff = Self::calculate_date_diff(a, b);
                Ok(Value::Duration(diff))
            }
            (Value::DateTime(a), Value::DateTime(b)) => {
                let diff = Self::calculate_datetime_diff(a, b);
                Ok(Value::Duration(diff))
            }
            _ => Err("只能对相同类型的日期时间进行差值计算".to_string()),
        }
    }

    fn calculate_date_diff(a: &DateValue, b: &DateValue) -> DurationValue {
        let days_a = Self::date_to_days(a);
        let days_b = Self::date_to_days(b);
        let diff_days = days_a - days_b;
        DurationValue {
            seconds: diff_days * 86400,
            microseconds: 0,
            months: 0,
        }
    }

    fn calculate_datetime_diff(a: &DateTimeValue, b: &DateTimeValue) -> DurationValue {
        let days_a = Self::date_to_days(&DateValue {
            year: a.year,
            month: a.month,
            day: a.day,
        });
        let days_b = Self::date_to_days(&DateValue {
            year: b.year,
            month: b.month,
            day: b.day,
        });

        let total_seconds_a = days_a * 86400 + a.hour as i64 * 3600 + a.minute as i64 * 60 + a.sec as i64;
        let total_seconds_b = days_b * 86400 + b.hour as i64 * 3600 + b.minute as i64 * 60 + b.sec as i64;

        let diff_seconds = total_seconds_a - total_seconds_b;
        let diff_microseconds = a.microsec as i32 - b.microsec as i32;

        DurationValue {
            seconds: diff_seconds,
            microseconds: diff_microseconds,
            months: 0,
        }
    }

    fn date_to_days(date: &DateValue) -> i64 {
        let year = date.year as i64;
        let month = date.month as i64;
        let day = date.day as i64;

        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;

        day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }
}
```

**实现步骤：**
1. 在 `src/core/value/operations.rs` 中添加上述方法
2. 在 `src/core/value/types.rs` 中为 DateValue、TimeValue、DateTimeValue 实现 add_duration 和 sub_duration 方法
3. 添加单元测试验证功能
4. 添加边界情况测试（闰年、月末等）

**优先级：** 高

**预计工作量：** 4-6 小时

#### 1.2.3 扩展地理运算支持

**目标文件：** 新建 `src/core/value/geography_ops.rs`

**实现内容：**

```rust
use super::types::{GeographyValue, Value};
use std::f64::consts::PI;

const EARTH_RADIUS_KM: f64 = 6371.0;
const DEG_TO_RAD: f64 = PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / PI;

impl GeographyValue {
    /// 计算两点之间的 Haversine 距离（单位：公里）
    pub fn distance(&self, other: &GeographyValue) -> f64 {
        let lat1 = self.latitude * DEG_TO_RAD;
        let lat2 = other.latitude * DEG_TO_RAD;
        let delta_lat = (other.latitude - self.latitude) * DEG_TO_RAD;
        let delta_lon = (other.longitude - self.longitude) * DEG_TO_RAD;

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    /// 计算两点之间的方位角（单位：度）
    pub fn bearing(&self, other: &GeographyValue) -> f64 {
        let lat1 = self.latitude * DEG_TO_RAD;
        let lat2 = other.latitude * DEG_TO_RAD;
        let delta_lon = (other.longitude - self.longitude) * DEG_TO_RAD;

        let y = delta_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();

        let bearing = y.atan2(x) * RAD_TO_DEG;
        (bearing + 360.0) % 360.0
    }

    /// 计算从当前点出发，沿指定方位角移动指定距离后的新坐标
    pub fn destination(&self, bearing: f64, distance: f64) -> GeographyValue {
        let lat1 = self.latitude * DEG_TO_RAD;
        let lon1 = self.longitude * DEG_TO_RAD;
        let brng = bearing * DEG_TO_RAD;
        let d = distance / EARTH_RADIUS_KM;

        let lat2 = lat1.sin() + d * lat1.cos() * brng.sin();
        let lat2 = lat2.asin().clamp(-PI / 2.0, PI / 2.0);

        let lon2 = lon1
            + (brng.cos() * lat1.cos() / lat2.cos()).atan2(
                d * lat1.cos() * brng.sin(),
                1.0 - d * lat1.cos() * (1.0 - lat2.cos()),
            );

        GeographyValue {
            latitude: lat2 * RAD_TO_DEG,
            longitude: (lon2 * RAD_TO_DEG + 540.0) % 360.0 - 180.0,
        }
    }

    /// 检查点是否在指定矩形区域内
    pub fn in_bbox(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> bool {
        self.latitude >= min_lat
            && self.latitude <= max_lat
            && self.longitude >= min_lon
            && self.longitude <= max_lon
    }
}

impl Value {
    /// 计算两个地理点之间的距离
    pub fn geo_distance(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Geography(a), Value::Geography(b)) => {
                Ok(Value::Float(a.distance(b)))
            }
            _ => Err("只能对地理类型计算距离".to_string()),
        }
    }

    /// 计算从当前地理点到目标点的方位角
    pub fn geo_bearing(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Geography(a), Value::Geography(b)) => {
                Ok(Value::Float(a.bearing(b)))
            }
            _ => Err("只能对地理类型计算方位角".to_string()),
        }
    }

    /// 检查地理点是否在指定区域内
    pub fn geo_in_bbox(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> Result<Value, String> {
        match self {
            Value::Geography(geo) => {
                Ok(Value::Bool(geo.in_bbox(min_lat, max_lat, min_lon, max_lon)))
            }
            _ => Err("只能对地理类型进行区域检查".to_string()),
        }
    }
}
```

**实现步骤：**
1. 创建新文件 `src/core/value/geography_ops.rs`
2. 在 `src/core/value/mod.rs` 中导出模块
3. 添加单元测试验证功能
4. 添加边界情况测试（极点、日期变更线等）

**优先级：** 中

**预计工作量：** 3-4 小时

---

## 二、类型转换完善方案

### 2.1 当前状态分析

**已实现的转换：**
- ✅ `to_bool()` - 转换为布尔值
- ✅ `to_int()` - 转换为整数
- ✅ `to_float()` - 转换为浮点数
- ✅ `to_string()` - 转换为字符串
- ✅ `to_list()` - 转换为列表
- ✅ `to_set()` - 转换为集合
- ✅ `to_map()` - 转换为映射
- ✅ `From` trait 实现

**问题分析：**
1. 缺少对日期时间类型的转换
2. 缺少对地理类型的转换
3. 缺少对 Duration 类型的转换
4. 转换错误处理不够完善
5. 缺少隐式类型转换机制

### 2.2 改进方案

#### 2.2.1 扩展日期时间类型转换

**目标文件：** `src/core/value/conversion.rs`

**实现内容：**

```rust
impl Value {
    /// 转换为日期
    pub fn to_date(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Date(d) => Value::Date(d.clone()),
            Value::DateTime(dt) => Value::Date(DateValue {
                year: dt.year,
                month: dt.month,
                day: dt.day,
            }),
            Value::String(s) => {
                Self::parse_date_string(s)
            }
            Value::Int(i) => {
                Self::days_to_date(*i)
            }
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 转换为时间
    pub fn to_time(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Time(t) => Value::Time(t.clone()),
            Value::DateTime(dt) => Value::Time(TimeValue {
                hour: dt.hour,
                minute: dt.minute,
                sec: dt.sec,
                microsec: dt.microsec,
            }),
            Value::String(s) => {
                Self::parse_time_string(s)
            }
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 转换为日期时间
    pub fn to_datetime(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::DateTime(dt) => Value::DateTime(dt.clone()),
            Value::Date(d) => Value::DateTime(DateTimeValue {
                year: d.year,
                month: d.month,
                day: d.day,
                hour: 0,
                minute: 0,
                sec: 0,
                microsec: 0,
            }),
            Value::Time(t) => Value::DateTime(DateTimeValue {
                year: 1970,
                month: 1,
                day: 1,
                hour: t.hour,
                minute: t.minute,
                sec: t.sec,
                microsec: t.microsec,
            }),
            Value::String(s) => {
                Self::parse_datetime_string(s)
            }
            Value::Int(i) => {
                let days = *i;
                let date = Self::days_to_date(days);
                Value::DateTime(DateTimeValue {
                    year: date.year,
                    month: date.month,
                    day: date.day,
                    hour: 0,
                    minute: 0,
                    sec: 0,
                    microsec: 0,
                })
            }
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 转换为持续时间
    pub fn to_duration(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Duration(d) => Value::Duration(d.clone()),
            Value::Int(i) => Value::Duration(DurationValue {
                seconds: *i,
                microseconds: 0,
                months: 0,
            }),
            Value::Float(f) => {
                let seconds = f.floor() as i64;
                let microseconds = ((f - seconds as f64) * 1_000_000.0) as i32;
                Value::Duration(DurationValue {
                    seconds,
                    microseconds,
                    months: 0,
                })
            }
            Value::String(s) => {
                Self::parse_duration_string(s)
            }
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 解析日期字符串
    fn parse_date_string(s: &str) -> Value {
        let formats = vec![
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%Y%m%d",
        ];

        for format in &formats {
            if let Ok(dt) = chrono::NaiveDate::parse_from_str(s, format) {
                return Value::Date(DateValue {
                    year: dt.year(),
                    month: dt.month(),
                    day: dt.day(),
                });
            }
        }

        Value::Null(NullType::BadData)
    }

    /// 解析时间字符串
    fn parse_time_string(s: &str) -> Value {
        let formats = vec![
            "%H:%M:%S",
            "%H:%M:%S%.f",
            "%H:%M",
        ];

        for format in &formats {
            if let Ok(time) = chrono::NaiveTime::parse_from_str(s, format) {
                return Value::Time(TimeValue {
                    hour: time.hour(),
                    minute: time.minute(),
                    sec: time.second(),
                    microsec: time.nanosecond() / 1000,
                });
            }
        }

        Value::Null(NullType::BadData)
    }

    /// 解析日期时间字符串
    fn parse_datetime_string(s: &str) -> Value {
        let formats = vec![
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y/%m/%d %H:%M:%S",
        ];

        for format in &formats {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, format) {
                return Value::DateTime(DateTimeValue {
                    year: dt.year(),
                    month: dt.month(),
                    day: dt.day(),
                    hour: dt.hour(),
                    minute: dt.minute(),
                    sec: dt.second(),
                    microsec: dt.nanosecond() / 1000,
                });
            }
        }

        Value::Null(NullType::BadData)
    }

    /// 解析持续时间字符串
    fn parse_duration_string(s: &str) -> Value {
        use regex::Regex;

        let re = Regex::new(r"(?:(\d+)d)?(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?").unwrap();
        let caps = re.captures(s);

        if let Some(caps) = caps {
            let days = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let hours = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let minutes = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let seconds = caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);

            let total_seconds = days * 86400 + hours * 3600 + minutes * 60 + seconds;

            return Value::Duration(DurationValue {
                seconds: total_seconds,
                microseconds: 0,
                months: 0,
            });
        }

        Value::Null(NullType::BadData)
    }

    /// 将天数转换为日期
    fn days_to_date(days: i64) -> DateValue {
        let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let date = epoch + chrono::Duration::days(days);
        DateValue {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}
```

**依赖项：**
需要在 `Cargo.toml` 中添加：
```toml
[dependencies]
chrono = "0.4"
regex = "1.10"
```

**实现步骤：**
1. 在 `src/core/value/conversion.rs` 中添加上述方法
2. 添加单元测试验证各种格式的解析
3. 添加边界情况测试（闰年、月末等）

**优先级：** 高

**预计工作量：** 4-5 小时

#### 2.2.2 实现隐式类型转换

**目标文件：** `src/core/value/conversion.rs`

**实现内容：**

```rust
impl Value {
    /// 尝试隐式转换为指定类型
    pub fn try_implicit_cast(&self, target_type: &DataType) -> Result<Value, String> {
        match target_type {
            DataType::Bool => Ok(self.to_bool()),
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
                Ok(self.to_int())
            }
            DataType::Float | DataType::Double => Ok(self.to_float()),
            DataType::String => self.to_string().map(Value::String),
            DataType::Date => Ok(self.to_date()),
            DataType::Time => Ok(self.to_time()),
            DataType::DateTime => Ok(self.to_datetime()),
            DataType::Duration => Ok(self.to_duration()),
            _ => Err(format!("无法隐式转换为 {:?}", target_type)),
        }
    }

    /// 检查是否可以隐式转换
    pub fn can_implicitly_cast_to(&self, target_type: &DataType) -> bool {
        self.try_implicit_cast(target_type).is_ok()
    }
}
```

**实现步骤：**
1. 在 `src/core/value/conversion.rs` 中添加上述方法
2. 添加单元测试验证转换规则
3. 更新类型系统工具以支持隐式转换

**优先级：** 中

**预计工作量：** 2-3 小时

---

## 三、地理空间扩展方案

### 3.1 当前状态分析

**已实现：**
- ✅ 基础坐标点（GeographyValue）
- ✅ 坐标存储（latitude、longitude）

**缺失功能：**
1. ❌ LineString（线）支持
2. ❌ Polygon（多边形）支持
3. ❌ WKT/WKB 格式支持
4. ❌ 空间索引支持
5. ❌ 空间关系判断（包含、相交等）
6. ❌ 地理库集成（S2、GEOS 等）

### 3.2 改进方案

#### 3.2.1 扩展地理类型定义

**目标文件：** `src/core/value/types.rs`

**实现内容：**

```rust
/// 地理形状类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum GeoShape {
    Point,
    LineString,
    Polygon,
}

/// 地理坐标
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn normalize(&mut self) {
        self.x = ((self.x + 180.0) % 360.0) - 180.0;
        self.y = self.y.clamp(-90.0, 90.0);
    }

    pub fn is_valid(&self) -> bool {
        self.x >= -180.0 && self.x <= 180.0 && self.y >= -90.0 && self.y <= 90.0
    }
}

/// 线
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct LineString {
    pub coordinates: Vec<Coordinate>,
}

impl LineString {
    pub fn new() -> Self {
        Self {
            coordinates: Vec::new(),
        }
    }

    pub fn add_point(&mut self, coord: Coordinate) {
        self.coordinates.push(coord);
    }

    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for window in self.coordinates.windows(2) {
            total += haversine_distance(
                window[0].y, window[0].x,
                window[1].y, window[1].x,
            );
        }
        total
    }

    pub fn is_valid(&self) -> bool {
        self.coordinates.len() >= 2
            && self.coordinates.iter().all(|c| c.is_valid())
    }
}

/// 多边形
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct Polygon {
    pub rings: Vec<Vec<Coordinate>>,
}

impl Polygon {
    pub fn new() -> Self {
        Self { rings: Vec::new() }
    }

    pub fn add_ring(&mut self, ring: Vec<Coordinate>) {
        self.rings.push(ring);
    }

    pub fn area(&self) -> f64 {
        if self.rings.is_empty() {
            return 0.0;
        }

        let exterior = &self.rings[0];
        let mut area = 0.0;
        let n = exterior.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area += exterior[i].x * exterior[j].y;
            area -= exterior[j].x * exterior[i].y;
        }

        area.abs() / 2.0
    }

    pub fn contains_point(&self, point: &Coordinate) -> bool {
        if self.rings.is_empty() {
            return false;
        }

        let exterior = &self.rings[0];
        Self::point_in_polygon(point, exterior)
    }

    fn point_in_polygon(point: &Coordinate, polygon: &[Coordinate]) -> bool {
        let mut inside = false;
        let n = polygon.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let xi = polygon[i].x;
            let yi = polygon[i].y;
            let xj = polygon[j].x;
            let yj = polygon[j].y;

            if ((yi > point.y) != (yj > point.y))
                && (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi)
            {
                inside = !inside;
            }
        }

        inside
    }

    pub fn is_valid(&self) -> bool {
        !self.rings.is_empty()
            && self.rings.iter().all(|ring| {
                ring.len() >= 3 && ring.iter().all(|c| c.is_valid())
            })
    }
}

/// 扩展的地理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub enum Geography {
    Point(GeographyValue),
    LineString(LineString),
    Polygon(Polygon),
}

impl Geography {
    pub fn shape(&self) -> GeoShape {
        match self {
            Geography::Point(_) => GeoShape::Point,
            Geography::LineString(_) => GeoShape::LineString,
            Geography::Polygon(_) => GeoShape::Polygon,
        }
    }

    pub fn as_wkt(&self) -> String {
        match self {
            Geography::Point(geo) => {
                format!("POINT({} {})", geo.longitude, geo.latitude)
            }
            Geography::LineString(ls) => {
                let coords: Vec<String> = ls.coordinates
                    .iter()
                    .map(|c| format!("{} {}", c.x, c.y))
                    .collect();
                format!("LINESTRING({})", coords.join(", "))
            }
            Geography::Polygon(poly) => {
                let rings: Vec<String> = poly.rings
                    .iter()
                    .map(|ring| {
                        let coords: Vec<String> = ring
                            .iter()
                            .map(|c| format!("{} {}", c.x, c.y))
                            .collect();
                        format!("({})", coords.join(", "))
                    })
                    .collect();
                format!("POLYGON({})", rings.join(", "))
            }
        }
    }

    pub fn from_wkt(wkt: &str) -> Result<Self, String> {
        let wkt = wkt.trim();
        
        if wkt.starts_with("POINT") {
            Self::parse_point_wkt(wkt)
        } else if wkt.starts_with("LINESTRING") {
            Self::parse_linestring_wkt(wkt)
        } else if wkt.starts_with("POLYGON") {
            Self::parse_polygon_wkt(wkt)
        } else {
            Err("不支持的 WKT 格式".to_string())
        }
    }

    fn parse_point_wkt(wkt: &str) -> Result<Self, String> {
        use regex::Regex;
        let re = Regex::new(r"POINT\s*\(\s*([-\d.]+)\s+([-\d.]+)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let lon = caps.get(1).unwrap().as_str().parse::<f64>().unwrap();
            let lat = caps.get(2).unwrap().as_str().parse::<f64>().unwrap();
            return Ok(Geography::Point(GeographyValue {
                latitude: lat,
                longitude: lon,
            }));
        }
        
        Err("无效的 POINT WKT 格式".to_string())
    }

    fn parse_linestring_wkt(wkt: &str) -> Result<Self, String> {
        use regex::Regex;
        let re = Regex::new(r"LINESTRING\s*\(\s*(.*?)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let coords_str = caps.get(1).unwrap().as_str();
            let coords: Result<Vec<Coordinate>, _> = coords_str
                .split(',')
                .map(|s| {
                    let parts: Vec<&str> = s.trim().split_whitespace().collect();
                    if parts.len() == 2 {
                        let x = parts[0].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                        let y = parts[1].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                        Ok(Coordinate::new(x, y))
                    } else {
                        Err("无效的坐标格式".to_string())
                    }
                })
                .collect();
            
            return coords.map(|coordinates| {
                Geography::LineString(LineString { coordinates })
            });
        }
        
        Err("无效的 LINESTRING WKT 格式".to_string())
    }

    fn parse_polygon_wkt(wkt: &str) -> Result<Self, String> {
        use regex::Regex;
        let re = Regex::new(r"POLYGON\s*\(\s*(.*?)\s*\)").unwrap();
        
        if let Some(caps) = re.captures(wkt) {
            let rings_str = caps.get(1).unwrap().as_str();
            let rings: Result<Vec<Vec<Coordinate>>, _> = rings_str
                .split("),(")
                .map(|s| {
                    let s = s.trim_start_matches('(').trim_end_matches(')');
                    let coords: Result<Vec<Coordinate>, _> = s
                        .split(',')
                        .map(|coord_str| {
                            let parts: Vec<&str> = coord_str.trim().split_whitespace().collect();
                            if parts.len() == 2 {
                                let x = parts[0].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                                let y = parts[1].parse::<f64>().map_err(|_| "无效的坐标".to_string())?;
                                Ok(Coordinate::new(x, y))
                            } else {
                                Err("无效的坐标格式".to_string())
                            }
                        })
                        .collect();
                    coords
                })
                .collect();
            
            return rings.map(|rings| {
                Geography::Polygon(Polygon { rings })
            });
        }
        
        Err("无效的 POLYGON WKT 格式".to_string())
    }
}

/// Haversine 距离计算
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0;
    const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

    let dlat = (lat2 - lat1) * DEG_TO_RAD;
    let dlon = (lon2 - lon1) * DEG_TO_RAD;

    let a = (dlat / 2.0).sin().powi(2)
        + lat1 * DEG_TO_RAD * lat2 * DEG_TO_RAD * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    R * c
}
```

**实现步骤：**
1. 在 `src/core/value/types.rs` 中添加上述类型定义
2. 更新 Value 枚举以支持新的 Geography 类型
3. 添加单元测试验证 WKT 解析和生成
4. 添加空间关系判断测试

**优先级：** 高

**预计工作量：** 8-10 小时

#### 3.2.2 集成地理库（可选）

**目标：** 集成 S2 或 GEOS 库以提供更强大的地理空间功能

**选项 1：S2 库**
```toml
[dependencies]
s2 = "0.0"
```

**选项 2：GEOS 库**
```toml
[dependencies]
geos = "0.9"
```

**实现步骤：**
1. 选择合适的地理库
2. 添加依赖项
3. 实现与库的集成接口
4. 提供高级地理操作（缓冲区、交集、并集等）

**优先级：** 低

**预计工作量：** 20-30 小时

---

## 四、数据集操作完善方案

### 4.1 当前状态分析

**已实现：**
- ✅ 基本的数据集结构（DataSet）
- ✅ 列名和行数据存储

**缺失功能：**
1. ❌ append 操作（垂直合并）
2. ❌ merge 操作（水平合并）
3. ❌ 列操作（添加、删除、重命名）
4. ❌ 行操作（过滤、排序、分组）
5. ❌ 数据集连接操作

### 4.2 改进方案

**目标文件：** `src/core/value/types.rs`

**实现内容：**

```rust
impl DataSet {
    /// 垂直追加数据集（列名必须相同）
    pub fn append(&mut self, other: DataSet) -> Result<(), String> {
        if self.col_names.is_empty() {
            self.col_names = other.col_names;
        } else if self.col_names != other.col_names {
            return Err("列名不匹配，无法追加数据集".to_string());
        }

        self.rows.extend(other.rows);
        Ok(())
    }

    /// 水平合并数据集（行数必须相同）
    pub fn merge(&mut self, other: DataSet) -> Result<(), String> {
        if self.rows.len() != other.rows.len() {
            return Err("行数不匹配，无法合并数据集".to_string());
        }

        self.col_names.extend(other.col_names);
        for (row, other_row) in self.rows.iter_mut().zip(other.rows.iter()) {
            row.extend(other_row.clone());
        }

        Ok(())
    }

    /// 添加列
    pub fn add_column(&mut self, name: String, default_value: Value) -> Result<(), String> {
        if self.col_names.contains(&name) {
            return Err(format!("列名 '{}' 已存在", name));
        }

        self.col_names.push(name);
        for row in &mut self.rows {
            row.push(default_value.clone());
        }

        Ok(())
    }

    /// 删除列
    pub fn remove_column(&mut self, name: &str) -> Result<(), String> {
        let index = self.col_names.iter().position(|n| n == name)
            .ok_or_else(|| format!("列名 '{}' 不存在", name))?;

        self.col_names.remove(index);
        for row in &mut self.rows {
            row.remove(index);
        }

        Ok(())
    }

    /// 重命名列
    pub fn rename_column(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        let index = self.col_names.iter().position(|n| n == old_name)
            .ok_or_else(|| format!("列名 '{}' 不存在", old_name))?;

        if self.col_names.contains(&new_name) {
            return Err(format!("列名 '{}' 已存在", new_name));
        }

        self.col_names[index] = new_name;
        Ok(())
    }

    /// 获取列索引
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.col_names.iter().position(|n| n == name)
    }

    /// 获取列数据
    pub fn column(&self, name: &str) -> Option<Vec<Value>> {
        let index = self.column_index(name)?;
        Some(self.rows.iter().map(|row| row[index].clone()).collect())
    }

    /// 过滤行
    pub fn filter<F>(&self, predicate: F) -> DataSet
    where
        F: Fn(&Vec<Value>) -> bool,
    {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().filter(|row| predicate(row)).cloned().collect(),
        }
    }

    /// 按列排序
    pub fn sort_by(&mut self, column_name: &str, ascending: bool) -> Result<(), String> {
        let index = self.column_index(column_name)
            .ok_or_else(|| format!("列名 '{}' 不存在", column_name))?;

        self.rows.sort_by(|a, b| {
            let cmp = a[index].cmp(&b[index]);
            if ascending { cmp } else { cmp.reverse() }
        });

        Ok(())
    }

    /// 按多列排序
    pub fn sort_by_multiple(&mut self, columns: &[(String, bool)]) -> Result<(), String> {
        let indices: Result<Vec<usize>, String> = columns
            .iter()
            .map(|(name, _)| {
                self.column_index(name)
                    .ok_or_else(|| format!("列名 '{}' 不存在", name))
            })
            .collect();

        let indices = indices?;

        self.rows.sort_by(|a, b| {
            for (idx, ascending) in indices.iter().zip(columns.iter().map(|(_, asc)| asc)) {
                let cmp = a[*idx].cmp(&b[*idx]);
                if cmp != std::cmp::Ordering::Equal {
                    return if *ascending { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(())
    }

    /// 分组
    pub fn group_by(&self, column_name: &str) -> Result<Vec<DataSet>, String> {
        let index = self.column_index(column_name)
            .ok_or_else(|| format!("列名 '{}' 不存在", column_name))?;

        let mut groups: std::collections::HashMap<Value, Vec<Vec<Value>>> = std::collections::HashMap::new();

        for row in &self.rows {
            let key = row[index].clone();
            groups.entry(key).or_insert_with(Vec::new).push(row.clone());
        }

        Ok(groups.into_values().map(|rows| DataSet {
            col_names: self.col_names.clone(),
            rows,
        }).collect())
    }

    /// 连接数据集
    pub fn join(
        &self,
        other: &DataSet,
        on_column: &str,
        join_type: JoinType,
    ) -> Result<DataSet, String> {
        let left_index = self.column_index(on_column)
            .ok_or_else(|| format!("左数据集列名 '{}' 不存在", on_column))?;
        let right_index = other.column_index(on_column)
            .ok_or_else(|| format!("右数据集列名 '{}' 不存在", on_column))?;

        let mut result = DataSet::new();
        result.col_names = self.col_names.clone();
        result.col_names.extend(other.col_names.iter().filter(|n| *n != on_column));

        match join_type {
            JoinType::Inner => {
                for left_row in &self.rows {
                    for right_row in &other.rows {
                        if left_row[left_index] == right_row[right_index] {
                            let mut new_row = left_row.clone();
                            new_row.extend(right_row.iter().enumerate()
                                .filter(|(i, _)| *i != right_index)
                                .map(|(_, v)| v.clone()));
                            result.rows.push(new_row);
                        }
                    }
                }
            }
            JoinType::Left => {
                for left_row in &self.rows {
                    let mut matched = false;
                    for right_row in &other.rows {
                        if left_row[left_index] == right_row[right_index] {
                            let mut new_row = left_row.clone();
                            new_row.extend(right_row.iter().enumerate()
                                .filter(|(i, _)| *i != right_index)
                                .map(|(_, v)| v.clone()));
                            result.rows.push(new_row);
                            matched = true;
                        }
                    }
                    if !matched {
                        let mut new_row = left_row.clone();
                        for _ in 0..other.col_names.len() - 1 {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        result.rows.push(new_row);
                    }
                }
            }
            JoinType::Right => {
                for right_row in &other.rows {
                    let mut matched = false;
                    for left_row in &self.rows {
                        if left_row[left_index] == right_row[right_index] {
                            let mut new_row = left_row.clone();
                            new_row.extend(right_row.iter().enumerate()
                                .filter(|(i, _)| *i != right_index)
                                .map(|(_, v)| v.clone()));
                            result.rows.push(new_row);
                            matched = true;
                        }
                    }
                    if !matched {
                        let mut new_row = Vec::new();
                        for _ in 0..self.col_names.len() - 1 {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        new_row.extend(right_row.iter().enumerate()
                            .filter(|(i, _)| *i != right_index)
                            .map(|(_, v)| v.clone()));
                        result.rows.push(new_row);
                    }
                }
            }
            JoinType::Full => {
                let mut left_matched = vec![false; self.rows.len()];
                let mut right_matched = vec![false; other.rows.len()];

                for (li, left_row) in self.rows.iter().enumerate() {
                    for (ri, right_row) in other.rows.iter().enumerate() {
                        if left_row[left_index] == right_row[right_index] {
                            let mut new_row = left_row.clone();
                            new_row.extend(right_row.iter().enumerate()
                                .filter(|(i, _)| *i != right_index)
                                .map(|(_, v)| v.clone()));
                            result.rows.push(new_row);
                            left_matched[li] = true;
                            right_matched[ri] = true;
                        }
                    }
                }

                for (li, left_row) in self.rows.iter().enumerate() {
                    if !left_matched[li] {
                        let mut new_row = left_row.clone();
                        for _ in 0..other.col_names.len() - 1 {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        result.rows.push(new_row);
                    }
                }

                for (ri, right_row) in other.rows.iter().enumerate() {
                    if !right_matched[ri] {
                        let mut new_row = Vec::new();
                        for _ in 0..self.col_names.len() - 1 {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        new_row.extend(right_row.iter().enumerate()
                            .filter(|(i, _)| *i != right_index)
                            .map(|(_, v)| v.clone()));
                        result.rows.push(new_row);
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
```

**实现步骤：**
1. 在 `src/core/value/types.rs` 中为 DataSet 添加上述方法
2. 添加单元测试验证各种操作
3. 添加性能测试（大数据集）

**优先级：** 中

**预计工作量：** 6-8 小时

---

## 五、内存优化方案

### 5.1 当前状态分析

**内存占用问题：**
1. Value 使用 enum，大小不固定
2. Vertex 和 Edge 使用 Box 包装
3. 内部 ID 增加了额外内存
4. 字符串使用 String，可能存在冗余

### 5.2 改进方案

#### 5.2.1 使用 Arc 优化共享数据

**目标文件：** `src/core/value/types.rs`

**实现内容：**

```rust
use std::sync::Arc;

/// 优化的顶点定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Vertex {
    pub vid: Arc<Value>,  // 使用 Arc 替代 Box
    pub id: i64,
    pub tags: Arc<Vec<Tag>>,  // 使用 Arc 共享标签列表
    pub properties: Arc<HashMap<String, Value>>,  // 使用 Arc 共享属性
}

impl Vertex {
    pub fn new(vid: Value, tags: Vec<Tag>) -> Self {
        Self {
            vid: Arc::new(vid),
            id: 0,
            tags: Arc::new(tags),
            properties: Arc::new(HashMap::new()),
        }
    }

    pub fn vid(&self) -> &Value {
        &self.vid
    }

    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    pub fn properties(&self) &HashMap<String, Value> {
        &self.properties
    }

    pub fn set_property(&mut self, name: String, value: Value) {
        Arc::make_mut(&mut self.properties).insert(name, value);
    }
}

/// 优化的边定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Edge {
    pub src: Arc<Value>,
    pub dst: Arc<Value>,
    pub edge_type: Arc<String>,  // 使用 Arc 共享边类型
    pub ranking: i64,
    pub id: i64,
    pub props: Arc<HashMap<String, Value>>,
}

impl Edge {
    pub fn new(
        src: Value,
        dst: Value,
        edge_type: String,
        ranking: i64,
        props: HashMap<String, Value>,
    ) -> Self {
        Self {
            src: Arc::new(src),
            dst: Arc::new(dst),
            edge_type: Arc::new(edge_type),
            ranking,
            id: 0,
            props: Arc::new(props),
        }
    }

    pub fn src(&self) -> &Value {
        &self.src
    }

    pub fn dst(&self) -> &Value {
        &self.dst
    }

    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    pub fn set_property(&mut self, name: String, value: Value) {
        Arc::make_mut(&mut self.props).insert(name, value);
    }
}
```

**优势：**
- Arc 允许多个引用共享同一数据
- Arc::make_mut 提供写时复制（COW）语义
- 减少内存复制

**实现步骤：**
1. 更新 Vertex 和 Edge 定义
2. 更新所有使用这些类型的代码
3. 添加性能测试验证内存使用

**优先级：** 中

**预计工作量：** 4-6 小时

#### 5.2.2 使用 Cow 优化字符串

**目标文件：** `src/core/value/types.rs`

**实现内容：**

```rust
use std::borrow::Cow;

/// 优化的边定义，使用 Cow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Edge {
    pub src: Arc<Value>,
    pub dst: Arc<Value>,
    pub edge_type: Cow<'static, str>,  // 使用 Cow 优化字符串
    pub ranking: i64,
    pub id: i64,
    pub props: Arc<HashMap<String, Value>>,
}

impl Edge {
    pub fn new(
        src: Value,
        dst: Value,
        edge_type: String,
        ranking: i64,
        props: HashMap<String, Value>,
    ) -> Self {
        Self {
            src: Arc::new(src),
            dst: Arc::new(dst),
            edge_type: Cow::Owned(edge_type),
            ranking,
            id: 0,
            props: Arc::new(props),
        }
    }

    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }
}
```

**优势：**
- Cow 允许在需要时才复制字符串
- 对于静态字符串可以直接使用引用
- 减少不必要的内存分配

**实现步骤：**
1. 更新 Edge 定义使用 Cow
2. 更新所有使用 edge_type 的代码
3. 添加性能测试

**优先级：** 低

**预计工作量：** 2-3 小时

#### 5.2.3 优化 Value 枚举大小

**目标文件：** `src/core/value/types.rs`

**实现内容：**

```rust
/// 优化的 Value 定义，使用更紧凑的存储
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Box<str>),  // 使用 Box<str> 替代 String
    Date(DateValue),
    Time(TimeValue),
    DateTime(DateTimeValue),
    Vertex(Box<Vertex>),
    Edge(Box<Edge>),
    Path(Box<Path>),
    List(Vec<Value>),
    Map(Box<HashMap<String, Value>>),  // 使用 Box 减小枚举大小
    Set(Box<HashSet<Value>>),  // 使用 Box 减小枚举大小
    Geography(GeographyValue),
    Duration(DurationValue),
    DataSet(Box<DataSet>),  // 使用 Box 减小枚举大小
}
```

**优势：**
- Box<str> 比 String 更紧凑
- Box 包裹大类型可以减小枚举整体大小
- 减少栈内存占用

**实现步骤：**
1. 更新 Value 定义
2. 更新所有使用 Value 的代码
3. 添加性能测试

**优先级：** 低

**预计工作量：** 3-4 小时

---

## 六、实施计划

### 6.1 优先级排序

| 优先级 | 任务 | 预计工作量 |
|--------|------|------------|
| 高 | 扩展日期时间运算支持 | 4-6 小时 |
| 高 | 扩展日期时间类型转换 | 4-5 小时 |
| 高 | 扩展地理类型定义 | 8-10 小时 |
| 中 | 添加位运算支持 | 2-3 小时 |
| 中 | 实现隐式类型转换 | 2-3 小时 |
| 中 | 扩展地理运算支持 | 3-4 小时 |
| 中 | 完善数据集操作 | 6-8 小时 |
| 中 | 使用 Arc 优化共享数据 | 4-6 小时 |
| 低 | 集成地理库 | 20-30 小时 |
| 低 | 使用 Cow 优化字符串 | 2-3 小时 |
| 低 | 优化 Value 枚举大小 | 3-4 小时 |

### 6.2 实施阶段

#### 第一阶段（高优先级）
- 扩展日期时间运算支持
- 扩展日期时间类型转换
- 扩展地理类型定义

**预计时间：** 16-21 小时（2-3 个工作日）

#### 第二阶段（中优先级）
- 添加位运算支持
- 实现隐式类型转换
- 扩展地理运算支持
- 完善数据集操作
- 使用 Arc 优化共享数据

**预计时间：** 17-24 小时（2-3 个工作日）

#### 第三阶段（低优先级）
- 集成地理库（可选）
- 使用 Cow 优化字符串
- 优化 Value 枚举大小

**预计时间：** 25-37 小时（3-5 个工作日）

### 6.3 测试计划

每个改进都需要：
1. 单元测试
2. 集成测试
3. 性能测试
4. 边界情况测试

### 6.4 文档更新

每个改进都需要：
1. 更新 API 文档
2. 添加使用示例
3. 更新变更日志

---

## 七、总结

本文档详细制定了 GraphDB 数据结构的改进方案，涵盖了：

1. **运算支持补充**：位运算、日期时间运算、地理运算
2. **类型转换完善**：日期时间转换、隐式类型转换
3. **地理空间扩展**：LineString、Polygon、WKT/WKB 支持
4. **数据集操作完善**：append、merge、连接操作
5. **内存优化**：Arc、Cow、Box 优化

通过实施这些改进，GraphDB 将在功能完整性和性能方面显著提升，同时保持 Rust 的类型安全和内存安全优势。

**总预计工作量：** 58-82 小时（7-11 个工作日）

**建议实施顺序：** 按优先级从高到低实施，确保核心功能优先完善。
