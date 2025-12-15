# StorageExpressionContext 改进方案

## 概述

本文档提供了对当前简化版 `StorageExpressionContext` 实现的详细改进方案，对照 nebula-graph 的原始实现，提出具体的修改建议和代码示例。

## 1. RowReaderWrapper 改进方案

### 当前问题
- 使用简单的固定长度字段偏移量计算（每个字段8字节）
- 缺乏对实际字段类型的支持
- 没有处理默认值和可空字段的逻辑
- 缺乏Schema字段定义的支持

### 改进方案

#### 1.1 增强字段定义结构

```rust
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

/// Schema定义
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, FieldDef>,
    pub version: i32,
}
```

#### 1.2 改进RowReaderWrapper实现

```rust
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

#[derive(Debug, Clone)]
pub enum EncodingFormat {
    /// Nebula的默认编码格式
    Nebula,
    /// 简化的编码格式（用于测试）
    Simple,
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
                Ok(Value::Timestamp(timestamp))
            }
            FieldType::Date => {
                if data.len() < 4 {
                    return Err("数据长度不足".to_string());
                }
                let days = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Value::Date(days))
            }
            FieldType::DateTime => {
                if data.len() < 8 {
                    return Err("数据长度不足".to_string());
                }
                let timestamp = i64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                Ok(Value::DateTime(timestamp))
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
}
```

## 2. StorageExpressionContext 改进方案

### 当前问题
- 缺乏对键值解析的支持（顶点ID、边ID、排名等）
- 没有实现索引模式下的值解析
- 特殊属性（_src, _dst, _rank, _type, _vid, _tag）处理不完整
- 缺乏错误处理和默认值逻辑

### 改进方案

#### 2.1 增强键值解析工具

```rust
/// 键值解析工具
pub struct KeyUtils;

impl KeyUtils {
    /// 从顶点键中提取顶点ID
    pub fn get_vertex_id(v_id_len: usize, key: &str) -> Result<String, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len {
            return Err("键长度不足".to_string());
        }
        
        // 假设顶点ID在键的固定位置
        let id_bytes = &key.as_bytes()[..v_id_len];
        let id_str = String::from_utf8(id_bytes.to_vec())
            .map_err(|e| format!("顶点ID解析失败: {}", e))?;
        
        // 移除null字符
        Ok(id_str.trim_end_matches('\0').to_string())
    }

    /// 从边键中提取源顶点ID
    pub fn get_src_id(v_id_len: usize, key: &str) -> Result<String, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len * 2 {
            return Err("键长度不足".to_string());
        }
        
        let src_bytes = &key.as_bytes()[..v_id_len];
        let src_str = String::from_utf8(src_bytes.to_vec())
            .map_err(|e| format!("源顶点ID解析失败: {}", e))?;
        
        Ok(src_str.trim_end_matches('\0').to_string())
    }

    /// 从边键中提取目标顶点ID
    pub fn get_dst_id(v_id_len: usize, key: &str) -> Result<String, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len * 2 {
            return Err("键长度不足".to_string());
        }
        
        let dst_bytes = &key.as_bytes()[v_id_len..v_id_len * 2];
        let dst_str = String::from_utf8(dst_bytes.to_vec())
            .map_err(|e| format!("目标顶点ID解析失败: {}", e))?;
        
        Ok(dst_str.trim_end_matches('\0').to_string())
    }

    /// 从边键中提取排名
    pub fn get_rank(v_id_len: usize, key: &str) -> Result<i64, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len * 2 + 8 {
            return Err("键长度不足".to_string());
        }
        
        let rank_bytes = &key.as_bytes()[v_id_len * 2..v_id_len * 2 + 8];
        let rank = i64::from_be_bytes([
            rank_bytes[0], rank_bytes[1], rank_bytes[2], rank_bytes[3],
            rank_bytes[4], rank_bytes[5], rank_bytes[6], rank_bytes[7],
        ]);
        
        Ok(rank)
    }

    /// 从边键中提取边类型
    pub fn get_edge_type(v_id_len: usize, key: &str) -> Result<i64, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len * 2 + 16 {
            return Err("键长度不足".to_string());
        }
        
        let type_bytes = &key.as_bytes()[v_id_len * 2 + 8..v_id_len * 2 + 16];
        let edge_type = i64::from_be_bytes([
            type_bytes[0], type_bytes[1], type_bytes[2], type_bytes[3],
            type_bytes[4], type_bytes[5], type_bytes[6], type_bytes[7],
        ]);
        
        Ok(edge_type)
    }

    /// 从顶点键中提取标签ID
    pub fn get_tag_id(v_id_len: usize, key: &str) -> Result<i64, String> {
        // 简化实现，实际需要根据Nebula的键格式解析
        if key.len() < v_id_len + 8 {
            return Err("键长度不足".to_string());
        }
        
        let tag_bytes = &key.as_bytes()[v_id_len..v_id_len + 8];
        let tag_id = i64::from_be_bytes([
            tag_bytes[0], tag_bytes[1], tag_bytes[2], tag_bytes[3],
            tag_bytes[4], tag_bytes[5], tag_bytes[6], tag_bytes[7],
        ]);
        
        Ok(tag_id)
    }
}
```

