//! 默认表达式上下文实现
//!
//! 包含默认上下文的实现，使用新的组件化设计

use crate::core::{Edge, Value, Vertex};
use crate::core::error::ExpressionError;
use crate::expression::context::{
    cache_manager::CacheManager,
    traits::*,
    version_manager::VersionManager,
};
use crate::expression::functions::registry::FunctionRegistry;
use std::collections::HashMap;

/// 存储层表达式上下文trait
///
/// 为存储层特定的表达式上下文提供额外接口
pub trait StorageExpressionContext: ExpressionContext {
    /// 获取变量值（最新版本）
    fn get_var(&self, name: &str) -> Result<Value, ExpressionError>;

    /// 获取指定版本的变量值
    fn get_versioned_var(&self, name: &str, version: i64) -> Result<Value, ExpressionError>;

    /// 设置变量值
    fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError>;

    /// 设置表达式内部变量
    fn set_inner_var(&mut self, var: &str, value: Value);

    /// 获取表达式内部变量
    fn get_inner_var(&self, var: &str) -> Option<Value>;

    /// 获取变量属性值
    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取目标顶点属性值
    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取输入属性值
    fn get_input_prop(&self, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取输入属性索引
    fn get_input_prop_index(&self, prop: &str) -> Result<usize, ExpressionError>;

    /// 按列索引获取值
    fn get_column(&self, index: i32) -> Result<Value, ExpressionError>;

    /// 获取标签属性值
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取边属性值
    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取源顶点属性值
    fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError>;

    /// 获取顶点（按标签名）
    fn get_vertex_by_tag(&self, name: &str) -> Result<Value, ExpressionError>;

    /// 获取边（作为值）
    fn get_edge_value(&self) -> Result<Value, ExpressionError>;
}

/// 简单的表达式上下文实现
///
/// 轻量级上下文，适用于大部分表达式求值场景
/// 使用 VersionManager 管理变量，支持版本控制
/// 使用 FunctionRegistry 管理函数，支持自定义函数
/// 使用 CacheManager 管理缓存，提升性能
#[derive(Debug)]
pub struct DefaultExpressionContext {
    /// 版本管理器
    version_manager: VersionManager,
    /// 函数注册表
    function_registry: FunctionRegistry,
    /// 缓存管理器
    cache_manager: CacheManager,
    /// 顶点
    vertex: Option<Vertex>,
    /// 边
    edge: Option<Edge>,
    /// 路径
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

impl DefaultExpressionContext {
    /// 创建新的简单上下文
    pub fn new() -> Self {
        Self {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            vertex: None,
            edge: None,
            paths: HashMap::new(),
        }
    }

    /// 创建带有全局函数注册表的上下文
    pub fn with_global_functions() -> Self {
        let mut context = Self::new();
        context.register_global_functions();
        context
    }

