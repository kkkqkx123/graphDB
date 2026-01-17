//! 索引键二进制编码模块
//!
//! 提供高效的索引键编码和解码功能，支持：
//! - 多种数据类型的二进制编码
//! - 复合索引键的编码
//! - 前缀查询支持
//! - 范围查询优化
//!
//! 编码格式：
//! - 固定长度类型（Int, Float, Bool）：直接使用字节表示
//! - 可变长度类型（String, Date, DateTime）：长度前缀 + 数据
//! - 复合索引：依次编码各字段

use crate::core::{DateTimeValue, DateValue, DurationValue, GeographyValue, TimeValue, Value, Vertex, Edge};
use std::cmp::Ordering;

pub const INDEX_KEY_SEPARATOR: u8 = 0xFF;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexKey {
    pub space_id: i32,
    pub index_id: i32,
    pub encoded_values: Vec<u8>,
}

impl IndexKey {
    pub fn new(space_id: i32, index_id: i32, encoded_values: Vec<u8>) -> Self {
        Self {
            space_id,
            index_id,
            encoded_values,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.encoded_values.len());
        bytes.extend_from_slice(&self.space_id.to_le_bytes());
        bytes.extend_from_slice(&self.index_id.to_le_bytes());
        bytes.extend_from_slice(&self.encoded_values);
        bytes
    }

    pub fn from_encoded(encoded: &[u8]) -> Option<Self> {
        if encoded.len() < 8 {
            return None;
        }
        let space_id = i32::from_le_bytes(encoded[0..4].try_into().ok()?);
        let index_id = i32::from_le_bytes(encoded[4..8].try_into().ok()?);
        let encoded_values = encoded[8..].to_vec();
        Some(Self {
            space_id,
            index_id,
            encoded_values,
        })
    }
}

impl PartialOrd for IndexKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IndexKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.encoded_values.cmp(&other.encoded_values)
    }
}

pub struct IndexBinaryEncoder;

impl IndexBinaryEncoder {
    pub fn encode_value(value: &Value) -> Vec<u8> {
        match value {
            Value::Int(i) => Self::encode_int64(*i),
            Value::Float(f) => Self::encode_float64(*f),
            Value::Bool(b) => Self::encode_bool(*b),
            Value::String(s) => Self::encode_string(s),
            Value::Date(d) => Self::encode_date(d),
            Value::Time(t) => Self::encode_time(t),
            Value::DateTime(dt) => Self::encode_datetime(dt),
            Value::Null(_) => Self::encode_null(),
            Value::List(l) => Self::encode_list(l),
            Value::Map(m) => Self::encode_map(&m),
            Value::Path(p) => Self::encode_string(&format!("{:?}", p)),
            Value::Edge(e) => Self::encode_string(&format!("{:?}", e)),
            Value::Vertex(v) => Self::encode_string(&format!("{:?}", v)),
            Value::Set(s) => Self::encode_set(&s),
            Value::Duration(d) => Self::encode_string(&format!("{:?}", d)),
            Value::Geography(g) => Self::encode_string(&format!("{:?}", g)),
            Value::DataSet(ds) => Self::encode_string(&format!("{:?}", ds)),
            Value::Empty => Self::encode_string("empty"),
        }
    }

    pub fn encode_int64(i: i64) -> Vec<u8> {
        i.to_le_bytes().to_vec()
    }

    pub fn decode_int64(bytes: &[u8]) -> Option<i64> {
        if bytes.len() >= 8 {
            Some(i64::from_le_bytes(bytes[0..8].try_into().ok()?))
        } else {
            None
        }
    }

    pub fn encode_float64(f: f64) -> Vec<u8> {
        f.to_le_bytes().to_vec()
    }

    pub fn decode_float64(bytes: &[u8]) -> Option<f64> {
        if bytes.len() >= 8 {
            Some(f64::from_le_bytes(bytes[0..8].try_into().ok()?))
        } else {
            None
        }
    }

    pub fn encode_bool(b: bool) -> Vec<u8> {
        vec![if b { 1u8 } else { 0u8 }]
    }

    pub fn decode_bool(bytes: &[u8]) -> Option<bool> {
        bytes.first().map(|b| *b != 0)
    }

    pub fn encode_string(s: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(s.len() + 4);
        bytes.extend_from_slice(&(s.len() as i32).to_le_bytes());
        bytes.extend_from_slice(s.as_bytes());
        bytes
    }

