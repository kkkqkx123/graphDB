//! 索引二进制编码模块
//!
//! 实现 NebulaGraph 风格的索引二进制编码，支持固定长度和可变长度字段

use crate::core::Value;
use std::io::{self, Read, Write, Cursor};

/// 索引二进制编码器
///
/// 将索引列值编码为字节序列，支持固定长度和可变长度字段
#[derive(Debug, Clone)]
pub struct IndexBinaryEncoder;

impl IndexBinaryEncoder {
    /// 编码索引值列表为二进制格式
    ///
    /// # 参数
    /// - `values`: 索引列值列表
    ///
    /// # 返回
    /// 编码后的字节序列
    pub fn encode(values: &[Value]) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut var_length_positions = Vec::new();

        for value in values {
            match value {
                Value::Int(i) => {
                    buffer.extend_from_slice(&i.to_le_bytes());
                }
                Value::Float(f) => {
                    buffer.extend_from_slice(&f.to_le_bytes());
                }
                Value::Bool(b) => {
                    buffer.push(if *b { 1u8 } else { 0u8 });
                }
                Value::String(s) => {
                    var_length_positions.push(buffer.len());
                    buffer.extend_from_slice(s.as_bytes());
                }
                _ => {
                    // 其他类型暂不支持索引
                    panic!("不支持的索引类型: {:?}", value);
                }
            }
        }

        // 在末尾添加可变长度字段的长度信息
        for pos in var_length_positions {
            let length = (buffer.len() - pos) as i32;
            buffer.extend_from_slice(&length.to_le_bytes());
        }

        buffer
    }

    /// 解码二进制为索引值列表
    ///
    /// # 参数
    /// - `data`: 编码后的字节序列
    /// - `field_types`: 字段类型列表
    ///
    /// # 返回
    /// 解码后的索引值列表
    pub fn decode(data: &[u8], field_types: &[ValueType]) -> Vec<Value> {
        let mut cursor = Cursor::new(data);
        let mut values = Vec::new();
        let mut var_length_positions = Vec::new();

        for field_type in field_types {
            match field_type {
                ValueType::Int => {
                    let mut buf = [0u8; 8];
                    cursor.read_exact(&mut buf).unwrap();
                    let i = i64::from_le_bytes(buf);
                    values.push(Value::Int(i));
                }
                ValueType::Float => {
                    let mut buf = [0u8; 8];
                    cursor.read_exact(&mut buf).unwrap();
                    let f = f64::from_le_bytes(buf);
                    values.push(Value::Float(f));
                }
                ValueType::Bool => {
                    let mut buf = [0u8; 1];
                    cursor.read_exact(&mut buf).unwrap();
                    let b = buf[0] != 0;
                    values.push(Value::Bool(b));
                }
                ValueType::String => {
                    var_length_positions.push(cursor.position() as usize);
                    // 字符串长度在末尾，先跳过
                    cursor.set_position(data.len() as u64 - var_length_positions.len() as u64 * 4);
                }
            }
        }

        // 读取可变长度字段的长度并解析字符串
        let mut length_cursor = Cursor::new(&data[data.len() - var_length_positions.len() * 4..]);
        for (i, pos) in var_length_positions.iter().enumerate() {
            let mut buf = [0u8; 4];
            length_cursor.read_exact(&mut buf).unwrap();
            let length = i32::from_le_bytes(buf) as usize;

            let start = *pos;
            let end = if i < var_length_positions.len() - 1 {
                var_length_positions[i + 1]
            } else {
                data.len() - var_length_positions.len() * 4
            };

            let s = String::from_utf8_lossy(&data[start..end]).to_string();
            values.push(Value::String(s));
        }

        values
    }

    /// 比较两个索引二进制编码
    ///
    /// # 参数
    /// - `a`: 第一个索引二进制
    /// - `b`: 第二个索引二进制
    ///
    /// # 返回
    /// 比较结果
    pub fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
        a.cmp(b)
    }
}

/// 索引字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    String,
}

impl ValueType {
    /// 从 Value 类型推断 ValueType
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
            Value::String(_) => ValueType::String,
            _ => panic!("不支持的索引类型: {:?}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_int() {
        let values = vec![Value::Int(42)];
        let encoded = IndexBinaryEncoder::encode(&values);
        assert_eq!(encoded.len(), 8);
    }

    #[test]
    fn test_encode_string() {
        let values = vec![Value::String("hello".to_string())];
        let encoded = IndexBinaryEncoder::encode(&values);
        assert_eq!(encoded.len(), 5 + 4);
    }

    #[test]
    fn test_encode_multiple_fields() {
        let values = vec![
            Value::Int(23),
            Value::String("abc".to_string()),
            Value::String("here".to_string()),
        ];
        let encoded = IndexBinaryEncoder::encode(&values);
        assert_eq!(encoded.len(), 8 + 3 + 4 + 4 + 4);
    }

    #[test]
    fn test_decode_int() {
        let values = vec![Value::Int(42)];
        let encoded = IndexBinaryEncoder::encode(&values);
        let field_types = vec![ValueType::Int];
        let decoded = IndexBinaryEncoder::decode(&encoded, &field_types);
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_decode_string() {
        let values = vec![Value::String("hello".to_string())];
        let encoded = IndexBinaryEncoder::encode(&values);
        let field_types = vec![ValueType::String];
        let decoded = IndexBinaryEncoder::decode(&encoded, &field_types);
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_compare() {
        let values1 = vec![Value::Int(1)];
        let values2 = vec![Value::Int(2)];
        let encoded1 = IndexBinaryEncoder::encode(&values1);
        let encoded2 = IndexBinaryEncoder::encode(&values2);
        assert_eq!(IndexBinaryEncoder::compare(&encoded1, &encoded2), std::cmp::Ordering::Less);
    }
}
