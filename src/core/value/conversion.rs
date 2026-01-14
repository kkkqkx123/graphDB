use super::types::{NullType, Value};
use crate::core::error::ExpressionError;

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value as i64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<NullType> for Value {
    fn from(value: NullType) -> Self {
        Value::Null(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(value)
    }
}

impl From<std::collections::HashMap<String, Value>> for Value {
    fn from(value: std::collections::HashMap<String, Value>) -> Self {
        Value::Map(value)
    }
}

impl From<std::collections::HashSet<Value>> for Value {
    fn from(value: std::collections::HashSet<Value>) -> Self {
        Value::Set(value)
    }
}

impl Value {
    /// 转换为布尔值
    pub fn to_bool(&self) -> Result<bool, ExpressionError> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Int(i) => Ok(*i != 0),
            Value::Float(f) => {
                if f.is_nan() {
                    Err(ExpressionError::type_error("无法将 NaN 转换为布尔值"))
                } else {
                    Ok(*f != 0.0)
                }
            }
            Value::String(s) => {
                if s.eq_ignore_ascii_case("true") {
                    Ok(true)
                } else if s.eq_ignore_ascii_case("false") {
                    Ok(false)
                } else {
                    Err(ExpressionError::type_error(format!("无法将字符串 '{}' 转换为布尔值", s)))
                }
            }
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为布尔值", self))),
        }
    }

    /// 转换为整数
    pub fn to_int(&self) -> Result<i64, ExpressionError> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Float(f) => {
                if f.is_nan() || f.is_infinite() {
                    Err(ExpressionError::type_error("无法将 NaN 或无穷大转换为整数"))
                } else if *f > i64::MAX as f64 || *f < i64::MIN as f64 {
                    Err(ExpressionError::type_error("浮点数超出整数范围"))
                } else {
                    Ok(*f as i64)
                }
            }
            Value::String(s) => s
                .parse::<i64>()
                .map_err(|e| ExpressionError::type_error(format!("无法将字符串 '{}' 转换为整数: {}", s, e))),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为整数", self))),
        }
    }

    /// 转换为浮点数
    pub fn to_float(&self) -> Result<f64, ExpressionError> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|e| ExpressionError::type_error(format!("无法将字符串 '{}' 转换为浮点数: {}", s, e))),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为浮点数", self))),
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> Result<String, ExpressionError> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Int(i) => Ok(i.to_string()),
            Value::Float(f) => {
                if f.is_nan() {
                    Ok("NaN".to_string())
                } else if f.is_infinite() {
                    if f.is_sign_positive() {
                        Ok("Infinity".to_string())
                    } else {
                        Ok("-Infinity".to_string())
                    }
                } else {
                    Ok(f.to_string())
                }
            }
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null(n) => Ok(format!("{:?}", n)),
            Value::Empty => Ok("Empty".to_string()),
            Value::Date(d) => {
                if d.year < 0 || d.year > 9999 || d.month < 1 || d.month > 12 || d.day < 1 || d.day > 31 {
                    return Err(ExpressionError::type_error("无效的日期值"));
                }
                Ok(format!("{}-{:02}-{:02}", d.year, d.month, d.day))
            }
            Value::Time(t) => {
                if t.hour < 0 || t.hour > 23 || t.minute < 0 || t.minute > 59 || t.sec < 0 || t.sec > 59 || t.microsec < 0 || t.microsec > 999999 {
                    return Err(ExpressionError::type_error("无效的时间值"));
                }
                Ok(format!(
                    "{:02}:{:02}:{:02}.{:06}",
                    t.hour, t.minute, t.sec, t.microsec
                ))
            }
            Value::DateTime(dt) => {
                if dt.year < 0 || dt.year > 9999 || dt.month < 1 || dt.month > 12 || dt.day < 1 || dt.day > 31 ||
                   dt.hour < 0 || dt.hour > 23 || dt.minute < 0 || dt.minute > 59 || dt.sec < 0 || dt.sec > 59 ||
                   dt.microsec < 0 || dt.microsec > 999999 {
                    return Err(ExpressionError::type_error("无效的日期时间值"));
                }
                Ok(format!(
                    "{}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.sec, dt.microsec
                ))
            }
            Value::Duration(d) => Ok(format!(
                "{}秒 {}微秒 {}月",
                d.seconds, d.microseconds, d.months
            )),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为字符串", self))),
        }
    }

    /// 转换为列表
    pub fn to_list(&self) -> Result<Vec<Value>, ExpressionError> {
        match self {
            Value::List(list) => Ok(list.clone()),
            Value::Set(set) => Ok(set.iter().cloned().collect()),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为列表", self))),
        }
    }

    /// 转换为映射
    pub fn to_map(&self) -> Result<std::collections::HashMap<String, Value>, ExpressionError> {
        match self {
            Value::Map(map) => Ok(map.clone()),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为映射", self))),
        }
    }

    /// 转换为集合
    pub fn to_set(&self) -> Result<std::collections::HashSet<Value>, ExpressionError> {
        match self {
            Value::Set(set) => Ok(set.clone()),
            Value::List(list) => Ok(list.iter().cloned().collect()),
            _ => Err(ExpressionError::type_error(format!("无法将 {:?} 转换为集合", self))),
        }
    }

    /// 检查是否可以转换为指定类型
    pub fn can_convert_to(&self, target_type: &str) -> bool {
        match target_type {
            "bool" => self.to_bool().is_ok(),
            "int" => self.to_int().is_ok(),
            "float" => self.to_float().is_ok(),
            "string" => self.to_string().is_ok(),
            "list" => self.to_list().is_ok(),
            "map" => self.to_map().is_ok(),
            "set" => self.to_set().is_ok(),
            _ => false,
        }
    }

    /// 检查值是否为有效的数字
    pub fn is_valid_number(&self) -> bool {
        match self {
            Value::Int(_) => true,
            Value::Float(f) => !f.is_nan() && !f.is_infinite(),
            _ => false,
        }
    }

    /// 检查值是否为有效的日期
    pub fn is_valid_date(&self) -> bool {
        match self {
            Value::Date(d) => {
                d.year >= 0 && d.year <= 9999 && d.month >= 1 && d.month <= 12 && d.day >= 1 && d.day <= 31
            }
            _ => false,
        }
    }

    /// 检查值是否为有效的时间
    pub fn is_valid_time(&self) -> bool {
        match self {
            Value::Time(t) => {
                t.hour >= 0 && t.hour <= 23 && t.minute >= 0 && t.minute <= 59 &&
                t.sec >= 0 && t.sec <= 59 && t.microsec >= 0 && t.microsec <= 999999
            }
            _ => false,
        }
    }

    /// 检查值是否为有效的日期时间
    pub fn is_valid_datetime(&self) -> bool {
        match self {
            Value::DateTime(dt) => {
                dt.year >= 0 && dt.year <= 9999 && dt.month >= 1 && dt.month <= 12 && dt.day >= 1 && dt.day <= 31 &&
                dt.hour >= 0 && dt.hour <= 23 && dt.minute >= 0 && dt.minute <= 59 &&
                dt.sec >= 0 && dt.sec <= 59 && dt.microsec >= 0 && dt.microsec <= 999999
            }
            _ => false,
        }
    }
}
