//! 表达式求值上下文接口
//!
//! 使用枚举实现零成本抽象，避免动态分发和循环依赖

use crate::core::{Edge, Value, Vertex};
use std::collections::HashMap;

/// 表达式求值上下文枚举
///
/// 使用枚举实现零成本抽象，避免动态分发的性能开销
#[derive(Clone, Debug)]
pub enum ExpressionContext {
    /// 简单上下文实现
    Simple(SimpleExpressionContext),
    /// 查询上下文适配器
    Query(QueryContextAdapter),
}

/// 简单的表达式上下文实现
#[derive(Clone, Debug)]
pub struct SimpleExpressionContext {
    vertex: Option<Vertex>,
    edge: Option<Edge>,
    vars: HashMap<String, Value>,
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

/// 查询上下文适配器
///
/// 用于适配query模块的EvalContext，避免循环依赖
#[derive(Clone, Debug)]
pub struct QueryContextAdapter {
    // 这里存储查询上下文的必要信息
    // 由于避免循环依赖，这里使用基本类型
    vertex: Option<Vertex>,
    edge: Option<Edge>,
    vars: HashMap<String, Value>,
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

impl SimpleExpressionContext {
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
    
    /// 添加路径
    pub fn add_path(mut self, name: String, path: crate::core::vertex_edge_path::Path) -> Self {
        self.paths.insert(name, path);
        self
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
    
    /// 从query模块的EvalContext创建适配器
    /// 这个函数在query模块中实现，避免循环依赖
    pub fn from_eval_context(_ctx: &crate::query::context::EvalContext) -> Self {
        // 这里需要query模块提供实现
        // 暂时返回空适配器
        Self::new()
    }
}

impl ExpressionContext {
    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.vars.get(name).cloned(),
            ExpressionContext::Query(ctx) => ctx.vars.get(name).cloned(),
        }
    }
    
    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        match self {
            ExpressionContext::Simple(ctx) => { ctx.vars.insert(name, value); }
            ExpressionContext::Query(ctx) => { ctx.vars.insert(name, value); }
        }
    }
    
    /// 获取顶点引用
    pub fn get_vertex(&self) -> Option<&Vertex> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.vertex.as_ref(),
            ExpressionContext::Query(ctx) => ctx.vertex.as_ref(),
        }
    }
    
    /// 获取边引用
    pub fn get_edge(&self) -> Option<&Edge> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.edge.as_ref(),
            ExpressionContext::Query(ctx) => ctx.edge.as_ref(),
        }
    }
    
    /// 获取路径
    pub fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.paths.get(name),
            ExpressionContext::Query(ctx) => ctx.paths.get(name),
        }
    }
    
    /// 获取所有变量
    pub fn get_variables(&self) -> HashMap<String, Value> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.vars.clone(),
            ExpressionContext::Query(ctx) => ctx.vars.clone(),
        }
    }
    
    /// 创建简单上下文
    pub fn simple() -> Self {
        ExpressionContext::Simple(SimpleExpressionContext::new())
    }
    
    /// 创建查询上下文适配器
    pub fn query() -> Self {
        ExpressionContext::Query(QueryContextAdapter::new())
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::simple()
    }
}

impl Default for SimpleExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QueryContextAdapter {
    fn default() -> Self {
        Self::new()
    }
}