//! 转换类访问者
//!
//! 这个模块提供了用于转换 Value 的访问者实现

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Step, Tag, Vertex};
use crate::core::visitor::core::{utils, ValueVisitor};
use std::collections::HashMap;

/// 深度克隆访问者 - 创建 Value 的深度副本
#[derive(Debug, Default)]
pub struct DeepCloneVisitor {
    #[allow(dead_code)]
    max_depth: usize,
}

impl DeepCloneVisitor {
    pub fn new() -> Self {
        Self {
            max_depth: 100, // 默认最大深度限制
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { max_depth }
    }

    pub fn clone_value(value: &Value) -> Result<Value, TransformationError> {
        value.accept(&mut Self::new())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("转换错误: {0}")]
    Transformation(String),
    #[error("递归深度超过限制")]
    MaxDepthExceeded,
}

impl ValueVisitor for DeepCloneVisitor {
    type Result = Result<Value, TransformationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        Ok(Value::Bool(value))
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        Ok(Value::Int(value))
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        Ok(Value::Float(value))
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        Ok(Value::String(value.to_string()))
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        Ok(Value::Date(value.clone()))
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        Ok(Value::Time(value.clone()))
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        Ok(Value::DateTime(value.clone()))
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        let mut cloned_tags = Vec::with_capacity(value.tags().len());
        for tag in value.tags() {
            let mut cloned_props = HashMap::new();
            for (name, prop_value) in &tag.properties {
                cloned_props.insert(name.clone(), Self::clone_value(prop_value)?);
            }
            cloned_tags.push(Tag::new(tag.name.clone(), cloned_props));
        }

        let mut cloned_vertex_props = HashMap::new();
        for (name, prop_value) in value.vertex_properties() {
            cloned_vertex_props.insert(name.clone(), Self::clone_value(prop_value)?);
        }

        Ok(Value::Vertex(Box::new(Vertex::new_with_properties(
            value.id().clone(),
            cloned_tags,
            cloned_vertex_props,
        ))))
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        let mut cloned_props = HashMap::new();
        for (name, prop_value) in value.get_all_properties() {
            cloned_props.insert(name.clone(), Self::clone_value(prop_value)?);
        }

        Ok(Value::Edge(Edge::new(
            (*value.src).clone(),
            (*value.dst).clone(),
            value.edge_type().to_string(),
            value.ranking(),
            cloned_props,
        )))
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        let cloned_src = Self::clone_value(&Value::Vertex(Box::new(value.src.as_ref().clone())))?;
        let mut cloned_steps = Vec::with_capacity(value.steps.len());

        for step in &value.steps {
            let cloned_dst =
                Self::clone_value(&Value::Vertex(Box::new(step.dst.as_ref().clone())))?;
            let cloned_edge = Self::clone_value(&Value::Edge(step.edge.as_ref().clone()))?;
            cloned_steps.push(Step {
                dst: Box::new(match cloned_dst {
                    Value::Vertex(v) => *v,
                    _ => {
                        return Err(TransformationError::Transformation(
                            "Expected vertex".to_string(),
                        ))
                    }
                }),
                edge: Box::new(match cloned_edge {
                    Value::Edge(e) => e,
                    _ => {
                        return Err(TransformationError::Transformation(
                            "Expected edge".to_string(),
                        ))
                    }
                }),
            });
        }