    /// 注册全局函数
    pub fn register_global_functions(&mut self) {
        use crate::expression::functions::{BuiltinFunction, MathFunction, StringFunction, RegexFunction, ConversionFunction, DateTimeFunction};
        
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Abs));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Sqrt));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Pow));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Log));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Log10));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Sin));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Cos));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Tan));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Round));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Ceil));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Floor));

        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Length));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Upper));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Lower));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Trim));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Substring));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Concat));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Replace));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Contains));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::StartsWith));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::EndsWith));

        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexMatch));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexReplace));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexFind));

        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToString));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToInt));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToFloat));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToBool));

        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Now));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Date));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Time));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Year));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Month));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Day));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Hour));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Minute));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Second));
    }

    /// 注册自定义函数
    pub fn register_function(&mut self, function: crate::expression::functions::CustomFunction) {
        self.function_registry.register_custom_full(function);
    }

    /// 获取函数注册表引用
    pub fn function_registry(&self) -> &FunctionRegistry {
        &self.function_registry
    }

    /// 获取函数注册表可变引用
    pub fn function_registry_mut(&mut self) -> &mut FunctionRegistry {
        &mut self.function_registry
    }

    /// 获取缓存管理器引用
    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    /// 获取缓存管理器可变引用
    pub fn cache_manager_mut(&mut self) -> &mut CacheManager {
        &mut self.cache_manager
    }

    /// 设置顶点
    pub fn with_vertex(mut self, vertex: Vertex) -> Self {
        self.vertex = Some(vertex);
        self
    }

    /// 设置边
    pub fn with_edge(mut self, edge: Edge) -> Self {
        self.edge = Some(edge);
        self
    }

    /// 添加变量
    pub fn add_variable(mut self, name: String, value: Value) -> Self {
        self.version_manager.set_version(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.version_manager.set_version(name, value);
        }
        self
    }

    /// 添加路径
    pub fn add_path(mut self, name: String, path: crate::core::vertex_edge_path::Path) -> Self {
        self.paths.insert(name, path);
        self
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.vertex.is_none() && self.edge.is_none() && self.version_manager.variable_names().is_empty()
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.version_manager.variable_names().len()
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.version_manager.variable_names().into_iter().map(|s| s.to_string()).collect()
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.vertex = None;
        self.edge = None;
        self.version_manager.clear();
        self.paths.clear();
    }

    /// 获取版本管理器引用
    pub fn version_manager(&self) -> &VersionManager {
        &self.version_manager
    }

    /// 获取版本管理器可变引用
    pub fn version_manager_mut(&mut self) -> &mut VersionManager {
        &mut self.version_manager
    }
}

impl VariableContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.version_manager.get_latest(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.version_manager.set_version(name, value);
    }

    fn get_variable_names(&self) -> Vec<&str> {
        self.version_manager.variable_names()
    }

    fn variable_count(&self) -> usize {
        self.version_manager.variable_names().len()
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        let mut all_vars = HashMap::new();
        for name in self.version_manager.variable_names() {
            if let Some(value) = self.version_manager.get_latest(name) {
                all_vars.insert(name.to_string(), value.clone());
            }
        }
        Some(all_vars)
    }

    fn clear_variables(&mut self) {
        self.version_manager.clear();
    }
}

impl GraphContext for DefaultExpressionContext {
    fn get_vertex(&self) -> Option<&Vertex> {
        self.vertex.as_ref()
    }

    fn get_edge(&self) -> Option<&Edge> {
        self.edge.as_ref()
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.paths.get(name)
    }

    fn set_vertex(&mut self, vertex: Vertex) {
        self.vertex = Some(vertex);
    }

    fn set_edge(&mut self, edge: Edge) {
        self.edge = Some(edge);
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        self.paths.insert(name, path);
    }
}

impl FunctionContext for DefaultExpressionContext {
    fn get_function(&self, _name: &str) -> Option<crate::expression::functions::FunctionRef> {
        None
    }

    fn get_function_names(&self) -> Vec<&str> {
        self.function_registry.function_names()
    }
}

impl CacheContext for DefaultExpressionContext {
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        self.cache_manager.get_regex_internal(pattern)
    }
}

impl ScopedContext for DefaultExpressionContext {
    fn get_depth(&self) -> usize {
        0
    }

    fn create_child_context(&self) -> Box<dyn crate::expression::evaluator::traits::ExpressionContext> {
        Box::new(Self::new())
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageExpressionContext for DefaultExpressionContext {
    fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        self.version_manager.get_latest(name)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(name))
    }

    fn get_versioned_var(&self, name: &str, _version: i64) -> Result<Value, ExpressionError> {
        self.version_manager.get_latest(name)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(name))
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError> {
        self.version_manager.set_version(name.to_string(), value);
        Ok(())
    }

    fn set_inner_var(&mut self, var: &str, value: Value) {
        self.version_manager.set_version(var.to_string(), value);
    }

    fn get_inner_var(&self, var: &str) -> Option<Value> {
        self.version_manager.get_latest(var).cloned()
    }

    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError> {
        let var_value = self.version_manager.get_latest(var)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(var))?;

