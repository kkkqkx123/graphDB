//! 额外的访问者实现
//!
//! 这个模块提供了更多的访问者实现，用于不同的 Value 操作
//!
//! 注意：此文件已被重构为模块化结构，请使用 `src/core/visitor/mod.rs` 中的新实现
//! 此文件保留是为了向后兼容，建议迁移到新的模块化结构

// 重新导出新模块化结构的内容，保持向后兼容性
pub use crate::core::visitor::{
    calculate_hash, calculate_size, convert_type, deep_clone, DeepCloneVisitor,
    HashCalculatorVisitor, SizeCalculatorVisitor, TransformationError, TypeConversionVisitor,
};

// 为了向后兼容，保留一些旧的类型别名
#[deprecated(note = "使用新的模块化结构 `crate::core::visitor`")]
pub mod legacy {
    //! 旧版访问者实现，保留用于向后兼容
    //! 建议使用新的模块化结构

    use crate::core::value::{
        DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue,
        Value,
    };
    use crate::core::vertex_edge_path::{Edge, Path, Vertex};
    use crate::core::visitor::ValueVisitor;
    use std::collections::HashMap;

    /// 旧版深度克隆访问者
    #[derive(Debug, Default)]
    pub struct DeepCloneVisitor;

    impl DeepCloneVisitor {
        pub fn new() -> Self {
            Self
        }

        pub fn clone_value(value: &Value) -> Value {
            let mut visitor = Self::new();
            value.accept(&mut visitor)
        }
    }

    impl ValueVisitor for DeepCloneVisitor {
        type Result = Value;

        fn visit_bool(&mut self, value: bool) -> Self::Result {
            Value::Bool(value)
        }

        fn visit_int(&mut self, value: i64) -> Self::Result {
            Value::Int(value)
        }

        fn visit_float(&mut self, value: f64) -> Self::Result {
            Value::Float(value)
        }

        fn visit_string(&mut self, value: &str) -> Self::Result {
            Value::String(value.to_string())
        }

        fn visit_date(&mut self, value: &DateValue) -> Self::Result {
            Value::Date(value.clone())
        }

        fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
            Value::Time(value.clone())
        }

        fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
            Value::DateTime(value.clone())
        }

        fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
            Value::Vertex(Box::new(value.clone()))
        }

        fn visit_edge(&mut self, value: &Edge) -> Self::Result {
            Value::Edge(value.clone())
        }

        fn visit_path(&mut self, value: &Path) -> Self::Result {
            Value::Path(value.clone())
        }

        fn visit_list(&mut self, value: &[Value]) -> Self::Result {
            let cloned_list: Vec<Value> = value.iter().map(|v| Self::clone_value(v)).collect();
            Value::List(cloned_list)
        }

        fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
            let cloned_map: HashMap<String, Value> = value
                .iter()
                .map(|(k, v)| (k.clone(), Self::clone_value(v)))
                .collect();
            Value::Map(cloned_map)
        }

        fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
            let cloned_set: std::collections::HashSet<Value> =
                value.iter().map(|v| Self::clone_value(v)).collect();
            Value::Set(cloned_set)
        }

        fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
            Value::Geography(value.clone())
        }

        fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
            Value::Duration(value.clone())
        }

        fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
            Value::DataSet(value.clone())
        }

        fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
            Value::Null(null_type.clone())
        }

        fn visit_empty(&mut self) -> Self::Result {
            Value::Empty
        }
    }

    /// 旧版大小计算访问者
    #[derive(Debug, Default)]
    pub struct SizeCalculatorVisitor {
        size: usize,
    }

    impl SizeCalculatorVisitor {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn calculate_size(value: &Value) -> usize {
            let mut visitor = Self::new();
            value.accept(&mut visitor);
            visitor.size
        }
    }

    impl ValueVisitor for SizeCalculatorVisitor {
        type Result = ();

        fn visit_bool(&mut self, _value: bool) -> Self::Result {
            self.size += std::mem::size_of::<bool>();
        }

        fn visit_int(&mut self, _value: i64) -> Self::Result {
            self.size += std::mem::size_of::<i64>();
        }

        fn visit_float(&mut self, _value: f64) -> Self::Result {
            self.size += std::mem::size_of::<f64>();
        }

        fn visit_string(&mut self, value: &str) -> Self::Result {
            self.size += std::mem::size_of::<String>() + value.len();
        }

        fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
            self.size += std::mem::size_of::<DateValue>();
        }

        fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
            self.size += std::mem::size_of::<TimeValue>();
        }

        fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
            self.size += std::mem::size_of::<DateTimeValue>();
        }

        fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
            self.size += std::mem::size_of::<Vertex>();
            // 递归计算顶点内容的大小
            self.size += std::mem::size_of_val(value.id());
            for tag in value.tags() {
                self.size += std::mem::size_of::<crate::core::vertex_edge_path::Tag>();
                self.size += tag.name.len();
                for (prop_name, prop_value) in &tag.properties {
                    self.size += prop_name.len();
                    self.size += Self::calculate_size(prop_value);
                }
            }
            for (prop_name, prop_value) in value.vertex_properties() {
                self.size += prop_name.len();
                self.size += Self::calculate_size(prop_value);
            }
        }

        fn visit_edge(&mut self, value: &Edge) -> Self::Result {
            self.size += std::mem::size_of::<Edge>();
            self.size += std::mem::size_of_val(value.src());
            self.size += std::mem::size_of_val(value.dst());
            self.size += value.edge_type().len();
            for (prop_name, prop_value) in value.get_all_properties() {
                self.size += prop_name.len();
                self.size += Self::calculate_size(prop_value);
            }
        }

        fn visit_path(&mut self, value: &Path) -> Self::Result {
            self.size += std::mem::size_of::<Path>();
            self.size += Self::calculate_size(&Value::Vertex(Box::new(value.src.as_ref().clone())));
            for step in &value.steps {
                self.size += std::mem::size_of::<crate::core::vertex_edge_path::Step>();
                self.size +=
                    Self::calculate_size(&Value::Vertex(Box::new(step.dst.as_ref().clone())));
                self.size += Self::calculate_size(&Value::Edge(step.edge.as_ref().clone()));
            }
        }

        fn visit_list(&mut self, value: &[Value]) -> Self::Result {
            self.size += std::mem::size_of::<Vec<Value>>();
            for item in value {
                self.size += Self::calculate_size(item);
            }
        }

        fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
            self.size += std::mem::size_of::<HashMap<String, Value>>();
            for (key, val) in value {
                self.size += key.len();
                self.size += Self::calculate_size(val);
            }
        }

        fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
            self.size += std::mem::size_of::<std::collections::HashSet<Value>>();
            for item in value {
                self.size += Self::calculate_size(item);
            }
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
                    self.size += std::mem::size_of::<Vec<(f64, f64)>>()
                        + ring.len() * std::mem::size_of::<(f64, f64)>();
                }
            }
        }

        fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
            self.size += std::mem::size_of::<DurationValue>();
        }

        fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
            self.size += std::mem::size_of::<DataSet>();
            self.size += value.col_names.len() * std::mem::size_of::<String>();
            for row in &value.rows {
                self.size += std::mem::size_of::<Vec<Value>>();
                for cell in row {
                    self.size += Self::calculate_size(cell);
                }
            }
        }

        fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
            self.size += std::mem::size_of::<NullType>();
        }

        fn visit_empty(&mut self) -> Self::Result {
            self.size += std::mem::size_of::<Value>();
        }
    }

    /// 旧版哈希计算访问者
    #[derive(Debug, Default)]
    pub struct HashCalculatorVisitor {
        hasher: std::collections::hash_map::DefaultHasher,
    }

    impl HashCalculatorVisitor {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn calculate_hash(value: &Value) -> u64 {
            let mut visitor = Self::new();
            value.accept(&mut visitor);
            use std::hash::Hasher;
            visitor.hasher.finish()
        }
    }

    impl ValueVisitor for HashCalculatorVisitor {
        type Result = ();

        fn visit_bool(&mut self, value: bool) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_int(&mut self, value: i64) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_float(&mut self, value: f64) -> Self::Result {
            // 特殊处理浮点数的哈希
            if value.is_nan() {
                use std::hash::Hash;
                (0x7ff80000u32 as u64).hash(&mut self.hasher);
            } else if value == 0.0 {
                use std::hash::Hash;
                0.0_f64.to_bits().hash(&mut self.hasher);
            } else {
                use std::hash::Hash;
                value.to_bits().hash(&mut self.hasher);
            }
        }

        fn visit_string(&mut self, value: &str) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_date(&mut self, value: &DateValue) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_edge(&mut self, value: &Edge) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_path(&mut self, value: &Path) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_list(&mut self, value: &[Value]) -> Self::Result {
            use std::hash::Hash;
            value.len().hash(&mut self.hasher);
            for item in value {
                let _ = Self::calculate_hash(item);
            }
        }

        fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
            use std::hash::Hash;
            value.len().hash(&mut self.hasher);
            // 对键值对进行排序以确保一致的哈希
            let mut pairs: Vec<_> = value.iter().collect();
            pairs.sort_by_key(|&(k, _)| k);
            for (k, v) in pairs {
                k.hash(&mut self.hasher);
                let _ = Self::calculate_hash(v);
            }
        }

        fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
            use std::hash::Hash;
            value.len().hash(&mut self.hasher);
            // 对集合元素进行排序以确保一致的哈希
            let mut items: Vec<_> = value.iter().collect();
            items.sort_by(|a, b| {
                let hash_a = Self::calculate_hash(a);
                let hash_b = Self::calculate_hash(b);
                hash_a.cmp(&hash_b)
            });
            for item in items {
                let _ = Self::calculate_hash(item);
            }
        }

        fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
            use std::hash::Hash;
            value.hash(&mut self.hasher);
        }

        fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
            use std::hash::Hash;
            null_type.hash(&mut self.hasher);
        }

        fn visit_empty(&mut self) -> Self::Result {
            use std::hash::Hash;
            0u8.hash(&mut self.hasher);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_backward_compatibility() {
        // 测试向后兼容性
        let original = Value::List(vec![
            Value::Int(42),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);

        let cloned = deep_clone(&original).expect("克隆失败");
        assert_eq!(original, cloned);

        let value = Value::String("test".to_string());
        let size = calculate_size(&value).expect("大小计算失败");
        assert!(size > std::mem::size_of::<String>());

        let value1 = Value::Int(42);
        let value2 = Value::Int(42);
        let hash1 = calculate_hash(&value1).expect("哈希计算失败");
        let hash2 = calculate_hash(&value2).expect("哈希计算失败");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_edge_src_dst() {
        // 测试边的 src 和 dst 访问
        use crate::core::vertex_edge_path::Edge;
        use crate::core::Value;

        let src_id = Value::Int(1);
        let dst_id = Value::Int(2);
        let edge = Edge::new(
            src_id,
            dst_id,
            "关系".to_string(),
            0,
            std::collections::HashMap::new(),
        );

        let _src = &edge.src;
        let _dst = &edge.dst;
    }

    #[test]
    fn test_new_api_compatibility() {
        // 测试新 API 的兼容性
        let original = Value::List(vec![
            Value::Int(42),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);

        // 使用新的便捷函数
        let cloned = deep_clone(&original).unwrap();
        assert_eq!(original, cloned);

        let value = Value::String("test".to_string());
        let size = calculate_size(&value).unwrap();
        assert!(size > std::mem::size_of::<String>());

        let value1 = Value::Int(42);
        let value2 = Value::Int(42);
        let hash1 = calculate_hash(&value1).unwrap();
        let hash2 = calculate_hash(&value2).unwrap();
        assert_eq!(hash1, hash2);
    }
}