        Ok(Value::Path(Path {
            src: Box::new(match cloned_src {
                Value::Vertex(v) => *v,
                _ => {
                    return Err(TransformationError::Transformation(
                        "Expected vertex".to_string(),
                    ))
                }
            }),
            steps: cloned_steps,
        }))
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        let cloned_list: Vec<Value> = value
            .iter()
            .map(|v| Self::clone_value(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(cloned_list))
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        let mut cloned_map = HashMap::new();
        for (k, v) in value {
            cloned_map.insert(k.clone(), Self::clone_value(v)?);
        }
        Ok(Value::Map(cloned_map))
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        let cloned_set: std::collections::HashSet<Value> = value
            .iter()
            .map(|v| Self::clone_value(v))
            .collect::<Result<std::collections::HashSet<_>, _>>()?;
        Ok(Value::Set(cloned_set))
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
        Ok(Value::Geography(value.clone()))
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        Ok(Value::Duration(value.clone()))
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        let cloned_col_names = value.col_names.clone();
        let mut cloned_rows = Vec::with_capacity(value.rows.len());

        for row in &value.rows {
            let cloned_row = row
                .iter()
                .map(|v| Self::clone_value(v))
                .collect::<Result<Vec<_>, _>>()?;
            cloned_rows.push(cloned_row);
        }

        Ok(Value::DataSet(DataSet {
            col_names: cloned_col_names,
            rows: cloned_rows,
        }))
    }

    fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
        Ok(Value::Null(null_type.clone()))
    }

    fn visit_empty(&mut self) -> Self::Result {
        Ok(Value::Empty)
    }
}

// Implement From trait for error conversion
impl From<utils::RecursionError> for TransformationError {
    fn from(err: utils::RecursionError) -> Self {
        match err {
            utils::RecursionError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        }
    }
}

/// 大小计算访问者 - 计算 Value 的内存大小
#[derive(Debug, Default)]
pub struct SizeCalculatorVisitor {
    size: usize,
    max_depth: usize,
}