    pub fn decode_string(bytes: &[u8]) -> Option<String> {
        if bytes.len() >= 4 {
            let len = i32::from_le_bytes(bytes[0..4].try_into().ok()?) as usize;
            if bytes.len() >= 4 + len {
                String::from_utf8(bytes[4..4 + len].to_vec()).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn encode_date(d: &DateValue) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6);
        bytes.extend_from_slice(&d.year.to_le_bytes()[0..2]);
        bytes.push(d.month as u8);
        bytes.push(d.day as u8);
        bytes
    }

    pub fn decode_date(bytes: &[u8]) -> Option<DateValue> {
        if bytes.len() >= 4 {
            let year = i16::from_le_bytes(bytes[0..2].try_into().ok()?) as i32;
            Some(DateValue {
                year,
                month: bytes[2] as u32,
                day: bytes[3] as u32,
            })
        } else {
            None
        }
    }

    pub fn encode_time(t: &TimeValue) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.extend_from_slice(&t.hour.to_le_bytes()[0..1]);
        bytes.push(t.minute as u8);
        bytes.push(t.sec as u8);
        bytes
    }

    pub fn decode_time(bytes: &[u8]) -> Option<TimeValue> {
        if bytes.len() >= 3 {
            Some(TimeValue {
                hour: bytes[0] as u32,
                minute: bytes[1] as u32,
                sec: bytes[2] as u32,
                ..Default::default()
            })
        } else {
            None
        }
    }

