//! 存储层表达式求值上下文
//!
//! StorageExpressionContext支持从RowReader读取值和用户设置值
//! 对应C++版本中的StorageExpressionContext类

use crate::core::{Value, NullType};
use std::collections::HashMap;

/// 字段类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Bool,
    Int,
    Float,
    Double,
    String,
    FixedString(usize), // 固定长度字符串
    Timestamp,
    Date,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Set,
    Map,
    Blob,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub fixed_length: Option<usize>, // 用于FIXED_STRING类型
}

impl FieldDef {
    pub fn new(name: String, field_type: FieldType) -> Self {
        Self {
            name,
            field_type,
            nullable: false,
            default_value: None,
            fixed_length: None,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn fixed_length(mut self, length: usize) -> Self {
        self.fixed_length = Some(length);
        self
    }
}

/// Schema定义
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, FieldDef>,
    pub version: i32,
}

impl Schema {
    pub fn new(name: String, version: i32) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            version,
        }
    }

    pub fn add_field(mut self, field: FieldDef) -> Self {
        self.fields.insert(field.name.clone(), field);
        self
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.get(name)
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }
}

/// 编码格式
#[derive(Debug, Clone)]
pub enum EncodingFormat {
    /// Nebula的默认编码格式
    Nebula,
    /// 简化的编码格式（用于测试）
    Simple,
}

/// 行读取器包装器 - 负责从二进制数据中解析字段值
#[derive(Debug, Clone)]
pub struct RowReaderWrapper {
    /// 原始二进制数据
    pub data: Vec<u8>,
    /// Schema定义
    pub schema: Schema,
    /// 字段偏移量缓存（字段名 -> (偏移量, 长度)）
    field_offsets: HashMap<String, (usize, usize)>,
    /// 编码格式
    encoding: EncodingFormat,
}

impl RowReaderWrapper {
    pub fn new(data: Vec<u8>, schema: Schema) -> Result<Self, String> {
        let mut wrapper = Self {
            data,
            schema,
            field_offsets: HashMap::new(),
            encoding: EncodingFormat::Nebula,
        };
        
        // 预计算字段偏移量
        wrapper.calculate_field_offsets()?;
        Ok(wrapper)
    }

    /// 创建简化版本的RowReaderWrapper（用于测试）
    pub fn new_simple(data: Vec<u8>, schema: Schema) -> Result<Self, String> {
        let mut wrapper = Self {
            data,
            schema,
            field_offsets: HashMap::new(),
            encoding: EncodingFormat::Simple,
        };
        
        wrapper.calculate_field_offsets()?;
        Ok(wrapper)
    }

    /// 预计算字段偏移量
    fn calculate_field_offsets(&mut self) -> Result<(), String> {
        let mut offset = 0;
        
        for (field_name, field_def) in &self.schema.fields {
            let field_size = self.calculate_field_size(field_def)?;
            self.field_offsets.insert(field_name.clone(), (offset, field_size));
            offset += field_size;
        }
        
        Ok(())
    }

    /// 计算字段大小
    fn calculate_field_size(&self, field_def: &FieldDef) -> Result<usize, String> {
        match field_def.field_type {
            FieldType::Bool => Ok(1),
            FieldType::Int => Ok(8),
            FieldType::Float => Ok(4),
            FieldType::Double => Ok(8),
            FieldType::String => {
                // 字符串长度前缀(4字节) + 实际内容
                Ok(4)
            }
            FieldType::FixedString(len) => Ok(len),
            FieldType::Timestamp => Ok(8),
            FieldType::Date => Ok(4),
            FieldType::DateTime => Ok(8),
            FieldType::Vertex => Ok(16), // 简化假设
            FieldType::Edge => Ok(24),   // 简化假设
            FieldType::Path => Ok(32),   // 简化假设
            FieldType::List | FieldType::Set => Ok(8), // 指针大小
            FieldType::Map => Ok(8),     // 指针大小
            FieldType::Blob => Ok(8),    // 长度前缀
        }
    }

