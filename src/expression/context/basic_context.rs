//! 基础表达式上下文模块
//!
//! 提供表达式求值过程中的基础上下文实现
//! 使用独立的组件管理不同职责

use crate::core::Value;
use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::expression::context::{
    cache_manager::CacheManager,
    version_manager::VersionManager,
    traits::*,
};
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::{BuiltinFunction, CustomFunction, FunctionRef};
use std::collections::HashMap;
use std::rc::Rc;

/// 表达式上下文枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum ExpressionContextType {
    /// 基础表达式上下文
    Basic(BasicExpressionContext),
}

/// 基础表达式上下文
///
/// 使用独立的组件管理不同职责：
/// - VersionManager: 管理变量版本历史
/// - FunctionRegistry: 管理函数注册
/// - CacheManager: 管理缓存
#[derive(Debug)]
pub struct BasicExpressionContext {
    /// 版本管理器
    pub version_manager: VersionManager,
    /// 函数注册表
    pub function_registry: FunctionRegistry,
    /// 缓存管理器
    pub cache_manager: CacheManager,
    /// 父上下文
    pub parent: Option<Box<BasicExpressionContext>>,
    /// 上下文深度
    pub depth: usize,
    /// 内部变量（用于 ListComprehension、Predicate 等表达式）
    pub inner_variables: HashMap<String, Value>,
    /// 路径存储（使用 Rc 以支持引用返回）
    pub paths: HashMap<String, Rc<crate::core::vertex_edge_path::Path>>,
}

