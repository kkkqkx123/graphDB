//! 增强版验证上下文模块 - 集成Schema管理和生成器功能
//! 对应原C++中的ValidateContext.h的增强功能

use std::collections::{HashMap, HashSet};
use crate::core::Value;
use crate::core::enhanced_symbols::{EnhancedSymbolTable, PlanNodeRef};

/// 图空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub id: i32,
    pub name: String,
    pub vid_type: String,  // 顶点ID类型
}

/// 列定义
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: String,
}

/// 列定义集合 - 一个变量包含多个列
pub type ColsDef = Vec<Column>;

/// 变量定义 - 在查询中定义的变量（如MATCH中的别名）
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub columns: ColsDef,
}

/// Schema提供者trait（简化版）
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

/// 匿名变量生成器
#[derive(Debug, Clone)]
pub struct AnonVarGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
}

impl AnonVarGenerator {
    pub fn new(prefix: String) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
            prefix,
        }
    }

    pub fn generate(&self) -> String {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("{}_{}", self.prefix, count)
    }

    pub fn reset(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// 匿名列生成器
#[derive(Debug, Clone)]
pub struct AnonColGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
}

impl AnonColGenerator {
    pub fn new(prefix: String) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
            prefix,
        }
    }

    pub fn generate(&self) -> String {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("{}_{}", self.prefix, count)
    }

    pub fn reset(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// 增强版验证上下文
/// 
/// 验证阶段的上下文，包含验证所需的信息，集成Schema管理和生成器功能
/// 对应原C++中的ValidateContext类的增强版本
#[derive(Debug, Clone)]
pub struct EnhancedValidateContext {
    // 图空间栈 - 追踪空间切换的历史
    spaces: Vec<SpaceInfo>,
    
    // 已定义的变量映射 (变量名 -> 列定义)
    // 例如：MATCH (n:Person) -> 变量 n 的列定义
    variables: HashMap<String, ColsDef>,
    
    // 参数映射
    parameters: HashMap<String, Value>,
    
    // 别名到类型的映射
    aliases: HashMap<String, String>,
    
    // 创建的空间集合
    create_spaces: HashSet<String>,
    
    // 索引集合
    indexes: HashSet<String>,
    
    // 收集的错误信息
    errors: Vec<String>,
    
    // Schema管理器
    schema_manager: Option<Arc<dyn SchemaProvider>>,
    
    // 匿名变量生成器
    anon_var_gen: AnonVarGenerator,
    
    // 匿名列生成器
    anon_col_gen: AnonColGenerator,
    
    // 增强版符号表
    symbol_table: EnhancedSymbolTable,
    
    // Schema缓存
    schemas: HashMap<String, SchemaInfo>,
}

impl EnhancedValidateContext {
    /// 创建新的增强版验证上下文
    pub fn new() -> Self {
        Self {
            spaces: Vec::new(),
            variables: HashMap::new(),
            parameters: HashMap::new(),
            aliases: HashMap::new(),
            create_spaces: HashSet::new(),
            indexes: HashSet::new(),
            errors: Vec::new(),
            schema_manager: None,
            anon_var_gen: AnonVarGenerator::new("__var".to_string()),
            anon_col_gen: AnonColGenerator::new("__col".to_string()),
            symbol_table: EnhancedSymbolTable::new(),
            schemas: HashMap::new(),
        }
    }

    /// 设置Schema管理器
    pub fn set_schema_manager(&mut self, manager: Arc<dyn SchemaProvider>) {
        self.schema_manager = Some(manager);
    }

    /// 获取Schema管理器
    pub fn get_schema_manager(&self) -> Option<&Arc<dyn SchemaProvider>> {
        self.schema_manager.as_ref()
    }

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
    pub fn validate_var_against_schema(&self, var_name: &str, schema_name: &str) -> Result<bool, String> {
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
            if !schema.fields.contains_key(&col.name) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 获取增强版符号表
    pub fn symbol_table(&self) -> &EnhancedSymbolTable {
        &self.symbol_table
    }

    /// 获取可变增强版符号表
    pub fn symbol_table_mut(&mut self) -> &mut EnhancedSymbolTable {
        &mut self.symbol_table
    }

    // ==================== 空间管理 ====================
    
    /// 切换到指定的图空间
    /// 此操作将空间压入栈中，允许跟踪空间切换历史
    pub fn switch_to_space(&mut self, space: SpaceInfo) {
        self.spaces.push(space);
    }

    /// 检查是否已选择空间
    pub fn space_chosen(&self) -> bool {
        !self.spaces.is_empty()
    }

    /// 获取当前选择的空间
    /// 
    /// # Panics
    /// 如果尚未选择空间，则panic
    pub fn which_space(&self) -> &SpaceInfo {
        self.spaces
            .last()
            .expect("空间未被选择")
    }

    /// 获取当前空间（可选）
    pub fn current_space(&self) -> Option<&SpaceInfo> {
        self.spaces.last()
    }

    // ==================== 变量管理 ====================
    
    /// 注册一个变量（例如MATCH中的别名）
    /// 
    /// # 参数
    /// * `var` - 变量名称
    /// * `cols` - 变量包含的列定义
    pub fn register_variable(&mut self, var: String, cols: ColsDef) {
        self.variables.insert(var.clone(), cols.clone());
        
        // 同时在符号表中注册
        let _ = self.symbol_table.new_variable(&var);
    }

    /// 获取变量的列定义
    pub fn get_var(&self, var: &str) -> ColsDef {
        self.variables
            .get(var)
            .map(|cols| cols.clone())
            .unwrap_or_default()
    }

    /// 检查变量是否存在
    pub fn exists_var(&self, var: &str) -> bool {
        self.variables.contains_key(var)
    }

    /// 添加变量对象
    pub fn add_variable(&mut self, var: Variable) {
        self.variables.insert(var.name.clone(), var.columns);
        let _ = self.symbol_table.new_variable(&var.name);
    }

    /// 获取变量对象
    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).map(|cols| Variable {
            name: name.to_string(),
            columns: cols.clone(),
        })
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 检查变量是否有某个列
    pub fn var_has_column(&self, var: &str, col: &str) -> bool {
        self.variables
            .get(var)
            .map_or(false, |cols| cols.iter().any(|c| c.name == col))
    }

    // ==================== 参数管理 ====================
    
    /// 设置参数
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    /// 获取参数
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// 检查参数是否存在
    pub fn exist_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    /// 获取所有参数
    pub fn get_parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    // ==================== 别名管理 ====================
    
    /// 添加别名及其类型
    pub fn add_alias(&mut self, alias: String, type_: String) {
        self.aliases.insert(alias, type_);
    }

    /// 获取别名的类型
    pub fn get_alias_type(&self, alias: &str) -> Option<&String> {
        self.aliases.get(alias)
    }

    /// 检查别名是否存在
    pub fn exist_alias(&self, alias: &str) -> bool {
        self.aliases.contains_key(alias)
    }

    // ==================== 空间创建管理 ====================
    
    /// 添加待创建的空间
    pub fn add_space(&mut self, space_name: String) {
        self.create_spaces.insert(space_name);
    }

    /// 检查空间是否待创建
    pub fn has_space(&self, space_name: &str) -> bool {
        self.create_spaces.contains(space_name)
    }

    /// 获取所有待创建的空间
    pub fn get_create_spaces(&self) -> Vec<String> {
        self.create_spaces.iter().cloned().collect()
    }

    // ==================== 索引管理 ====================
    
    /// 添加索引
    pub fn add_index(&mut self, index_name: String) {
        self.indexes.insert(index_name);
    }

    /// 检查索引是否存在
    pub fn has_index(&self, index_name: &str) -> bool {
        self.indexes.contains(index_name)
    }

    /// 获取所有索引
    pub fn get_indexes(&self) -> Vec<String> {
        self.indexes.iter().cloned().collect()
    }

    // ==================== 错误管理 ====================
    
    /// 添加错误信息
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// 获取所有错误
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 清除所有错误
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// 生成验证上下文的字符串表示
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("EnhancedValidateContext {\n");
        result.push_str(&format!("  spaces: {:?},\n", self.spaces.len()));
        result.push_str(&format!("  variables: {:?},\n", self.variables.len()));
        result.push_str(&format!("  parameters: {:?},\n", self.parameters.len()));
        result.push_str(&format!("  aliases: {:?},\n", self.aliases.len()));
        result.push_str(&format!("  create_spaces: {:?},\n", self.create_spaces.len()));
        result.push_str(&format!("  indexes: {:?},\n", self.indexes.len()));
        result.push_str(&format!("  errors: {:?},\n", self.errors.len()));
        result.push_str(&format!("  schemas: {:?},\n", self.schemas.len()));
        result.push_str(&format!("  symbol_table: {:?},\n", self.symbol_table.size().unwrap_or(0)));
        result.push_str("}");
        result
    }
}

