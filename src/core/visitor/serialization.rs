//! 序列化类访问者
//!
//! 这个模块提供了用于序列化 Value 的访问者实现

use crate::core::visitor::core::ValueVisitor;
use crate::core::value::{Value, NullType, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, DataSet};
use crate::core::vertex_edge_path::{Vertex, Edge, Path};
use std::collections::HashMap;

/// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Xml,
    Yaml,
    Custom,
}

/// JSON 序列化访问者 - 将 Value 转换为 JSON 字符串
#[derive(Debug, Default)]
pub struct JsonSerializationVisitor {
    result: String,
    indent_level: usize,
    pretty: bool,
}

impl JsonSerializationVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_pretty() -> Self {
        Self {
            result: String::new(),
            indent_level: 0,
            pretty: true,
        }
    }

    pub fn serialize(value: &Value) -> Result<String, SerializationError> {
        let mut visitor = Self::new();
        value.accept(&mut visitor)?;
        Ok(visitor.result)
    }

    pub fn serialize_pretty(value: &Value) -> Result<String, SerializationError> {
        let mut visitor = Self::new_pretty();
        value.accept(&mut visitor)?;
        Ok(visitor.result)
    }

    fn indent(&mut self) {
        if self.pretty {
            self.result.push_str("\n");
            for _ in 0..self.indent_level {
                self.result.push_str("  ");
            }
        }
    }

    fn start_object(&mut self) {
        self.result.push('{');
        if self.pretty {
            self.indent_level += 1;
        }
    }

    fn end_object(&mut self) {
        if self.pretty {
            self.indent_level -= 1;
            self.indent();
        }
        self.result.push('}');
    }

    fn start_array(&mut self) {
        self.result.push('[');
        if self.pretty {
            self.indent_level += 1;
        }
    }

    fn end_array(&mut self) {
        if self.pretty {
            self.indent_level -= 1;
            self.indent();
        }
        self.result.push(']');
    }

    fn add_comma(&mut self) {
        self.result.push(',');
        if self.pretty {
            self.result.push(' ');
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("序列化错误: {0}")]
    Serialization(String),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}

impl ValueVisitor for JsonSerializationVisitor {
    type Result = Result<(), SerializationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.result.push('"');
        // 简单的 JSON 转义
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        self.result.push_str(&escaped);
        self.result.push('"');
        Ok(())
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        self.result.push('"');
        self.result.push_str(&format!("{}-{}-{}", value.year, value.month, value.day));
        self.result.push('"');
        Ok(())
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        self.result.push('"');
        self.result.push_str(&format!("{}:{}:{}", value.hour, value.minute, value.sec));
        self.result.push('"');
        Ok(())
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        self.result.push('"');
        self.result.push_str(&format!(
            "{}-{}-{} {}:{}:{}",
            value.year, value.month, value.day, value.hour, value.minute, value.sec
        ));
        self.result.push('"');
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.start_object();
        self.result.push_str("\"vertex_id\": ");
        self.result.push_str(&format!("{:?}", value.id()));
        self.add_comma();
        self.indent();
        self.result.push_str("\"tags\": ");
        self.result.push_str(&value.tags().len().to_string());
        self.add_comma();
        self.indent();
        self.result.push_str("\"properties\": ");
        
        // 简化的顶点属性序列化
        let mut props = Vec::new();
        for (name, prop_value) in value.get_all_properties() {
            props.push(format!("\"{}\": {}", name, Self::serialize_value(prop_value)?));
        }
        self.result.push_str(&format!("{{{}}}", props.join(", ")));
        
        self.end_object();
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.start_object();
        self.result.push_str("\"src\": ");
        self.result.push_str(&format!("{:?}", value.src));
        self.add_comma();
        self.indent();
        self.result.push_str("\"dst\": ");
        self.result.push_str(&format!("{:?}", value.dst));
        self.add_comma();
        self.indent();
        self.result.push_str("\"type\": \"");
        self.result.push_str(value.edge_type());
        self.result.push('"');
        self.add_comma();
        self.indent();
        self.result.push_str("\"ranking\": ");
        self.result.push_str(&value.ranking().to_string());
        self.add_comma();
        self.indent();
        self.result.push_str("\"properties\": ");
        
        // 简化的边属性序列化
        let mut props = Vec::new();
        for (name, prop_value) in value.get_all_properties() {
            props.push(format!("\"{}\": {}", name, Self::serialize_value(prop_value)?));
        }
        self.result.push_str(&format!("{{{}}}", props.join(", ")));
        