        match var_value {
            Value::Map(map) => map.get(prop)
                .cloned()
                .ok_or_else(|| ExpressionError::property_not_found(prop)),
            _ => Err(ExpressionError::type_error(format!("变量 '{}' 不是映射类型，无法获取属性", var))),
        }
    }

    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = &self.edge {
            if let Value::Vertex(dst_vertex) = edge.dst.as_ref() {
                if dst_vertex.has_tag(tag) {
                    dst_vertex.get_property(tag, prop)
                        .cloned()
                        .ok_or_else(|| ExpressionError::property_not_found(prop))
                } else {
                    Err(ExpressionError::label_not_found(tag))
                }
            } else {
                Err(ExpressionError::type_error("边的目标顶点不是顶点类型"))
            }
        } else {
            Err(ExpressionError::type_error("上下文中没有边"))
        }
    }

    fn get_input_prop(&self, prop: &str) -> Result<Value, ExpressionError> {
        self.version_manager.get_latest(prop)
            .cloned()
            .ok_or_else(|| ExpressionError::property_not_found(prop))
    }

    fn get_input_prop_index(&self, prop: &str) -> Result<usize, ExpressionError> {
        if let Some(Value::List(list)) = self.version_manager.get_latest(prop) {
            Ok(list.len())
        } else {
            Err(ExpressionError::type_error(format!("属性 '{}' 不是列表类型", prop)))
        }
    }

    fn get_column(&self, index: i32) -> Result<Value, ExpressionError> {
        if index < 0 {
            return Err(ExpressionError::index_out_of_bounds(index as isize, 0));
        }
        let idx = index as usize;
        for value in self.version_manager.variable_names() {
            if let Some(val) = self.version_manager.get_latest(value) {
                if let Value::List(list) = val {
                    if idx < list.len() {
                        return Ok(list[idx].clone());
                    }
                }
            }
        }
        Err(ExpressionError::index_out_of_bounds(idx as isize, 0))
    }

    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(vertex) = &self.vertex {
            if vertex.has_tag(tag) {
                vertex.get_property(tag, prop)
                    .cloned()
                    .ok_or_else(|| ExpressionError::property_not_found(prop))
            } else {
                Err(ExpressionError::label_not_found(tag))
            }
        } else {
            Err(ExpressionError::type_error("上下文中没有顶点"))
        }
    }

    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(current_edge) = &self.edge {
            if current_edge.edge_type == edge {
                current_edge.properties().get(prop)
                    .cloned()
                    .ok_or_else(|| ExpressionError::property_not_found(prop))
            } else {
                Err(ExpressionError::label_not_found(edge))
            }
        } else {
            Err(ExpressionError::type_error("上下文中没有边"))
        }
    }

    fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = &self.edge {
            if let Value::Vertex(src_vertex) = edge.src.as_ref() {
                if src_vertex.has_tag(tag) {
                    src_vertex.get_property(tag, prop)
                        .cloned()
                        .ok_or_else(|| ExpressionError::property_not_found(prop))
                } else {
                    Err(ExpressionError::label_not_found(tag))
                }
            } else {
                Err(ExpressionError::type_error("边的源顶点不是顶点类型"))
            }
        } else {
            Err(ExpressionError::type_error("上下文中没有边"))
        }
    }

    fn get_vertex_by_tag(&self, name: &str) -> Result<Value, ExpressionError> {
        if let Some(vertex) = &self.vertex {
            if vertex.has_tag(name) {
                Ok(Value::Vertex(Box::new(vertex.clone())))
            } else {
                Err(ExpressionError::label_not_found(name))
            }
        } else {
            Err(ExpressionError::type_error("上下文中没有顶点"))
        }
    }

    fn get_edge_value(&self) -> Result<Value, ExpressionError> {
        self.edge.as_ref()
            .map(|e| Value::Edge(e.clone()))
            .ok_or_else(|| ExpressionError::type_error("上下文中没有边"))
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for DefaultExpressionContext {
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

    fn get_function(&self, name: &str) -> Option<crate::expression::functions::FunctionRef> {
        FunctionContext::get_function(self, name)
    }

    fn supports_cache(&self) -> bool {
        true
    }

    fn get_cache(&mut self) -> Option<&mut crate::expression::context::cache_manager::CacheManager> {
        Some(&mut self.cache_manager)
    }
}