impl SizeCalculatorVisitor {
    pub fn new() -> Self {
        Self {
            size: 0,
            max_depth: 100, // 默认最大深度限制
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { size: 0, max_depth }
    }

    pub fn calculate_size(value: &Value) -> Result<usize, TransformationError> {
        let mut visitor = Self::new();
        let max_depth = visitor.max_depth;
        let _ = utils::visit_recursive(value, &mut visitor, 0, max_depth)?;
        Ok(visitor.size)
    }
}

impl ValueVisitor for SizeCalculatorVisitor {
    type Result = Result<(), TransformationError>;

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.size += std::mem::size_of::<bool>();
        Ok(())
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.size += std::mem::size_of::<i64>();
        Ok(())
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        self.size += std::mem::size_of::<f64>();
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.size += std::mem::size_of::<String>() + value.len();
        Ok(())
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        self.size += std::mem::size_of::<DateValue>();
        Ok(())
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        self.size += std::mem::size_of::<TimeValue>();
        Ok(())
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        self.size += std::mem::size_of::<DateTimeValue>();
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.size += std::mem::size_of::<Vertex>();
        // 递归计算顶点内容的大小
        self.size += std::mem::size_of_val(value.id());
        for tag in value.tags() {
            self.size += std::mem::size_of::<Tag>();
            self.size += tag.name.len();
            for (prop_name, prop_value) in &tag.properties {
                self.size += prop_name.len();
                self.size += Self::calculate_size(prop_value)?;
            }
        }
        for (prop_name, prop_value) in value.vertex_properties() {
            self.size += prop_name.len();
            self.size += Self::calculate_size(prop_value)?;
        }
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.size += std::mem::size_of::<Edge>();
        self.size += std::mem::size_of_val(&value.src);
        self.size += std::mem::size_of_val(&value.dst);
        self.size += value.edge_type.len();
        for (prop_name, prop_value) in value.get_all_properties() {
            self.size += prop_name.len();
            self.size += Self::calculate_size(prop_value)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.size += std::mem::size_of::<Path>();
        self.size += Self::calculate_size(&Value::Vertex(Box::new(value.src.as_ref().clone())))?;
        for step in &value.steps {
            self.size += std::mem::size_of::<Step>();
            self.size += Self::calculate_size(&Value::Vertex(Box::new(step.dst.as_ref().clone())))?;
            self.size += Self::calculate_size(&Value::Edge(step.edge.as_ref().clone()))?;
        }
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        self.size += std::mem::size_of::<Vec<Value>>();
        for item in value {
            self.size += Self::calculate_size(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        self.size += std::mem::size_of::<HashMap<String, Value>>();
        for (key, val) in value {
            self.size += key.len();
            self.size += Self::calculate_size(val)?;
        }
        Ok(())
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        self.size += std::mem::size_of::<std::collections::HashSet<Value>>();
        for item in value {
            self.size += Self::calculate_size(item)?;
        }
        Ok(())
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
        self.size += std::mem::size_of::<GeographyValue>();
        // 计算地理数据的大小
        if let Some(_) = value.point {
            self.size += std::mem::size_of::<(f64, f64)>();
        }
        if let Some(ref line) = value.linestring {
            self.size += std::mem::size_of::<Vec<(f64, f64)>>()
                + line.len() * std::mem::size_of::<(f64, f64)>();
        }
        if let Some(ref poly) = value.polygon {
            self.size += std::mem::size_of::<Vec<Vec<(f64, f64)>>>();
            for ring in poly {
                self.size += std::mem::size_of::<Vec<(f64, f64)>>();
                self.size += ring.len() * std::mem::size_of::<(f64, f64)>();
            }
        }
        Ok(())
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        self.size += std::mem::size_of::<DurationValue>();
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.size += std::mem::size_of::<DataSet>();
        self.size += value.col_names.len() * std::mem::size_of::<String>();
        for row in &value.rows {
            self.size += std::mem::size_of::<Vec<Value>>();
            for cell in row {
                self.size += Self::calculate_size(cell)?;
            }
        }
        Ok(())
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.size += std::mem::size_of::<NullType>();
        Ok(())
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.size += std::mem::size_of::<Value>();
        Ok(())
    }
}

/// 哈希计算访问者 - 计算 Value 的哈希值
#[derive(Debug, Default)]
pub struct HashCalculatorVisitor {
    hasher: std::collections::hash_map::DefaultHasher,
    max_depth: usize,
}

impl HashCalculatorVisitor {
    pub fn new() -> Self {
        Self {
            hasher: std::collections::hash_map::DefaultHasher::new(),
            max_depth: 100, // 默认最大深度限制
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            hasher: std::collections::hash_map::DefaultHasher::new(),
            max_depth,
        }
    }

    pub fn calculate_hash(value: &Value) -> Result<u64, TransformationError> {
        let mut visitor = Self::new();
        let max_depth = visitor.max_depth;
        let _ = utils::visit_recursive(value, &mut visitor, 0, max_depth)?;
        use std::hash::Hasher;
        Ok(visitor.hasher.finish())
    }
}

impl ValueVisitor for HashCalculatorVisitor {
    type Result = Result<(), TransformationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        use std::hash::Hash;
        // 特殊处理浮点数的哈希
        if value.is_nan() {
            (0x7ff80000u32 as u64).hash(&mut self.hasher);
        } else if value == 0.0 {
            0.0_f64.to_bits().hash(&mut self.hasher);
        } else {
            value.to_bits().hash(&mut self.hasher);
        }
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        use std::hash::Hash;
        value.len().hash(&mut self.hasher);
        for item in value {
            HashCalculatorVisitor::calculate_hash(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        use std::hash::Hash;
        value.len().hash(&mut self.hasher);
        // 对键值对进行排序以确保一致的哈希
        let mut pairs: Vec<_> = value.iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        for (k, v) in pairs {
            k.hash(&mut self.hasher);
            HashCalculatorVisitor::calculate_hash(v)?;
        }
        Ok(())
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        use std::hash::Hash;
        value.len().hash(&mut self.hasher);
        // 对集合元素进行排序以确保一致的哈希
        let mut items: Vec<_> = value.iter().collect();
        // Sort by hash of each item to ensure consistent ordering
        items.sort_by(|a, b| {
            let hash_a = HashCalculatorVisitor::calculate_hash(a).unwrap_or_else(|_| 0);
            let hash_b = HashCalculatorVisitor::calculate_hash(b).unwrap_or_else(|_| 0);
            hash_a.cmp(&hash_b)
        });
        for item in items {
            HashCalculatorVisitor::calculate_hash(item)?;
        }
        Ok(())
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        use std::hash::Hash;
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
        use std::hash::Hash;
        null_type.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_empty(&mut self) -> Self::Result {
        use std::hash::Hash;
        0u8.hash(&mut self.hasher);
        Ok(())
    }
}

/// 类型转换访问者 - 在不同 Value 类型之间进行转换
#[derive(Debug, Default)]
pub struct TypeConversionVisitor {
    target_type: Option<ValueTypeDef>,
}

use crate::core::value::ValueTypeDef;

impl TypeConversionVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target_type(target_type: ValueTypeDef) -> Self {
        Self {
            target_type: Some(target_type),
        }
    }

    pub fn convert(value: &Value, target_type: ValueTypeDef) -> Result<Value, TransformationError> {
        let mut visitor = Self::with_target_type(target_type);
        value.accept(&mut visitor)
    }
}

impl ValueVisitor for TypeConversionVisitor {
    type Result = Result<Value, TransformationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(value.to_string())),
            Some(ValueTypeDef::Float) => Ok(Value::Float(if value { 1.0 } else { 0.0 })),
            Some(ValueTypeDef::Int) => Ok(Value::Int(value as i64)),
            Some(ValueTypeDef::Bool) => Ok(Value::Bool(value)),
            _ => Ok(Value::Bool(value)),
        }
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(value.to_string())),
            Some(ValueTypeDef::Float) => Ok(Value::Float(value as f64)),
            Some(ValueTypeDef::Int) => Ok(Value::Int(value)),
            Some(ValueTypeDef::Bool) => Ok(Value::Bool(value != 0)),
            _ => Ok(Value::Int(value)),
        }
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(value.to_string())),
            Some(ValueTypeDef::Float) => Ok(Value::Float(value)),
            Some(ValueTypeDef::Int) => Ok(Value::Int(value as i64)),
            Some(ValueTypeDef::Bool) => Ok(Value::Bool(value != 0.0)),
            _ => Ok(Value::Float(value)),
        }
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::Int) => value.parse::<i64>().map(Value::Int).map_err(|_| {
                TransformationError::Transformation(format!("无法将字符串 '{}' 转换为整数", value))
            }),
            Some(ValueTypeDef::Float) => value.parse::<f64>().map(Value::Float).map_err(|_| {
                TransformationError::Transformation(format!(
                    "无法将字符串 '{}' 转换为浮点数",
                    value
                ))
            }),
            Some(ValueTypeDef::Bool) => match value.to_lowercase().as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(TransformationError::Transformation(format!(
                    "无法将字符串 '{}' 转换为布尔值",
                    value
                ))),
            },
            _ => Ok(Value::String(value.to_string())),
        }
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!(
                "{}-{}-{}",
                value.year, value.month, value.day
            ))),
            Some(ValueTypeDef::DateTime) => Ok(Value::DateTime(DateTimeValue {
                year: value.year,
                month: value.month,
                day: value.day,
                hour: 0,
                minute: 0,
                sec: 0,
                microsec: 0,
            })),
            _ => Ok(Value::Date(value.clone())),
        }
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!(
                "{}:{}:{}",
                value.hour, value.minute, value.sec
            ))),
            Some(ValueTypeDef::DateTime) => Ok(Value::DateTime(DateTimeValue {
                year: 1970,
                month: 1,
                day: 1,
                hour: value.hour,
                minute: value.minute,
                sec: value.sec,
                microsec: 0,
            })),
            _ => Ok(Value::Time(value.clone())),
        }
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!(
                "{}-{}-{} {}:{}:{}",
                value.year, value.month, value.day, value.hour, value.minute, value.sec
            ))),
            _ => Ok(Value::DateTime(value.clone())),
        }
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!("Vertex({:?})", value.id()))),
            _ => Ok(Value::Vertex(Box::new(value.clone()))),
        }
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!(
                "Edge({:?} -> {:?}, type: {})",
                &*value.src, &*value.dst, value.edge_type
            ))),
            _ => Ok(Value::Edge(value.clone())),
        }
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => {
                Ok(Value::String(format!("Path(length: {})", value.len())))
            }
            _ => Ok(Value::Path(value.clone())),
        }
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => {
                let items: Vec<String> = value
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Ok(Value::String(format!("[{}]", items.join(", "))))
            }
            _ => {
                let target_type = self.target_type.as_ref().ok_or_else(||
                    TransformationError::Transformation("Target type not specified for list conversion".to_string())
                )?;
                Ok(Value::List(
                    value
                        .iter()
                        .map(|v| Self::convert(v, target_type.clone()))
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }
        }
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => {
                let pairs: Vec<String> = value
                    .iter()
                    .map(|(k, v)| {
                        let serialized_v = match v {
                            Value::String(s) => s.clone(),
                            _ => format!("{:?}", v),
                        };
                        format!("\"{}\": {}", k, serialized_v)
                    })
                    .collect();
                Ok(Value::String(format!("{{{}}}", pairs.join(", "))))
            }
            _ => {
                let target_type = self.target_type.as_ref().ok_or_else(||
                    TransformationError::Transformation("Target type not specified for map conversion".to_string())
                )?;
                Ok(Value::Map(
                    value
                        .iter()
                        .map(|(k, v)| {
                            let converted = Self::convert(v, target_type.clone())?;
                            Ok::<(String, Value), TransformationError>((k.clone(), converted))
                        })
                        .collect::<Result<HashMap<_, _>, _>>()?,
                ))
            }
        }
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => {
                let items: Vec<String> = value
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Ok(Value::String(format!("[{}]", items.join(", "))))
            }
            _ => {
                let target_type = self.target_type.as_ref().ok_or_else(||
                    TransformationError::Transformation("Target type not specified for set conversion".to_string())
                )?;
                Ok(Value::Set(
                    value
                        .iter()
                        .map(|v| Self::convert(v, target_type.clone()))
                        .collect::<Result<std::collections::HashSet<_>, TransformationError>>()?,
                ))
            }
        }
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String("\"geography\"".to_string())),
            _ => Ok(Value::Geography(value.clone())),
        }
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!("{} seconds", value.seconds))),
            _ => Ok(Value::Duration(value.clone())),
        }
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String(format!(
                "Dataset({} rows, {} columns)",
                value.rows.len(),
                value.col_names.len()
            ))),
            _ => Ok(Value::DataSet(value.clone())),
        }
    }

    fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String("null".to_string())),
            _ => Ok(Value::Null(null_type.clone())),
        }
    }

    fn visit_empty(&mut self) -> Self::Result {
        match self.target_type {
            Some(ValueTypeDef::String) => Ok(Value::String("\"empty\"".to_string())),
            _ => Ok(Value::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_deep_clone_visitor() {
        let original = Value::List(vec![
            Value::Int(42),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);

        let cloned = DeepCloneVisitor::clone_value(&original).unwrap();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_size_calculator_visitor() {
        let value = Value::String("test".to_string());
        let size = SizeCalculatorVisitor::calculate_size(&value).expect("Failed to calculate size");
        assert!(size > std::mem::size_of::<String>());
    }

    #[test]
    fn test_hash_calculator_visitor() {
        let value1 = Value::Int(42);
        let value2 = Value::Int(42);
        let hash1 = HashCalculatorVisitor::calculate_hash(&value1).expect("Failed to calculate hash");
        let hash2 = HashCalculatorVisitor::calculate_hash(&value2).expect("Failed to calculate hash");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_type_conversion_visitor() {
        let string_value = Value::String("123".to_string());
        let int_value = TypeConversionVisitor::convert(&string_value, ValueTypeDef::Int)
            .expect("Failed to convert string to int");
        assert_eq!(int_value, Value::Int(123));

        let bool_value = TypeConversionVisitor::convert(&string_value, ValueTypeDef::Bool)
            .expect("Failed to convert string to bool");
        assert_eq!(bool_value, Value::Bool(true));
    }
}
