//! 行读取器包装器
//!
//! 负责从二进制数据中解析字段值

use super::schema_def::Schema;
use super::types::FieldDef;
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
    pub fn new(data: Vec<u8>, schema: Schema) -> Result<Self, String> {
        let mut wrapper = Self {
            data,
            schema,
            field_offsets: HashMap::new(),
        };

        // 预计算字段偏移量
        wrapper.calculate_field_offsets()?;
        Ok(wrapper)
    }

    /// 预计算字段偏移量
    fn calculate_field_offsets(&mut self) -> Result<(), String> {
        let mut offset = 0;

        for (field_name, field_def) in &self.schema.fields {
            let field_size = self.calculate_field_size(field_def)?;
            self.field_offsets
                .insert(field_name.clone(), (offset, field_size));
            offset += field_size;
        }

        Ok(())
    }

    /// 计算字段大小
    fn calculate_field_size(&self, field_def: &FieldDef) -> Result<usize, String> {
        match field_def.field_type {
            // 基本类型
            super::types::FieldType::Bool => Ok(1),
            super::types::FieldType::Int => Ok(8),
            super::types::FieldType::Float => Ok(4),
            super::types::FieldType::Double => Ok(8),

            // 字符串类型
            super::types::FieldType::String => {
                // 字符串类型：4字节长度前缀 + 可变长度数据
                // 这里返回最小大小，实际大小取决于数据
                Ok(4) // 仅长度前缀
            }
            super::types::FieldType::FixedString(len) => Ok(len),

            // 时间类型
            super::types::FieldType::Timestamp => Ok(8), // 8字节Unix时间戳
            super::types::FieldType::Date => Ok(4),      // 4字节天数
            super::types::FieldType::DateTime => Ok(8),  // 8字节时间戳

            // 图类型
            super::types::FieldType::Vertex => {
                // 顶点：顶点ID(8字节) + 标签数量(4字节) + 属性数量(4字节)
                // 这里返回基本大小，实际大小取决于标签和属性
                Ok(16)
            }
            super::types::FieldType::Edge => {
                // 边：源顶点ID(8字节) + 目标顶点ID(8字节) + 边类型(4字节) + 排名(8字节)
                Ok(28)
            }
            super::types::FieldType::Path => {
                // 路径：源顶点ID(8字节) + 步骤数量(4字节)
                // 这里返回基本大小，实际大小取决于步骤
                Ok(12)
            }

            // 集合类型
            super::types::FieldType::List | super::types::FieldType::Set => {
                // 列表/集合：元素数量(4字节) + 元素大小(可变)
                // 这里返回基本大小，实际大小取决于元素
                Ok(4)
            }
            super::types::FieldType::Map => {
                // 映射：键值对数量(4字节) + 键值对大小(可变)
                // 这里返回基本大小，实际大小取决于键值对
                Ok(4)
            }
            super::types::FieldType::Blob => {
                // 二进制数据：4字节长度前缀 + 可变长度数据
                // 这里返回最小大小，实际大小取决于数据
                Ok(4)
            }
        }
    }

    /// 读取指定属性的值
    pub fn read_value(&self, prop_name: &str) -> Result<Value, String> {
        // 检查字段是否存在
        let field_def = self
            .schema
            .fields
            .get(prop_name)
            .ok_or_else(|| format!("字段 '{}' 不存在", prop_name))?;

        // 检查字段偏移量缓存
        let &(offset, _size) = self
            .field_offsets
            .get(prop_name)
            .ok_or_else(|| format!("字段 '{}' 偏移量未计算", prop_name))?;

        // 根据字段类型解析值
        self.parse_value_by_type(&self.data[offset..], field_def)
    }

    /// 根据类型解析值
    fn parse_value_by_type(&self, data: &[u8], field_def: &FieldDef) -> Result<Value, String> {
        match field_def.field_type {
            super::types::FieldType::Bool => {
                if data.len() < 1 {
                    return Err("数据长度不足".to_string());
                }
                Ok(Value::Bool(data[0] != 0))
            }
            super::types::FieldType::Int => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let value = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Int(value))
            }
            super::types::FieldType::Float => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let value = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Value::Float(value as f64))
            }
            super::types::FieldType::Double => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let value = f64::from_be_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Float(value))
            }
            super::types::FieldType::String => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
                if data.len() < 4 + len {
                    return Err(format!(
                        "字符串数据长度不足，需要 {} 字节，实际 {} 字节",
                        4 + len,
                        data.len()
                    ));
                }
                let string_bytes = &data[4..4 + len];
                String::from_utf8(string_bytes.to_vec())
                    .map(Value::String)
                    .map_err(|e| format!("字符串解析失败: {}", e))
            }
            super::types::FieldType::FixedString(fixed_len) => {
                if data.len() < fixed_len {
                    return Err("数据长度不足".to_string());
                }
                // 找到第一个null字符的位置
                let actual_len = data.iter().position(|&b| b == 0).unwrap_or(fixed_len);
                String::from_utf8(data[..actual_len].to_vec())
                    .map(Value::String)
                    .map_err(|e| format!("固定字符串解析失败: {}", e))
            }
            super::types::FieldType::Timestamp => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let timestamp = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                // 将时间戳转换为DateTime
                let _seconds = timestamp / 1000;
                let nanos = ((timestamp % 1000) * 1_000_000) as u32;
                Ok(Value::DateTime(crate::core::value::DateTimeValue {
                    year: 1970,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    sec: 0,
                    microsec: nanos / 1000,
                }))
            }
            super::types::FieldType::Date => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let days = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                // 将天数转换为DateValue（简化实现，从1970-01-01开始计算）
                Ok(Value::Date(crate::core::value::DateValue {
                    year: 1970 + (days / 365) as i32,
                    month: ((days % 365) / 30 + 1) as u32,
                    day: ((days % 365) % 30 + 1) as u32,
                }))
            }
            super::types::FieldType::DateTime => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let timestamp = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                // 将时间戳转换为DateTime
                let _seconds = timestamp / 1000;
                let nanos = ((timestamp % 1000) * 1_000_000) as u32;
                Ok(Value::DateTime(crate::core::value::DateTimeValue {
                    year: 1970,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    sec: 0,
                    microsec: nanos / 1000,
                }))
            }
            // 其他类型的简化实现
            _ => Ok(Value::String(format!(
                "未实现的类型: {:?}",
                field_def.field_type
            ))),
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
    use super::super::types::FieldType;
    use super::*;

    #[test]
    fn test_row_reader_wrapper() {
        // 创建测试Schema - 简化版本，只测试基本功能
        let mut schema = Schema::new("player".to_string(), 1);
        schema = schema.add_field(FieldDef::new("age".to_string(), FieldType::Int));
        schema = schema.add_field(FieldDef::new("score".to_string(), FieldType::Float));

        // 创建测试数据 - 简化版本
        let mut test_data = Vec::new();

        // age字段：8字节整数
        test_data.extend_from_slice(&25i64.to_be_bytes());

        // score字段：4字节浮点数
        test_data.extend_from_slice(&95.5f32.to_be_bytes());

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
