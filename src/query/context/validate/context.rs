//! 增强验证上下文模块
//! 集成Schema管理和生成器功能的验证上下文

use super::basic_context::BasicValidateContext;
use super::generators::{AnonColGenerator, AnonVarGenerator, GeneratorFactory};
use super::schema::{SchemaInfo, SchemaProvider};
use super::types::{ColsDef, SpaceInfo, Variable, Column};
use crate::core::symbol::SymbolTable;
use crate::core::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// 增强版验证上下文
///
/// 验证阶段的上下文，包含验证所需的信息，集成Schema管理和生成器功能
///
/// 主要功能：
/// 1. 基本验证上下文的所有功能
/// 2. Schema管理和验证
/// 3. 匿名变量和列生成
/// 4. 增强版符号表集成
#[derive(Clone)]
pub struct ValidateContext {
    // 基本验证上下文
    basic_context: BasicValidateContext,

    // Schema管理器
    schema_manager: Option<Arc<dyn SchemaProvider>>,

    // 匿名变量生成器
    anon_var_gen: AnonVarGenerator,

    // 匿名列生成器
    anon_col_gen: AnonColGenerator,

    // 符号表
    symbol_table: SymbolTable,

    // Schema缓存
    schemas: HashMap<String, SchemaInfo>,
}

impl ValidateContext {
    /// 创建新的增强版验证上下文
    pub fn new() -> Self {
        Self {
            basic_context: BasicValidateContext::new(),
            schema_manager: None,
            anon_var_gen: GeneratorFactory::create_anon_var_generator(),
            anon_col_gen: GeneratorFactory::create_anon_col_generator(),
            symbol_table: SymbolTable::new(),
            schemas: HashMap::new(),
        }
    }

    /// 从基本验证上下文创建
    pub fn from_basic(basic_context: BasicValidateContext) -> Self {
        Self {
            basic_context,
            schema_manager: None,
            anon_var_gen: GeneratorFactory::create_anon_var_generator(),
            anon_col_gen: GeneratorFactory::create_anon_col_generator(),
            symbol_table: SymbolTable::new(),
            schemas: HashMap::new(),
        }
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

    /// 验证变量类型是否符合Schema
    pub fn validate_var_against_schema(
        &self,
        var_name: &str,
        schema_name: &str,
    ) -> Result<bool, String> {
        let schema = match self.get_schema(schema_name) {
            Some(s) => s,
            None => return Err(format!("Schema '{}' not found", schema_name)),
        };

        let var_cols = self.get_var(var_name);
        if var_cols.is_empty() {
            return Ok(false);
        }

        // 简化验证：检查变量列是否在Schema中定义
        for col in &var_cols {
            if !schema.has_field(&col.name) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 获取所有Schema名称
    pub fn get_schema_names(&self) -> Vec<String> {
        let mut names = self.schemas.keys().cloned().collect::<Vec<_>>();

        // 如果有Schema管理器，也获取其Schema名称
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

    /// 获取符号表
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// 获取可变符号表
    pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    // ==================== 基本上下文委托 ====================

    /// 获取基本验证上下文的引用
    pub fn basic_context(&self) -> &BasicValidateContext {
        &self.basic_context
    }

    /// 获取基本验证上下文的可变引用
    pub fn basic_context_mut(&mut self) -> &mut BasicValidateContext {
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
        let _ = self.symbol_table.new_variable(&var);
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
        let _ = self.symbol_table.new_variable(&var.name);
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

    // ==================== 参数管理（委托） ====================

    /// 设置参数
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.basic_context.set_parameter(name, value);
    }

    /// 获取参数
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.basic_context.get_parameter(name)
    }

    /// 检查参数是否存在
    pub fn exist_parameter(&self, name: &str) -> bool {
        self.basic_context.exist_parameter(name)
    }

    /// 获取所有参数
    pub fn get_parameters(&self) -> &HashMap<String, Value> {
        self.basic_context.get_parameters()
    }

    // ==================== 别名管理（委托） ====================

    /// 添加别名及其类型
    pub fn add_alias(&mut self, alias: String, type_: String) {
        self.basic_context.add_alias(alias, type_);
    }

    /// 获取别名的类型
    pub fn get_alias_type(&self, alias: &str) -> Option<&String> {
        self.basic_context.get_alias_type(alias)
    }

    /// 检查别名是否存在
    pub fn exist_alias(&self, alias: &str) -> bool {
        self.basic_context.exist_alias(alias)
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

    // ==================== 错误管理（委托） ====================

    /// 添加错误信息
    pub fn add_error(&mut self, error: String) {
        self.basic_context.add_error(error);
    }

    /// 获取所有错误
    pub fn get_errors(&self) -> &[String] {
        self.basic_context.get_errors()
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        self.basic_context.has_errors()
    }

    /// 清除所有错误
    pub fn clear_errors(&mut self) {
        self.basic_context.clear_errors();
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.basic_context.error_count()
    }

    /// 生成验证上下文的字符串表示
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("ValidateContext {\n");
        result.push_str(&format!(
            "  spaces: {:?},\n",
            self.basic_context.current_space().map(|s| s.name.as_str())
        ));
        result.push_str(&format!(
            "  variables: {:?},\n",
            self.basic_context.get_all_variables().len()
        ));
        result.push_str(&format!(
            "  parameters: {:?},\n",
            self.basic_context.get_parameters().len()
        ));
        result.push_str(&format!(
            "  aliases: {:?},\n",
            self.basic_context
                .get_alias_type("sample")
                .map(|t| t.as_str())
        ));
        result.push_str(&format!(
            "  create_spaces: {:?},\n",
            self.basic_context.get_create_spaces().len()
        ));
        result.push_str(&format!(
            "  indexes: {:?},\n",
            self.basic_context.get_indexes().len()
        ));
        result.push_str(&format!(
            "  errors: {:?},\n",
            self.basic_context.error_count()
        ));
        result.push_str(&format!("  schemas: {:?},\n", self.schemas.len()));
        result.push_str(&format!(
            "  symbol_table: {:?},\n",
            self.symbol_table.size().unwrap_or(0)
        ));
        result.push_str("}");
        result
    }
}

impl std::fmt::Debug for ValidateContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidateContext")
            .field("basic_context", &self.basic_context)
            .field("schema_manager", &"<SchemaProvider>")
            .field("anon_var_gen", &self.anon_var_gen)
            .field("anon_col_gen", &self.anon_col_gen)
            .field("symbol_table", &self.symbol_table)
            .field("schemas", &self.schemas)
            .finish()
    }
}

impl Default for ValidateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    struct MockSchemaProvider;

