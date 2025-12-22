//! Schema管理模块
//! 提供Schema相关的数据结构和管理功能
//!

use std::collections::HashMap;

/// Schema验证错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    /// 字段在Schema中不存在
    FieldNotFound(String),
    /// 字段类型不匹配 (字段名, 期望类型, 实际类型)
    TypeMismatch(String, String, String),
    /// 缺少必需字段
    MissingRequiredField(String),
    /// 变量中有Schema中未定义的字段
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
    /// 是否验证通过
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<SchemaValidationError>,
}

impl SchemaValidationResult {
    /// 创建成功的验证结果
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn failure(errors: Vec<SchemaValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// 添加错误
    pub fn add_error(&mut self, error: SchemaValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

/// Schema验证模式
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationMode {
    /// 严格模式：变量字段必须与Schema完全匹配
    Strict,
    /// 宽松模式：允许变量中有额外字段，但类型必须匹配
    Lenient,
    /// 必需字段模式：只验证必需字段存在且类型匹配
    RequiredOnly,
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
    pub fields: HashMap<String, String>, // 字段名 -> 类型
    pub is_vertex: bool,
}

impl SchemaInfo {
    /// 创建新的Schema信息
    pub fn new(name: String, is_vertex: bool) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            is_vertex,
        }
    }

    /// 添加字段
    pub fn add_field(&mut self, name: String, type_: String) {
        self.fields.insert(name, type_);
    }

    /// 获取字段类型
    pub fn get_field_type(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }

    /// 检查字段是否存在
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// 获取所有字段名
    pub fn get_field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    /// 验证字段类型是否匹配
    pub fn validate_field_type(&self, name: &str, expected_type: &str) -> bool {
        self.fields.get(name).map_or(false, |t| t == expected_type)
    }

    /// 验证变量列定义是否符合Schema
    ///
    /// # 参数
    /// * `var_cols` - 变量的列定义
    /// * `mode` - 验证模式
    /// * `required_fields` - 必需字段列表（可选）
    ///
    /// # 返回值
    /// 返回验证结果，包含详细的错误信息
    pub fn validate_columns(
        &self,
        var_cols: &super::types::ColsDef,
        mode: &ValidationMode,
        required_fields: Option<&[String]>,
    ) -> SchemaValidationResult {
        let mut result = SchemaValidationResult::success();

        // 检查变量中的每个字段
        for col in var_cols {
            // 检查字段是否在Schema中定义
            if !self.has_field(&col.name) {
                result.add_error(SchemaValidationError::FieldNotFound(col.name.clone()));
                continue;
            }

            // 检查字段类型是否匹配
            if let Some(schema_type) = self.get_field_type(&col.name) {
                if schema_type != &col.type_ {
                    result.add_error(SchemaValidationError::TypeMismatch(
                        col.name.clone(),
                        schema_type.clone(),
                        col.type_.clone(),
                    ));
                }
            }
        }

        // 根据验证模式进行额外检查
        match mode {
            ValidationMode::Strict => {
                // 严格模式：检查Schema中的所有字段是否都在变量中
                for schema_field in self.get_field_names() {
                    if !var_cols.iter().any(|c| c.name == schema_field) {
                        result.add_error(SchemaValidationError::MissingRequiredField(schema_field));
                    }
                }
            }
            ValidationMode::Lenient => {
                // 宽松模式：不需要额外检查
            }
            ValidationMode::RequiredOnly => {
                // 必需字段模式：检查指定的必需字段是否存在
                if let Some(required) = required_fields {
                    for required_field in required {
                        if !var_cols.iter().any(|c| c.name == *required_field) {
                            result.add_error(SchemaValidationError::MissingRequiredField(
                                required_field.clone(),
                            ));
                        }
                    }
                }
            }
        }

        result
    }

    /// 获取字段的详细信息
    pub fn get_field_info(&self, name: &str) -> Option<(String, &String)> {
        self.fields.get(name).map(|t| (name.to_string(), t))
    }

    /// 检查变量列是否包含Schema中未定义的字段
    pub fn has_extra_fields(&self, var_cols: &super::types::ColsDef) -> Vec<String> {
        var_cols
            .iter()
            .filter(|col| !self.has_field(&col.name))
            .map(|col| col.name.clone())
            .collect()
    }

    /// 获取缺失的字段列表
    pub fn get_missing_fields(&self, var_cols: &super::types::ColsDef) -> Vec<String> {
        self.get_field_names()
            .into_iter()
            .filter(|field| !var_cols.iter().any(|c| c.name == *field))
            .collect()
    }
}

/// Schema管理器
#[derive(Debug, Clone)]
pub struct SchemaManager {
    schemas: HashMap<String, SchemaInfo>,
}

impl SchemaManager {
    /// 创建新的Schema管理器
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// 添加Schema
    pub fn add_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// 获取Schema
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas.get(name)
    }

