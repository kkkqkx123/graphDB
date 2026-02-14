//! 值运算模块
//!
//! 此模块包含的值运算方法当前未被项目使用。
/// 算术运算方法保留供将来使用，如果确认不需要可以安全移除。
/// 当前聚合运算逻辑实现在 query/executor/aggregation.rs 中。

use super::types::Value;
use super::date_time::{DateValue, DateTimeValue, DurationValue};

impl Value {
    /// 加法运算
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            _ => Err("无法对这些类型的值进行加法运算".to_string()),
        }
    }

    /// 减法运算
    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - *b as f64)),
            _ => Err("无法对这些值进行减法运算".to_string()),
        }
    }

    /// 乘法运算
    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * *b as f64)),
            _ => Err("无法对这些类型的值进行乘法运算".to_string()),
        }
    }

    /// 除法运算
    pub fn div(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int(a / b))
                }
            }
            (Float(a), Float(b)) => {
                if *b == 0.0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(a / b))
                }
            }
            (Int(a), Float(b)) => {
                if *b == 0.0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(*a as f64 / b))
                }
            }
            (Float(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(a / *b as f64))
                }
            }
            _ => Err("无法对这些类型的值进行除法运算".to_string()),
        }
    }

    /// 取模运算
    pub fn rem(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int(a % b))
                }
            }
            _ => Err("只能对整数类型进行取模运算".to_string()),
        }
    }

    /// 幂运算
    pub fn pow(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int(a.pow(*b as u32)))
                }
            }
            (Float(a), Float(b)) => Ok(Float(a.powf(*b))),
            (Int(a), Float(b)) => Ok(Float((*a as f64).powf(*b))),
            (Float(a), Int(b)) => Ok(Float(a.powi(*b as i32))),
            _ => Err("无法对这些类型的值进行幂运算".to_string()),
        }
    }

    /// 取负运算
    pub fn neg(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(-a)),
            Float(a) => Ok(Float(-a)),
            _ => Err("只能对数值类型进行取负运算".to_string()),
        }
    }

    /// 逻辑与运算
    pub fn and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a && *b)),
            _ => Err("只能对布尔类型进行逻辑与运算".to_string()),
        }
    }

    /// 逻辑或运算
    pub fn or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a || *b)),
            _ => Err("只能对布尔类型进行逻辑或运算".to_string()),
        }
    }

    /// 逻辑非运算
    pub fn not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Bool(a) => Ok(Bool(!a)),
            _ => Err("只能对布尔类型进行逻辑非运算".to_string()),
        }
    }

    /// 位与运算
    pub fn bit_and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a & b)),
            _ => Err("只能对整数类型进行位与运算".to_string()),
        }
    }

    /// 位或运算
    pub fn bit_or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a | b)),
            _ => Err("只能对整数类型进行位或运算".to_string()),
        }
    }

    /// 位异或运算
    pub fn bit_xor(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a ^ b)),
            _ => Err("只能对整数类型进行位异或运算".to_string()),
        }
    }

    /// 位左移运算
    pub fn bit_shl(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int(a << *b as u32))
                }
            }
            _ => Err("只能对整数类型进行位左移运算".to_string()),
        }
    }

    /// 位右移运算
    pub fn bit_shr(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int(a >> *b as u32))
                }
            }
            _ => Err("只能对整数类型进行位右移运算".to_string()),
        }
    }

    /// 位取反运算
    pub fn bit_not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(!a)),
            _ => Err("只能对整数类型进行位取反运算".to_string()),
        }
    }

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

    /// 计算两点之间的距离
    pub fn geo_distance(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Geography(a), Value::Geography(b)) => {
                let distance = a.distance(b);
                Ok(Value::Float(distance))
            }
            _ => Err("只能对地理点类型进行距离计算".to_string()),
        }
    }

    /// 计算两点之间的方位角
    pub fn geo_bearing(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Geography(a), Value::Geography(b)) => {
                let bearing = a.bearing(b);
                Ok(Value::Float(bearing))
            }
            _ => Err("只能对地理点类型进行方位角计算".to_string()),
        }
    }

    /// 检查点是否在矩形区域内
    pub fn geo_in_bbox(&self, min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Result<Value, String> {
        match self {
            Value::Geography(geo) => {
                let result = geo.in_bbox(min_lat, max_lat, min_lon, max_lon);
                Ok(Value::Bool(result))
            }
            _ => Err("只能对地理点类型进行区域检查".to_string()),
        }
    }

    /// 计算线的长度
    pub fn geo_length(&self) -> Result<Value, String> {
        Err("线长度计算暂未实现".to_string())
    }

    /// 计算多边形的面积
    pub fn geo_area(&self) -> Result<Value, String> {
        Err("多边形面积计算暂未实现".to_string())
    }

    /// 检查点是否在多边形内
    pub fn geo_contains(&self, _other: &Value) -> Result<Value, String> {
        Err("点在多边形内判断暂未实现".to_string())
    }

    fn calculate_date_diff(a: &DateValue, b: &DateValue) -> DurationValue {
        let days_a = a.to_days();
        let days_b = b.to_days();
        let diff_days = days_a - days_b;
        DurationValue {
            seconds: diff_days * 86400,
            microseconds: 0,
            months: 0,
        }
    }

    fn calculate_datetime_diff(a: &DateTimeValue, b: &DateTimeValue) -> DurationValue {
        let date_a = DateValue {
            year: a.year,
            month: a.month,
            day: a.day,
        };
        let date_b = DateValue {
            year: b.year,
            month: b.month,
            day: b.day,
        };

        let days_a = date_a.to_days();
        let days_b = date_b.to_days();

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
}
