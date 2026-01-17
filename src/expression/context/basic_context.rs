//! 基础表达式上下文模块
//!
//! 提供表达式求值过程中的基础上下文实现

use crate::core::context::traits::BaseContext;
use crate::core::context::ContextType;
use crate::core::Value;
use crate::expression::functions::{
    BuiltinFunction, CustomFunction, ExpressionFunction, FunctionRef,
};
use std::collections::HashMap;
use std::sync::Arc;

/// 表达式上下文枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum ExpressionContextType {
    /// 基础表达式上下文
    Basic(BasicExpressionContext),
}

/// 表达式上下文特征
pub trait ExpressionContextCoreExtended {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<&Value>;

    /// 获取函数
    fn get_function(&self, name: &str) -> Option<FunctionRef>;

    /// 检查变量是否存在
    fn has_variable(&self, name: &str) -> bool;

    /// 获取所有变量名
    fn get_variable_names(&self) -> Vec<&str>;

    /// 获取上下文深度
    fn get_depth(&self) -> usize;

    /// 创建子上下文
    fn create_child_context(&self) -> ExpressionContextType;
}

/// 基础表达式上下文
#[derive(Debug)]
pub struct BasicExpressionContext {
    /// 变量绑定
    pub variables: HashMap<String, Value>,
    /// 函数注册表
    pub functions: HashMap<String, BuiltinFunction>,
    /// 自定义函数注册表
    pub custom_functions: HashMap<String, CustomFunction>,
    /// 父上下文
    pub parent: Option<Box<BasicExpressionContext>>,
    /// 上下文深度
    pub depth: usize,
}

impl ExpressionContextCoreExtended for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        // 在当前上下文中查找
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }

        // 如果在当前上下文中找不到，则在父上下文中查找
        if let Some(parent) = &self.parent {
            ExpressionContextCoreExtended::get_variable(parent, name)
        } else {
            None
        }
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        // 在当前上下文中查找内置函数
        if let Some(function) = self.functions.get(name) {
            return Some(FunctionRef::Builtin(function));
        }

        // 然后查找自定义函数
        if let Some(function) = self.custom_functions.get(name) {
            return Some(FunctionRef::Custom(function));
        }

        // 如果在当前上下文中找不到，则在父上下文中查找
        if let Some(parent) = &self.parent {
            parent.get_function(name)
        } else {
            None
        }
    }

    fn has_variable(&self, name: &str) -> bool {
        ExpressionContextCoreExtended::get_variable(self, name).is_some()
    }

    fn get_variable_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.variables.keys().map(|k| k.as_str()).collect();

        // 添加父上下文中的变量名（去重）
        if let Some(parent) = &self.parent {
            let parent_names = parent.get_variable_names();
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names
    }

    fn get_depth(&self) -> usize {
        self.depth
    }

    fn create_child_context(&self) -> ExpressionContextType {
        ExpressionContextType::Basic(BasicExpressionContext {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: Some(Box::new(self.clone())),
            depth: self.get_depth() + 1,
        })
    }
}

impl ExpressionContextCoreExtended for Box<BasicExpressionContext> {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        (**self).get_variable(name)
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        (**self).get_function(name)
    }

    fn has_variable(&self, name: &str) -> bool {
        (**self).has_variable(name)
    }

    fn get_variable_names(&self) -> Vec<&str> {
        (**self).get_variable_names()
    }

    fn get_depth(&self) -> usize {
        (**self).get_depth()
    }

    fn create_child_context(&self) -> ExpressionContextType {
        (**self).create_child_context()
    }
}

impl ExpressionContextCoreExtended for ExpressionContextType {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        match self {
            ExpressionContextType::Basic(ctx) => {
                ExpressionContextCoreExtended::get_variable(ctx, name)
            }
        }
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_function(name),
        }
    }

    fn has_variable(&self, name: &str) -> bool {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.has_variable(name),
        }
    }

    fn get_variable_names(&self) -> Vec<&str> {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_variable_names(),
        }
    }

    fn get_depth(&self) -> usize {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_depth(),
        }
    }

    fn create_child_context(&self) -> ExpressionContextType {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.create_child_context(),
        }
    }
}

impl BasicExpressionContext {
    /// 创建新的基础表达式上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: None,
            depth: 0,
        }
    }

    /// 创建带父上下文的基础表达式上下文
    pub fn with_parent(parent: BasicExpressionContext) -> Self {
        let parent_depth = parent.get_depth();
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: Some(Box::new(parent)),
            depth: parent_depth + 1,
        }
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: impl Into<String>, value: Value) {
        self.variables.insert(name.into(), value);
    }

    /// 批量设置变量
    pub fn set_variables(&mut self, variables: HashMap<String, Value>) {
        self.variables = variables;
    }

    /// 注册内置函数
    pub fn register_builtin_function(&mut self, function: BuiltinFunction) {
        self.functions.insert(function.name().to_string(), function);
    }

    /// 注册自定义函数
    pub fn register_custom_function(&mut self, function: CustomFunction) {
        self.custom_functions
            .insert(function.name.clone(), function);
    }

    /// 获取内置函数
    pub fn get_builtin_function(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }

    /// 获取自定义函数
    pub fn get_custom_function(&self, name: &str) -> Option<&CustomFunction> {
        self.custom_functions.get(name)
    }

    /// 移除变量
    pub fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 清空所有变量
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// 检查变量是否在当前上下文中定义
    pub fn is_local_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取当前上下文中的变量名
    pub fn get_local_variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for BasicExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BasicExpressionContext {
    fn clone(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            custom_functions: self.custom_functions.clone(),
            parent: self.parent.clone(),
            depth: self.get_depth(),
        }
    }
}

impl BaseContext for BasicExpressionContext {
    fn id(&self) -> &str {
        "expression_context"
    }

    fn context_type(&self) -> ContextType {
        ContextType::Expression
    }

    fn created_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    fn updated_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn touch(&mut self) {}

    fn invalidate(&mut self) {}

    fn revalidate(&mut self) -> bool {
        true
    }

    fn parent_id(&self) -> Option<&str> {
        self.parent.as_ref().map(|_| "parent_expression")
    }

    fn depth(&self) -> usize {
        self.depth
    }
}

// 为BasicExpressionContext实现统一的ExpressionContext trait
impl crate::expression::evaluator::traits::ExpressionContext for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<crate::core::Value> {
        ExpressionContextCoreExtended::get_variable(self, name).cloned()
    }

    fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.set_variable(name, value);
    }

    fn get_vertex(&self) -> Option<&crate::core::Vertex> {
        // BasicExpressionContext没有直接的vertex字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn get_edge(&self) -> Option<&crate::core::Edge> {
        // BasicExpressionContext没有直接的edge字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn get_path(&self, _name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        // BasicExpressionContext没有path字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn set_vertex(&mut self, vertex: crate::core::Vertex) {
        self.set_variable(
            "_vertex".to_string(),
            crate::core::Value::Vertex(Box::new(vertex)),
        );
    }

    fn set_edge(&mut self, edge: crate::core::Edge) {
        self.set_variable("_edge".to_string(), crate::core::Value::Edge(edge));
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        self.set_variable(name, crate::core::Value::Path(path));
    }

    fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.variables.len()
    }

    fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, crate::core::Value>> {
        let mut value_map = std::collections::HashMap::new();
        for (name, value) in &self.variables {
            value_map.insert(name.clone(), value.clone());
        }
        Some(value_map)
    }

    fn clear(&mut self) {
        self.variables.clear();
    }

    fn get_variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }
}