#### 2.2 改进StorageExpressionContext实现

```rust
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
    pub fields: Vec<FieldDef>,
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
        fields: Vec<FieldDef>,
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
    pub fn get_index_value(&self, prop: &str, is_edge: bool) -> Value {
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
    fn parse_index_value(&self, prop: &str, field_def: &FieldDef) -> Value {
        // 简化实现，实际需要根据索引键格式解析
        match field_def.field_type {
            FieldType::Int => Value::Int(0),
            FieldType::Float => Value::Float(0.0),
            FieldType::String => Value::String(format!("index_{}", prop)),
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

    fn get_versioned_var(&self, name: &str, version: i64) -> Result<Value, String> {
        if let Some(values) = self.value_map.get(name) {
            if version >= 0 && version < values.len() as i64 {
                Ok(values[values.len() - 1 - version as usize].clone())
            } else {
                Ok(Value::Null(NullType::Null))
            }
        } else {
            Ok(Value::Null(NullType::Null))
        }
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

    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String> {
        // 简化实现：不支持变量属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        // 简化实现：不支持目标顶点属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_input_prop(&self, prop: &str) -> Result<Value, String> {
        // 简化实现：不支持输入属性访问
        Ok(Value::Null(NullType::Null))
    }

    fn get_input_prop_index(&self, prop: &str) -> Result<usize, String> {
        // 简化实现：不支持输入属性索引
        Err("不支持输入属性索引".to_string())
    }

    fn get_column(&self, index: i32) -> Result<Value, String> {
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
            if let Some(ref reader) = self.reader {
                if edge_name != self.name {
                    return Ok(Value::Empty);
                }

                // 处理特殊属性
                match prop {
                    "_src" => {
                        // 从键值中提取源顶点ID
                        match KeyUtils::get_src_id(self.v_id_len, &self.key) {
                            Ok(src_id) => {
                                if self.is_int_id {
                                    if let Ok(int_id) = src_id.parse::<i64>() {
                                        Ok(Value::Int(int_id))
                                    } else {
                                        Ok(Value::String(src_id))
                                    }
                                } else {
                                    Ok(Value::String(src_id))
                                }
                            }
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
                    }
                    "_dst" => {
                        // 从键值中提取目标顶点ID
                        match KeyUtils::get_dst_id(self.v_id_len, &self.key) {
                            Ok(dst_id) => {
                                if self.is_int_id {
                                    if let Ok(int_id) = dst_id.parse::<i64>() {
                                        Ok(Value::Int(int_id))
                                    } else {
                                        Ok(Value::String(dst_id))
                                    }
                                } else {
                                    Ok(Value::String(dst_id))
                                }
                            }
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
                    }
                    "_rank" => {
                        // 从键值中提取排名
                        match KeyUtils::get_rank(self.v_id_len, &self.key) {
                            Ok(rank) => Ok(Value::Int(rank)),
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
                    }
                    "_type" => {
                        // 从键值中提取边类型
                        match KeyUtils::get_edge_type(self.v_id_len, &self.key) {
                            Ok(edge_type) => Ok(Value::Int(edge_type)),
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
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
            if let Some(ref reader) = self.reader {
                if tag_name != self.name {
                    return Ok(Value::Empty);
                }

                // 处理特殊属性
                match prop {
                    "_vid" => {
                        // 从键值中提取顶点ID
                        match KeyUtils::get_vertex_id(self.v_id_len, &self.key) {
                            Ok(vertex_id) => {
                                if self.is_int_id {
                                    if let Ok(int_id) = vertex_id.parse::<i64>() {
                                        Ok(Value::Int(int_id))
                                    } else {
                                        Ok(Value::String(vertex_id))
                                    }
                                } else {
                                    Ok(Value::String(vertex_id))
                                }
                            }
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
                    }
                    "_tag" => {
                        // 从键值中提取标签ID
                        match KeyUtils::get_tag_id(self.v_id_len, &self.key) {
                            Ok(tag_id) => Ok(Value::Int(tag_id)),
                            Err(_) => Ok(Value::Null(NullType::BadData)),
                        }
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
```

