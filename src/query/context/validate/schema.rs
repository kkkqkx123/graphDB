//! Schema管理模块
//! 提供Schema相关的数据结构和管理功能（简化版本）

use std::collections::HashMap;

/// Schema验证错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    FieldNotFound(String),
    TypeMismatch(String, String, String),
    MissingRequiredField(String),
    ExtraField(String),
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaValidationError::FieldNotFound(field) => {
                write!(f, "字段 '{}' 在Schema中不存在", field)
            }
            SchemaValidationError::TypeMismatch(field, expected, actual) => {
                write!(
                    f,
                    "字段 '{}' 类型不匹配: 期望 '{}', 实际 '{}'",
                    field, expected, actual
                )
            }
            SchemaValidationError::MissingRequiredField(field) => {
                write!(f, "缺少必需字段 '{}'", field)
            }
            SchemaValidationError::ExtraField(field) => {
                write!(f, "变量中包含Schema中未定义的字段 '{}'", field)
            }
        }
    }
}

/// Schema验证结果
#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub errors: Vec<SchemaValidationError>,
}

impl SchemaValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<SchemaValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: SchemaValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

/// Schema提供者trait
pub trait SchemaProvider: Send + Sync {
    fn get_schema(&self, name: &str) -> Option<SchemaInfo>;
    fn list_schemas(&self) -> Vec<String>;
}

/// Schema信息
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub fields: HashMap<String, String>,
    pub is_vertex: bool,
}

impl SchemaInfo {
    pub fn new(name: String, is_vertex: bool) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            is_vertex,
        }
    }

    pub fn add_field(&mut self, name: String, type_: String) {
        self.fields.insert(name, type_);
    }

    pub fn get_field_type(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    pub fn get_field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }
}

/// Schema管理器
#[derive(Debug, Clone)]
pub struct SchemaManager {
    schemas: HashMap<String, SchemaInfo>,
}

impl SchemaManager {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn add_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas.get(name)
    }

    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    pub fn remove_schema(&mut self, name: &str) -> Option<SchemaInfo> {
        self.schemas.remove(name)
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaProvider for SchemaManager {
    fn get_schema(&self, name: &str) -> Option<SchemaInfo> {
        self.schemas.get(name).cloned()
    }

    fn list_schemas(&self) -> Vec<String> {
        self.list_schemas()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_info_creation() {
        let mut schema = SchemaInfo::new("person".to_string(), true);
        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());

        assert_eq!(schema.name, "person");
        assert!(schema.is_vertex);
        assert_eq!(schema.fields.len(), 2);
        assert!(schema.has_field("id"));
        assert!(schema.has_field("name"));
        assert!(!schema.has_field("age"));
        assert_eq!(schema.get_field_type("id"), Some(&"INT".to_string()));
    }

    #[test]
    fn test_schema_manager() {
        let mut manager = SchemaManager::new();

        let mut person_schema = SchemaInfo::new("person".to_string(), true);
        person_schema.add_field("id".to_string(), "INT".to_string());
        manager.add_schema(person_schema);

        assert!(manager.has_schema("person"));
        assert!(!manager.has_schema("company"));

        let schemas = manager.list_schemas();
        assert_eq!(schemas.len(), 1);
        assert!(schemas.contains(&"person".to_string()));
    }

    #[test]
    fn test_schema_provider_trait() {
        let mut manager = SchemaManager::new();
        let schema = SchemaInfo::new("test".to_string(), true);
        manager.add_schema(schema);

        let provider: &dyn SchemaProvider = &manager;
        assert!(provider.get_schema("test").is_some());
        assert!(provider.get_schema("nonexistent").is_none());

        let schemas = provider.list_schemas();
        assert_eq!(schemas.len(), 1);
    }
}