    pub fn encode_datetime(dt: &DateTimeValue) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10);
        bytes.extend_from_slice(&dt.year.to_le_bytes()[0..2]);
        bytes.push(dt.month as u8);
        bytes.push(dt.day as u8);
        bytes.push(dt.hour as u8);
        bytes.push(dt.minute as u8);
        bytes.push(dt.sec as u8);
        bytes
    }

    pub fn decode_datetime(bytes: &[u8]) -> Option<DateTimeValue> {
        if bytes.len() >= 5 {
            let year = i16::from_le_bytes(bytes[0..2].try_into().ok()?) as i32;
            Some(DateTimeValue {
                year,
                month: bytes[2] as u32,
                day: bytes[3] as u32,
                hour: bytes[4] as u32,
                minute: bytes[5] as u32,
                sec: bytes[6] as u32,
                ..Default::default()
            })
        } else {
            None
        }
    }

    pub fn encode_null() -> Vec<u8> {
        vec![0xFFu8; 8]
    }

    pub fn encode_binary(data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(data.len() + 4);
        bytes.extend_from_slice(&(data.len() as i32).to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    pub fn encode_list(values: &[Value]) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(values.len() * 8);
        encoded.extend_from_slice(&(values.len() as i32).to_le_bytes());
        for value in values {
            encoded.extend_from_slice(&Self::encode_value(value));
        }
        encoded
    }

    pub fn encode_map(map: &std::collections::HashMap<String, Value>) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(map.len() * 16);
        encoded.extend_from_slice(&(map.len() as i32).to_le_bytes());
        for (key, value) in map {
            encoded.extend_from_slice(&(key.len() as i32).to_le_bytes());
            encoded.extend_from_slice(key.as_bytes());
            encoded.extend_from_slice(&Self::encode_value(value));
        }
        encoded
    }

    pub fn encode_set(values: &std::collections::HashSet<Value>) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(values.len() * 8);
        encoded.extend_from_slice(&(values.len() as i32).to_le_bytes());
        for value in values {
            encoded.extend_from_slice(&Self::encode_value(value));
        }
        encoded
    }

    pub fn encode_values(values: &[Value]) -> Vec<u8> {
        let mut encoded = Vec::new();
        for value in values {
            encoded.extend_from_slice(&Self::encode_value(value));
        }
        encoded
    }

    pub fn encode_composite_key(values: &[Value]) -> Vec<u8> {
        Self::encode_values(values)
    }

    pub fn encode_prefix(values: &[Value], prefix_len: usize) -> Vec<u8> {
        let mut encoded = Vec::new();
        for (i, value) in values.iter().enumerate() {
            if i >= prefix_len {
                break;
            }
            encoded.extend_from_slice(&Self::encode_value(value));
        }
        encoded
    }

    pub fn encode_prefix_range(prefix: &[u8]) -> (Vec<u8>, Vec<u8>) {
        if prefix.is_empty() {
            return (Vec::new(), vec![0xFFu8]);
        }
        let mut start = prefix.to_vec();
        let mut end = prefix.to_vec();
        end.push(0xFFu8);
        (start, end)
    }

    pub fn encode_range(start: &Value, end: &Value) -> (Vec<u8>, Vec<u8>) {
        let start_encoded = Self::encode_value(start);
        let mut end_encoded = Self::encode_value(end);
        end_encoded.push(0xFFu8);
        (start_encoded, end_encoded)
    }

    pub fn decode_value(bytes: &[u8], value_type: &Value) -> Option<Value> {
        match value_type {
            Value::Int(_) => Self::decode_int64(bytes).map(Value::Int),
            Value::Float(_) => Self::decode_float64(bytes).map(Value::Float),
            Value::Bool(_) => Self::decode_bool(bytes).map(Value::Bool),
            Value::String(_) => Self::decode_string(bytes).map(Value::String),
            Value::Date(_) => Self::decode_date(bytes).map(Value::Date),
            Value::Time(_) => Self::decode_time(bytes).map(Value::Time),
            Value::DateTime(_) => Self::decode_datetime(bytes).map(Value::DateTime),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{DateValue, DateTimeValue, TimeValue};

    #[test]
    fn test_encode_decode_int64() {
        let test_values = vec![0i64, 1, -1, i64::MAX, i64::MIN, 1234567890];
        for value in test_values {
            let encoded = IndexBinaryEncoder::encode_int64(value);
            let decoded = IndexBinaryEncoder::decode_int64(&encoded).expect("Failed to decode int64 in test");
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_encode_decode_float64() {
        let test_values = vec![0.0f64, 1.0, -1.0, f64::MAX, f64::MIN, 3.1415926];
        for value in test_values {
            let encoded = IndexBinaryEncoder::encode_float64(value);
            let decoded = IndexBinaryEncoder::decode_float64(&encoded).expect("Failed to decode float64 in test");
            assert!((value - decoded).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_encode_decode_bool() {
        for value in [true, false] {
            let encoded = IndexBinaryEncoder::encode_bool(value);
            let decoded = IndexBinaryEncoder::decode_bool(&encoded).expect("Failed to decode bool in test");
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_encode_decode_string() {
        let test_values = vec!["", "hello", "中文", "特殊字符!@#$%"];
        for value in test_values {
            let encoded = IndexBinaryEncoder::encode_string(value);
            let decoded = IndexBinaryEncoder::decode_string(&encoded).expect("Failed to decode string in test");
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_encode_decode_date() {
        let date = DateValue {
            year: 2024,
            month: 1,
            day: 17,
        };
        let encoded = IndexBinaryEncoder::encode_date(&date);
        let decoded = IndexBinaryEncoder::decode_date(&encoded).expect("Failed to decode date in test");
        assert_eq!(date.year, decoded.year);
        assert_eq!(date.month, decoded.month);
        assert_eq!(date.day, decoded.day);
    }

    #[test]
    fn test_encode_decode_datetime() {
        let datetime = DateTimeValue {
            year: 2024,
            month: 1,
            day: 17,
            hour: 12,
            minute: 30,
            sec: 45,
            ..Default::default()
        };
        let encoded = IndexBinaryEncoder::encode_datetime(&datetime);
        let decoded = IndexBinaryEncoder::decode_datetime(&encoded).expect("Failed to decode datetime in test");
        assert_eq!(datetime.year, decoded.year);
        assert_eq!(datetime.month, decoded.month);
        assert_eq!(datetime.day, decoded.day);
        assert_eq!(datetime.hour, decoded.hour);
        assert_eq!(datetime.minute, decoded.minute);
        assert_eq!(datetime.sec, decoded.sec);
    }

    #[test]
    fn test_encode_composite_key() {
        let values = vec![
            Value::Int(100),
            Value::String("test".to_string()),
            Value::Bool(true),
        ];
        let encoded = IndexBinaryEncoder::encode_composite_key(&values);
        assert!(!encoded.is_empty());
        assert!(encoded.len() > 8);
    }

    #[test]
    fn test_encode_prefix_range() {
        let prefix = vec![0x01, 0x02, 0x03];
        let (start, end) = IndexBinaryEncoder::encode_prefix_range(&prefix);
        assert_eq!(start, vec![0x01, 0x02, 0x03]);
        assert_eq!(end, vec![0x01, 0x02, 0x03, 0xFF]);
    }

    #[test]
    fn test_index_key_ordering() {
        let key1 = IndexKey::new(1, 1, vec![0x01, 0x02]);
        let key2 = IndexKey::new(1, 1, vec![0x01, 0x03]);
        let key3 = IndexKey::new(1, 1, vec![0x01, 0x02, 0x04]);

        assert!(key1 < key2);
        assert!(key2 > key1);
        assert!(key1 < key3);
    }

    #[test]
    fn test_index_key_encode_decode() {
        let key = IndexKey::new(1, 2, vec![0x01, 0x02, 0x03]);
        let encoded = key.encode();
        let decoded = IndexKey::from_encoded(&encoded).expect("Failed to decode index key in test");
        assert_eq!(key.space_id, decoded.space_id);
        assert_eq!(key.index_id, decoded.index_id);
        assert_eq!(key.encoded_values, decoded.encoded_values);
    }
}