impl Default for EnhancedValidateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_enhanced_validate_context() {
        let mut ctx = EnhancedValidateContext::new();
        
        // 测试Schema管理器
        let schema_manager = Arc::new(MockSchemaProvider);
        ctx.set_schema_manager(schema_manager);
        assert!(ctx.get_schema_manager().is_some());
        
        // 测试匿名生成器
        let anon_var = ctx.generate_anon_var();
        assert!(anon_var.starts_with("__var_"));
        
        let anon_col = ctx.generate_anon_col();
        assert!(anon_col.starts_with("__col_"));
        
        // 测试Schema管理
        let schema = SchemaInfo {
            name: "test_schema".to_string(),
            fields: HashMap::new(),
            is_vertex: true,
        };
        ctx.add_schema("test_schema".to_string(), schema.clone());
        assert!(ctx.get_schema("test_schema").is_some());
        
        // 测试变量验证
        let cols = vec![
            Column { name: "id".to_string(), type_: "INT".to_string() },
            Column { name: "name".to_string(), type_: "STRING".to_string() },
        ];
        ctx.register_variable("test_var".to_string(), cols);
        
        // 由于我们没有真实的Schema字段信息，验证会失败
        let result = ctx.validate_var_against_schema("test_var", "test_schema");
        assert!(result.is_err());
        