    /// 列出所有Schema名称
    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// 检查Schema是否存在
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    /// 移除Schema
    pub fn remove_schema(&mut self, name: &str) -> Option<SchemaInfo> {
        self.schemas.remove(name)
    }

    /// 获取所有顶点Schema
    pub fn get_vertex_schemas(&self) -> Vec<&SchemaInfo> {
        self.schemas.values().filter(|s| s.is_vertex).collect()
    }

    /// 获取所有边Schema
    pub fn get_edge_schemas(&self) -> Vec<&SchemaInfo> {
        self.schemas.values().filter(|s| !s.is_vertex).collect()
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
    fn test_schema_info_field_validation() {
        let mut schema = SchemaInfo::new("person".to_string(), true);

        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());

        assert!(schema.validate_field_type("id", "INT"));
        assert!(!schema.validate_field_type("id", "STRING"));
        assert!(!schema.validate_field_type("age", "INT"));
    }

    #[test]
    fn test_schema_manager() {
        let mut manager = SchemaManager::new();

        // 添加顶点Schema
        let mut person_schema = SchemaInfo::new("person".to_string(), true);
        person_schema.add_field("id".to_string(), "INT".to_string());
        person_schema.add_field("name".to_string(), "STRING".to_string());
        manager.add_schema(person_schema);

        // 添加边Schema
        let mut knows_schema = SchemaInfo::new("knows".to_string(), false);
        knows_schema.add_field("since".to_string(), "DATETIME".to_string());
        knows_schema.add_field("weight".to_string(), "DOUBLE".to_string());
        manager.add_schema(knows_schema);

        // 测试基本功能
        assert!(manager.has_schema("person"));
        assert!(manager.has_schema("knows"));
        assert!(!manager.has_schema("company"));

        let schemas = manager.list_schemas();
        assert_eq!(schemas.len(), 2);
        assert!(schemas.contains(&"person".to_string()));
        assert!(schemas.contains(&"knows".to_string()));

        // 测试顶点和边Schema分离
        let vertex_schemas = manager.get_vertex_schemas();
        assert_eq!(vertex_schemas.len(), 1);
        assert_eq!(vertex_schemas[0].name, "person");

        let edge_schemas = manager.get_edge_schemas();
        assert_eq!(edge_schemas.len(), 1);
        assert_eq!(edge_schemas[0].name, "knows");
    }

