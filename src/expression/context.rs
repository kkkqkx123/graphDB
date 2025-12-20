//! 表达式求值上下文接口 - 优化版本
//!
//! 使用枚举实现零成本抽象，避免动态分发和循环依赖
//! 提供高性能、类型安全的表达式求值上下文实现

use crate::core::{Edge, Value, Vertex};
use std::collections::HashMap;

/// 表达式求值上下文枚举
///
/// 使用枚举实现零成本抽象，避免动态分发的性能开销
/// 同时提供更好的类型安全性和扩展性
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

    /// 从query模块的EvalContext创建适配器
    /// 这个函数在query模块中实现，避免循环依赖
    pub fn from_eval_context(_ctx: &crate::query::context::EvalContext) -> Self {
        // 这里需要query模块提供实现
        // 暂时返回空适配器
        Self::new()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.vertex.is_none() && self.edge.is_none() && self.vars.is_empty()
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
            ExpressionContext::Simple(ctx) => {
                ctx.vars.insert(name, value);
            }
            ExpressionContext::Query(ctx) => {
                ctx.vars.insert(name, value);
            }
        }
    }

    /// 批量设置变量
    pub fn set_variables<I>(&mut self, variables: I)
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.set_variable(name, value);
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

    /// 设置顶点
    pub fn set_vertex(&mut self, vertex: Vertex) {
        match self {
            ExpressionContext::Simple(ctx) => {
                ctx.vertex = Some(vertex);
            }
            ExpressionContext::Query(ctx) => {
                ctx.vertex = Some(vertex);
            }
        }
    }

    /// 设置边
    pub fn set_edge(&mut self, edge: Edge) {
        match self {
            ExpressionContext::Simple(ctx) => {
                ctx.edge = Some(edge);
            }
            ExpressionContext::Query(ctx) => {
                ctx.edge = Some(edge);
            }
        }
    }

    /// 添加路径
    pub fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        match self {
            ExpressionContext::Simple(ctx) => {
                ctx.paths.insert(name, path);
            }
            ExpressionContext::Query(ctx) => {
                ctx.paths.insert(name, path);
            }
        }
    }

    /// 检查是否为空上下文
    pub fn is_empty(&self) -> bool {
        match self {
            ExpressionContext::Simple(ctx) => ctx.is_empty(),
            ExpressionContext::Query(ctx) => ctx.is_empty(),
        }
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        match self {
            ExpressionContext::Simple(ctx) => ctx.variable_count(),
            ExpressionContext::Query(ctx) => ctx.vars.len(),
        }
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        match self {
            ExpressionContext::Simple(ctx) => ctx.variable_names(),
            ExpressionContext::Query(ctx) => ctx.vars.keys().cloned().collect(),
        }
    }

    /// 获取所有变量
    pub fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        match self {
            ExpressionContext::Simple(ctx) => Some(ctx.vars.clone()),
            ExpressionContext::Query(ctx) => Some(ctx.vars.clone()),
        }
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        match self {
            ExpressionContext::Simple(ctx) => ctx.clear(),
            ExpressionContext::Query(ctx) => {
                ctx.vertex = None;
                ctx.edge = None;
                ctx.vars.clear();
                ctx.paths.clear();
            }
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

    /// 从简单上下文创建
    pub fn from_simple(simple: SimpleExpressionContext) -> Self {
        ExpressionContext::Simple(simple)
    }

    /// 从查询上下文适配器创建
    pub fn from_query(query: QueryContextAdapter) -> Self {
        ExpressionContext::Query(query)
    }

    /// 转换为简单上下文（如果可能）
    pub fn as_simple(&self) -> Option<&SimpleExpressionContext> {
        match self {
            ExpressionContext::Simple(ctx) => Some(ctx),
            ExpressionContext::Query(_) => None,
        }
    }

    /// 转换为可变简单上下文（如果可能）
    pub fn as_simple_mut(&mut self) -> Option<&mut SimpleExpressionContext> {
        match self {
            ExpressionContext::Simple(ctx) => Some(ctx),
            ExpressionContext::Query(_) => None,
        }
    }

    /// 转换为查询上下文适配器（如果可能）
    pub fn as_query(&self) -> Option<&QueryContextAdapter> {
        match self {
            ExpressionContext::Query(ctx) => Some(ctx),
            ExpressionContext::Simple(_) => None,
        }
    }

    /// 转换为可变查询上下文适配器（如果可能）
    pub fn as_query_mut(&mut self) -> Option<&mut QueryContextAdapter> {
        match self {
            ExpressionContext::Query(ctx) => Some(ctx),
            ExpressionContext::Simple(_) => None,
        }
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

/// 便捷的构建器
pub struct ExpressionContextBuilder {
    context: ExpressionContext,
}

impl ExpressionContextBuilder {
    pub fn new() -> Self {
        Self {
            context: ExpressionContext::simple(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_context() {
        let mut ctx = ExpressionContext::simple();

        // 测试变量操作
        ctx.set_variable("x".to_string(), Value::Int(42));
        assert_eq!(ctx.get_variable("x"), Some(Value::Int(42)));

        // 测试批量设置变量
        ctx.set_variables(vec![
            ("y".to_string(), Value::String("test".to_string())),
            ("z".to_string(), Value::Float(3.14)),
        ]);
        assert_eq!(
            ctx.get_variable("y"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(ctx.get_variable("z"), Some(Value::Float(3.14)));

        // 测试顶点操作
        let vertex = Vertex::new(Value::Int(1), vec![]);
        ctx.set_vertex(vertex.clone());
        assert_eq!(ctx.get_vertex(), Some(&vertex));

        // 测试构建器
        let ctx2 = ExpressionContextBuilder::new()
            .with_variable("y".to_string(), Value::String("test".to_string()))
            .with_vertex(vertex.clone())
            .build();

        assert_eq!(
            ctx2.get_variable("y"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(ctx2.get_vertex(), Some(&vertex));
    }

    #[test]
    fn test_context_cloning() {
        let mut ctx = ExpressionContext::simple();
        ctx.set_variable("x".to_string(), Value::Int(42));

        let cloned = ctx.clone();
        assert_eq!(cloned.get_variable("x"), Some(Value::Int(42)));

        // 修改原上下文不应影响克隆
        ctx.set_variable("x".to_string(), Value::Int(100));
        assert_eq!(cloned.get_variable("x"), Some(Value::Int(42)));
        assert_eq!(ctx.get_variable("x"), Some(Value::Int(100)));
    }

    #[test]
    fn test_builder_pattern() {
        let vertex = Vertex::new(Value::Int(1), vec![]);
        let edge = Edge::new_empty(Value::Int(1), Value::Int(2), "test".to_string(), 0);

        let ctx = ExpressionContextBuilder::new()
            .with_vertex(vertex.clone())
            .with_edge(edge)
            .with_variables(vec![
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
            ])
            .build();

        assert!(ctx.get_vertex().is_some());
        assert!(ctx.get_edge().is_some());
        assert_eq!(
            ctx.get_variable("name"),
            Some(Value::String("Alice".to_string()))
        );
        assert_eq!(ctx.get_variable("age"), Some(Value::Int(30)));
    }

    #[test]
    fn test_convenience_functions() {
        let vertex = Vertex::new(Value::Int(1), vec![]);

        // 测试便捷函数
        let ctx1 = with_vertex(vertex.clone());
        assert_eq!(ctx1.get_vertex(), Some(&vertex));

        let ctx2 = with_variables(vec![
            ("x".to_string(), Value::Int(42)),
            ("y".to_string(), Value::String("test".to_string())),
        ]);
        assert_eq!(ctx2.get_variable("x"), Some(Value::Int(42)));
        assert_eq!(
            ctx2.get_variable("y"),
            Some(Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_context_operations() {
        let mut ctx = ExpressionContext::simple();

        // 测试空状态
        assert!(ctx.is_empty());
        assert_eq!(ctx.variable_count(), 0);
        assert!(ctx.variable_names().is_empty());

        // 添加数据
        ctx.set_variable("x".to_string(), Value::Int(42));
        assert!(!ctx.is_empty());
        assert_eq!(ctx.variable_count(), 1);
        assert_eq!(ctx.variable_names(), vec!["x"]);

        // 清空数据
        ctx.clear();
        assert!(ctx.is_empty());
        assert_eq!(ctx.variable_count(), 0);
        assert!(ctx.variable_names().is_empty());
    }
}
