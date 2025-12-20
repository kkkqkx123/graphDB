//! 默认表达式上下文实现
//!
//! 包含默认上下文和查询上下文适配器的实现

use crate::core::{Edge, Value, Vertex};
use crate::expression::context::ExpressionContextCore;
use std::collections::HashMap;

/// 表达式求值上下文枚举
///
/// 使用枚举实现零成本抽象，避免动态分发的性能开销
/// 同时提供更好的类型安全性和扩展性
#[derive(Clone, Debug)]
pub enum ExpressionContext {
    /// 默认上下文实现
    Default(DefaultExpressionContext),
    /// 查询上下文适配器
    Query(QueryContextAdapter),
}

/// 简单的表达式上下文实现
#[derive(Clone, Debug)]
pub struct DefaultExpressionContext {
    vertex: Option<Vertex>,
    edge: Option<Edge>,
    vars: HashMap<String, Value>,
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

/// 查询上下文适配器
///
/// 用于适配query模块的上下文，避免循环依赖
#[derive(Clone, Debug)]
pub struct QueryContextAdapter {
    // 这里存储查询上下文的必要信息
    // 由于避免循环依赖，这里使用基本类型
    vertex: Option<Vertex>,
    edge: Option<Edge>,
    vars: HashMap<String, Value>,
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

impl DefaultExpressionContext {
    /// 创建新的简单上下文
    pub fn new() -> Self {
        Self {
            vertex: None,
            edge: None,
            vars: HashMap::new(),
            paths: HashMap::new(),
        }
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
        self.vars.insert(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.vars.insert(name, value);
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
        self.vertex.is_none() && self.edge.is_none() && self.vars.is_empty()
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.vars.len()
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.vertex = None;
        self.edge = None;
        self.vars.clear();
        self.paths.clear();
    }
}

impl QueryContextAdapter {
    /// 创建新的查询上下文适配器
    pub fn new() -> Self {
        Self {
            vertex: None,
            edge: None,
            vars: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.vertex.is_none() && self.edge.is_none() && self.vars.is_empty()
    }
}

impl super::core::ExpressionContextCore for ExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        match self {
            ExpressionContext::Default(ctx) => ctx.vars.get(name).cloned(),
            ExpressionContext::Query(ctx) => ctx.vars.get(name).cloned(),
        }
    }

    fn set_variable(&mut self, name: String, value: Value) {
        match self {
            ExpressionContext::Default(ctx) => {
                ctx.vars.insert(name, value);
            }
            ExpressionContext::Query(ctx) => {
                ctx.vars.insert(name, value);
            }
        }
    }

    fn get_vertex(&self) -> Option<&Vertex> {
        match self {
            ExpressionContext::Default(ctx) => ctx.vertex.as_ref(),
            ExpressionContext::Query(ctx) => ctx.vertex.as_ref(),
        }
    }

    fn get_edge(&self) -> Option<&Edge> {
        match self {
            ExpressionContext::Default(ctx) => ctx.edge.as_ref(),
            ExpressionContext::Query(ctx) => ctx.edge.as_ref(),
        }
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        match self {
            ExpressionContext::Default(ctx) => ctx.paths.get(name),
            ExpressionContext::Query(ctx) => ctx.paths.get(name),
        }
    }

    fn set_vertex(&mut self, vertex: Vertex) {
        match self {
            ExpressionContext::Default(ctx) => {
                ctx.vertex = Some(vertex);
            }
            ExpressionContext::Query(ctx) => {
                ctx.vertex = Some(vertex);
            }
        }
    }

    fn set_edge(&mut self, edge: Edge) {
        match self {
            ExpressionContext::Default(ctx) => {
                ctx.edge = Some(edge);
            }
            ExpressionContext::Query(ctx) => {
                ctx.edge = Some(edge);
            }
        }
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        match self {
            ExpressionContext::Default(ctx) => {
                ctx.paths.insert(name, path);
            }
            ExpressionContext::Query(ctx) => {
                ctx.paths.insert(name, path);
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            ExpressionContext::Default(ctx) => ctx.is_empty(),
            ExpressionContext::Query(ctx) => ctx.is_empty(),
        }
    }

    fn variable_count(&self) -> usize {
        match self {
            ExpressionContext::Default(ctx) => ctx.variable_count(),
            ExpressionContext::Query(ctx) => ctx.vars.len(),
        }
    }

    fn variable_names(&self) -> Vec<String> {
        match self {
            ExpressionContext::Default(ctx) => ctx.variable_names(),
            ExpressionContext::Query(ctx) => ctx.vars.keys().cloned().collect(),
        }
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        match self {
            ExpressionContext::Default(ctx) => Some(ctx.vars.clone()),
            ExpressionContext::Query(ctx) => Some(ctx.vars.clone()),
        }
    }

    fn clear(&mut self) {
        match self {
            ExpressionContext::Default(ctx) => ctx.clear(),
            ExpressionContext::Query(ctx) => {
                ctx.vertex = None;
                ctx.edge = None;
                ctx.vars.clear();
                ctx.paths.clear();
            }
        }
    }
}

impl super::core::ExpressionContextCore for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
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
        self.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.variable_count()
    }