    #[test]
    fn test_schema_provider_trait() {
        let mut manager = SchemaManager::new();

        let mut schema = SchemaInfo::new("test".to_string(), true);
        schema.add_field("id".to_string(), "INT".to_string());
        manager.add_schema(schema);

        // 测试trait方法
        let provider: &dyn SchemaProvider = &manager;
        assert!(provider.get_schema("test").is_some());
        assert!(provider.get_schema("nonexistent").is_none());

        let schemas = provider.list_schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0], "test");
    }

    #[test]
    fn test_schema_manager_remove() {
        let mut manager = SchemaManager::new();

        let schema = SchemaInfo::new("test".to_string(), true);
        manager.add_schema(schema);

        assert!(manager.has_schema("test"));

        let removed = manager.remove_schema("test");
        assert!(removed.is_some());
        assert!(!manager.has_schema("test"));

        let removed_again = manager.remove_schema("test");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_schema_validation_errors() {
        let error = SchemaValidationError::FieldNotFound("id".to_string());
        assert_eq!(error.to_string(), "字段 'id' 在Schema中不存在");

        let error = SchemaValidationError::TypeMismatch(
            "age".to_string(),
            "INT".to_string(),
            "STRING".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "字段 'age' 类型不匹配: 期望 'INT', 实际 'STRING'"
        );

        let error = SchemaValidationError::MissingRequiredField("name".to_string());
        assert_eq!(error.to_string(), "缺少必需字段 'name'");

        let error = SchemaValidationError::ExtraField("email".to_string());
        assert_eq!(error.to_string(), "变量中包含Schema中未定义的字段 'email'");
    }

    #[test]
    fn test_schema_validation_result() {
        let mut result = SchemaValidationResult::success();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        result.add_error(SchemaValidationError::FieldNotFound("id".to_string()));
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        let failure = SchemaValidationResult::failure(vec![SchemaValidationError::TypeMismatch(
            "age".to_string(),
            "INT".to_string(),
            "STRING".to_string(),
        )]);
        assert!(!failure.is_valid);
        assert_eq!(failure.errors.len(), 1);
    }

    #[test]
    fn test_schema_info_validate_columns() {
        let mut schema = SchemaInfo::new("person".to_string(), true);
        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());
        schema.add_field("age".to_string(), "INT".to_string());

        // 测试完全匹配的情况
        let cols = vec![
            crate::query::context::validate::types::Column::new(
                "id".to_string(),
                "INT".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "name".to_string(),
                "STRING".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "age".to_string(),
                "INT".to_string(),
            ),
        ];

        let result = schema.validate_columns(&cols, &ValidationMode::Strict, None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        // 测试类型不匹配
        let wrong_type_cols = vec![
            crate::query::context::validate::types::Column::new(
                "id".to_string(),
                "STRING".to_string(),
            ), // 错误类型
            crate::query::context::validate::types::Column::new(
                "name".to_string(),
                "STRING".to_string(),
            ),
        ];

        let result = schema.validate_columns(&wrong_type_cols, &ValidationMode::Lenient, None);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        match &result.errors[0] {
            SchemaValidationError::TypeMismatch(field, expected, actual) => {
                assert_eq!(field, "id");
                assert_eq!(expected, "INT");
                assert_eq!(actual, "STRING");
            }
            _ => panic!("期望类型不匹配错误"),
        }

        // 测试缺少字段
        let missing_cols = vec![
            crate::query::context::validate::types::Column::new(
                "id".to_string(),
                "INT".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "name".to_string(),
                "STRING".to_string(),
            ),
            // 缺少 age 字段
        ];

        let result = schema.validate_columns(&missing_cols, &ValidationMode::Strict, None);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        match &result.errors[0] {
            SchemaValidationError::MissingRequiredField(field) => {
                assert_eq!(field, "age");
            }
            _ => panic!("期望缺少必需字段错误"),
        }

        // 测试额外字段
        let extra_cols = vec![
            crate::query::context::validate::types::Column::new(
                "id".to_string(),
                "INT".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "name".to_string(),
                "STRING".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "email".to_string(),
                "STRING".to_string(),
            ), // 额外字段
        ];

        let result = schema.validate_columns(&extra_cols, &ValidationMode::Lenient, None);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        match &result.errors[0] {
            SchemaValidationError::FieldNotFound(field) => {
                assert_eq!(field, "email");
            }
            _ => panic!("期望字段未找到错误"),
        }

        // 测试必需字段模式
        let required_fields = vec!["id".to_string(), "name".to_string()];
        let result = schema.validate_columns(
            &missing_cols,
            &ValidationMode::RequiredOnly,
            Some(&required_fields),
        );
        assert!(result.is_valid); // 只验证必需字段，应该成功
    }

    #[test]
    fn test_schema_info_helper_methods() {
        let mut schema = SchemaInfo::new("person".to_string(), true);
        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());

        let cols = vec![
            crate::query::context::validate::types::Column::new(
                "id".to_string(),
                "INT".to_string(),
            ),
            crate::query::context::validate::types::Column::new(
                "email".to_string(),
                "STRING".to_string(),
            ), // 额外字段
        ];

        // 测试额外字段检测
        let extra_fields = schema.has_extra_fields(&cols);
        assert_eq!(extra_fields.len(), 1);
        assert!(extra_fields.contains(&"email".to_string()));

        // 测试缺失字段检测
        let missing_fields = schema.get_missing_fields(&cols);
        assert_eq!(missing_fields.len(), 1);
        assert!(missing_fields.contains(&"name".to_string()));

        // 测试字段信息获取
        let field_info = schema.get_field_info("id");
        assert!(field_info.is_some());
        let (name, type_) = field_info.expect("Expected field info for 'id' to exist");
        assert_eq!(name, "id");
        assert_eq!(type_, "INT");
    }
}
