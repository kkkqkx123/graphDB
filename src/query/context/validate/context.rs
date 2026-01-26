//! 增强验证上下文模块
//! 集成Schema管理和生成器功能的验证上下文

use super::basic_context::BasicValidationContext;
use super::generators::{AnonColGenerator, AnonVarGenerator, GeneratorFactory};
use super::schema::{SchemaInfo, SchemaProvider};
use super::types::{ColsDef, SpaceInfo, Variable};
use crate::query::context::SymbolTable;
use crate::query::validator::structs::{AliasType, QueryPart};
use crate::query::validator::validation_interface::{
    ValidationContext as ValidationContextTrait, ValidationError,
};
use std::collections::HashMap;
use std::sync::Arc;

/// 增强版验证上下文
///
/// 验证阶段的上下文，包含验证所需的信息，集成Schema管理和生成器功能
///
/// 注意：符号表由 QueryContext 持有，这里通过 sym_table 引用访问
///
/// 主要功能：
/// 1. 基本验证上下文的所有功能
/// 2. Schema管理和验证
/// 3. 匿名变量和列生成
/// 4. 符号表引用（来自 QueryContext）
#[derive(Clone)]
pub struct ValidationContext {
    // 基本验证上下文
    basic_context: BasicValidationContext,

    // Schema管理器
    schema_manager: Option<Arc<dyn SchemaProvider>>,

    // 匿名变量生成器
    anon_var_gen: AnonVarGenerator,

    // 匿名列生成器
    anon_col_gen: AnonColGenerator,

    // 符号表引用（来自 QueryContext）
    sym_table: Option<Arc<SymbolTable>>,

    // Schema缓存
    schemas: HashMap<String, SchemaInfo>,

    // 查询部分列表（用于ValidationContext trait）
    query_parts: Vec<QueryPart>,

    // 别名类型映射（用于ValidationContext trait）
    alias_types: HashMap<String, AliasType>,

    // 验证错误列表（用于ValidationContext trait）
    validation_errors: Vec<ValidationError>,
}