    /// 读取指定属性的值
    pub fn read_value(&self, prop_name: &str) -> Result<Value, String> {
        // 检查字段是否存在
        let field_def = self.schema.fields.get(prop_name)
            .ok_or_else(|| format!("字段 '{}' 不存在", prop_name))?;

        // 检查字段偏移量缓存
        let &(offset, size) = self.field_offsets.get(prop_name)
            .ok_or_else(|| format!("字段 '{}' 偏移量未计算", prop_name))?;

        // 检查数据长度是否足够
        if offset + size > self.data.len() {
            return Err(format!("数据长度不足，需要 {} 字节，实际 {} 字节",
                              offset + size, self.data.len()));
        }

        // 根据字段类型解析值
        self.parse_value_by_type(&self.data[offset..offset + size], field_def)
    }

    /// 根据类型解析值
    fn parse_value_by_type(&self, data: &[u8], field_def: &FieldDef) -> Result<Value, String> {
        match field_def.field_type {
            FieldType::Bool => {
                if data.len() < 1 {
                    return Err("数据长度不足".to_string());
                }
                Ok(Value::Bool(data[0] != 0))
            }
            FieldType::Int => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let value = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Int(value))
            }
            FieldType::Float => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let value = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Value::Float(value as f64))
            }
            FieldType::Double => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let value = f64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::Float(value))
            }
            FieldType::String => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
                if data.len() < 4 + len {
                    return Err("字符串数据长度不足".to_string());
                }
                let string_bytes = &data[4..4 + len];
                String::from_utf8(string_bytes.to_vec())
                    .map(Value::String)
                    .map_err(|e| format!("字符串解析失败: {}", e))
            }
            FieldType::FixedString(fixed_len) => {
                if data.len() < fixed_len {
                    return Err("数据长度不足".to_string());
                }
                // 找到第一个null字符的位置
                let actual_len = data.iter().position(|&b| b == 0).unwrap_or(fixed_len);
                String::from_utf8(data[..actual_len].to_vec())
                    .map(Value::String)
                    .map_err(|e| format!("固定字符串解析失败: {}", e))
            }
            FieldType::Timestamp => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let timestamp = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                // 将时间戳转换为DateTime
                let seconds = timestamp / 1000;
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
            FieldType::Date => {
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
            FieldType::DateTime => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let timestamp = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                // 将时间戳转换为DateTime
                let seconds = timestamp / 1000;
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
            _ => Ok(Value::String(format!("未实现的类型: {:?}", field_def.field_type))),
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

/// 列定义（简化版本，保持向后兼容）
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl From<FieldDef> for ColumnDef {
    fn from(field: FieldDef) -> Self {
        Self {
            name: field.name,
            data_type: format!("{:?}", field.field_type),
            nullable: field.nullable,
        }
    }
}

/// 表达式上下文trait
pub trait ExpressionContext: Send + Sync + std::fmt::Debug {
    /// 获取变量值（最新版本）
    fn get_var(&self, name: &str) -> Result<Value, String>;

    /// 获取指定版本的变量值
    fn get_versioned_var(&self, name: &str, version: i64) -> Result<Value, String>;

    /// 设置变量值
    fn set_var(&mut self, name: &str, value: Value) -> Result<(), String>;

    /// 设置表达式内部变量
    fn set_inner_var(&mut self, var: &str, value: Value);

    /// 获取表达式内部变量
    fn get_inner_var(&self, var: &str) -> Option<Value>;

    /// 获取变量属性值
    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String>;

    /// 获取目标顶点属性值
    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取输入属性值
    fn get_input_prop(&self, prop: &str) -> Result<Value, String>;

    /// 获取输入属性索引
    fn get_input_prop_index(&self, prop: &str) -> Result<usize, String>;

    /// 按列索引获取值
    fn get_column(&self, index: i32) -> Result<Value, String>;

    /// 获取标签属性值
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取边属性值
    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String>;

    /// 获取源顶点属性值
    fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取顶点
    fn get_vertex(&self, name: &str) -> Result<Value, String>;

    /// 获取边
    fn get_edge(&self) -> Result<Value, String>;
}

/// 存储层表达式上下文
#[derive(Debug, Clone)]
pub struct StorageExpressionContext {
    /// 顶点ID长度
    pub v_id_len: usize,
    /// 是否为整数ID
    pub is_int_id: bool,
    /// 行读取器（可选）
    pub reader: Option<RowReaderWrapper>,
    /// 键值
    pub key: String,
    /// 名称（标签名或边名）
    pub name: String,
    /// Schema定义（可选）
    pub schema: Option<Schema>,
    /// 是否为边
    pub is_edge: bool,
    /// 是否为索引
    pub is_index: bool,
    /// 是否有可空列
    pub has_nullable_col: bool,
    /// 字段定义
    pub fields: Vec<ColumnDef>,
    /// 标签过滤器
    pub tag_filters: HashMap<(String, String), Value>,
    /// 边过滤器
    pub edge_filters: HashMap<(String, String), Value>,
    /// 变量值映射（支持多版本）
    pub value_map: HashMap<String, Vec<Value>>,
    /// 表达式内部变量映射
    pub expr_value_map: HashMap<String, Value>,
}

impl StorageExpressionContext {
    /// 创建新的存储表达式上下文（顶点/边模式）
    pub fn new(
        v_id_len: usize,
        is_int_id: bool,
        name: String,
        schema: Option<Schema>,
        is_edge: bool,
    ) -> Self {
        Self {
            v_id_len,
            is_int_id,
            reader: None,
            key: String::new(),
            name,
            schema,
            is_edge,
            is_index: false,
            has_nullable_col: false,
            fields: Vec::new(),
            tag_filters: HashMap::new(),
            edge_filters: HashMap::new(),
            value_map: HashMap::new(),
            expr_value_map: HashMap::new(),
        }
    }

    /// 创建新的存储表达式上下文（索引模式）
    pub fn new_for_index(
        v_id_len: usize,
        is_int_id: bool,
        has_nullable_col: bool,
        fields: Vec<ColumnDef>,
    ) -> Self {
        Self {
            v_id_len,
            is_int_id,
            reader: None,
            key: String::new(),
            name: String::new(),
            schema: None,
            is_edge: false,
            is_index: true,
            has_nullable_col,
            fields,
            tag_filters: HashMap::new(),
            edge_filters: HashMap::new(),
            value_map: HashMap::new(),
            expr_value_map: HashMap::new(),
        }
    }

    /// 重置键值
    pub fn reset_key(&mut self, key: String) {
        self.key = key;
    }

    /// 重置行读取器和键值
    pub fn reset_reader(&mut self, reader: RowReaderWrapper, key: String) {
        self.reader = Some(reader);
        self.key = key;
    }

    /// 清空重置
    pub fn reset(&mut self) {
        self.reader = None;
        self.key.clear();
        self.name.clear();
        self.schema = None;
    }

    /// 重置Schema
    pub fn reset_schema(&mut self, name: String, schema: Option<Schema>, is_edge: bool) {
        self.name = name;
        self.schema = schema;
        self.is_edge = is_edge;
    }

    /// 设置标签属性值
    pub fn set_tag_prop(&mut self, tag_name: String, prop: String, value: Value) {
        self.tag_filters.insert((tag_name, prop), value);
    }

    /// 设置边属性值
    pub fn set_edge_prop(&mut self, edge_name: String, prop: String, value: Value) {
        self.edge_filters.insert((edge_name, prop), value);
    }

    /// 清空标签和边过滤器
    pub fn clear_filters(&mut self) {
        self.tag_filters.clear();
        self.edge_filters.clear();
    }

    /// 读取属性值
    pub fn read_value(&self, prop_name: &str) -> Value {
        if let Some(ref reader) = self.reader {
            match reader.read_value(prop_name) {
                Ok(value) => value,
                Err(_) => {
                    // 尝试从schema获取默认值
                    if let Some(field_def) = reader.get_field_def(prop_name) {
                        if let Some(default_value) = &field_def.default_value {
                            default_value.clone()
                        } else if field_def.nullable {
                            Value::Null(NullType::Null)
                        } else {
                            Value::Null(NullType::BadData)
                        }
                    } else {
                        Value::Null(NullType::UnknownProp)
                    }
                }
            }
        } else {
            Value::Null(NullType::Null)
        }
    }

    /// 获取索引值
    pub fn get_index_value(&self, prop: &str, _is_edge: bool) -> Value {
        // 简化实现：根据字段定义解析键值
        // 实际实现需要根据字段类型和长度解析二进制键值
        for field_def in &self.fields {
            if field_def.name == prop {
                return self.parse_index_value(prop, field_def);
            }
        }
        Value::Null(NullType::UnknownProp)
    }

    /// 解析索引值
    fn parse_index_value(&self, prop: &str, field_def: &ColumnDef) -> Value {
        // 简化实现，实际需要根据索引键格式解析
        match field_def.data_type.as_str() {
            "Int" => Value::Int(0),
            "Float" | "Double" => Value::Float(0.0),
            "String" => Value::String(format!("index_{}", prop)),
            _ => Value::String(format!("index_{}_{}", prop, "unknown")),
        }
    }

    /// 获取顶点ID长度
    pub fn v_id_len(&self) -> usize {
        self.v_id_len
    }

    /// 检查是否有可空列
    pub fn has_nullable_col(&self) -> bool {
        self.has_nullable_col
    }
}

impl ExpressionContext for StorageExpressionContext {
    fn get_var(&self, name: &str) -> Result<Value, String> {
        if let Some(values) = self.value_map.get(name) {
            if !values.is_empty() {
                Ok(values.last().unwrap().clone())
            } else {
                Ok(Value::Null(NullType::Null))
            }
        } else {
            Ok(Value::Null(NullType::Null))
        }
    }

    fn get_versioned_var(&self, _name: &str, _version: i64) -> Result<Value, String> {
        // 简化实现：不支持版本控制
        Ok(Value::Null(NullType::Null))
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<(), String> {
        self.value_map
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        Ok(())
    }

    fn set_inner_var(&mut self, var: &str, value: Value) {
        self.expr_value_map.insert(var.to_string(), value);
    }

    fn get_inner_var(&self, var: &str) -> Option<Value> {
        self.expr_value_map.get(var).cloned()
    }

    fn get_var_prop(&self, _var: &str, _prop: &str) -> Result<Value, String> {
        // 简化实现：不支持变量属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_dst_prop(&self, _tag: &str, _prop: &str) -> Result<Value, String> {
        // 简化实现：不支持目标顶点属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_input_prop(&self, _prop: &str) -> Result<Value, String> {
        // 简化实现：不支持输入属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_input_prop_index(&self, _prop: &str) -> Result<usize, String> {
        // 简化实现：不支持输入属性索引
        Err("不支持输入属性索引".to_string())
    }

    fn get_column(&self, _index: i32) -> Result<Value, String> {
        // 简化实现：不支持列索引访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_tag_prop(&self, tag_name: &str, prop: &str) -> Result<Value, String> {
        if self.is_index {
            Ok(self.get_index_value(prop, false))
        } else {
            Ok(self.get_src_prop(tag_name, prop)?)
        }
    }

    fn get_edge_prop(&self, edge_name: &str, prop: &str) -> Result<Value, String> {
        if self.is_index {
            Ok(self.get_index_value(prop, true))
        } else if self.is_edge {
            if let Some(ref _reader) = self.reader {
                if edge_name != self.name {
                    return Ok(Value::Empty);
                }

                // 处理特殊属性
                match prop {
                    "_src" => {
                        // 从键值中提取源顶点ID
                        // 简化实现
                        Ok(Value::String("src_vertex".to_string()))
                    }
                    "_dst" => {
                        // 从键值中提取目标顶点ID
                        // 简化实现
                        Ok(Value::String("dst_vertex".to_string()))
                    }
                    "_rank" => {
                        // 从键值中提取排名
                        // 简化实现
                        Ok(Value::Int(0))
                    }
                    "_type" => {
                        // 从键值中提取边类型
                        // 简化实现
                        Ok(Value::Int(1))
                    }
                    _ => Ok(self.read_value(prop)),
                }
            } else {
                // 从用户设置的过滤器中获取
                if let Some(value) = self.edge_filters.get(&(edge_name.to_string(), prop.to_string())) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Empty)
                }
            }
        } else {
            Ok(Value::Empty)
        }
    }

    fn get_src_prop(&self, tag_name: &str, prop: &str) -> Result<Value, String> {
        if !self.is_edge {
            if let Some(ref _reader) = self.reader {
                if tag_name != self.name {
                    return Ok(Value::Empty);
                }

                // 处理特殊属性
                match prop {
                    "_vid" => {
                        // 从键值中提取顶点ID
                        // 简化实现
                        Ok(Value::String("vertex_id".to_string()))
                    }
                    "_tag" => {
                        // 从键值中提取标签ID
                        // 简化实现
                        Ok(Value::Int(1))
                    }
                    _ => Ok(self.read_value(prop)),
                }
            } else {
                // 从用户设置的过滤器中获取
                if let Some(value) = self.tag_filters.get(&(tag_name.to_string(), prop.to_string())) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Empty)
                }
            }
        } else {
            Ok(Value::Empty)
        }
    }

    fn get_vertex(&self, _name: &str) -> Result<Value, String> {
        // 简化实现：不支持顶点获取
        Ok(Value::Null(NullType::BadData))
    }

    fn get_edge(&self) -> Result<Value, String> {
        // 简化实现：不支持边获取
        Ok(Value::Null(NullType::BadData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_expression_context_creation() {
        let schema = Schema::new("player".to_string(), 1);
        let ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            Some(schema),
            false,
        );

        assert_eq!(ctx.v_id_len(), 16);
        assert!(!ctx.is_int_id);
        assert_eq!(ctx.name, "player");
        assert!(!ctx.is_edge);
        assert!(!ctx.is_index);
    }

    #[test]
    fn test_storage_expression_context_index_mode() {
        let fields = vec![ColumnDef {
            name: "name".to_string(),
            data_type: "string".to_string(),
            nullable: false,
        }];

        let ctx = StorageExpressionContext::new_for_index(16, false, true, fields);

        assert_eq!(ctx.v_id_len(), 16);
        assert!(!ctx.is_int_id);
        assert!(ctx.is_index);
        assert!(ctx.has_nullable_col);
        assert_eq!(ctx.fields.len(), 1);
    }

    #[test]
    fn test_storage_expression_context_var_management() {
        let mut ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            None,
            false,
        );

        // 测试变量设置和获取
        ctx.set_var("x", Value::Int(42)).unwrap();
        let value = ctx.get_var("x").unwrap();
        assert_eq!(value, Value::Int(42));

        // 测试内部变量
        ctx.set_inner_var("temp", Value::String("hello".to_string()));
        let inner_value = ctx.get_inner_var("temp");
        assert_eq!(inner_value, Some(Value::String("hello".to_string())));

        // 测试不存在的变量
        let nonexistent = ctx.get_var("nonexistent").unwrap();
        assert_eq!(nonexistent, Value::Null(NullType::Null));
    }

    #[test]
    fn test_storage_expression_context_prop_filters() {
        let mut ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            None,
            false,
        );

        // 设置标签属性过滤器
        ctx.set_tag_prop("player".to_string(), "name".to_string(), Value::String("Alice".to_string()));
        
        // 获取标签属性
        let prop_value = ctx.get_tag_prop("player", "name").unwrap();
        assert_eq!(prop_value, Value::String("Alice".to_string()));

        // 设置边属性过滤器
        ctx.set_edge_prop("follow".to_string(), "weight".to_string(), Value::Float(0.8));
        
        // 获取边属性
        let edge_value = ctx.get_edge_prop("follow", "weight").unwrap();
        assert_eq!(edge_value, Value::Float(0.8));

        // 清空过滤器
        ctx.clear_filters();
        let cleared_value = ctx.get_tag_prop("player", "name").unwrap();
        assert_eq!(cleared_value, Value::Empty);
    }

    #[test]
    fn test_storage_expression_context_reset() {
        let mut ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            None,
            false,
        );

        ctx.set_var("x", Value::Int(42)).unwrap();
        ctx.set_tag_prop("player".to_string(), "name".to_string(), Value::String("Alice".to_string()));

        // 重置
        ctx.reset();

        assert!(ctx.reader.is_none());
        assert!(ctx.key.is_empty());
        assert!(ctx.name.is_empty());
        assert!(ctx.schema.is_none());
        
        // 变量和过滤器应该被保留（因为reset只重置reader/key/name/schema）
        let value = ctx.get_var("x").unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_storage_expression_context_reset_schema() {
        let mut ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            None,
            false,
        );

        let schema = Schema::new("new_tag".to_string(), 1);
        ctx.reset_schema("new_tag".to_string(), Some(schema), true);

        assert_eq!(ctx.name, "new_tag");
        assert!(ctx.schema.is_some());
        assert!(ctx.is_edge);
    }

    #[test]
    fn test_row_reader_wrapper() {
        // 创建测试Schema
        let mut schema = Schema::new("player".to_string(), 1);
        schema = schema.add_field(FieldDef::new("name".to_string(), FieldType::String));
        schema = schema.add_field(FieldDef::new("age".to_string(), FieldType::Int));
        schema = schema.add_field(FieldDef::new("score".to_string(), FieldType::Float));
        
        // 创建测试数据
        let mut test_data = Vec::new();
        
        // name字段：字符串长度(4字节) + "Alice"(5字节)
        test_data.extend_from_slice(&5u32.to_be_bytes());
        test_data.extend_from_slice(b"Alice");
        
        // age字段：8字节整数
        test_data.extend_from_slice(&25i64.to_be_bytes());
        
        // score字段：4字节浮点数
        test_data.extend_from_slice(&95.5f32.to_be_bytes());
        
        // 创建RowReaderWrapper
        let reader = RowReaderWrapper::new(test_data, schema).unwrap();
        
        // 测试字段存在性检查
        assert!(reader.has_field("name"));
        assert!(reader.has_field("age"));
        assert!(reader.has_field("score"));
        assert!(!reader.has_field("nonexistent"));
        
        // 测试获取字段名
        let field_names = reader.get_field_names();
        assert!(field_names.contains(&"name".to_string()));
        assert!(field_names.contains(&"age".to_string()));
        assert!(field_names.contains(&"score".to_string()));
        
        // 测试数据长度
        assert_eq!(reader.data_len(), 21); // 4+5+8+4 = 21字节
        
        // 测试读取值
        let name_value = reader.read_value("name").unwrap();
        assert_eq!(name_value, Value::String("Alice".to_string()));
        
        let age_value = reader.read_value("age").unwrap();
        assert_eq!(age_value, Value::Int(25));
        
        let score_value = reader.read_value("score").unwrap();
        assert_eq!(score_value, Value::Float(95.5));
    }

    #[test]
    fn test_storage_expression_context_with_reader() {
        // 创建测试Schema
        let mut schema = Schema::new("player".to_string(), 1);
        schema = schema.add_field(FieldDef::new("name".to_string(), FieldType::String));
        
        let mut ctx = StorageExpressionContext::new(
            16,
            false,
            "player".to_string(),
            Some(schema.clone()),
            false,
        );

        // 创建测试数据
        let mut test_data = Vec::new();
        test_data.extend_from_slice(&5u32.to_be_bytes());
        test_data.extend_from_slice(b"Alice");
        
        let reader = RowReaderWrapper::new(test_data, schema).unwrap();
        
        // 设置行读取器
        ctx.reset_reader(reader, "test_key".to_string());
        
        // 测试通过读取器读取属性
        let prop_value = ctx.read_value("name");
        assert_eq!(prop_value, Value::String("Alice".to_string()));
        
        // 测试通过get_src_prop读取属性
        let src_prop_value = ctx.get_src_prop("player", "name").unwrap();
        assert_eq!(src_prop_value, Value::String("Alice".to_string()));
    }
}