        self.end_object();
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.start_object();
        self.result.push_str("\"length\": ");
        self.result.push_str(&value.len().to_string());
        self.add_comma();
        self.indent();
        self.result.push_str("\"src\": ");
        self.result.push_str(&format!("{:?}", value.src));
        self.end_object();
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        if value.is_empty() {
            self.result.push_str("[]");
            return Ok(());
        }

        self.start_array();
        for (i, item) in value.iter().enumerate() {
            Self::serialize_value(item)?;
            if i < value.len() - 1 {
                self.add_comma();
            }
        }
        self.end_array();
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        if value.is_empty() {
            self.result.push_str("{}");
            return Ok(());
        }

        self.start_object();
        let mut pairs: Vec<(&String, &Value)> = value.iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        
        for (i, (key, val)) in pairs.iter().enumerate() {
            self.result.push('"');
            self.result.push_str(key);
            self.result.push_str("\": ");
            Self::serialize_value(val)?;
            if i < pairs.len() - 1 {
                self.add_comma();
            }
        }
        self.end_object();
        Ok(())
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        if value.is_empty() {
            self.result.push_str("[]");
            return Ok(());
        }

        self.start_array();
        let mut items: Vec<_> = value.iter().collect();
        items.sort();
        
        for (i, item) in items.iter().enumerate() {
            Self::serialize_value(item)?;
            if i < items.len() - 1 {
                self.add_comma();
            }
        }
        self.end_array();
        Ok(())
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.result.push_str("\"geography\"");
        Ok(())
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        self.result.push('"');
        self.result.push_str(&format!("{} seconds", value.seconds));
        self.result.push('"');
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.start_object();
        self.result.push_str("\"columns\": ");
        self.result.push_str(&value.col_names.len().to_string());
        self.add_comma();
        self.indent();
        self.result.push_str("\"rows\": ");
        self.result.push_str(&value.rows.len().to_string());
        self.end_object();
        Ok(())
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.result.push_str("null");
        Ok(())
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.result.push_str("\"empty\"");
        Ok(())
    }
}

impl JsonSerializationVisitor {
    fn serialize_value(value: &Value) -> Result<String, SerializationError> {
        let mut visitor = Self::new();
        value.accept(&mut visitor)?;
        Ok(visitor.result)
    }
}

/// XML 序列化访问者 - 将 Value 转换为 XML 字符串
#[derive(Debug, Default)]
pub struct XmlSerializationVisitor {
    result: String,
    indent_level: usize,
}

impl XmlSerializationVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(value: &Value) -> Result<String, SerializationError> {
        let mut visitor = Self::new();
        value.accept(&mut visitor)?;
        Ok(visitor.result)
    }

    fn indent(&mut self) {
        self.result.push('\n');
        for _ in 0..self.indent_level {
            self.result.push_str("  ");
        }
    }

    fn start_tag(&mut self, tag: &str) {
        self.result.push('<');
        self.result.push_str(tag);
        self.result.push('>');
    }

    fn end_tag(&mut self, tag: &str) {
        self.result.push_str("</");
        self.result.push_str(tag);
        self.result.push('>');
    }

    fn start_element(&mut self, tag: &str) {
        self.start_tag(tag);
        self.indent_level += 1;
    }

    fn end_element(&mut self, tag: &str) {
        self.indent_level -= 1;
        self.indent();
        self.end_tag(tag);
    }
}