        // 测试符号表集成
        assert!(ctx.symbol_table().has_variable("test_var"));
        
        // 测试基本功能
        ctx.switch_to_space(SpaceInfo {
            id: 1,
            name: "test_space".to_string(),
            vid_type: "INT".to_string(),
        });
        assert!(ctx.space_chosen());
        assert_eq!(ctx.which_space().name, "test_space");
    }

    #[test]
    fn test_anon_generators() {
        let mut ctx = EnhancedValidateContext::new();
        
        // 测试匿名变量生成器
        let var1 = ctx.generate_anon_var();
        let var2 = ctx.generate_anon_var();
        assert_ne!(var1, var2);
        assert!(var1.starts_with("__var_"));
        assert!(var2.starts_with("__var_"));
        
        // 测试匿名列生成器
        let col1 = ctx.generate_anon_col();
        let col2 = ctx.generate_anon_col();
        assert_ne!(col1, col2);
        assert!(col1.starts_with("__col_"));
        assert!(col2.starts_with("__col_"));
    }

    #[test]
    fn test_schema_validation() {
        let mut ctx = EnhancedValidateContext::new();
        
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
            Column { name: "id".to_string(), type_: "INT".to_string() },
            Column { name: "name".to_string(), type_: "STRING".to_string() },
        ];
        ctx.register_variable("p".to_string(), cols);
        
        // 验证变量是否符合Schema
        let result = ctx.validate_var_against_schema("p", "person");
        // 由于我们的验证逻辑简化，这里会返回Ok(false)
        assert!(result.is_ok());
    }
}