impl BasicExpressionContext {
    /// 创建新的基础表达式上下文
    pub fn new() -> Self {
        Self {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            parent: None,
            depth: 0,
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// 创建带父上下文的基础表达式上下文
    pub fn with_parent(parent: BasicExpressionContext) -> Self {
        let parent_depth = parent.get_depth();
        Self {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            parent: Some(Box::new(parent)),
            depth: parent_depth + 1,
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// 设置变量（添加新版本）
    pub fn set_variable(&mut self, name: impl Into<String>, value: Value) {
        self.version_manager.set_version(name.into(), value);
    }

    /// 批量设置变量
    pub fn set_variables(&mut self, variables: HashMap<String, Value>) {
        for (name, value) in variables {
            self.set_variable(name, value);
        }
    }

    /// 注册内置函数
    pub fn register_builtin_function(&mut self, function: BuiltinFunction) {
        self.function_registry.register_builtin(function);
    }

    /// 注册自定义函数
    pub fn register_custom_function(&mut self, function: CustomFunction) {
        self.function_registry.register_custom_full(function);
    }

    /// 获取内置函数
    pub fn get_builtin_function(&self, name: &str) -> Option<&BuiltinFunction> {
        self.function_registry.get_builtin(name)
    }

    /// 获取自定义函数
    pub fn get_custom_function(&self, name: &str) -> Option<&CustomFunction> {
        self.function_registry.get_custom(name)
    }

    /// 移除变量
    pub fn remove_variable(&mut self, name: &str) -> Option<Vec<Value>> {
        self.version_manager.remove(name)
    }

    /// 清空所有变量
    pub fn clear_variables(&mut self) {
        self.version_manager.clear();
    }

    /// 检查变量是否在当前上下文中定义
    pub fn is_local_variable(&self, name: &str) -> bool {
        self.version_manager.exists(name)
    }

    /// 获取当前上下文中的变量名
    pub fn get_local_variable_names(&self) -> Vec<&str> {
        self.version_manager.variable_names()
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
    pub fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        self.cache_manager.get_regex(pattern)
    }

    /// 获取变量属性值
    pub fn get_var_prop(&self, var: &str, prop: &str) -> Option<Value> {
        if let Some(var_value) = VariableContext::get_variable(self, var) {
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
        self.version_manager.get_version(name, version).cloned()
    }

    /// 获取变量属性在元组中的索引
    pub fn get_var_prop_index(&self, _var: &str, prop: &str) -> Option<usize> {
        self.version_manager.variable_names().iter().position(|&k| k == prop)
    }

    /// 设置变量（带错误处理）
    pub fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError> {
        self.set_variable(name, value);
        Ok(())
    }

    /// 获取变量值（带错误处理）
    pub fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        VariableContext::get_variable(self, name)
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
        self.version_manager.get_latest(prop)
            .cloned()
            .ok_or_else(|| ExpressionError::new(
                ExpressionErrorType::PropertyNotFound,
                format!("输入属性 '{}' 不存在", prop)
            ))
    }

    /// 获取输入属性在元组中的索引
    pub fn get_input_prop_index(&self, prop: &str) -> Result<usize, ExpressionError> {
        self.version_manager.variable_names()
            .iter()
            .position(|&k| k == prop)
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
        for value in self.version_manager.variable_names() {
            if let Some(val) = self.version_manager.get_version(value, 0) {
                if let Value::List(list) = val {
                    if idx < list.len() {
                        return Ok(list[idx].clone());
                    }
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
        if let Some(Value::Vertex(v)) = self.version_manager.get_latest("_vertex") {
            Some(v.as_ref())
        } else {
            None
        }
    }

    /// 内部方法：获取边（从变量中解析）
    fn get_edge_internal(&self) -> Option<&crate::core::Edge> {
        if let Some(Value::Edge(e)) = self.version_manager.get_latest("_edge") {
            Some(e)
        } else {
            None
        }
    }

    /// 获取上下文深度
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// 创建子上下文
    pub fn create_child_context(&self) -> ExpressionContextType {
        ExpressionContextType::Basic(BasicExpressionContext {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            parent: Some(Box::new(self.clone())),
            depth: self.get_depth() + 1,
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
        })
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
            version_manager: self.version_manager.clone(),
            function_registry: FunctionRegistry::new(),
            cache_manager: self.cache_manager.clone(),
            parent: self.parent.clone(),
            depth: self.get_depth(),
            inner_variables: self.inner_variables.clone(),
            paths: self.paths.clone(),
        }
    }
}

impl VariableContext for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        // 首先在当前上下文中查找
        if let Some(value) = self.version_manager.get_latest(name) {
            return Some(value.clone());
        }

        // 然后在内部变量中查找
        if let Some(value) = self.inner_variables.get(name) {
            return Some(value.clone());
        }

        // 如果在当前上下文中找不到，则在父上下文中查找
        if let Some(parent) = &self.parent {
            VariableContext::get_variable(parent, name)
        } else {
            None
        }
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.version_manager.set_version(name, value);
    }

    fn get_variable_names(&self) -> Vec<&str> {
        let mut names = self.version_manager.variable_names();
        names.extend(self.inner_variables.keys().map(|k| k.as_str()));

        // 添加父上下文中的变量名（去重）
        if let Some(parent) = &self.parent {
            let parent_names = VariableContext::get_variable_names(parent);
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names
    }

    fn variable_count(&self) -> usize {
        let mut count = self.version_manager.variable_names().len() + self.inner_variables.len();
        if let Some(parent) = &self.parent {
            count += VariableContext::variable_count(parent);
        }
        count
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        let mut all_vars = HashMap::new();

        // 收集当前上下文的变量
        for name in self.version_manager.variable_names() {
            if let Some(value) = self.version_manager.get_latest(name) {
                all_vars.insert(name.to_string(), value.clone());
            }
        }

        // 收集内部变量
        for (name, value) in &self.inner_variables {
            all_vars.insert(name.clone(), value.clone());
        }

        // 收集父上下文的变量
        if let Some(parent) = &self.parent {
            if let Some(parent_vars) = VariableContext::get_all_variables(parent) {
                for (name, value) in parent_vars {
                    if !all_vars.contains_key(&name) {
                        all_vars.insert(name, value);
                    }
                }
            }
        }

        Some(all_vars)
    }

    fn clear_variables(&mut self) {
        self.version_manager.clear();
        self.inner_variables.clear();
    }
}

impl GraphContext for BasicExpressionContext {
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
}

impl FunctionContext for BasicExpressionContext {
    fn get_function(&self, _name: &str) -> Option<FunctionRef> {
        None
    }

    fn get_function_names(&self) -> Vec<&str> {
        let mut names = self.function_registry.function_names();

        // 添加父上下文中的函数名（去重）
        if let Some(parent) = &self.parent {
            let parent_names = FunctionContext::get_function_names(parent);
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names
    }
}

impl CacheContext for BasicExpressionContext {
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        self.cache_manager.get_regex(pattern)
    }
}

impl ScopedContext for BasicExpressionContext {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn create_child_context(&self) -> Box<dyn crate::expression::evaluator::traits::ExpressionContext> {
        Box::new(BasicExpressionContext {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            parent: Some(Box::new(self.clone())),
            depth: self.get_depth() + 1,
            inner_variables: HashMap::new(),
            paths: HashMap::new(),
        })
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<crate::core::Value> {
        VariableContext::get_variable(self, name)
    }

    fn set_variable(&mut self, name: String, value: crate::core::Value) {
        VariableContext::set_variable(self, name, value);
    }

    fn get_function(&self, _name: &str) -> Option<crate::expression::functions::FunctionRef> {
        None
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

    fn set_vertex(&mut self, vertex: crate::core::Vertex) {
        GraphContext::set_vertex(self, vertex);
    }

    fn set_edge(&mut self, edge: crate::core::Edge) {
        GraphContext::set_edge(self, edge);
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        GraphContext::add_path(self, name, path);
    }

    fn is_empty(&self) -> bool {
        self.get_vertex_internal().is_none()
            && self.get_edge_internal().is_none()
            && VariableContext::variable_count(self) == 0
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
        VariableContext::clear_variables(self);
    }
}

impl VariableContext for Box<BasicExpressionContext> {
    fn get_variable(&self, name: &str) -> Option<Value> {
        VariableContext::get_variable(&**self, name)
    }

    fn set_variable(&mut self, name: String, value: Value) {
        VariableContext::set_variable(&mut **self, name, value);
    }

    fn get_variable_names(&self) -> Vec<&str> {
        VariableContext::get_variable_names(&**self)
    }

    fn variable_count(&self) -> usize {
        VariableContext::variable_count(&**self)
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        VariableContext::get_all_variables(&**self)
    }

    fn clear_variables(&mut self) {
        VariableContext::clear_variables(&mut **self);
    }
}

impl FunctionContext for Box<BasicExpressionContext> {
    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        FunctionContext::get_function(&**self, name)
    }

    fn get_function_names(&self) -> Vec<&str> {
        FunctionContext::get_function_names(&**self)
    }
}
