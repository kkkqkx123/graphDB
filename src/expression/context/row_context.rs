//! 行表达式上下文实现
//!
//! 为Join操作和行级表达式求值提供专用的上下文实现
//! 支持按列名和列索引访问行数据

use crate::core::Value;
use crate::expression::context::traits::*;
use std::collections::HashMap;

/// 行表达式上下文
///
/// 专门用于在行数据上求值表达式的上下文实现
/// 支持两种访问模式：
/// 1. 按列名访问：通过 col_names 映射
/// 2. 按变量名访问：通过 variables 映射
#[derive(Debug, Clone)]
pub struct RowExpressionContext {
    /// 当前行数据
    row: Vec<Value>,
    /// 列名到索引的映射
    col_names: Vec<String>,
    /// 列名索引映射（快速查找）
    col_name_index: HashMap<String, usize>,
    /// 额外变量（用于存储计算中间结果）
    variables: HashMap<String, Value>,
}

impl RowExpressionContext {
    /// 创建新的行上下文
    pub fn new(row: Vec<Value>, col_names: Vec<String>) -> Self {
        let col_name_index: HashMap<String, usize> = col_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        Self {
            row,
            col_names,
            col_name_index,
            variables: HashMap::new(),
        }
    }

    /// 从现有数据创建上下文
    pub fn from_dataset(row: &[Value], col_names: &[String]) -> Self {
        Self::new(row.to_vec(), col_names.to_vec())
    }

    /// 获取当前行引用
    pub fn row(&self) -> &Vec<Value> {
        &self.row
    }

    /// 获取当前行的可变引用
    pub fn row_mut(&mut self) -> &mut Vec<Value> {
        &mut self.row
    }

    /// 按列索引获取值
    pub fn get_value_by_index(&self, index: usize) -> Option<&Value> {
        self.row.get(index)
    }

    /// 按列名获取值
    pub fn get_value_by_name(&self, name: &str) -> Option<&Value> {
        self.col_name_index
            .get(name)
            .and_then(|&idx| self.row.get(idx))
    }

    /// 检查列是否存在
    pub fn has_column(&self, name: &str) -> bool {
        self.col_name_index.contains_key(name)
    }

    /// 获取列索引
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.col_name_index.get(name).copied()
    }

    /// 获取所有列名
    pub fn get_col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取列数量
    pub fn num_columns(&self) -> usize {
        self.col_names.len()
    }

    /// 获取行大小
    pub fn row_size(&self) -> usize {
        self.row.len()
    }

    /// 添加计算变量
    pub fn add_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 转换为普通表达式上下文
    pub fn to_default_context(&self) -> crate::expression::context::default_context::DefaultExpressionContext {
        let mut ctx = crate::expression::context::default_context::DefaultExpressionContext::new();
        for (name, value) in &self.variables {
            ctx = ctx.add_variable(name.clone(), value.clone());
        }
        ctx
    }
}

impl VariableContext for RowExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        // 首先检查变量映射
        if let Some(value) = self.variables.get(name) {
            return Some(value.clone());
        }

        // 然后检查列名（支持将列名作为变量访问）
        if let Some(value) = self.get_value_by_name(name) {
            return Some(value.clone());
        }

        None
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_variable_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.variables.keys().map(|s| s.as_str()).collect();
        names.extend(self.col_names.iter().map(|s| s.as_str()));
        names
    }

    fn variable_count(&self) -> usize {
        self.variables.len() + self.col_names.len()
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        let mut all_vars = HashMap::new();
        for (name, value) in &self.variables {
            all_vars.insert(name.clone(), value.clone());
        }
        for (name, &idx) in &self.col_name_index {
            if let Some(value) = self.row.get(idx) {
                all_vars.insert(name.clone(), value.clone());
            }
        }
        Some(all_vars)
    }

    fn clear_variables(&mut self) {
        self.row.clear();
        self.col_names.clear();
        self.col_name_index.clear();
        self.variables.clear();
    }
}

impl GraphContext for RowExpressionContext {
    fn get_vertex(&self) -> Option<&crate::core::Vertex> {
        None
    }

    fn get_edge(&self) -> Option<&crate::core::Edge> {
        None
    }

    fn get_path(&self, _name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        None
    }

    fn set_vertex(&mut self, _vertex: crate::core::Vertex) {
        // 不支持
    }

    fn set_edge(&mut self, _edge: crate::core::Edge) {
        // 不支持
    }

    fn add_path(&mut self, _name: String, _path: crate::core::vertex_edge_path::Path) {
        // 不支持
    }
}

impl FunctionContext for RowExpressionContext {
    fn get_function(&self, _name: &str) -> Option<crate::expression::functions::FunctionRef> {
        // RowExpressionContext 不支持函数注册
        None
    }

    fn get_function_names(&self) -> Vec<&str> {
        Vec::new()
    }
}

impl CacheContext for RowExpressionContext {
    fn get_regex(&mut self, _pattern: &str) -> Option<&regex::Regex> {
        // RowExpressionContext 不支持缓存
        None
    }
}

impl ScopedContext for RowExpressionContext {
    fn get_depth(&self) -> usize {
        0
    }

    fn create_child_context(&self) -> Box<dyn ExpressionContext> {
        Box::new(Self::new(self.row.clone(), self.col_names.clone()))
    }
}

impl ExpressionContext for RowExpressionContext {
    fn is_empty(&self) -> bool {
        self.row.is_empty() && self.variables.is_empty()
    }

