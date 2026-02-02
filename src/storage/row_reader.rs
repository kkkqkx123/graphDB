//! 行读取器包装器
//!
//! 负责从二进制数据中解析字段值

use super::schema::Schema;
use super::types::FieldDef;
use crate::core::error::ExpressionError;
use crate::core::Value;
use std::collections::HashMap;

/// 行读取器包装器 - 负责从二进制数据中解析字段值
#[derive(Debug, Clone)]
pub struct RowReaderWrapper {
    /// 原始二进制数据
    pub data: Vec<u8>,
    /// Schema定义
    pub schema: Schema,
    /// 字段偏移量缓存（字段名 -> (偏移量, 长度)）
    field_offsets: HashMap<String, (usize, usize)>,
}

impl RowReaderWrapper {
    pub fn new(data: Vec<u8>, schema: Schema) -> Result<Self, ExpressionError> {
        let mut wrapper = Self {
            data,
            schema,
            field_offsets: HashMap::new(),
        };

        wrapper.calculate_field_offsets()?;
        Ok(wrapper)
    }

    fn check_length(&self, data: &[u8], required: usize, type_name: &str) -> Result<(), ExpressionError> {
        if data.len() < required {
            Err(ExpressionError::type_error(format!(
                "{} 数据长度不足，需要{}字节，实际{}字节",
                type_name, required, data.len()
            )))
        } else {
            Ok(())
        }
    }

    fn read_offset_data(&self, data: &[u8], type_name: &str) -> Result<(usize, usize), ExpressionError> {
        if data.len() < 8 {
            return Err(ExpressionError::type_error(format!(
                "{} 数据头部不足，需要8字节，实际{}字节",
                type_name, data.len()
            )));
        }
        let offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

        if self.data.len() < offset || self.data.len() < offset + len {
            return Err(ExpressionError::type_error(format!(
                "{} 数据偏移量越界：offset={}, len={}, data_len={}",
                type_name, offset, len, self.data.len()
            )));
        }

        Ok((offset, len))
    }

