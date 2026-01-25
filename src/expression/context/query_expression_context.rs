//! 查询表达式上下文
//!
//! 提供与查询上下文集成的表达式求值上下文。
//! 整合变量管理、属性访问、输入列访问等功能。

use crate::core::{Edge, Value, Vertex};
use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::expression::context::traits::*;
use crate::expression::context::{
    cache_manager::CacheManager,
    function_registry::FunctionRegistry,
    version_manager::VersionManager,
};
use crate::query::context::CoreQueryContext;
use std::collections::HashMap;

/// 查询表达式上下文
///
/// 整合查询上下文的变量和执行上下文，提供完整的表达式求值能力。
/// 适用于查询验证和执行阶段的表达式求值。
///
/// # 与 QueryContext 的集成
///
/// ```ignore
/// let qctx = CoreQueryContext::new();
/// let mut expr_ctx = QueryExpressionContext::from_query_context(&qctx);
///
/// // 设置当前行（用于 $-.prop 访问）
/// expr_ctx.set_current_row(row);
///
/// // 设置顶点（用于 $^.tag.prop 访问）
/// expr_ctx.set_vertex(vertex);
/// ```
#[derive(Debug, Clone)]
pub struct QueryExpressionContext {
    version_manager: VersionManager,
    function_registry: FunctionRegistry,
    cache_manager: CacheManager,
    inner_variables: HashMap<String, Value>,
    current_row: Option<HashMap<String, Value>>,
    current_vertex: Option<Vertex>,
    current_edge: Option<Edge>,
    paths: HashMap<String, Value>,
}

impl QueryExpressionContext {
    /// 从查询上下文创建表达式上下文
    ///
    /// 继承查询上下文中的变量
    pub fn from_query_context(qctx: &CoreQueryContext) -> Self {
        let mut ctx = Self::new();

        // 继承执行上下文的变量
        for (name, value) in qctx.ectx().variables() {
            ctx.set_variable(name.clone(), value.clone());
        }

        ctx
    }

    /// 创建新的查询表达式上下文
    pub fn new() -> Self {
        Self {
            version_manager: VersionManager::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
            inner_variables: HashMap::new(),
            current_row: None,
            current_vertex: None,
            current_edge: None,
            paths: HashMap::new(),
        }
    }

    /// 设置当前行（用于输入属性访问）
    pub fn set_current_row(&mut self, row: HashMap<String, Value>) {
        self.current_row = Some(row);
    }

    /// 获取当前行
    pub fn current_row(&self) -> Option<&HashMap<String, Value>> {
        self.current_row.as_ref()
    }

    /// 设置当前顶点
    pub fn set_vertex(&mut self, vertex: Vertex) {
        self.current_vertex = Some(vertex);
    }

    /// 获取当前顶点
    pub fn current_vertex(&self) -> Option<&Vertex> {
        self.current_vertex.as_ref()
    }

    /// 设置当前边
    pub fn set_edge(&mut self, edge: Edge) {
        self.current_edge = Some(edge);
    }

    /// 获取当前边
    pub fn current_edge(&self) -> Option<&Edge> {
        self.current_edge.as_ref()
    }

    /// 设置内部变量
    pub fn set_inner_var(&mut self, name: &str, value: Value) {
        self.inner_variables.insert(name.to_string(), value);
    }

    /// 获取内部变量
    pub fn get_inner_var(&self, name: &str) -> Option<&Value> {
        self.inner_variables.get(name)
    }

    /// 获取变量值（带错误处理）
    pub fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        // 首先在当前行中查找
        if let Some(row) = &self.current_row {
            if let Some(value) = row.get(name) {
                return Ok(value.clone());
            }
        }

