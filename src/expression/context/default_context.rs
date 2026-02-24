//! 默认表达式上下文实现
//!
//! 提供表达式求值过程中的上下文管理

use crate::core::{Edge, Value, Vertex};
use crate::core::error::ExpressionError;
use crate::expression::context::{
    cache_manager::CacheManager,
    version_manager::VersionManager,
};
use crate::expression::functions::registry::FunctionRegistry;
use std::collections::HashMap;

/// 表达式上下文
///
/// 提供表达式求值所需的上下文环境，包括：
/// - 变量存储
/// - 函数注册
/// - 正则缓存
/// - 当前顶点/边/路径
#[derive(Debug)]
pub struct DefaultExpressionContext {
    /// 变量管理器
    variables: VersionManager,
    /// 函数注册表
    function_registry: FunctionRegistry,
    /// 缓存管理器
    cache_manager: CacheManager,
    /// 当前顶点
    vertex: Option<Vertex>,
    /// 当前边
    edge: Option<Edge>,
    /// 路径存储
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

impl DefaultExpressionContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            variables: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            vertex: None,
            edge: None,
            paths: HashMap::new(),
        }
    }

    /// 创建带有全局函数的上下文
    pub fn with_global_functions() -> Self {
        let mut context = Self::new();
        context.register_global_functions();
        context
    }

    /// 注册全局函数
    fn register_global_functions(&mut self) {
        use crate::expression::functions::{BuiltinFunction, MathFunction, StringFunction, RegexFunction, ConversionFunction, DateTimeFunction};
        
        // 数学函数
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

        // 字符串函数
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

        // 正则函数
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexMatch));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexReplace));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexFind));

        // 转换函数
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToString));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToInt));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToFloat));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToBool));

        // 日期时间函数
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
        self.variables.set(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.variables.set(name, value);
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
        self.vertex.is_none() && self.edge.is_none() && self.variables.is_empty()
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.variable_names().into_iter().map(|s| s.to_string()).collect()
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.vertex = None;
        self.edge = None;
        self.variables.clear();
        self.paths.clear();
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.set(name, value);
    }

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

    fn is_empty(&self) -> bool {
        self.vertex.is_none() && self.edge.is_none() && self.variables.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.variables.len()
    }

    fn variable_names(&self) -> Vec<String> {
        self.variables.variable_names().into_iter().map(|s| s.to_string()).collect()
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        let mut all_vars = HashMap::new();
        for name in self.variables.variable_names() {
            if let Some(value) = self.variables.get(name) {
                all_vars.insert(name.to_string(), value.clone());
            }
        }
        Some(all_vars)
    }

    fn clear(&mut self) {
        self.variables.clear();
        self.vertex = None;
        self.edge = None;
        self.paths.clear();
    }

    fn get_function(&self, name: &str) -> Option<crate::expression::functions::FunctionRef> {
        self.function_registry.get_builtin(name).map(|f| crate::expression::functions::FunctionRef::Builtin(f))
            .or_else(|| self.function_registry.get_custom(name).map(|f| crate::expression::functions::FunctionRef::Custom(f)))
    }

    fn supports_cache(&self) -> bool {
        true
    }

    fn get_cache(&mut self) -> Option<&mut CacheManager> {
        Some(&mut self.cache_manager)
    }
}

/// 存储层表达式上下文 trait
///
/// 为存储层提供额外的属性访问接口
pub trait StorageExpressionContext: crate::expression::evaluator::traits::ExpressionContext {
    /// 获取变量值
    fn get_var(&self, name: &str) -> Result<Value, ExpressionError>;
    
    /// 设置变量值
    fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError>;
    
    /// 获取变量属性值
    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError>;
    
    /// 获取标签属性值
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError>;
    
    /// 获取边属性值
    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, ExpressionError>;
}

impl StorageExpressionContext for DefaultExpressionContext {
    fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        self.variables.get(name)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(name))
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError> {
        self.variables.set(name.to_string(), value);
        Ok(())
    }

    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError> {
        let var_value = self.variables.get(var)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(var))?;

        match var_value {
            Value::Map(map) => map.get(prop)
                .cloned()
                .ok_or_else(|| ExpressionError::property_not_found(prop)),
            _ => Err(ExpressionError::type_error(format!("变量 '{}' 不是映射类型", var))),
        }
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
}