    fn clear(&mut self) {
        self.row.clear();
        self.col_names.clear();
        self.col_name_index.clear();
        self.variables.clear();
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for RowExpressionContext {
    fn get_variable(&self, name: &str) -> Option<crate::core::Value> {
        VariableContext::get_variable(self, name)
    }

    fn set_variable(&mut self, name: String, value: crate::core::Value) {
        VariableContext::set_variable(self, name, value);
    }

    fn get_vertex(&self) -> Option<&crate::core::Vertex> {
        GraphContext::get_vertex(self)
    }

    fn get_edge(&self) -> Option<&crate::core::Edge> {
        GraphContext::get_edge(self)
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        GraphContext::get_path(self, name)
    }

    fn set_vertex(&mut self, _vertex: crate::core::Vertex) {
        // 不支持
    }

    fn set_edge(&mut self, _edge: crate::core::Edge) {
        // 不支持
    }

    fn add_path(&mut self, _name: String, _path: crate::core::vertex_edge_path::Path) {
        // 不支持
    }

    fn is_empty(&self) -> bool {
        ExpressionContext::is_empty(self)
    }

    fn variable_count(&self) -> usize {
        VariableContext::variable_count(self)
    }

    fn variable_names(&self) -> Vec<String> {
        VariableContext::get_variable_names(self)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, crate::core::Value>> {
        VariableContext::get_all_variables(self)
    }

    fn clear(&mut self) {
        ExpressionContext::clear(self);
    }
}

/// 简化的行上下文引用
///
/// 避免克隆的轻量级上下文引用，适用于只需要读取的场景
#[derive(Debug, Clone)]
pub struct RowContextRef<'a> {
    row: &'a [Value],
    col_names: &'a [String],
}

impl<'a> RowContextRef<'a> {
    /// 创建行上下文引用
    pub fn new(row: &'a [Value], col_names: &'a [String]) -> Self {
        Self { row, col_names }
    }

    /// 按列名获取值
    pub fn get_value_by_name(&self, name: &str) -> Option<&Value> {
        self.col_names
            .iter()
            .position(|n| n == name)
            .and_then(|idx| self.row.get(idx))
    }

    /// 按列索引获取值
    pub fn get_value_by_index(&self, index: usize) -> Option<&Value> {
        self.row.get(index)
    }

    /// 检查列是否存在
    pub fn has_column(&self, name: &str) -> bool {
        self.col_names.iter().any(|n| n == name)
    }
}

/// 用于Join操作的构建器
pub struct RowExpressionContextBuilder {
    col_names: Vec<String>,
}

impl RowExpressionContextBuilder {
    pub fn new() -> Self {
        Self {
            col_names: Vec::new(),
        }
    }

    pub fn with_columns(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    pub fn build(&self, row: Vec<Value>) -> RowExpressionContext {
        RowExpressionContext::new(row, self.col_names.clone())
    }
}

impl Default for RowExpressionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_row_context_basic() {
        let row = vec![Value::Int(1), Value::String("Alice".to_string()), Value::Int(25)];
        let col_names = vec!["id".to_string(), "name".to_string(), "age".to_string()];

        let ctx = RowExpressionContext::new(row, col_names.clone());

        // 测试按列名获取
        assert_eq!(ctx.get_value_by_name("id"), Some(&Value::Int(1)));
        assert_eq!(ctx.get_value_by_name("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(ctx.get_value_by_name("age"), Some(&Value::Int(25)));

        // 测试按索引获取
        assert_eq!(ctx.get_value_by_index(0), Some(&Value::Int(1)));
        assert_eq!(ctx.get_value_by_index(1), Some(&Value::String("Alice".to_string())));

        // 测试列不存在
        assert_eq!(ctx.get_value_by_name("nonexistent"), None);
        assert_eq!(ctx.get_value_by_index(100), None);
    }

    #[test]
    fn test_row_context_expression_context() {
        let row = vec![Value::Int(1), Value::String("Alice".to_string())];
        let col_names = vec!["id".to_string(), "name".to_string()];

        let ctx = RowExpressionContext::new(row, col_names);

        // 测试ExpressionContext接口
        assert_eq!(ctx.get_variable("id"), Some(Value::Int(1)));
        assert_eq!(ctx.get_variable("name"), Some(Value::String("Alice".to_string())));
        assert!(ctx.has_variable("id"));
        assert!(!ctx.has_variable("nonexistent"));
    }

    #[test]
    fn test_row_context_with_variables() {
        let row = vec![Value::Int(1)];
        let col_names = vec!["id".to_string()];

        let mut ctx = RowExpressionContext::new(row, col_names);
        ctx.add_variable("computed".to_string(), Value::Float(3.14));

        assert_eq!(ctx.get_variable("computed"), Some(Value::Float(3.14)));
        assert_eq!(ctx.get_variable("id"), Some(Value::Int(1)));
    }

    #[test]
    fn test_row_context_ref() {
        let row = vec![Value::Int(42), Value::String("test".to_string())];
        let col_names = vec!["col1".to_string(), "col2".to_string()];

        let ctx_ref = RowContextRef::new(&row, &col_names);

        assert_eq!(ctx_ref.get_value_by_name("col1"), Some(&Value::Int(42)));
        assert_eq!(ctx_ref.get_value_by_index(1), Some(&Value::String("test".to_string())));
        assert!(ctx_ref.has_column("col1"));
        assert!(!ctx_ref.has_column("col3"));
    }

    #[test]
    fn test_row_expression_context_builder() {
        let builder = RowExpressionContextBuilder::new()
            .with_columns(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let ctx = builder.build(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        assert_eq!(ctx.num_columns(), 3);
        assert_eq!(ctx.get_value_by_name("a"), Some(&Value::Int(1)));
        assert_eq!(ctx.get_value_by_name("b"), Some(&Value::Int(2)));
        assert_eq!(ctx.get_value_by_name("c"), Some(&Value::Int(3)));
    }
}
