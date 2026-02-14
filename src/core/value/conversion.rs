use super::types::{NullType, Value};
use super::date_time::{DateValue, DateTimeValue, DurationValue, TimeValue};
use super::dataset::List;
use crate::core::types::DataType;
use chrono::{Datelike, Timelike};

impl Value {
    /// 转换为布尔值
    /// 
    /// 返回 Value 类型
    /// - 空值和 Null 返回 Null
    /// - 布尔值直接返回
    /// - 字符串 "true"/"false" 返回对应布尔值，其他返回 Null
    /// - 其他类型返回 BadData
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
            _ => Value::Null(NullType::BadData),
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
            _ => Value::Null(NullType::BadData),
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
            _ => Value::Null(NullType::BadData),
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
            Value::Set(set) => Value::List(List::from(set.iter().cloned().collect::<Vec<_>>())),
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 转换为映射
    pub fn to_map(&self) -> Value {
        match self {
            Value::Map(map) => Value::Map(map.clone()),
            _ => Value::Null(NullType::BadData),
        }
    }

    /// 转换为集合
    pub fn to_set(&self) -> Value {
        match self {
            Value::Set(set) => Value::Set(set.clone()),
            Value::List(list) => Value::Set(list.iter().cloned().collect()),
            _ => Value::Null(NullType::BadData),
        }
    }

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
            Value::String(s) => Self::parse_date_string(s),
            Value::Int(i) => Value::Date(Self::days_to_date(*i)),
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
            Value::String(s) => Self::parse_time_string(s),
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
            Value::String(s) => Self::parse_datetime_string(s),
            Value::Int(i) => {
                let date = Self::days_to_date(*i);
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
            Value::String(s) => Self::parse_duration_string(s),
            _ => Value::Null(NullType::BadData),
        }
    }

    fn parse_date_string(s: &str) -> Value {
        let formats = vec!["%Y-%m-%d", "%Y/%m/%d", "%Y%m%d"];

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

    fn parse_time_string(s: &str) -> Value {
        let formats = vec!["%H:%M:%S", "%H:%M:%S%.f", "%H:%M"];

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

    fn days_to_date(days: i64) -> DateValue {
        let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let date = epoch + chrono::Duration::days(days);
        DateValue {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }

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
        Value::List(List::from(value))
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

impl From<(i64, &str)> for Value {
    fn from(value: (i64, &str)) -> Self {
        Value::List(super::dataset::List::from(vec![Value::Int(value.0), Value::String(value.1.to_string())]))
    }
}

impl From<(i64, String)> for Value {
    fn from(value: (i64, String)) -> Self {
        Value::List(super::dataset::List::from(vec![Value::Int(value.0), Value::String(value.1)]))
    }
}
