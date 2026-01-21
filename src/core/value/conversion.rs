use super::types::{NullType, Value};

impl Value {
    /// 转换为布尔值
    /// 
    /// 返回 Value 类型
    /// - 空值和 Null 返回 Null
    /// - 布尔值直接返回
    /// - 字符串 "true"/"false" 返回对应布尔值，其他返回 Null
    /// - 其他类型返回 BadType
    pub fn to_bool(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => {
                let lower = s.to_lowercase();
                if lower == "true" {
                    Value::Bool(true)
                } else if lower == "false" {
                    Value::Bool(false)
                } else {
                    Value::Null(NullType::Null)
                }
            }
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 转换为整数
    /// 
    /// 参考 Nebula-Graph 设计：
    /// - 空值和 Null 返回 Null
    /// - 整数直接返回
    /// - 浮点数截断为整数，溢出时返回边界值
    /// - 字符串解析为整数，失败返回 Null
    /// - 布尔值转换为 1/0
    pub fn to_int(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Int(i) => Value::Int(*i),
            Value::Float(f) => {
                if f.is_nan() || f.is_infinite() {
                    Value::Null(NullType::Null)
                } else if *f <= i64::MIN as f64 {
                    Value::Int(i64::MIN)
                } else if *f >= i64::MAX as f64 {
                    Value::Int(i64::MAX)
                } else {
                    Value::Int(*f as i64)
                }
            }
            Value::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => Value::Int(i),
                    Err(_) => Value::Null(NullType::Null),
                }
            }
            Value::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 转换为浮点数
    /// 
    /// 参考 Nebula-Graph 设计：
    /// - 空值和 Null 返回 Null
    /// - 浮点数直接返回
    /// - 整数转换为浮点数
    /// - 字符串解析为浮点数，失败返回 Null
    pub fn to_float(&self) -> Value {
        match self {
            Value::Empty | Value::Null(_) => Value::Null(NullType::Null),
            Value::Float(f) => Value::Float(*f),
            Value::Int(i) => Value::Float(*i as f64),
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(f) => Value::Float(f),
                    Err(_) => Value::Null(NullType::Null),
                }
            }
            Value::Bool(b) => Value::Float(if *b { 1.0 } else { 0.0 }),
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> Result<String, String> {
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
            Value::Empty => Ok("EMPTY".to_string()),
            Value::Date(d) => Ok(format!("{}-{:02}-{:02}", d.year, d.month, d.day)),
            Value::Time(t) => Ok(format!(
                "{:02}:{:02}:{:02}.{:06}",
                t.hour, t.minute, t.sec, t.microsec
            )),
            Value::DateTime(dt) => Ok(format!(
                "{}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.sec, dt.microsec
            )),
            Value::Duration(d) => Ok(format!(
                "{}秒 {}微秒 {}月",
                d.seconds, d.microseconds, d.months
            )),
            Value::List(list) => {
                let items: Result<Vec<String>, _> = list
                    .iter()
                    .map(|v| v.to_string().map_err(|e| e.to_string()))
                    .collect();
                items.map(|items_str| format!("[{}]", items_str.join(", ")))
            }
            Value::Map(map) => {
                let items: Result<Vec<String>, _> = map
                    .iter()
                    .map(|(k, v)| v.to_string().map(|v_str| format!("{}: {}", k, v_str)))
                    .collect();
                items.map(|items_str| format!("{{{}}}", items_str.join(", ")))
            }
            _ => Err(format!("无法将 {:?} 转换为字符串", self)),
        }
    }

    /// 转换为列表
    pub fn to_list(&self) -> Value {
        match self {
            Value::List(list) => Value::List(list.clone()),
            Value::Set(set) => Value::List(set.iter().cloned().collect()),
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 转换为映射
    pub fn to_map(&self) -> Value {
        match self {
            Value::Map(map) => Value::Map(map.clone()),
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 转换为集合
    pub fn to_set(&self) -> Value {
        match self {
            Value::Set(set) => Value::Set(set.clone()),
            Value::List(list) => Value::Set(list.iter().cloned().collect()),
            _ => Value::Null(NullType::BadType),
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
                t.hour <= 23 && t.minute <= 59 &&
                t.sec <= 59 && t.microsec <= 999999
            }
            _ => false,
        }
    }

    /// 检查值是否为有效的日期时间
    pub fn is_valid_datetime(&self) -> bool {
        match self {
            Value::DateTime(dt) => {
                dt.year >= 0 && dt.year <= 9999 && dt.month >= 1 && dt.month <= 12 && dt.day >= 1 && dt.day <= 31 &&
                dt.hour <= 23 && dt.minute <= 59 &&
                dt.sec <= 59 && dt.microsec <= 999999
            }
            _ => false,
        }
    }
}

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