impl ValidationContext {
    /// 创建新的增强版验证上下文
    pub fn new() -> Self {
        Self {
            basic_context: BasicValidationContext::new(),
            schema_manager: None,
            anon_var_gen: GeneratorFactory::create_anon_var_generator(),
            anon_col_gen: GeneratorFactory::create_anon_col_generator(),
            sym_table: None,
            schemas: HashMap::new(),
            query_parts: Vec::new(),
            alias_types: HashMap::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 从基本验证上下文创建
    pub fn from_basic(basic_context: BasicValidationContext) -> Self {
        Self {
            basic_context,
            schema_manager: None,
            anon_var_gen: GeneratorFactory::create_anon_var_generator(),
            anon_col_gen: GeneratorFactory::create_anon_col_generator(),
            sym_table: None,
            schemas: HashMap::new(),
            query_parts: Vec::new(),
            alias_types: HashMap::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 设置符号表引用
    pub fn set_symbol_table(&mut self, sym_table: Arc<SymbolTable>) {
        self.sym_table = Some(sym_table);
    }

    /// 获取符号表引用
    pub fn symbol_table(&self) -> Option<&SymbolTable> {
        self.sym_table.as_deref()
    }

    // ==================== Schema管理 ====================

    /// 设置Schema管理器
    pub fn set_schema_manager(&mut self, manager: Arc<dyn SchemaProvider>) {
        self.schema_manager = Some(manager);
    }

    /// 获取Schema管理器
    pub fn get_schema_manager(&self) -> Option<&Arc<dyn SchemaProvider>> {
        self.schema_manager.as_ref()
    }

    /// 添加Schema
    pub fn add_schema(&mut self, name: String, schema: SchemaInfo) {
        self.schemas.insert(name.clone(), schema);
    }

    /// 获取Schema
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas.get(name)
    }

    /// 从Schema管理器获取Schema
    pub fn get_schema_from_manager(&self, name: &str) -> Option<SchemaInfo> {
        if let Some(manager) = &self.schema_manager {
            manager.get_schema(name)
        } else {
            None
        }
    }

    /// 获取所有Schema名称
    pub fn get_schema_names(&self) -> Vec<String> {
        let mut names = self.schemas.keys().cloned().collect::<Vec<_>>();

        if let Some(manager) = &self.schema_manager {
            let manager_names = manager.list_schemas();
            for name in manager_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names
    }

    // ==================== QueryPart管理 ====================

    /// 添加查询部分
    pub fn add_query_part(&mut self, query_part: QueryPart) {
        self.query_parts.push(query_part);
    }

    /// 获取所有查询部分
    pub fn get_query_parts(&self) -> &[QueryPart] {
        &self.query_parts
    }

    /// 清空查询部分
    pub fn clear_query_parts(&mut self) {
        self.query_parts.clear();
    }

    // ==================== AliasType管理 ====================

    /// 添加别名类型
    pub fn add_alias_type(&mut self, alias: String, alias_type: AliasType) {
        self.alias_types.insert(alias, alias_type);
    }

    /// 获取别名类型
    pub fn get_alias_type(&self, alias: &str) -> Option<&AliasType> {
        self.alias_types.get(alias)
    }

    /// 获取所有别名类型
    pub fn get_alias_types(&self) -> &HashMap<String, AliasType> {
        &self.alias_types
    }

    /// 检查别名是否存在
    pub fn has_alias_type(&self, alias: &str) -> bool {
        self.alias_types.contains_key(alias)
    }

    // ==================== 验证错误管理 ====================

    /// 添加验证错误
    pub fn add_validation_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// 获取所有验证错误
    pub fn get_validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// 检查是否有验证错误
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// 清空验证错误
    pub fn clear_validation_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 获取验证错误数量
    pub fn validation_error_count(&self) -> usize {
        self.validation_errors.len()
    }

    // ==================== 匿名生成器 ====================

    /// 获取匿名变量生成器
    pub fn anon_var_gen(&self) -> &AnonVarGenerator {
        &self.anon_var_gen
    }

    /// 获取匿名列生成器
    pub fn anon_col_gen(&self) -> &AnonColGenerator {
        &self.anon_col_gen
    }

    /// 生成匿名变量名
    pub fn generate_anon_var(&self) -> String {
        self.anon_var_gen.generate()
    }

    /// 生成匿名列名
    pub fn generate_anon_col(&self) -> String {
        self.anon_col_gen.generate()
    }

    /// 重置所有生成器
    pub fn reset_generators(&self) {
        self.anon_var_gen.reset();
        self.anon_col_gen.reset();
    }

    // ==================== 符号表 ====================

    // ==================== 基本上下文委托 ====================

    /// 获取基本验证上下文的引用
    pub fn basic_context(&self) -> &BasicValidationContext {
        &self.basic_context
    }

    /// 获取基本验证上下文的可变引用
    pub fn basic_context_mut(&mut self) -> &mut BasicValidationContext {
        &mut self.basic_context
    }

    // ==================== 空间管理（委托） ====================

    /// 切换到指定的图空间
    pub fn switch_to_space(&mut self, space: SpaceInfo) {
        self.basic_context.switch_to_space(space);
    }

    /// 检查是否已选择空间
    pub fn space_chosen(&self) -> bool {
        self.basic_context.space_chosen()
    }

    /// 获取当前选择的空间
    pub fn which_space(&self) -> &SpaceInfo {
        self.basic_context.which_space()
    }

    /// 获取当前空间（可选）
    pub fn current_space(&self) -> Option<&SpaceInfo> {
        self.basic_context.current_space()
    }

    // ==================== 变量管理（委托） ====================

    /// 注册一个变量（例如MATCH中的别名）
    pub fn register_variable(&mut self, var: String, cols: ColsDef) {
        self.basic_context
            .register_variable(var.clone(), cols.clone());

        // 同时在符号表中注册
        if let Some(sym_table) = &mut self.sym_table {
            let sym_table_mut = Arc::make_mut(sym_table);
            let _ = sym_table_mut.new_variable(&var);
        }
    }

    /// 获取变量的列定义
    pub fn get_var(&self, var: &str) -> ColsDef {
        self.basic_context.get_var(var)
    }

    /// 检查变量是否存在
    pub fn exists_var(&self, var: &str) -> bool {
        self.basic_context.exists_var(var)
    }

    /// 添加变量对象
    pub fn add_variable(&mut self, var: Variable) {
        self.basic_context.add_variable(var.clone());
        if let Some(sym_table) = &mut self.sym_table {
            let sym_table_mut = Arc::make_mut(sym_table);
            let _ = sym_table_mut.new_variable(&var.name);
        }
    }

    /// 获取变量对象
    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.basic_context.get_variable(name)
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        self.basic_context.get_all_variables()
    }

    /// 检查变量是否有某个列
    pub fn var_has_column(&self, var: &str, col: &str) -> bool {
        self.basic_context.var_has_column(var, col)
    }

    // ==================== 空间创建管理（委托） ====================

    /// 添加待创建的空间
    pub fn add_space(&mut self, space_name: String) {
        self.basic_context.add_space(space_name);
    }

    /// 检查空间是否待创建
    pub fn has_space(&self, space_name: &str) -> bool {
        self.basic_context.has_space(space_name)
    }

    /// 获取所有待创建的空间
    pub fn get_create_spaces(&self) -> Vec<String> {
        self.basic_context.get_create_spaces()
    }

    // ==================== 索引管理（委托） ====================

    /// 添加索引
    pub fn add_index(&mut self, index_name: String) {
        self.basic_context.add_index(index_name);
    }

    /// 检查索引是否存在
    pub fn has_index(&self, index_name: &str) -> bool {
        self.basic_context.has_index(index_name)
    }

    /// 获取所有索引
    pub fn get_indexes(&self) -> Vec<String> {
        self.basic_context.get_indexes()
    }

    // ==================== Schema验证方法 ====================

    /// 详细验证变量与Schema的匹配情况
    pub fn validate_var_against_schema_detailed(
        &self,
        var_name: &str,
        schema_name: &str,
        mode: &super::schema::ValidationMode,
        required_fields: Option<&Vec<String>>,
    ) -> Result<super::schema::SchemaValidationResult, String> {
        let schema = self.get_schema(schema_name);
        let columns = self.get_var(var_name);

        if schema.is_none() {
            return Ok(super::schema::SchemaValidationResult::failure(vec![
                super::schema::SchemaValidationError::FieldNotFound(schema_name.to_string()),
            ]));
        }
        let schema = schema.unwrap();
        let mut result = super::schema::SchemaValidationResult::success();

        for col in &columns {
            match mode {
                super::schema::ValidationMode::Strict => {
                    if let Some(expected_type) = schema.get_field_type(&col.name) {
                        if &col.type_ != expected_type {
                            result.add_error(super::schema::SchemaValidationError::TypeMismatch(
                                col.name.clone(),
                                expected_type.clone(),
                                col.type_.clone(),
                            ));
                        }
                    } else {
                        result.add_error(super::schema::SchemaValidationError::FieldNotFound(
                            col.name.clone(),
                        ));
                    }
                }
                super::schema::ValidationMode::Lenient => {
                    if !schema.has_field(&col.name) {
                        result.add_error(super::schema::SchemaValidationError::ExtraField(
                            col.name.clone(),
                        ));
                    }
                }
                super::schema::ValidationMode::RequiredOnly => {
                    if let Some(required) = required_fields {
                        if !required.contains(&col.name) && !schema.has_field(&col.name) {
                            result.add_error(super::schema::SchemaValidationError::ExtraField(
                                col.name.clone(),
                            ));
                        }
                    }
                }
            }
        }

        match mode {
            super::schema::ValidationMode::Strict => {
                for field_name in schema.get_field_names() {
                    let mut found = false;
                    for col in &columns {
                        if col.name == field_name {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        result.add_error(super::schema::SchemaValidationError::MissingRequiredField(
                            field_name.clone(),
                        ));
                    }
                }
            }
            super::schema::ValidationMode::RequiredOnly => {
                if let Some(required) = required_fields {
                    for field_name in required {
                        let mut found = false;
                        for col in &columns {
                            if &col.name == field_name {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            result.add_error(super::schema::SchemaValidationError::MissingRequiredField(
                                field_name.clone(),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(result)
    }

    /// 验证变量的字段类型
    pub fn validate_var_field_types(
        &self,
        var_name: &str,
        schema_name: &str,
    ) -> Result<Vec<String>, String> {
        let schema = self.get_schema(schema_name);
        let columns = self.get_var(var_name);

        if schema.is_none() {
            return Err(format!("Schema '{}' 不存在", schema_name));
        }
        let schema = schema.unwrap();
        let mut errors = Vec::new();

        for col in &columns {
            if let Some(expected_type) = schema.get_field_type(&col.name) {
                if &col.type_ != expected_type {
                    errors.push(format!(
                        "字段 '{}' 类型不匹配: 期望 '{:?}', 实际 '{:?}'",
                        col.name, expected_type, col.type_
                    ));
                }
            }
        }

        Ok(errors)
    }

    /// 检查变量是否缺少Schema必需的字段
    pub fn check_var_missing_fields(
        &self,
        var_name: &str,
        schema_name: &str,
    ) -> Vec<String> {
        let schema = self.get_schema(schema_name);
        let columns = self.get_var(var_name);

        if schema.is_none() {
            return vec![format!("Schema '{}' 不存在", schema_name)];
        }
        let schema = schema.unwrap();
        let mut missing_fields = Vec::new();

        for field_name in schema.get_field_names() {
            let mut found = false;
            for col in &columns {
                if col.name == field_name {
                    found = true;
                    break;
                }
            }
            if !found {
                missing_fields.push(field_name);
            }
        }

        missing_fields
    }

    /// 检查变量是否包含Schema中未定义的额外字段
    pub fn check_var_extra_fields(
        &self,
        var_name: &str,
        schema_name: &str,
    ) -> Vec<String> {
        let schema = self.get_schema(schema_name);
        let columns = self.get_var(var_name);

        if schema.is_none() {
            return vec![format!("Schema '{}' 不存在", schema_name)];
        }
        let schema = schema.unwrap();
        let mut extra_fields = Vec::new();

        for col in &columns {
            if !schema.has_field(&col.name) {
                extra_fields.push(col.name.clone());
            }
        }

        extra_fields
    }

    // ==================== 生成验证上下文的字符串表示 ====================
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("ValidationContext {\n");
        result.push_str(&format!(
            "  spaces: {:?},\n",
            self.basic_context.current_space().map(|s| s.space_name.as_str())
        ));
        result.push_str(&format!(
            "  variables: {:?},\n",
            self.basic_context.get_all_variables().len()
        ));
        result.push_str(&format!(
            "  create_spaces: {:?},\n",
            self.basic_context.get_create_spaces().len()
        ));
        result.push_str(&format!(
            "  indexes: {:?},\n",
            self.basic_context.get_indexes().len()
        ));
        result.push_str(&format!("  schemas: {:?},\n", self.schemas.len()));
        let sym_table_size = self.sym_table.as_ref().map(|t| t.size()).unwrap_or(0);
        result.push_str(&format!(
            "  symbol_table: {:?},\n",
            sym_table_size
        ));
        result.push_str(&format!("  query_parts: {:?},\n", self.query_parts.len()));
        result.push_str(&format!("  alias_types: {:?},\n", self.alias_types.len()));
        result.push_str(&format!(
            "  validation_errors: {:?},\n",
            self.validation_errors.len()
        ));
        result.push_str("}");
        result
    }
}

impl std::fmt::Debug for ValidationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationContext")
            .field("basic_context", &self.basic_context)
            .field("schema_manager", &"<SchemaProvider>")
            .field("anon_var_gen", &self.anon_var_gen)
            .field("anon_col_gen", &self.anon_col_gen)
            .field("symbol_table", &self.sym_table)
            .field("schemas", &self.schemas)
            .field("query_parts", &self.query_parts)
            .field("alias_types", &self.alias_types)
            .field("validation_errors", &self.validation_errors)
            .finish()
    }
}

// 实现ValidationContext trait
impl ValidationContextTrait for ValidationContext {
    fn get_query_parts(&self) -> &[QueryPart] {
        &self.query_parts
    }

    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.alias_types
    }

    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    fn get_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::Column;
    use super::super::schema::ValidationMode;
    use super::*;
    use crate::core::types::DataType;

    struct MockSchemaProvider;

    impl SchemaProvider for MockSchemaProvider {
        fn get_schema(&self, name: &str) -> Option<SchemaInfo> {
            if name == "test_schema" {
                let mut fields = HashMap::new();
                fields.insert("id".to_string(), DataType::Int);
                fields.insert("name".to_string(), DataType::String);

                Some(SchemaInfo {
                    name: name.to_string(),
                    fields,
                    is_vertex: true,
                })
            } else {
                None
            }
        }

        fn list_schemas(&self) -> Vec<String> {
            vec!["test_schema".to_string()]
        }
    }

    #[test]
    fn test_validate_context_creation() {
        let ctx = ValidationContext::new();

        // 测试基本功能
        assert!(!ctx.space_chosen());
        assert!(ctx.get_all_variables().is_empty());
        assert!(!ctx.has_errors());

        // 测试生成器
        let anon_var = ctx.generate_anon_var();
        let anon_col = ctx.generate_anon_col();
        assert!(anon_var.starts_with("__var_"));
        assert!(anon_col.starts_with("__col_"));
    }

    #[test]
    fn test_schema_integration() {
        let mut ctx = ValidationContext::new();

        // 测试Schema管理器
        let schema_manager = Arc::new(MockSchemaProvider);
        ctx.set_schema_manager(schema_manager);
        assert!(ctx.get_schema_manager().is_some());

        // 测试Schema添加和获取
        let schema = SchemaInfo {
            name: "local_schema".to_string(),
            fields: HashMap::new(),
            is_vertex: true,
        };
        ctx.add_schema("local_schema".to_string(), schema.clone());
        assert!(ctx.get_schema("local_schema").is_some());

        // 测试从管理器获取Schema
        let manager_schema = ctx.get_schema_from_manager("test_schema");
        assert!(manager_schema.is_some());
        assert_eq!(
            manager_schema.expect("Expected test schema to exist").name,
            "test_schema"
        );
    }

    #[test]
    fn test_variable_with_symbol_table() {
        let mut ctx = ValidationContext::new();

        // 设置符号表
        let sym_table = Arc::new(SymbolTable::new());
        ctx.set_symbol_table(sym_table.clone());

        // 注册变量
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("test_var".to_string(), cols);

        // 验证变量存在
        assert!(ctx.exists_var("test_var"));

        // 验证符号表中有该变量
        assert!(ctx.symbol_table().expect("符号表已设置").has_variable("test_var"));
    }

    #[test]
    fn test_schema_validation() {
        let mut ctx = ValidationContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), DataType::Int);
        fields.insert("name".to_string(), DataType::String);
        fields.insert("age".to_string(), DataType::Int);

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量 - 完全匹配Schema
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "age".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 测试详细验证 - 严格模式
        let detailed_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Strict,
            None,
        );
        assert!(detailed_result.is_ok());
        let validation_result =
            detailed_result.expect("Expected successful validation in strict mode");
        assert!(validation_result.is_valid);
        assert!(validation_result.errors.is_empty());

        // 测试详细验证 - 宽松模式
        let lenient_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Lenient,
            None,
        );
        assert!(lenient_result.is_ok());
        let validation_result =
            lenient_result.expect("Expected successful validation in lenient mode");
        assert!(validation_result.is_valid);
        assert!(validation_result.errors.is_empty());
    }

    #[test]
    fn test_schema_validation_type_mismatch() {
        let mut ctx = ValidationContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), DataType::Int);
        fields.insert("name".to_string(), DataType::String);

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量 - 类型不匹配
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::String, // 错误：应该是INT
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 测试详细验证 - 严格模式（检测类型不匹配）
        let detailed_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Strict,
            None,
        );
        assert!(detailed_result.is_ok());
        let validation_result =
            detailed_result.expect("Expected successful validation with errors");
        assert!(!validation_result.is_valid);
        assert!(!validation_result.errors.is_empty());

        // 检查错误类型
        let type_errors = ctx
            .validate_var_field_types("p", "person")
            .expect("Expected successful type validation");
        assert!(!type_errors.is_empty());
        assert!(type_errors[0].contains("类型不匹配"));
    }

    #[test]
    fn test_schema_validation_missing_fields() {
        let mut ctx = ValidationContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), DataType::Int);
        fields.insert("name".to_string(), DataType::String);
        fields.insert("age".to_string(), DataType::Int);

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量 - 缺少age字段
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 测试宽松模式 - 应该成功（允许缺少字段）
        let lenient_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Lenient,
            None,
        );
        assert!(lenient_result.is_ok());
        let validation_result =
            lenient_result.expect("Expected successful validation in lenient mode");
        assert!(validation_result.is_valid);

        // 测试严格模式 - 应该失败（不允许缺少字段）
        let strict_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Strict,
            None,
        );
        assert!(strict_result.is_ok());
        let validation_result =
            strict_result.expect("Expected successful validation in strict mode");
        assert!(!validation_result.is_valid);

        // 检查缺失字段
        let missing_fields = ctx.check_var_missing_fields("p", "person");
        assert!(missing_fields.contains(&"age".to_string()));
    }

    #[test]
    fn test_schema_validation_extra_fields() {
        let mut ctx = ValidationContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), DataType::Int);
        fields.insert("name".to_string(), DataType::String);

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量 - 包含额外字段
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "email".to_string(), // 额外字段
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 测试宽松模式 - 应该失败（不允许Schema中未定义的字段）
        let lenient_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::Lenient,
            None,
        );
        assert!(lenient_result.is_ok());
        let validation_result =
            lenient_result.expect("Expected successful validation with extra fields error");
        assert!(!validation_result.is_valid);

        // 检查额外字段
        let extra_fields = ctx.check_var_extra_fields("p", "person");
        assert!(extra_fields.contains(&"email".to_string()));
    }

    #[test]
    fn test_schema_validation_required_only() {
        let mut ctx = ValidationContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), DataType::Int);
        fields.insert("name".to_string(), DataType::String);
        fields.insert("age".to_string(), DataType::Int);
        fields.insert("email".to_string(), DataType::String);

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量 - 只包含必需字段
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: DataType::Int,
                nullable: false,
                default_value: None,
                comment: None,
            },
            Column {
                name: "name".to_string(),
                type_: DataType::String,
                nullable: false,
                default_value: None,
                comment: None,
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 定义必需字段
        let required_fields = vec!["id".to_string(), "name".to_string()];

        // 测试必需字段模式 - 应该成功
        let required_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::RequiredOnly,
            Some(&required_fields),
        );
        assert!(required_result.is_ok());
        let validation_result =
            required_result.expect("Expected successful validation in required only mode");
        assert!(validation_result.is_valid);

        // 测试缺少必需字段的情况
        let required_fields_missing = vec!["id".to_string(), "name".to_string(), "age".to_string()];
        let missing_result = ctx.validate_var_against_schema_detailed(
            "p",
            "person",
            &ValidationMode::RequiredOnly,
            Some(&required_fields_missing),
        );
        assert!(missing_result.is_ok());
        let validation_result =
            missing_result.expect("Expected successful validation with missing fields error");
        assert!(!validation_result.is_valid);
    }

    #[test]
    fn test_generators() {
        let ctx = ValidationContext::new();

        // 测试生成器
        let var1 = ctx.generate_anon_var();
        let var2 = ctx.generate_anon_var();
        assert_ne!(var1, var2);

        let col1 = ctx.generate_anon_col();
        let col2 = ctx.generate_anon_col();
        assert_ne!(col1, col2);

        // 测试重置
        ctx.reset_generators();
        let var3 = ctx.generate_anon_var();
        assert_eq!(var3, "__var_0");
    }

    #[test]
    fn test_from_basic_context() {
        let mut basic = BasicValidationContext::new();
        basic.switch_to_space(SpaceInfo {
            space_id: Some(1),
            space_name: "test_space".to_string(),
            is_default: false,
        });

        let ctx = ValidationContext::from_basic(basic);
        assert!(ctx.space_chosen());
        assert_eq!(ctx.which_space().space_name, "test_space");
    }
}