    fn variable_names(&self) -> Vec<String> {
        self.variable_names()
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        Some(self.vars.clone())
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl super::core::ExpressionContextCore for QueryContextAdapter {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
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
        self.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.vars.len()
    }

    fn variable_names(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        Some(self.vars.clone())
    }

    fn clear(&mut self) {
        self.vertex = None;
        self.edge = None;
        self.vars.clear();
        self.paths.clear();
    }
}

impl ExpressionContext {
    /// 创建简单上下文
    pub fn default() -> Self {
        ExpressionContext::Default(DefaultExpressionContext::new())
    }

    /// 创建查询上下文适配器
    pub fn query() -> Self {
        ExpressionContext::Query(QueryContextAdapter::new())
    }

    /// 从简单上下文创建
    pub fn from_default(default: DefaultExpressionContext) -> Self {
        ExpressionContext::Default(default)
    }

    /// 从查询上下文适配器创建
    pub fn from_query(query: QueryContextAdapter) -> Self {
        ExpressionContext::Query(query)
    }

    /// 转换为简单上下文（如果可能）
    pub fn as_default(&self) -> Option<&DefaultExpressionContext> {
        match self {
            ExpressionContext::Default(ctx) => Some(ctx),
            ExpressionContext::Query(_) => None,
        }
    }

    /// 转换为可变简单上下文（如果可能）
    pub fn as_fault_mut(&mut self) -> Option<&mut DefaultExpressionContext> {
        match self {
            ExpressionContext::Default(ctx) => Some(ctx),
            ExpressionContext::Query(_) => None,
        }
    }

    /// 转换为查询上下文适配器（如果可能）
    pub fn as_query(&self) -> Option<&QueryContextAdapter> {
        match self {
            ExpressionContext::Query(ctx) => Some(ctx),
            ExpressionContext::Default(_) => None,
        }
    }

    /// 转换为可变查询上下文适配器（如果可能）
    pub fn as_query_mut(&mut self) -> Option<&mut QueryContextAdapter> {
        match self {
            ExpressionContext::Query(ctx) => Some(ctx),
            ExpressionContext::Default(_) => None,
        }
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::default()
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QueryContextAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷的构建器
pub struct ExpressionContextBuilder {
    context: ExpressionContext,
}

impl ExpressionContextBuilder {
    pub fn new() -> Self {
        Self {
            context: ExpressionContext::default(),
        }
    }

    pub fn query() -> Self {
        Self {
            context: ExpressionContext::query(),
        }
    }

    pub fn with_vertex(mut self, vertex: Vertex) -> Self {
        self.context.set_vertex(vertex);
        self
    }

    pub fn with_edge(mut self, edge: Edge) -> Self {
        self.context.set_edge(edge);
        self
    }

    pub fn with_variable(mut self, name: String, value: Value) -> Self {
        self.context.set_variable(name, value);
        self
    }

    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        self.context.set_variables(variables);
        self
    }

    pub fn with_path(mut self, name: String, path: crate::core::vertex_edge_path::Path) -> Self {
        self.context.add_path(name, path);
        self
    }

    pub fn build(self) -> ExpressionContext {
        self.context
    }
}

impl Default for ExpressionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建带有初始数据的上下文
pub fn with_variables<I>(variables: I) -> ExpressionContext
where
    I: IntoIterator<Item = (String, Value)>,
{
    ExpressionContextBuilder::new()
        .with_variables(variables)
        .build()
}

/// 便捷函数：创建带有顶点的上下文
pub fn with_vertex(vertex: Vertex) -> ExpressionContext {
    ExpressionContextBuilder::new().with_vertex(vertex).build()
}

/// 便捷函数：创建带有边的上下文
pub fn with_edge(edge: Edge) -> ExpressionContext {
    ExpressionContextBuilder::new().with_edge(edge).build()
}

// 为ExpressionContext添加批量设置变量的方法
impl ExpressionContext {
    /// 批量设置变量
    pub fn set_variables<I>(&mut self, variables: I)
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.set_variable(name, value);
        }
    }
}