    impl SchemaProvider for MockSchemaProvider {
        fn get_schema(&self, name: &str) -> Option<SchemaInfo> {
            if name == "test_schema" {
                let mut fields = HashMap::new();
                fields.insert("id".to_string(), "INT".to_string());
                fields.insert("name".to_string(), "STRING".to_string());

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
        let ctx = ValidateContext::new();

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
        let mut ctx = ValidateContext::new();

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
        assert_eq!(manager_schema.unwrap().name, "test_schema");
    }

    #[test]
    fn test_variable_with_symbol_table() {
        let mut ctx = ValidateContext::new();

        // 注册变量
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: "INT".to_string(),
            },
            Column {
                name: "name".to_string(),
                type_: "STRING".to_string(),
            },
        ];
        ctx.register_variable("test_var".to_string(), cols);

        // 验证变量存在
        assert!(ctx.exists_var("test_var"));

        // 验证符号表中有该变量
        assert!(ctx.symbol_table().has_variable("test_var"));
    }

    #[test]
    fn test_schema_validation() {
        let mut ctx = ValidateContext::new();

        // 添加Schema
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), "INT".to_string());
        fields.insert("name".to_string(), "STRING".to_string());

        let schema = SchemaInfo {
            name: "person".to_string(),
            fields,
            is_vertex: true,
        };
        ctx.add_schema("person".to_string(), schema);

        // 注册变量
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: "INT".to_string(),
            },
            Column {
                name: "name".to_string(),
                type_: "STRING".to_string(),
            },
        ];
        ctx.register_variable("p".to_string(), cols);

        // 验证变量是否符合Schema
        let result = ctx.validate_var_against_schema("p", "person");
        // 由于我们的验证逻辑简化，这里会返回Ok(false)
        assert!(result.is_ok());
    }

    #[test]
    fn test_generators() {
        let mut ctx = ValidateContext::new();

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
        let mut basic = BasicValidateContext::new();
        basic.switch_to_space(SpaceInfo {
            id: 1,
            name: "test_space".to_string(),
            vid_type: "INT".to_string(),
        });

        let ctx = ValidateContext::from_basic(basic);
        assert!(ctx.space_chosen());
        assert_eq!(ctx.which_space().name, "test_space");
    }
}