        // 然后在版本管理器中查找
        self.version_manager.get_latest(name)
            .cloned()
            .ok_or_else(|| ExpressionError::new(
                ExpressionErrorType::UndefinedVariable,
                format!("变量 '{}' 未定义", name)
            ))
    }

    /// 获取变量属性值 ($var.prop)
    pub fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError> {
        let var_value = self.get_var(var)?;

        match var_value {
            Value::Map(map) => map.get(prop)
                .cloned()
                .ok_or_else(|| ExpressionError::new(
                    ExpressionErrorType::PropertyNotFound,
                    format!("变量 '{}' 的属性 '{}' 不存在", var, prop)
                )),
            _ => Err(ExpressionError::new(
                ExpressionErrorType::TypeError,
                format!("变量 '{}' 不是映射类型", var)
            )),
        }
    }

    /// 获取输入属性值 ($-.prop)
    pub fn get_input_prop(&self, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(row) = &self.current_row {
            row.get(prop)
                .cloned()
                .ok_or_else(|| ExpressionError::new(
                    ExpressionErrorType::PropertyNotFound,
                    format!("输入属性 '{}' 不存在", prop)
                ))
        } else {
            Err(ExpressionError::new(
                ExpressionErrorType::PropertyNotFound,
                "当前行未设置".to_string()
            ))
        }
    }

    /// 获取源顶点属性值 ($^.tag.prop)
    pub fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(edge) = &self.current_edge {
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
        if let Some(edge) = &self.current_edge {
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

    /// 获取标签属性值
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, ExpressionError> {
        if let Some(vertex) = &self.current_vertex {
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
        if let Some(edge) = &self.current_edge {
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

    /// 清空上下文
    pub fn clear(&mut self) {
        self.version_manager.clear();
        self.inner_variables.clear();
        self.current_row = None;
        self.current_vertex = None;
        self.current_edge = None;
        self.paths.clear();
    }
}

impl VariableContext for QueryExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        // 首先在当前行中查找
        if let Some(row) = &self.current_row {
            if let Some(value) = row.get(name) {
                return Some(value.clone());
            }
        }

        // 然后在版本管理器中查找
        self.version_manager.get_latest(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.version_manager.set_version(name, value);
    }

    fn get_variable_names(&self) -> Vec<&str> {
        let mut names = self.version_manager.variable_names();

        if let Some(row) = &self.current_row {
            for key in row.keys() {
                if !names.contains(&key.as_str()) {
                    names.push(key.as_str());
                }
            }
        }

        names
    }

    fn variable_count(&self) -> usize {
        let mut count = self.version_manager.variable_names().len();

        if let Some(row) = &self.current_row {
            count += row.len();
        }

        count
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        let mut all_vars = HashMap::new();

        for name in self.version_manager.variable_names() {
            if let Some(value) = self.version_manager.get_latest(name) {
                all_vars.insert(name.to_string(), value.clone());
            }
        }

        if let Some(row) = &self.current_row {
            for (name, value) in row {
                if !all_vars.contains_key(name) {
                    all_vars.insert(name.clone(), value.clone());
                }
            }
        }

        Some(all_vars)
    }

    fn clear_variables(&mut self) {
        self.version_manager.clear();
    }
}

impl GraphContext for QueryExpressionContext {
    fn get_vertex(&self) -> Option<&Vertex> {
        self.current_vertex.as_ref()
    }

    fn get_edge(&self) -> Option<&Edge> {
        self.current_edge.as_ref()
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.paths.get(name)
            .and_then(|p| match p {
                Value::Path(path) => Some(path),
                _ => None,
            })
    }

    fn set_vertex(&mut self, vertex: Vertex) {
        self.current_vertex = Some(vertex);
    }

    fn set_edge(&mut self, edge: Edge) {
        self.current_edge = Some(edge);
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        self.paths.insert(name, Value::Path(path));
    }
}

impl FunctionContext for QueryExpressionContext {
    fn get_function(&self, name: &str) -> Option<crate::expression::functions::FunctionRef> {
        self.function_registry.get(name)
    }

    fn get_function_names(&self) -> Vec<&str> {
        self.function_registry.function_names()
    }
}

impl CacheContext for QueryExpressionContext {
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        self.cache_manager.get_regex(pattern)
    }
}

impl ScopedContext for QueryExpressionContext {
    fn get_depth(&self) -> usize {
        0
    }

    fn create_child_context(&self) -> Box<dyn ExpressionContext> {
        Box::new(Self::new())
    }
}

impl ExpressionContext for QueryExpressionContext {
    fn is_empty(&self) -> bool {
        self.version_manager.variable_names().is_empty()
            && self.inner_variables.is_empty()
            && self.current_row.is_none()
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for QueryExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        VariableContext::get_variable(self, name)
    }

    fn set_variable(&mut self, name: String, value: Value) {
        VariableContext::set_variable(self, name, value);
    }

    fn get_vertex(&self) -> Option<&Vertex> {
        GraphContext::get_vertex(self)
    }

    fn get_edge(&self) -> Option<&Edge> {
        GraphContext::get_edge(self)
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        GraphContext::get_path(self, name)
    }

    fn set_vertex(&mut self, vertex: Vertex) {
        GraphContext::set_vertex(self, vertex);
    }

    fn set_edge(&mut self, edge: Edge) {
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

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
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

    fn get_cache(&mut self) -> Option<&mut CacheManager> {
        Some(&mut self.cache_manager)
    }
}

impl Default for QueryExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}
