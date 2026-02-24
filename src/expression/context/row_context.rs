//! 行表达式上下文实现
//!
//! 为Join操作和行级表达式求值提供专用的上下文实现
//! 支持按列名和列索引访问行数据

use crate::core::Value;
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

impl crate::expression::evaluator::traits::ExpressionContext for RowExpressionContext {
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

    fn is_empty(&self) -> bool {
        self.variables.is_empty()
            && self.row.is_empty()
            && self.col_names.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.variables.len() + self.col_names.len()
    }

    fn variable_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.variables.keys().cloned().collect();
        names.extend(self.col_names.iter().cloned());
        names
    }

    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>> {
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

    fn clear(&mut self) {
        self.row.clear();
        self.col_names.clear();
        self.col_name_index.clear();
        self.variables.clear();
    }

    fn get_function(&self, _name: &str) -> Option<crate::expression::functions::FunctionRef> {
        None
    }

    fn supports_cache(&self) -> bool {
        false
    }

    fn get_cache(&mut self) -> Option<&mut crate::expression::context::CacheManager> {
        None
    }
}

/// 行上下文引用
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
}

/// 行表达式上下文构建器
///
/// 提供流畅的 API 用于构建行表达式上下文
#[derive(Debug)]
pub struct RowExpressionContextBuilder {
    row: Vec<Value>,
    col_names: Vec<String>,
    variables: HashMap<String, Value>,
}

impl RowExpressionContextBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            row: Vec::new(),
            col_names: Vec::new(),
            variables: HashMap::new(),
        }
    }

    /// 设置行数据
    pub fn with_row(mut self, row: Vec<Value>) -> Self {
        self.row = row;
        self
    }

    /// 设置列名
    pub fn with_col_names(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    /// 添加变量
    pub fn with_variable(mut self, name: String, value: Value) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// 构建上下文
    pub fn build(self) -> RowExpressionContext {
        let mut ctx = RowExpressionContext::new(self.row, self.col_names);
        for (name, value) in self.variables {
            ctx.add_variable(name, value);
        }
        ctx
    }
}

impl Default for RowExpressionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