## 3. ExpressionContext trait 改进方案

### 当前问题
- 多个方法返回简化值或不支持的功能
- 缺乏版本控制变量支持
- 变量属性访问不支持
- 输入属性访问不支持
- 列索引访问不支持

### 改进方案

#### 3.1 增强ExpressionContext trait

```rust
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

    /// 检查变量是否存在
    fn has_var(&self, name: &str) -> bool {
        self.get_var(name).is_ok()
    }

    /// 获取所有变量名
    fn get_var_names(&self) -> Vec<String> {
        Vec::new() // 默认实现
    }

    /// 清空所有变量
    fn clear_vars(&mut self) {
        // 默认实现为空
    }

    /// 获取变量历史版本数量
    fn get_var_version_count(&self, name: &str) -> usize {
        // 默认实现返回0
        0
    }
}
```

## 4. 实施建议

### 4.1 分阶段实施

1. **第一阶段**：完善基础数据结构
   - 实现增强的FieldDef和Schema
   - 改进RowReaderWrapper的基础功能
   - 添加基本的错误处理

2. **第二阶段**：实现键值解析
   - 完善KeyUtils的实现
   - 实现特殊属性的解析
   - 添加对整数ID和字符串ID的支持

3. **第三阶段**：完善表达式上下文
   - 实现完整的ExpressionContext trait
   - 添加版本控制支持
   - 完善错误处理和默认值逻辑

4. **第四阶段**：性能优化
   - 优化字段偏移量计算
   - 添加缓存机制
   - 优化内存使用

### 4.2 测试策略

1. **单元测试**：为每个组件编写详细的单元测试
2. **集成测试**：测试组件之间的交互
3. **性能测试**：确保性能满足要求
4. **兼容性测试**：确保与现有代码兼容

### 4.3 文档更新

1. **API文档**：更新所有公共API的文档
2. **使用示例**：提供详细的使用示例
3. **迁移指南**：为现有用户提供迁移指南

## 5. 总结

本改进方案提供了对当前简化版 `StorageExpressionContext` 的全面升级，主要改进包括：

1. **增强的数据结构**：支持完整的字段类型定义和Schema管理
2. **完善的键值解析**：支持顶点ID、边ID、排名等特殊属性的解析
3. **完整的表达式上下文**：实现完整的ExpressionContext trait
4. **错误处理**：添加完善的错误处理和默认值逻辑
5. **性能优化**：通过缓存和优化算法提高性能

这些改进将使Rust版本的实现更加接近nebula-graph的原始实现，提供更好的功能和性能。