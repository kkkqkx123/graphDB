//! AST上下文 - AST信息管理
//!
//! AST上下文，包含AST相关信息
//! 对应原C++中的AstContext.h/cpp

use std::collections::HashMap;
use std::result::Result;

/// 列定义
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可为空
    pub nullable: bool,
    /// 默认值
    pub default_value: Option<String>,
}

impl ColumnDefinition {
    /// 创建新的列定义
    pub fn new(name: String, data_type: String) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            default_value: None,
        }
    }

    /// 设置是否可为空
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// 设置默认值
    pub fn with_default_value(mut self, default_value: String) -> Self {
        self.default_value = Some(default_value);
        self
    }
}

/// 变量信息
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// 变量类型
    pub var_type: VariableType,
    /// 标签列表（用于顶点和边）
    pub labels: Vec<String>,
    /// 属性定义
    pub properties: HashMap<String, String>,
    /// 是否可选
    pub is_optional: bool,
    /// 作用域开始位置
    pub scope_start: Option<usize>,
    /// 作用域结束位置
    pub scope_end: Option<usize>,
}

/// 变量类型
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    /// 顶点
    Vertex,
    /// 边
    Edge,
    /// 路径
    Path,
    /// 标量值
    Scalar,
    /// 列表
    List,
    /// 映射
    Map,
    /// 数据集
    DataSet,
    /// 未知类型
    Unknown,
}

impl VariableInfo {
    /// 创建新的变量信息
    pub fn new(var_type: VariableType) -> Self {
        Self {
            var_type,
            labels: Vec::new(),
            properties: HashMap::new(),
            is_optional: false,
            scope_start: None,
            scope_end: None,
        }
    }

    /// 添加标签
    pub fn add_label(&mut self, label: String) {
        self.labels.push(label);
    }

    /// 添加属性
    pub fn add_property(&mut self, name: String, data_type: String) {
        self.properties.insert(name, data_type);
    }

    /// 设置作用域
    pub fn set_scope(&mut self, start: usize, end: usize) {
        self.scope_start = Some(start);
        self.scope_end = Some(end);
    }

    /// 检查是否在作用域内
    pub fn in_scope(&self, position: usize) -> bool {
        match (self.scope_start, self.scope_end) {
            (Some(start), Some(end)) => position >= start && position <= end,
            _ => true, // 如果没有定义作用域，则认为始终在作用域内
        }
    }
}

/// 语句接口
pub trait Statement: Send + Sync + std::fmt::Debug {
    /// 语句类型
    fn statement_type(&self) -> &str;
    /// 执行语句
    fn execute(&self) -> Result<(), String>;
}

/// AST上下文，包含AST相关信息
#[derive(Debug, Clone)]
pub struct AstContext {
    /// 查询类型
    pub query_type: String,
    /// 语句
    pub statement: Option<Box<dyn Statement>>,
    /// 变量信息
    variables: HashMap<String, VariableInfo>,
    /// 输出列定义
    output_columns: Vec<ColumnDefinition>,
    /// 输入列定义
    input_columns: Vec<ColumnDefinition>,
    /// 查询文本
    query_text: String,
    /// 是否包含路径查询
    contains_path: bool,
}

impl AstContext {
    /// 创建新的AST上下文
    pub fn new(query_type: String, query_text: String) -> Self {
        let contains_path = query_text.to_lowercase().contains("path");
        
        Self {
            query_type,
            statement: None,
            variables: HashMap::new(),
            output_columns: Vec::new(),
            input_columns: Vec::new(),
            query_text,
            contains_path,
        }
    }

    /// 设置语句
    pub fn set_statement(&mut self, statement: Box<dyn Statement>) {
        self.statement = Some(statement);
    }

    /// 添加变量
    pub fn add_variable(&mut self, name: String, info: VariableInfo) {
        self.variables.insert(name, info);
    }

