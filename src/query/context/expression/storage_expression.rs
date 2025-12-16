//! 存储层表达式上下文
//!
//! StorageExpressionContext支持从RowReader读取值和用户设置值
//! 对应C++版本中的StorageExpressionContext类

use crate::core::{NullType, Value};
use std::collections::HashMap;
use super::schema::{Schema, RowReaderWrapper, ColumnDef, FieldType};

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
                    // 字段不存在，返回Empty
                    Value::Empty
                }
            }
        } else {
            Value::Null(NullType::Null)
        }
    }

    /// 获取索引值
    pub fn get_index_value(&self, prop: &str, _is_edge: bool) -> Value {
        // 根据字段定义解析键值
        for field_def in &self.fields {
            if field_def.name == prop {
                return self.parse_index_value(prop, field_def);
            }
        }
        Value::Null(NullType::UnknownProp)
    }

    /// 解析索引值
    fn parse_index_value(&self, prop: &str, field_def: &ColumnDef) -> Value {
        // 从二进制索引键中解析索引值
        // 索引键格式：
        // - 顶点索引：PartitionID(4) + IndexID(4) + 编码字段值 + VertexID(vIdLen) [+ 可空标志(2)]
        // - 边索引：PartitionID(4) + IndexID(4) + 编码字段值 + SrcID(vIdLen) + DstID(vIdLen) + EdgeRanking(8) [+ 可空标志(2)]

        if self.key.is_empty() {
            return self.get_default_value_for_type(&field_def.data_type);
        }

        // 将字符串键转换为字节数组
        let key_bytes = self.key.as_bytes();

        // 基本偏移量：PartitionID + IndexID
        let mut offset = 8; // 4 + 4 bytes

        // 计算尾部信息长度
        let tail_len = if self.is_edge {
            self.v_id_len * 2 + 8 // srcId + dstId + ranking
        } else {
            self.v_id_len // vertexId
        };

        // 检查可空标志位位置
        let nullable_bit_offset = if self.has_nullable_col {
            Some(key_bytes.len() - tail_len - 2)
        } else {
            None
        };

        // 解析可空标志位
        let nullable_flags = if let Some(bit_offset) = nullable_bit_offset {
            if key_bytes.len() > bit_offset + 1 {
                let flags = u16::from_be_bytes([key_bytes[bit_offset], key_bytes[bit_offset + 1]]);
                Some(flags)
            } else {
                None
            }
        } else {
            None
        };

        // 查找目标字段在索引中的位置
        let mut field_index = 0;
        let mut found = false;

        for (i, field) in self.fields.iter().enumerate() {
            if field.name == prop {
                field_index = i;
                found = true;
                break;
            }
            // 计算当前字段的长度，更新偏移量
            offset += self.get_field_encoded_size(&field.data_type);
        }

        if !found {
            return Value::Null(NullType::UnknownProp);
        }

        // 检查字段是否为空
        if let Some(flags) = nullable_flags {
            let bit_position = 15 - field_index as u16; // 从高位开始
            if (flags & (1 << bit_position)) != 0 {
                return Value::Null(NullType::Null);
            }
        }

        // 解码字段值
        self.decode_value_from_bytes(key_bytes, offset, &field_def.data_type)
            .unwrap_or_else(|_| self.get_default_value_for_type(&field_def.data_type))
    }

    /// 获取字段编码后的字节长度
    fn get_field_encoded_size(&self, data_type: &str) -> usize {
        match data_type {
            "Bool" => 1,
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Timestamp" => 8,
            "Float" => 4,
            "Double" => 8,
            "String" => {
                // 字符串长度是可变的，这里返回最小值
                // 实际实现需要从索引定义中获取固定长度
                0
            }
            "FixedString" => {
                // 需要从字段定义中获取长度
                0
            }
            "Date" => 4,      // year(2) + month(1) + day(1)
            "Time" => 8,      // hour(1) + minute(1) + second(1) + microsec(4)
            "DateTime" => 12, // year(2) + month(1) + day(1) + hour(1) + minute(1) + second(1) + microsec(4)
            "Geography" => 8, // S2CellId
            _ => 0,
        }
    }

    /// 从字节数组解码值
    fn decode_value_from_bytes(
        &self,
        bytes: &[u8],
        offset: usize,
        data_type: &str,
    ) -> Result<Value, String> {
        if bytes.len() < offset {
            return Err("字节长度不足".to_string());
        }

        match data_type {
            "Bool" => {
                if bytes.len() < offset + 1 {
                    return Err("字节长度不足".to_string());
                }
                Ok(Value::Bool(bytes[offset] != 0))
            }
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Timestamp" => {
                if bytes.len() < offset + 8 {
                    return Err("字节长度不足".to_string());
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[offset..offset + 8]);
                // NebulaGraph 使用特殊的整数编码：第一位取反
                arr[0] ^= 0x80;
                let value = i64::from_be_bytes(arr);
                Ok(Value::Int(value))
            }
            "Float" => {
                if bytes.len() < offset + 4 {
                    return Err("字节长度不足".to_string());
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&bytes[offset..offset + 4]);
                let value = f32::from_be_bytes(arr);
                Ok(Value::Float(value as f64))
            }
            "Double" => {
                if bytes.len() < offset + 8 {
                    return Err("字节长度不足".to_string());
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[offset..offset + 8]);
                // NebulaGraph 使用特殊的浮点数编码
                if arr[0] & 0x80 != 0 {
                    // 正数
                    arr[0] &= 0x7F;
                } else {
                    // 负数
                    for byte in arr.iter_mut() {
                        *byte = !*byte;
                    }
                }
                let value = f64::from_be_bytes(arr);
                Ok(Value::Float(value))
            }
            "String" => {
                // 字符串以null结尾或使用固定长度
                let end_pos = bytes[offset..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|p| offset + p)
                    .unwrap_or(bytes.len());
                let string_bytes = &bytes[offset..end_pos];
                String::from_utf8(string_bytes.to_vec())
                    .map(Value::String)
                    .map_err(|e| format!("字符串解析失败: {}", e))
            }
            "Date" => {
                if bytes.len() < offset + 4 {
                    return Err("字节长度不足".to_string());
                }
                let year = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as i32;
                let month = bytes[offset + 2];
                let day = bytes[offset + 3];
                Ok(Value::Date(crate::core::value::DateValue {
                    year,
                    month: month as u32,
                    day: day as u32,
                }))
            }
            "Time" => {
                if bytes.len() < offset + 8 {
                    return Err("字节长度不足".to_string());
                }
                let hour = bytes[offset];
                let minute = bytes[offset + 1];
                let sec = bytes[offset + 2];
                let microsec = u32::from_be_bytes([
                    bytes[offset + 3],
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                ]);
                Ok(Value::Time(crate::core::value::TimeValue {
                    hour: hour as u32,
                    minute: minute as u32,
                    sec: sec as u32,
                    microsec,
                }))
            }
            "DateTime" => {
                if bytes.len() < offset + 12 {
                    return Err("字节长度不足".to_string());
                }
                let year = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as i32;
                let month = bytes[offset + 2];
                let day = bytes[offset + 3];
                let hour = bytes[offset + 4];
                let minute = bytes[offset + 5];
                let sec = bytes[offset + 6];
                let microsec = u32::from_be_bytes([
                    bytes[offset + 7],
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                ]);
                Ok(Value::DateTime(crate::core::value::DateTimeValue {
                    year,
                    month: month as u32,
                    day: day as u32,
                    hour: hour as u32,
                    minute: minute as u32,
                    sec: sec as u32,
                    microsec,
                }))
            }
            "Geography" => {
                if bytes.len() < offset + 8 {
                    return Err("字节长度不足".to_string());
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[offset..offset + 8]);
                let s2_cell_id = u64::from_be_bytes(arr);
                // 这里简化处理，实际应该解码为地理对象
                Ok(Value::String(format!("S2CellId:{}", s2_cell_id)))
            }
            _ => Err(format!("不支持的数据类型: {}", data_type)),
        }
    }

    /// 根据类型获取默认值
    fn get_default_value_for_type(&self, data_type: &str) -> Value {
        match data_type {
            "Bool" => Value::Bool(false),
            "Int" => Value::Int(0),
            "Float" | "Double" => Value::Float(0.0),
            "String" => Value::String(String::new()),
            "Timestamp" => Value::DateTime(crate::core::value::DateTimeValue {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                sec: 0,
                microsec: 0,
            }),
            "Date" => Value::Date(crate::core::value::DateValue {
                year: 1970,
                month: 1,
                day: 1,
            }),
            "Vertex" => Value::Vertex(Box::new(crate::core::vertex_edge_path::Vertex::default())),
            "Edge" => Value::Edge(crate::core::vertex_edge_path::Edge::new_empty(
                Value::Int(0),
                Value::Int(0),
                "unknown".to_string(),
                0,
            )),
            "Path" => Value::Path(crate::core::vertex_edge_path::Path::default()),
            "List" => Value::List(Vec::new()),
            "Set" => Value::Set(std::collections::HashSet::new()),
            "Map" => Value::Map(std::collections::HashMap::new()),
            "Blob" => Value::String("blob_data".to_string()),
            _ => Value::Null(NullType::BadType),
        }
    }

    /// 从字符串解析值
    fn parse_value_from_string(&self, value_str: &str, data_type: &str) -> Value {
        match data_type {
            "Bool" => {
                if let Ok(b) = value_str.parse::<bool>() {
                    Value::Bool(b)
                } else {
                    Value::Bool(false)
                }
            }
            "Int" => {
                if let Ok(i) = value_str.parse::<i64>() {
                    Value::Int(i)
                } else {
                    Value::Int(0)
                }
            }
            "Float" | "Double" => {
                if let Ok(f) = value_str.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::Float(0.0)
                }
            }
            "String" => Value::String(value_str.to_string()),
            _ => Value::String(value_str.to_string()),
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

    /// 从值中获取属性
    fn get_property_from_value(&self, value: &Value, prop: &str) -> Result<Value, String> {
        match value {
            Value::Vertex(vertex) => {
                // 从顶点中获取属性
                if let Some(prop_value) = vertex.get_property_any(prop) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(NullType::UnknownProp))
                }
            }
            Value::Edge(edge) => {
                // 从边中获取属性
                if let Some(prop_value) = edge.get_property(prop) {
                    Ok(prop_value.clone())
                } else {
                    // 检查是否是特殊属性
                    match prop {
                        "_src" => Ok(edge.src().clone()),
                        "_dst" => Ok(edge.dst().clone()),
                        "_type" => Ok(Value::String(edge.edge_type().to_string())),
                        "_rank" => Ok(Value::Int(edge.ranking())),
                        _ => Ok(Value::Null(NullType::UnknownProp)),
                    }
                }
            }
            Value::Map(map) => {
                // 从映射中获取属性
                if let Some(prop_value) = map.get(prop) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(NullType::UnknownProp))
                }
            }
            Value::List(list) => {
                // 从列表中获取属性（假设prop是数字索引）
                if let Ok(index) = prop.parse::<usize>() {
                    if index < list.len() {
                        Ok(list[index].clone())
                    } else {
                        Ok(Value::Null(NullType::OutOfRange))
                    }
                } else {
                    Ok(Value::Null(NullType::BadType))
                }
            }
            _ => Ok(Value::Null(NullType::BadType)),
        }
    }

    /// 解析键值，提取顶点ID信息
    fn parse_key_value(&self) -> Result<(Value, Value), String> {
        if self.key.is_empty() {
            return Err("键值为空".to_string());
        }

        let key_bytes = self.key.as_bytes();

        // 检查键值长度
        if key_bytes.len() < 8 {
            return Err("键值长度不足".to_string());
        }

        // 解析PartitionID (前4字节)
        let partition_id =
            u32::from_be_bytes([key_bytes[0], key_bytes[1], key_bytes[2], key_bytes[3]]);

        // 根据是否为边模式解析不同的键值格式
        if self.is_edge {
            // 边键格式：PartitionID(4) + SrcID(vIdLen) + EdgeType(4) + Ranking(8) + DstID(vIdLen)
            let src_offset = 4;
            let edge_type_offset = src_offset + self.v_id_len;
            let ranking_offset = edge_type_offset + 4;
            let dst_offset = ranking_offset + 8;

            if key_bytes.len() < dst_offset + self.v_id_len {
                return Err("边键值长度不足".to_string());
            }

            // 解析源顶点ID
            let src_id =
                self.parse_vertex_id(&key_bytes[src_offset..src_offset + self.v_id_len])?;

            // 解析边类型
            let edge_type = i32::from_be_bytes([
                key_bytes[edge_type_offset],
                key_bytes[edge_type_offset + 1],
                key_bytes[edge_type_offset + 2],
                key_bytes[edge_type_offset + 3],
            ]);

            // 解析排名
            let ranking = i64::from_be_bytes([
                key_bytes[ranking_offset],
                key_bytes[ranking_offset + 1],
                key_bytes[ranking_offset + 2],
                key_bytes[ranking_offset + 3],
                key_bytes[ranking_offset + 4],
                key_bytes[ranking_offset + 5],
                key_bytes[ranking_offset + 6],
                key_bytes[ranking_offset + 7],
            ]);

            // 解析目标顶点ID
            let dst_id =
                self.parse_vertex_id(&key_bytes[dst_offset..dst_offset + self.v_id_len])?;

            Ok((src_id, dst_id))
        } else {
            // 顶点键格式：PartitionID(4) + VertexID(vIdLen)
            let vertex_offset = 4;

            if key_bytes.len() < vertex_offset + self.v_id_len {
                return Err("顶点键值长度不足".to_string());
            }

            // 解析顶点ID
            let vertex_id =
                self.parse_vertex_id(&key_bytes[vertex_offset..vertex_offset + self.v_id_len])?;

            Ok((vertex_id.clone(), vertex_id))
        }
    }

    /// 解析顶点ID
    fn parse_vertex_id(&self, id_bytes: &[u8]) -> Result<Value, String> {
        if self.is_int_id {
            // 整数ID：直接解析为整数
            if id_bytes.len() < 8 {
                return Err("整数ID长度不足".to_string());
            }
            let id = i64::from_be_bytes([
                id_bytes[0],
                id_bytes[1],
                id_bytes[2],
                id_bytes[3],
                id_bytes[4],
                id_bytes[5],
                id_bytes[6],
                id_bytes[7],
            ]);
            Ok(Value::Int(id))
        } else {
            // 字符串ID：解析为字符串
            let id_str = String::from_utf8(id_bytes.to_vec())
                .map_err(|e| format!("顶点ID解析失败: {}", e))?;
            Ok(Value::String(id_str))
        }
    }

    /// 解析边类型
    fn parse_edge_type(&self) -> Result<i32, String> {
        if !self.is_edge {
            return Err("非边模式，无法解析边类型".to_string());
        }

        if self.key.is_empty() {
            return Err("键值为空".to_string());
        }

        let key_bytes = self.key.as_bytes();
        let src_offset = 4;
        let edge_type_offset = src_offset + self.v_id_len;

        if key_bytes.len() < edge_type_offset + 4 {
            return Err("键值长度不足，无法解析边类型".to_string());
        }

        let edge_type = i32::from_be_bytes([
            key_bytes[edge_type_offset],
            key_bytes[edge_type_offset + 1],
            key_bytes[edge_type_offset + 2],
            key_bytes[edge_type_offset + 3],
        ]);

        Ok(edge_type)
    }

    /// 解析边排名
    fn parse_edge_ranking(&self) -> Result<i64, String> {
        if !self.is_edge {
            return Err("非边模式，无法解析边排名".to_string());
        }

        if self.key.is_empty() {
            return Err("键值为空".to_string());
        }

        let key_bytes = self.key.as_bytes();
        let src_offset = 4;
        let edge_type_offset = src_offset + self.v_id_len;
        let ranking_offset = edge_type_offset + 4;

        if key_bytes.len() < ranking_offset + 8 {
            return Err("键值长度不足，无法解析边排名".to_string());
        }

        let ranking = i64::from_be_bytes([
            key_bytes[ranking_offset],
            key_bytes[ranking_offset + 1],
            key_bytes[ranking_offset + 2],
            key_bytes[ranking_offset + 3],
            key_bytes[ranking_offset + 4],
            key_bytes[ranking_offset + 5],
            key_bytes[ranking_offset + 6],
            key_bytes[ranking_offset + 7],
        ]);

        Ok(ranking)
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
        // 获取指定版本的变量值
        if let Some(values) = self.value_map.get(name) {
            if version < 0 {
                // 负版本号表示从最新版本开始计数
                let index_from_end = values.len().saturating_sub((-version) as usize);
                if index_from_end < values.len() {
                    return Ok(values[index_from_end].clone());
                }
            } else if version >= 0 {
                // 正版本号表示从第一个版本开始计数
                let index = version as usize;
                if index < values.len() {
                    return Ok(values[index].clone());
                }
            }
        }
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

    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String> {
        // 获取变量的属性值
        if let Some(values) = self.value_map.get(var) {
            if !values.is_empty() {
                let latest_value = values.last().unwrap();
                return self.get_property_from_value(latest_value, prop);
            }
        }
        Ok(Value::Null(NullType::Null))
    }

    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        // 获取目标顶点属性值
        // 在边上下文中，目标顶点信息可能存储在键值或特殊变量中
        if let Some(dst_var) = self.expr_value_map.get("_dst") {
            return self.get_property_from_value(dst_var, prop);
        }

        // 如果没有目标顶点变量，尝试从键值中解析
        if !self.key.is_empty() && self.is_edge {
            match self.parse_key_value() {
                Ok((_src_id, dst_id)) => {
                    // 创建一个简单的顶点值
                    let dst_vertex = Value::Vertex(Box::new(
                        crate::core::vertex_edge_path::Vertex::new(dst_id, vec![]),
                    ));
                    return self.get_property_from_value(&dst_vertex, prop);
                }
                Err(e) => {
                    // 键值解析失败，返回更具体的错误
                    return Err(format!("目标顶点属性解析失败: {}", e));
                }
            }
        }

        // 如果是标签属性，尝试从标签过滤器中获取
        if let Some(value) = self.tag_filters.get(&(tag.to_string(), prop.to_string())) {
            return Ok(value.clone());
        }

        Ok(Value::Null(NullType::UnknownProp))
    }

    fn get_input_prop(&self, prop: &str) -> Result<Value, String> {
        // 获取输入属性值
        // 输入属性通常来自查询的输入参数
        if let Some(input_vars) = self.expr_value_map.get("_input") {
            return self.get_property_from_value(input_vars, prop);
        }

        // 如果没有输入变量，尝试从内部变量中查找
        if let Some(value) = self.expr_value_map.get(prop) {
            return Ok(value.clone());
        }

        Ok(Value::Null(NullType::Null))
    }

    fn get_input_prop_index(&self, prop: &str) -> Result<usize, String> {
        // 获取输入属性的索引位置
        // 简化实现：假设输入属性按字母顺序排序
        if let Some(input_vars) = self.expr_value_map.get("_input") {
            if let Value::Map(map) = input_vars {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for (index, key) in keys.iter().enumerate() {
                    if *key == prop {
                        return Ok(index);
                    }
                }
            }
        }

        Err(format!("输入属性 '{}' 不存在", prop))
    }

    fn get_column(&self, index: i32) -> Result<Value, String> {
        // 根据列索引获取值
        if index < 0 {
            return Err("列索引不能为负数".to_string());
        }

        let index = index as usize;

        // 尝试从行读取器中获取
        if let Some(ref reader) = self.reader {
            let field_names = reader.get_field_names();
            if index < field_names.len() {
                let field_name = &field_names[index];
                return Ok(reader
                    .read_value(field_name)
                    .unwrap_or(Value::Null(NullType::Null)));
            }
        }

        // 尝试从内部变量中获取
        if let Some(column_vars) = self.expr_value_map.get("_columns") {
            if let Value::List(columns) = column_vars {
                if index < columns.len() {
                    return Ok(columns[index].clone());
                }
            }
        }

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
                        if !self.key.is_empty() {
                            match self.parse_key_value() {
                                Ok((src_id, _)) => Ok(src_id),
                                Err(e) => Err(format!("源顶点ID解析失败: {}", e)),
                            }
                        } else {
                            Ok(Value::String("src_vertex".to_string()))
                        }
                    }
                    "_dst" => {
                        // 从键值中提取目标顶点ID
                        if !self.key.is_empty() {
                            match self.parse_key_value() {
                                Ok((_, dst_id)) => Ok(dst_id),
                                Err(e) => Err(format!("目标顶点ID解析失败: {}", e)),
                            }
                        } else {
                            Ok(Value::String("dst_vertex".to_string()))
                        }
                    }
                    "_rank" => {
                        // 从键值中提取排名
                        match self.parse_edge_ranking() {
                            Ok(ranking) => Ok(Value::Int(ranking)),
                            Err(e) => Err(format!("边排名解析失败: {}", e)),
                        }
                    }
                    "_type" => {
                        // 从键值中提取边类型
                        match self.parse_edge_type() {
                            Ok(edge_type) => Ok(Value::Int(edge_type.into())),
                            Err(e) => Err(format!("边类型解析失败: {}", e)),
                        }
                    }
                    _ => Ok(self.read_value(prop)),
                }
            } else {
                // 从用户设置的过滤器中获取
                if let Some(value) = self
                    .edge_filters
                    .get(&(edge_name.to_string(), prop.to_string()))
                {
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
                        if !self.key.is_empty() {
                            match self.parse_key_value() {
                                Ok((vertex_id, _)) => Ok(vertex_id),
                                Err(e) => Err(format!("源顶点ID解析失败: {}", e)),
                            }
                        } else {
                            Ok(Value::String("vertex_id".to_string()))
                        }
                    }
                    "_tag" => {
                        // 从键值中提取标签ID
                        // 简化实现，实际应该从Schema中获取
                        Ok(Value::Int(1))
                    }
                    _ => Ok(self.read_value(prop)),
                }
            } else {
                // 从用户设置的过滤器中获取
                if let Some(value) = self
                    .tag_filters
                    .get(&(tag_name.to_string(), prop.to_string()))
                {
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