    fn read_fixed_data<'a>(&self, data: &'a [u8], type_name: &str, size: usize) -> Result<&'a [u8], ExpressionError> {
        self.check_length(data, size, type_name)?;
        Ok(&data[..size])
    }

    fn bytes_to_string(&self, bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }

    fn calculate_field_offsets(&mut self) -> Result<(), ExpressionError> {
        let mut offset = 0;

        for (field_name, field_def) in &self.schema.fields {
            let field_size = self.calculate_field_size(field_def)?;
            self.field_offsets
                .insert(field_name.clone(), (offset, field_size));
            offset += field_size;
        }

        Ok(())
    }

    fn calculate_field_size(&self, field_def: &FieldDef) -> Result<usize, ExpressionError> {
        match field_def.field_type {
            crate::core::DataType::Bool => Ok(1),
            crate::core::DataType::Int8 => Ok(1),
            crate::core::DataType::Int16 => Ok(2),
            crate::core::DataType::Int32 => Ok(4),
            crate::core::DataType::Int64 => Ok(8),
            crate::core::DataType::Float => Ok(4),
            crate::core::DataType::Double => Ok(8),
            crate::core::DataType::String => Ok(8),
            crate::core::DataType::FixedString(len) => Ok(len),
            crate::core::DataType::VID => Ok(8),
            crate::core::DataType::Timestamp => Ok(8),
            crate::core::DataType::Date => Ok(4),
            crate::core::DataType::Time => Ok(8),
            crate::core::DataType::DateTime => Ok(10),
            crate::core::DataType::Vertex => Ok(16),
            crate::core::DataType::Edge => Ok(32),
            crate::core::DataType::Path => Ok(24),
            crate::core::DataType::List | crate::core::DataType::Set => Ok(8),
            crate::core::DataType::Map => Ok(8),
            crate::core::DataType::Blob => Ok(8),
            crate::core::DataType::Geography => Ok(8),
            crate::core::DataType::Duration => Ok(16),
            _ => Err(ExpressionError::type_error(format!("不支持的类型: {:?}", field_def.field_type))),
        }
    }

    pub fn read_value(&self, prop_name: &str) -> Result<Value, ExpressionError> {
        let field_def = self
            .schema
            .fields
            .get(prop_name)
            .ok_or_else(|| ExpressionError::property_not_found(prop_name))?;

        let &(offset, _size) = self
            .field_offsets
            .get(prop_name)
            .ok_or_else(|| ExpressionError::runtime_error(format!("字段 '{}' 偏移量未计算", prop_name)))?;

        self.parse_value_by_type(&self.data[offset..], field_def)
    }

    fn parse_value_by_type(&self, data: &[u8], field_def: &FieldDef) -> Result<Value, ExpressionError> {
        match field_def.field_type {
            crate::core::DataType::Bool => {
                self.check_length(data, 1, "Bool")?;
                Ok(Value::Bool(data[0] != 0))
            }
            crate::core::DataType::Int8 => {
                self.check_length(data, 1, "Int8")?;
                Ok(Value::Int(data[0] as i8 as i64))
            }
            crate::core::DataType::Int16 => {
                self.check_length(data, 2, "Int16")?;
                let value = i16::from_le_bytes([data[0], data[1]]);
                Ok(Value::Int(value as i64))
            }
            crate::core::DataType::Int32 => {
                self.check_length(data, 4, "Int32")?;
                let value = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Value::Int(value as i64))
            }
            crate::core::DataType::Int64 => {
                self.check_length(data, 8, "Int64")?;
                let value = i64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Int(value))
            }
            crate::core::DataType::Float => {
                self.check_length(data, 4, "Float")?;
                let value = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Value::Float(value as f64))
            }
            crate::core::DataType::Double => {
                self.check_length(data, 8, "Double")?;
                let value = f64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Float(value))
            }
            crate::core::DataType::String => {
                let (offset, len) = self.read_offset_data(data, "String")?;
                if len == 0 {
                    return Ok(Value::String(String::new()));
                }
                let string_bytes = &self.data[offset..offset + len];
                String::from_utf8(string_bytes.to_vec())
                    .map(Value::String)
                    .map_err(|e| ExpressionError::type_error(format!("String 解析失败: {}", e)))
            }
            crate::core::DataType::FixedString(fixed_len) => {
                self.check_length(data, fixed_len, "FixedString")?;
                let actual_len = data.iter().position(|&b| b == 0).unwrap_or(fixed_len);
                String::from_utf8(data[..actual_len].to_vec())
                    .map(Value::String)
                    .map_err(|e| ExpressionError::type_error(format!("FixedString 解析失败: {}", e)))
            }
            crate::core::DataType::VID => {
                let vid_data = self.read_fixed_data(data, "VID", 8)?;
                Ok(Value::String(self.bytes_to_string(vid_data)))
            }
            crate::core::DataType::Blob => {
                let (offset, len) = self.read_offset_data(data, "Blob")?;
                if len == 0 {
                    return Ok(Value::String(String::new()));
                }
                let blob_data = &self.data[offset..offset + len];
                Ok(Value::String(self.bytes_to_string(blob_data)))
            }
            crate::core::DataType::Geography => {
                let (offset, len) = self.read_offset_data(data, "Geography")?;
                if len == 0 {
                    return Ok(Value::String(String::new()));
                }
                let wkb_data = &self.data[offset..offset + len];
                Ok(Value::String(self.bytes_to_string(wkb_data)))
            }
            crate::core::DataType::Vertex => {
                let vid_data = self.read_fixed_data(data, "Vertex", 16)?;
                let vid_slice = &vid_data[..8];
                let vertex = crate::core::vertex_edge_path::Vertex {
                    vid: Box::new(Value::String(self.bytes_to_string(vid_slice))),
                    id: 0,
                    tags: std::default::Default::default(),
                    properties: std::default::Default::default(),
                };
                Ok(Value::Vertex(Box::new(vertex)))
            }
            crate::core::DataType::Edge => {
                let edge_data = self.read_fixed_data(data, "Edge", 32)?;
                let src_slice = &edge_data[..8];
                let dst_slice = &edge_data[8..16];
                let edge_type = i32::from_le_bytes([edge_data[16], edge_data[17], edge_data[18], edge_data[19]]);
                let rank = i64::from_le_bytes([
                    edge_data[20], edge_data[21], edge_data[22], edge_data[23], edge_data[24], edge_data[25], edge_data[26], edge_data[27],
                ]);
                let edge = crate::core::vertex_edge_path::Edge {
                    src: Box::new(Value::String(self.bytes_to_string(src_slice))),
                    dst: Box::new(Value::String(self.bytes_to_string(dst_slice))),
                    edge_type: format!("{}", edge_type),
                    ranking: rank,
                    id: 0,
                    props: std::default::Default::default(),
                };
                Ok(Value::Edge(edge))
            }
            crate::core::DataType::Timestamp => {
                self.check_length(data, 8, "Timestamp")?;
                let timestamp = i64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                let (year, month, day, hour, minute, second, microsec) = super::date_utils::timestamp_to_datetime(timestamp);
                Ok(Value::DateTime(crate::core::value::DateTimeValue {
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    sec: second,
                    microsec,
                }))
            }
            crate::core::DataType::Date => {
                self.check_length(data, 4, "Date")?;
                let year = i16::from_le_bytes([data[0], data[1]]);
                let month = data[2];
                let day = data[3];
                Ok(Value::Date(crate::core::value::DateValue {
                    year: year as i32,
                    month: month as u32,
                    day: day as u32,
                }))
            }
            crate::core::DataType::Time => {
                self.check_length(data, 8, "Time")?;
                let hour = data[0];
                let minute = data[1];
                let sec = data[2];
                let _padding = data[3];
                let microsec = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                Ok(Value::Time(crate::core::value::TimeValue {
                    hour: hour as u32,
                    minute: minute as u32,
                    sec: sec as u32,
                    microsec,
                }))
            }
            crate::core::DataType::Duration => {
                self.check_length(data, 16, "Duration")?;
                let seconds = i64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                let microseconds = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
                let months = i32::from_le_bytes([data[12], data[13], data[14], data[15]]);
                Ok(Value::Duration(crate::core::value::DurationValue {
                    seconds,
                    microseconds,
                    months,
                }))
            }
            crate::core::DataType::Path => {
                Err(ExpressionError::unsupported_operation(
                    "Path 类型解析",
                    "Path 类型暂不支持"
                ))
            }
            crate::core::DataType::List => {
                Err(ExpressionError::unsupported_operation(
                    "List 类型解析",
                    "List 类型暂不支持"
                ))
            }
            crate::core::DataType::Set => {
                Err(ExpressionError::unsupported_operation(
                    "Set 类型解析",
                    "Set 类型暂不支持"
                ))
            }
            crate::core::DataType::Map => {
                Err(ExpressionError::unsupported_operation(
                    "Map 类型解析",
                    "Map 类型暂不支持"
                ))
            }
            _ => Err(ExpressionError::unsupported_operation(
                format!("类型解析: {:?}", field_def.field_type),
                "请使用支持的类型"
            )),
        }
    }

    /// 获取所有可用字段名
    pub fn get_field_names(&self) -> Vec<String> {
        self.schema.fields.keys().cloned().collect()
    }

    /// 检查字段是否存在
    pub fn has_field(&self, prop_name: &str) -> bool {
        self.schema.fields.contains_key(prop_name)
    }

    /// 获取原始数据长度
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// 获取字段定义
    pub fn get_field_def(&self, prop_name: &str) -> Option<&FieldDef> {
        self.schema.fields.get(prop_name)
    }

    /// 获取Schema
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::DataType;
    use super::*;

    #[test]
    fn test_row_reader_wrapper() {
        // 创建测试Schema - 简化版本，只测试基本功能
        let mut schema = Schema::new("player".to_string(), 1);
        schema = schema.add_field(FieldDef::new("age".to_string(), DataType::Int64));
        schema = schema.add_field(FieldDef::new("score".to_string(), DataType::Double));

        // 创建测试数据 - 简化版本
        let mut test_data = Vec::new();

        // age字段：8字节整数
        test_data.extend_from_slice(&25i64.to_le_bytes());

        // score字段：4字节浮点数
        test_data.extend_from_slice(&95.5f32.to_le_bytes());

        // 创建RowReaderWrapper
        let reader = RowReaderWrapper::new(test_data, schema)
            .expect("RowReaderWrapper creation should succeed with valid data and schema");

        // 测试字段存在性检查
        assert!(reader.has_field("age"));
        assert!(reader.has_field("score"));
        assert!(!reader.has_field("nonexistent"));

        // 测试获取字段名
        let field_names = reader.get_field_names();
        assert!(field_names.contains(&"age".to_string()));
        assert!(field_names.contains(&"score".to_string()));

        // 测试数据长度
        assert_eq!(reader.data_len(), 12); // 8+4 = 12字节

        // 测试读取值
        let age_value = reader
            .read_value("age")
            .expect("Reading 'age' field should succeed");
        assert_eq!(age_value, Value::Int(25));

        let score_value = reader
            .read_value("score")
            .expect("Reading 'score' field should succeed");
        assert_eq!(score_value, Value::Float(95.5));
    }
}
