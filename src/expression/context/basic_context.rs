//! 基础表达式上下文模块
//!
//! 提供表达式求值过程中的基础上下文实现

use crate::core::Value;
use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::expression::functions::{
    BuiltinFunction, CustomFunction, ExpressionFunction, FunctionRef,
};
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

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
    /// 正则表达式缓存
    pub regex_cache: HashMap<String, Regex>,
    /// 内部变量（用于 ListComprehension、Predicate 等表达式）
    pub inner_variables: HashMap<String, Value>,
    /// 路径存储（使用 Rc 以支持引用返回）
    pub paths: HashMap<String, Rc<crate::core::vertex_edge_path::Path>>,
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
            regex_cache: HashMap::new(),
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
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
            regex_cache: HashMap::new(),
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
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
            regex_cache: HashMap::new(),
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
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

    /// 设置内部变量（用于 ListComprehension、Predicate 等表达式）
    pub fn set_inner_var(&mut self, name: &str, value: Value) {
        self.inner_variables.insert(name.to_string(), value);
    }

    /// 获取内部变量
    pub fn get_inner_var(&self, name: &str) -> Option<&Value> {
        self.inner_variables.get(name)
    }

    /// 获取正则表达式（自动缓存）
    pub fn get_regex(&mut self, pattern: &str) -> Option<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            if let Ok(regex) = Regex::new(pattern) {
                self.regex_cache.insert(pattern.to_string(), regex);
            } else {
                return None;
            }
        }
        self.regex_cache.get(pattern)
    }

    /// 获取变量属性值
    pub fn get_var_prop(&self, var: &str, prop: &str) -> Option<Value> {
        if let Some(var_value) = self.get_variable(var) {
            if let Value::Map(map) = var_value {
                map.get(prop).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 获取指定版本的变量值
    pub fn get_versioned_var(&self, name: &str, version: i64) -> Option<Value> {
        if let Some(var_value) = self.get_variable(name) {
            if let Value::List(list) = var_value {
                let size = list.len() as i64;
                if version <= 0 {
                    let idx = (-version).min(size - 1) as usize;
                    Some(list[idx].clone())
                } else {
                    let idx = (size - version).max(0) as usize;
                    if idx < list.len() {
                        Some(list[idx].clone())
                    } else {
                        None
                    }
                }
            } else {
                Some(var_value.clone())
            }
        } else {
            None
        }
    }

    /// 获取变量属性在元组中的索引
    pub fn get_var_prop_index(&self, _var: &str, prop: &str) -> Option<usize> {
        self.variables.keys().position(|k| k == prop)
    }

    /// 设置变量（带错误处理）
    pub fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// 获取变量值（带错误处理）
    pub fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        self.variables.get(name)
            .cloned()
            .ok_or_else(|| ExpressionError::new(
                ExpressionErrorType::UndefinedVariable,
                format!("变量 '{}' 未定义", name)
            ))
    }

    /// 获取标签属性值
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(vertex) = self.get_vertex_internal() {
            if vertex.has_tag(tag) {
                vertex.get_property(tag, prop)
                    .cloned()
                    .ok_or_else(|| ExpressionError::new(
                        ExpressionErrorType::PropertyNotFound,
                        format!("标签 '{}' 的属性 '{}' 不存在", tag, prop)
                    ))
            } else {
                Err(ExpressionError::new(
                    ExpressionErrorType::LabelNotFound,
                    format!("顶点没有标签 '{}'", tag)
                ))
            }
        } else {
            Err(ExpressionError::new(
                ExpressionErrorType::TypeError,
                "上下文中没有顶点".to_string()
            ))
        }
    }

    /// 获取边属性值
    pub fn get_edge_prop(&self, edge_type: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = self.get_edge_internal() {
            if edge.edge_type == edge_type {
                edge.properties().get(prop)
                    .cloned()
                    .ok_or_else(|| ExpressionError::new(
                        ExpressionErrorType::PropertyNotFound,
                        format!("边类型 '{}' 的属性 '{}' 不存在", edge_type, prop)
                    ))
            } else {
                Err(ExpressionError::new(
                    ExpressionErrorType::LabelNotFound,
                    format!("边类型不匹配: expected {}, found {}", edge_type, edge.edge_type)
                ))
            }
        } else {
            Err(ExpressionError::new(
                ExpressionErrorType::TypeError,
                "上下文中没有边".to_string()
            ))
        }
    }

    /// 获取源顶点属性值 ($^.tag.prop)
    pub fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = self.get_edge_internal() {
            if let Value::Vertex(src_vertex) = edge.src.as_ref() {
                if src_vertex.has_tag(tag) {
                    src_vertex.get_property(tag, prop)
                        .cloned()
                        .ok_or_else(|| ExpressionError::new(
                            ExpressionErrorType::PropertyNotFound,
                            format!("源顶点标签 '{}' 的属性 '{}' 不存在", tag, prop)
                        ))
                } else {
                    Err(ExpressionError::new(
                        ExpressionErrorType::LabelNotFound,
                        format!("源顶点没有标签 '{}'", tag)
                    ))
                }
            } else {
                Err(ExpressionError::new(
                    ExpressionErrorType::TypeError,
                    "边的源顶点不是顶点类型".to_string()
                ))
            }
        } else {
            Err(ExpressionError::new(
                ExpressionErrorType::TypeError,
                "上下文中没有边".to_string()
            ))
        }
    }

    /// 获取目标顶点属性值 ($$.tag.prop)
    pub fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = self.get_edge_internal() {
            if let Value::Vertex(dst_vertex) = edge.dst.as_ref() {
                if dst_vertex.has_tag(tag) {
                    dst_vertex.get_property(tag, prop)
                        .cloned()
                        .ok_or_else(|| ExpressionError::new(
                            ExpressionErrorType::PropertyNotFound,
                            format!("目标顶点标签 '{}' 的属性 '{}' 不存在", tag, prop)
                        ))
                } else {
                    Err(ExpressionError::new(
                        ExpressionErrorType::LabelNotFound,
                        format!("目标顶点没有标签 '{}'", tag)
                    ))
                }
            } else {
                Err(ExpressionError::new(
                    ExpressionErrorType::TypeError,
                    "边的目标顶点不是顶点类型".to_string()
                ))
            }
        } else {
            Err(ExpressionError::new(
                ExpressionErrorType::TypeError,
                "上下文中没有边".to_string()
            ))
        }
    }

    /// 获取输入属性值 ($-.prop)
    pub fn get_input_prop(&self, prop: &str) -> Result<Value, ExpressionError> {
        self.variables.get(prop)
            .cloned()
            .ok_or_else(|| ExpressionError::new(
                ExpressionErrorType::PropertyNotFound,
                format!("输入属性 '{}' 不存在", prop)
            ))
    }

    /// 获取输入属性在元组中的索引
    pub fn get_input_prop_index(&self, prop: &str) -> Result<usize, ExpressionError> {
        self.variables.keys()
            .position(|k| k == prop)
            .ok_or_else(|| ExpressionError::new(
                ExpressionErrorType::PropertyNotFound,
                format!("输入属性 '{}' 不存在", prop)
            ))
    }

    /// 按列索引获取值
    pub fn get_column(&self, index: i32) -> Result<Value, ExpressionError> {
        if index < 0 {
            return Err(ExpressionError::new(
                ExpressionErrorType::IndexOutOfBounds,
                format!("列索引不能为负数: {}", index)
            ));
        }
        let idx = index as usize;
        for value in self.variables.values() {
            if let Value::List(list) = value {
                if idx < list.len() {
                    return Ok(list[idx].clone());
                }
            }
        }
        Err(ExpressionError::new(
            ExpressionErrorType::IndexOutOfBounds,
            format!("列索引 {} 超出范围", index)
        ))
    }

    /// 内部方法：获取顶点（从变量中解析）
    fn get_vertex_internal(&self) -> Option<&crate::core::Vertex> {
        if let Some(Value::Vertex(v)) = self.variables.get("_vertex") {
            Some(v)
        } else {
            None
        }
    }

    /// 内部方法：获取边（从变量中解析）
    fn get_edge_internal(&self) -> Option<&crate::core::Edge> {
        if let Some(Value::Edge(e)) = self.variables.get("_edge") {
            Some(e)
        } else {
            None
        }
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
            regex_cache: self.regex_cache.clone(),
            inner_variables: self.inner_variables.clone(),
            paths: self.paths.clone(),
        }
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
        self.get_vertex_internal()
    }

    fn get_edge(&self) -> Option<&crate::core::Edge> {
        self.get_edge_internal()
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.paths.get(name).map(|p| p.as_ref())
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
        let rc_path = Rc::new(path);
        self.paths.insert(name.clone(), rc_path.clone());
        self.set_variable(name, crate::core::Value::Path(rc_path.as_ref().clone()));
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