impl ValueVisitor for XmlSerializationVisitor {
    type Result = Result<(), SerializationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        self.result.push_str(&value.to_string());
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.result.push_str(value);
        Ok(())
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        self.start_element("date");
        self.indent();
        self.result.push_str(&format!(
            "<year>{}</year>",
            value.year
        ));
        self.indent();
        self.result.push_str(&format!(
            "<month>{}</month>",
            value.month
        ));
        self.indent();
        self.result.push_str(&format!(
            "<day>{}</day>",
            value.day
        ));
        self.end_element("date");
        Ok(())
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        self.start_element("time");
        self.indent();
        self.result.push_str(&format!(
            "<hour>{}</hour>",
            value.hour
        ));
        self.indent();
        self.result.push_str(&format!(
            "<minute>{}</minute>",
            value.minute
        ));
        self.indent();
        self.result.push_str(&format!(
            "<second>{}</second>",
            value.sec
        ));
        self.end_element("time");
        Ok(())
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        self.start_element("datetime");
        self.indent();
        self.result.push_str(&format!(
            "<year>{}</year>",
            value.year
        ));
        self.indent();
        self.result.push_str(&format!(
            "<month>{}</month>",
            value.month
        ));
        self.indent();
        self.result.push_str(&format!(
            "<day>{}</day>",
            value.day
        ));
        self.indent();
        self.result.push_str(&format!(
            "<hour>{}</hour>",
            value.hour
        ));
        self.indent();
        self.result.push_str(&format!(
            "<minute>{}</minute>",
            value.minute
        ));
        self.indent();
        self.result.push_str(&format!(
            "<second>{}</second>",
            value.sec
        ));
        self.end_element("datetime");
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.start_element("vertex");
        self.indent();
        self.result.push_str(&format!(
            "<id>{:?}</id>",
            value.id()
        ));
        self.indent();
        self.result.push_str(&format!(
            "<tags>{}</tags>",
            value.tags().len()
        ));
        self.end_element("vertex");
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.start_element("edge");
        self.indent();
        self.result.push_str(&format!(
            "<src>{:?}</src>",
            value.src
        ));
        self.indent();
        self.result.push_str(&format!(
            "<dst>{:?}</dst>",
            value.dst
        ));
        self.indent();
        self.result.push_str(&format!(
            "<type>{}</type>",
            value.edge_type()
        ));
        self.indent();
        self.result.push_str(&format!(
            "<ranking>{}</ranking>",
            value.ranking()
        ));
        self.end_element("edge");
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.start_element("path");
        self.indent();
        self.result.push_str(&format!(
            "<length>{}</length>",
            value.len()
        ));
        self.end_element("path");
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        self.start_element("list");
        for item in value {
            self.indent();
            item.accept(self)?;
        }
        self.end_element("list");
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        self.start_element("map");
        for (key, val) in value {
            self.indent();
            self.start_element("entry");
            self.indent();
            self.start_element("key");
            self.result.push_str(key);
            self.end_element("key");
            self.indent();
            self.start_element("value");
            val.accept(self)?;
            self.end_element("value");
            self.end_element("entry");
        }
        self.end_element("map");
        Ok(())
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        self.start_element("set");
        for item in value {
            self.indent();
            item.accept(self)?;
        }
        self.end_element("set");
        Ok(())
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.start_element("geography");
        self.end_element("geography");
        Ok(())
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        self.start_element("duration");
        self.indent();
        self.result.push_str(&format!(
            "<seconds>{}</seconds>",
            value.seconds
        ));
        self.end_element("duration");
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.start_element("dataset");
        self.indent();
        self.result.push_str(&format!(
            "<columns>{}</columns>",
            value.col_names.len()
        ));
        self.indent();
        self.result.push_str(&format!(
            "<rows>{}</rows>",
            value.rows.len()
        ));
        self.end_element("dataset");
        Ok(())
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.start_element("null");
        self.end_element("null");
        Ok(())
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.start_element("empty");
        self.end_element("empty");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_json_serialization_visitor() {
        let int_value = Value::Int(42);
        let json = JsonSerializationVisitor::serialize(&int_value).unwrap();
        assert_eq!(json, "42");
        
        let string_value = Value::String("test".to_string());
        let json = JsonSerializationVisitor::serialize(&string_value).unwrap();
        assert_eq!(json, "\"test\"");
        
        let bool_value = Value::Bool(true);
        let json = JsonSerializationVisitor::serialize(&bool_value).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn test_json_pretty_serialization() {
        let complex_value = Value::Map(std::collections::HashMap::from([
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ]));
        
        let json = JsonSerializationVisitor::serialize_pretty(&complex_value).unwrap();
        assert!(json.contains("{\n"));
        assert!(json.contains("\"name\": \"Alice\""));
        assert!(json.contains("\"age\": 30"));
    }

    #[test]
    fn test_xml_serialization_visitor() {
        let int_value = Value::Int(42);
        let xml = XmlSerializationVisitor::serialize(&int_value).unwrap();
        assert!(xml.contains("<int>42</int>"));
        
        let string_value = Value::String("test".to_string());
        let xml = XmlSerializationVisitor::serialize(&string_value).unwrap();
        assert!(xml.contains("<test>test</test>"));
    }
}