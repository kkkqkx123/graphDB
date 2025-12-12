//! 访问者模式实现 for Value 类型
//!
//! 这个模块提供了访问者模式的实现，允许对 Value 类型进行操作而不修改其结构
//! 
//! 注意：此文件已被重构为模块化结构，请使用 `src/core/visitor/mod.rs` 中的新实现
//! 此文件保留是为了向后兼容，建议迁移到新的模块化结构

// 重新导出新模块化结构的内容，保持向后兼容性
pub use crate::core::visitor::{
    ValueVisitor, ValueAcceptor,
    TypeCheckerVisitor, ComplexityAnalyzerVisitor, TypeCategory,
    JsonSerializationVisitor, XmlSerializationVisitor, SerializationFormat, SerializationError,
    DeepCloneVisitor, SizeCalculatorVisitor, HashCalculatorVisitor, TypeConversionVisitor, TransformationError,
    BasicValidationVisitor, TypeValidationVisitor, ValidationConfig, ValidationRule, ValidationError,
    
    // 便捷函数
    check_type, analyze_complexity, to_json, to_json_pretty, to_xml,
    deep_clone, calculate_size, calculate_hash, convert_type,
    validate_basic, validate_with_config, validate_type, validate_type_strict,
};

// 为了向后兼容，保留一些旧的类型别名
pub type JsonVisitor = JsonSerializationVisitor;
pub type TypeChecker = TypeCheckerVisitor;

#[deprecated(note = "使用新的模块化结构 `crate::core::visitor`")]
pub mod legacy {
    //! 旧版访问者实现，保留用于向后兼容
    //! 建议使用新的模块化结构
    
    use crate::core::value::{Value, NullType, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, DataSet};
    use crate::core::vertex_edge_path::{Vertex, Edge, Path};
    use std::collections::HashMap;

    /// 旧版 Value 访问者 trait
    pub trait ValueVisitor {
        type Result;
        
        fn visit_bool(&mut self, value: bool) -> Self::Result;
        fn visit_int(&mut self, value: i64) -> Self::Result;
        fn visit_float(&mut self, value: f64) -> Self::Result;
        fn visit_string(&mut self, value: &str) -> Self::Result;
        fn visit_date(&mut self, value: &DateValue) -> Self::Result;
        fn visit_time(&mut self, value: &TimeValue) -> Self::Result;
        fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result;
        fn visit_vertex(&mut self, value: &Vertex) -> Self::Result;
        fn visit_edge(&mut self, value: &Edge) -> Self::Result;
        fn visit_path(&mut self, value: &Path) -> Self::Result;
        fn visit_list(&mut self, value: &[Value]) -> Self::Result;
        fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result;
        fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result;
        fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result;
        fn visit_duration(&mut self, value: &DurationValue) -> Self::Result;
        fn visit_dataset(&mut self, value: &DataSet) -> Self::Result;
        fn visit_null(&mut self, null_type: &NullType) -> Self::Result;
        fn visit_empty(&mut self) -> Self::Result;
    }

    /// 旧版类型检查访问者
    #[derive(Debug, Default)]
    pub struct TypeCheckerVisitor {
        is_numeric: bool,
        is_string: bool,
        is_boolean: bool,
        is_collection: bool,
        is_graph_element: bool,
        is_temporal: bool,
        is_null: bool,
        is_empty: bool,
    }

    impl TypeCheckerVisitor {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn is_numeric(&self) -> bool {
            self.is_numeric
        }

        pub fn is_string(&self) -> bool {
            self.is_string
        }

        pub fn is_boolean(&self) -> bool {
            self.is_boolean
        }

        pub fn is_collection(&self) -> bool {
            self.is_collection
        }

        pub fn is_graph_element(&self) -> bool {
            self.is_graph_element
        }

        pub fn is_temporal(&self) -> bool {
            self.is_temporal
        }

        pub fn is_null(&self) -> bool {
            self.is_null
        }

        pub fn is_empty(&self) -> bool {
            self.is_empty
        }

        pub fn get_type_name(&self) -> &'static str {
            if self.is_empty { return "Empty"; }
            if self.is_null { return "Null"; }
            if self.is_boolean { return "Bool"; }
            if self.is_numeric { return "Numeric"; }
            if self.is_string { return "String"; }
            if self.is_temporal { return "Temporal"; }
            if self.is_graph_element { return "GraphElement"; }
            if self.is_collection { return "Collection"; }
            "Unknown"
        }

