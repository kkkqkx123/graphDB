use super::types::{NullType, Value};

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
    pub fn to_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Int(i) => Ok(*i != 0),
            Value::Float(f) => Ok(*f != 0.0),
            Value::String(s) => {
                if s.eq_ignore_ascii_case("true") {
                    Ok(true)
                } else if s.eq_ignore_ascii_case("false") {
                    Ok(false)
                } else {
                    Err(format!("无法将字符串 '{}' 转换为布尔值", s))
                }
            }
            _ => Err(format!("无法将 {:?} 转换为布尔值", self)),
        }
    }

    /// 转换为整数
    pub fn to_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            Value::String(s) => s
                .parse::<i64>()
                .map_err(|e| format!("无法将字符串 '{}' 转换为整数: {}", s, e)),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(format!("无法将 {:?} 转换为整数", self)),
        }
    }

    /// 转换为浮点数
    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|e| format!("无法将字符串 '{}' 转换为浮点数: {}", s, e)),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(format!("无法将 {:?} 转换为浮点数", self)),
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> Result<String, String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Int(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null(n) => Ok(format!("{:?}", n)),
            Value::Empty => Ok("Empty".to_string()),
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
            _ => Err(format!("无法将 {:?} 转换为字符串", self)),
        }
    }

    /// 尝试转换为列表
    pub fn to_list(&self) -> Result<Vec<Value>, String> {
        match self {
            Value::List(list) => Ok(list.clone()),
            Value::Set(set) => Ok(set.iter().cloned().collect()),
            _ => Err(format!("无法将 {:?} 转换为列表", self)),
        }
    }

    /// 尝试转换为映射
    pub fn to_map(&self) -> Result<std::collections::HashMap<String, Value>, String> {
        match self {
            Value::Map(map) => Ok(map.clone()),
            _ => Err(format!("无法将 {:?} 转换为映射", self)),
        }
    }

    /// 尝试转换为集合
    pub fn to_set(&self) -> Result<std::collections::HashSet<Value>, String> {
        match self {
            Value::Set(set) => Ok(set.clone()),
            Value::List(list) => Ok(list.iter().cloned().collect()),
            _ => Err(format!("无法将 {:?} 转换为集合", self)),
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
}