    /// 获取变量信息
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }

    /// 删除变量
    pub fn remove_variable(&mut self, name: &str) -> Option<VariableInfo> {
        self.variables.remove(name)
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 获取指定类型的变量
    pub fn get_variables_by_type(&self, var_type: &VariableType) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, info)| &info.var_type == var_type)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// 获取指定位置的变量
    pub fn get_variables_at_position(&self, position: usize) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, info)| info.in_scope(position))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// 添加输出列
    pub fn add_output_column(&mut self, column: ColumnDefinition) {
        self.output_columns.push(column);
    }

    /// 获取输出列
    pub fn get_output_column(&self, index: usize) -> Option<&ColumnDefinition> {
        self.output_columns.get(index)
    }

    /// 按名称获取输出列
    pub fn get_output_column_by_name(&self, name: &str) -> Option<&ColumnDefinition> {
        self.output_columns.iter().find(|col| col.name == name)
    }

    /// 获取所有输出列
    pub fn output_columns(&self) -> &[ColumnDefinition] {
        &self.output_columns
    }

    /// 获取输出列数量
    pub fn output_column_count(&self) -> usize {
        self.output_columns.len()
    }

    /// 添加输入列
    pub fn add_input_column(&mut self, column: ColumnDefinition) {
        self.input_columns.push(column);
    }

    /// 获取输入列
    pub fn get_input_column(&self, index: usize) -> Option<&ColumnDefinition> {
        self.input_columns.get(index)
    }

    /// 按名称获取输入列
    pub fn get_input_column_by_name(&self, name: &str) -> Option<&ColumnDefinition> {
        self.input_columns.iter().find(|col| col.name == name)
    }

    /// 获取所有输入列
    pub fn input_columns(&self) -> &[ColumnDefinition] {
        &self.input_columns
    }

    /// 获取输入列数量
    pub fn input_column_count(&self) -> usize {
        self.input_columns.len()
    }

    /// 获取查询文本
    pub fn query_text(&self) -> &str {
        &self.query_text
    }

    /// 检查是否包含路径查询
    pub fn contains_path_query(&self) -> bool {
        self.contains_path
    }

    /// 清除所有变量
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// 清除所有输出列
    pub fn clear_output_columns(&mut self) {
        self.output_columns.clear();
    }

    /// 清除所有输入列
    pub fn clear_input_columns(&mut self) {
        self.input_columns.clear();
    }

    /// 重置AST上下文
    pub fn reset(&mut self) {
        self.statement = None;
        self.clear_variables();
        self.clear_output_columns();
        self.clear_input_columns();
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> &str {
        if let Some(ref stmt) = self.statement {
            stmt.statement_type()
        } else {
            ""
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestStatement {
        stmt_type: String,
    }

    impl TestStatement {
        fn new(stmt_type: String) -> Self {
            Self { stmt_type }
        }
    }

    impl Statement for TestStatement {
        fn statement_type(&self) -> &str {
            &self.stmt_type
        }

        fn execute(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_ast_context_creation() {
        let ctx = AstContext::new(
            "SELECT".to_string(),
            "SELECT * FROM users".to_string(),
        );

        assert_eq!(ctx.query_type, "SELECT");
        assert_eq!(ctx.query_text(), "SELECT * FROM users");
        assert!(!ctx.contains_path_query());
        assert!(ctx.statement.is_none());
        assert_eq!(ctx.variable_names().len(), 0);
        assert_eq!(ctx.output_column_count(), 0);
        assert_eq!(ctx.input_column_count(), 0);
    }

    #[test]
    fn test_ast_context_with_path() {
        let ctx = AstContext::new(
            "FIND PATH".to_string(),
            "FIND PATH FROM a TO b".to_string(),
        );

        assert_eq!(ctx.query_type, "FIND PATH");
        assert!(ctx.contains_path_query());
    }

    #[test]
    fn test_statement_management() {
        let mut ctx = AstContext::new(
            "SELECT".to_string(),
            "SELECT * FROM users".to_string(),
        );

        let stmt = TestStatement::new("SELECT".to_string());
        ctx.set_statement(Box::new(stmt));

        assert!(ctx.statement.is_some());
        assert_eq!(ctx.statement.as_ref().unwrap().statement_type(), "SELECT");
    }

    #[test]
    fn test_variable_management() {
        let mut ctx = AstContext::new(
            "MATCH".to_string(),
            "MATCH (n:Person) RETURN n".to_string(),
        );

        // 添加顶点变量
        let mut vertex_info = VariableInfo::new(VariableType::Vertex);
        vertex_info.add_label("Person".to_string());
        vertex_info.add_property("name".to_string(), "string".to_string());
        vertex_info.set_scope(0, 20);
        ctx.add_variable("n".to_string(), vertex_info);

        // 添加边变量
        let mut edge_info = VariableInfo::new(VariableType::Edge);
        edge_info.add_label("KNOWS".to_string());
        edge_info.set_scope(5, 15);
        ctx.add_variable("e".to_string(), edge_info);

        // 测试变量获取
        assert!(ctx.has_variable("n"));
        assert!(ctx.has_variable("e"));
        assert!(!ctx.has_variable("nonexistent"));

        let n_info = ctx.get_variable("n").unwrap();
        assert_eq!(n_info.var_type, VariableType::Vertex);
        assert!(n_info.labels.contains(&"Person".to_string()));
        assert!(n_info.in_scope(10));

        // 测试按类型获取变量
        let vertex_vars = ctx.get_variables_by_type(&VariableType::Vertex);
        assert_eq!(vertex_vars.len(), 1);
        assert!(vertex_vars.contains(&"n"));

        let edge_vars = ctx.get_variables_by_type(&VariableType::Edge);
        assert_eq!(edge_vars.len(), 1);
        assert!(edge_vars.contains(&"e"));

        // 测试按位置获取变量
        let vars_at_pos = ctx.get_variables_at_position(10);
        assert_eq!(vars_at_pos.len(), 2); // n和e都在位置10的作用域内

        let vars_at_pos = ctx.get_variables_at_position(25);
        assert_eq!(vars_at_pos.len(), 1); // 只有n在位置25的作用域内

        // 测试变量删除
        let removed = ctx.remove_variable("e");
        assert!(removed.is_some());
        assert!(!ctx.has_variable("e"));
    }

    #[test]
    fn test_column_management() {
        let mut ctx = AstContext::new(
            "SELECT".to_string(),
            "SELECT name, age FROM users".to_string(),
        );

        // 添加输出列
        let name_col = ColumnDefinition::new("name".to_string(), "string".to_string());
        let age_col = ColumnDefinition::new("age".to_string(), "integer".to_string())
            .with_nullable(false);
        ctx.add_output_column(name_col);
        ctx.add_output_column(age_col);

        // 添加输入列
        let id_col = ColumnDefinition::new("id".to_string(), "integer".to_string())
            .with_default_value("0".to_string());
        ctx.add_input_column(id_col);

        // 测试输出列
        assert_eq!(ctx.output_column_count(), 2);
        assert_eq!(ctx.get_output_column(0).unwrap().name, "name");
        assert_eq!(ctx.get_output_column(1).unwrap().name, "age");
        assert_eq!(ctx.get_output_column_by_name("name").unwrap().data_type, "string");
        assert!(ctx.get_output_column_by_name("nonexistent").is_none());

        // 测试输入列
        assert_eq!(ctx.input_column_count(), 1);
        assert_eq!(ctx.get_input_column(0).unwrap().name, "id");
        assert_eq!(ctx.get_input_column_by_name("id").unwrap().default_value, Some("0".to_string()));

        // 测试清除
        ctx.clear_output_columns();
        assert_eq!(ctx.output_column_count(), 0);
        assert_eq!(ctx.input_column_count(), 1); // 输入列不受影响
    }

    #[test]
    fn test_reset() {
        let mut ctx = AstContext::new(
            "SELECT".to_string(),
            "SELECT * FROM users".to_string(),
        );

        // 添加一些数据
        ctx.set_statement(Box::new(TestStatement::new("SELECT".to_string())));
        ctx.add_variable("test".to_string(), VariableInfo::new(VariableType::Scalar));
        ctx.add_output_column(ColumnDefinition::new("col".to_string(), "string".to_string()));
        ctx.add_input_column(ColumnDefinition::new("input".to_string(), "integer".to_string()));

        // 重置
        ctx.reset();

        // 检查是否已清除
        assert!(ctx.statement.is_none());
        assert_eq!(ctx.variable_names().len(), 0);
        assert_eq!(ctx.output_column_count(), 0);
        assert_eq!(ctx.input_column_count(), 0);
    }

    #[test]
    fn test_variable_info() {
        let mut info = VariableInfo::new(VariableType::Path);
        info.add_label("Person".to_string());
        info.add_label("Knows".to_string());
        info.add_property("name".to_string(), "string".to_string());
        info.add_property("weight".to_string(), "double".to_string());
        info.set_scope(10, 20);

        assert_eq!(info.var_type, VariableType::Path);
        assert_eq!(info.labels.len(), 2);
        assert!(info.labels.contains(&"Person".to_string()));
        assert!(info.labels.contains(&"Knows".to_string()));
        assert_eq!(info.properties.get("name"), Some(&"string".to_string()));
        assert_eq!(info.properties.get("weight"), Some(&"double".to_string()));
        assert!(info.in_scope(15));
        assert!(!info.in_scope(5));
        assert!(!info.in_scope(25));
    }

    #[test]
    fn test_column_definition() {
        let col = ColumnDefinition::new("test".to_string(), "integer".to_string())
            .with_nullable(false)
            .with_default_value("42".to_string());

        assert_eq!(col.name, "test");
        assert_eq!(col.data_type, "integer");
        assert!(!col.nullable);
        assert_eq!(col.default_value, Some("42".to_string()));
    }
}