        pub fn reset(&mut self) {
            *self = Self::default();
        }
    }

    impl ValueVisitor for TypeCheckerVisitor {
        type Result = ();

        fn visit_bool(&mut self, _value: bool) -> Self::Result {
            self.is_boolean = true;
        }

        fn visit_int(&mut self, _value: i64) -> Self::Result {
            self.is_numeric = true;
        }

        fn visit_float(&mut self, _value: f64) -> Self::Result {
            self.is_numeric = true;
        }

        fn visit_string(&mut self, _value: &str) -> Self::Result {
            self.is_string = true;
        }

        fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
            self.is_temporal = true;
        }

        fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
            self.is_temporal = true;
        }

        fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
            self.is_temporal = true;
        }

        fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
            self.is_graph_element = true;
        }

        fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
            self.is_graph_element = true;
        }

        fn visit_path(&mut self, _value: &Path) -> Self::Result {
            self.is_graph_element = true;
        }

        fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
            self.is_collection = true;
        }

        fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
            self.is_collection = true;
        }

        fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
            self.is_collection = true;
        }

        fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
            // Geography 可以被视为特殊类型
        }

        fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
            self.is_temporal = true;
        }

        fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
            self.is_collection = true;
        }

        fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
            self.is_null = true;
        }

        fn visit_empty(&mut self) -> Self::Result {
            self.is_empty = true;
        }
    }

    /// 旧版 JSON 序列化访问者
    #[derive(Debug, Default)]
    pub struct JsonSerializationVisitor {
        result: String,
    }

    impl JsonSerializationVisitor {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn serialize(value: &Value) -> Result<String, serde_json::Error> {
            let mut visitor = Self::new();
            value.accept(&mut visitor);
            Ok(visitor.result)
        }
    }

    impl ValueVisitor for JsonSerializationVisitor {
        type Result = ();

        fn visit_bool(&mut self, value: bool) -> Self::Result {
            self.result = value.to_string();
        }

        fn visit_int(&mut self, value: i64) -> Self::Result {
            self.result = value.to_string();
        }

        fn visit_float(&mut self, value: f64) -> Self::Result {
            self.result = value.to_string();
        }

        fn visit_string(&mut self, value: &str) -> Self::Result {
            self.result = format!("\"{}\"", value);
        }

        fn visit_date(&mut self, value: &DateValue) -> Self::Result {
            self.result = format!("\"{}-{}-{}\"", value.year, value.month, value.day);
        }

        fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
            self.result = format!("\"{}:{}:{}\"", value.hour, value.minute, value.sec);
        }

        fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
            self.result = format!(
                "\"{}-{}-{} {}:{}:{}\"",
                value.year, value.month, value.day, value.hour, value.minute, value.sec
            );
        }

        fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
            // 简化的顶点序列化
            self.result = format!("{{\"vertex_id\": {:?}, \"tags\": {}}}", value.id(), value.tags().len());
        }

        fn visit_edge(&mut self, value: &Edge) -> Self::Result {
            // 简化的边序列化
            self.result = format!(
                "{{\"src\": {:?}, \"dst\": {:?}, \"type\": \"{}\"}}",
                value.src(), value.dst(), value.edge_type()
            );
        }

        fn visit_path(&mut self, value: &Path) -> Self::Result {
            // 简化的路径序列化
            self.result = format!("{{\"path_length\": {}}}", value.len());
        }

        fn visit_list(&mut self, value: &[Value]) -> Self::Result {
            let items: Vec<String> = value
                .iter()
                .map(|v| Self::serialize(v).unwrap_or_else(|_| "null".to_string()))
                .collect();
            self.result = format!("[{}]", items.join(", "));
        }

        fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
            let pairs: Vec<String> = value
                .iter()
                .map(|(k, v)| {
                    let serialized_v = Self::serialize(v).unwrap_or_else(|_| "null".to_string());
                    format!("\"{}\": {}", k, serialized_v)
                })
                .collect();
            self.result = format!("{{{}}}", pairs.join(", "));
        }

        fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
            let items: Vec<String> = value
                .iter()
                .map(|v| Self::serialize(v).unwrap_or_else(|_| "null".to_string()))
                .collect();
            self.result = format!("[{}]", items.join(", "));
        }

        fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
            self.result = "\"geography\"".to_string();
        }

        fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
            self.result = format!("\"{} seconds\"", value.seconds);
        }

        fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
            self.result = format!("{{\"dataset\": {{\"rows\": {}}}}}", value.rows.len());
        }

        fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
            self.result = "null".to_string();
        }

        fn visit_empty(&mut self) -> Self::Result {
            self.result = "\"empty\"".to_string();
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
        let mut visitor = TypeCheckerVisitor::new();
        
        let int_value = Value::Int(42);
        int_value.accept(&mut visitor);
        assert!(visitor.is_numeric());
        assert_eq!(visitor.get_type_name(), "Numeric");
        
        let json = JsonSerializationVisitor::serialize(&int_value).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_new_api_compatibility() {
        // 测试新 API 的兼容性
        let int_value = Value::Int(42);
        
        // 使用新的便捷函数
        let category = check_type(&int_value);
        assert_eq!(category, TypeCategory::Numeric);
        
        let json = to_json(&int_value).unwrap();
        assert_eq!(json, "42");
    }
}