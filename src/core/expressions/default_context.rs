//! 默认表达式上下文实现
//!
//! 包含默认上下文和查询上下文适配器的实现

use crate::core::{Edge, Value, Vertex};
use std::collections::HashMap;

// 重新导出统一的ExpressionContext trait
pub use crate::core::evaluator::traits::ExpressionContext;

/// 存储层表达式上下文trait
///
/// 为存储层特定的表达式上下文提供额外接口
pub trait StorageExpressionContext: ExpressionContext {
    /// 获取变量值（最新版本）
    fn get_var(&self, name: &str) -> Result<Value, String>;

    /// 获取指定版本的变量值
    fn get_versioned_var(&self, name: &str, version: i64) -> Result<Value, String>;

    /// 设置变量值
    fn set_var(&mut self, name: &str, value: Value) -> Result<(), String>;

    /// 设置表达式内部变量
    fn set_inner_var(&mut self, var: &str, value: Value);

    /// 获取表达式内部变量
    fn get_inner_var(&self, var: &str) -> Option<Value>;

    /// 获取变量属性值
    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String>;

    /// 获取目标顶点属性值
    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取输入属性值
    fn get_input_prop(&self, prop: &str) -> Result<Value, String>;

    /// 获取输入属性索引
    fn get_input_prop_index(&self, prop: &str) -> Result<usize, String>;

    /// 按列索引获取值
    fn get_column(&self, index: i32) -> Result<Value, String>;

    /// 获取标签属性值
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取边属性值
    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String>;

    /// 获取源顶点属性值
    fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;

    /// 获取顶点
    fn get_vertex(&self, name: &str) -> Result<Value, String>;

    /// 获取边
    fn get_edge(&self) -> Result<Value, String>;
}

/// 表达式求值上下文枚举
///
/// 使用枚举实现零成本抽象，避免动态分发的性能开销
/// 同时提供更好的类型安全性和扩展性
#[derive(Clone, Debug)]
pub enum ExpressionContextEnum {
    /// 默认上下文实现
    Default(DefaultExpressionContext),
    /// 查询上下文适配器
    Query(QueryContextAdapter),
    /// 基础表达式上下文
    Basic(crate::core::expressions::BasicExpressionContext),
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

impl ExpressionContext for ExpressionContextEnum {
    fn get_variable(&self, name: &str) -> Option<Value> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.vars.get(name).cloned(),
            ExpressionContextEnum::Query(ctx) => ctx.vars.get(name).cloned(),
            ExpressionContextEnum::Basic(ctx) => ctx.get_variable(name),
        }
    }

    fn set_variable(&mut self, name: String, value: Value) {
        match self {
            ExpressionContextEnum::Default(ctx) => {
                ctx.vars.insert(name, value);
            }
            ExpressionContextEnum::Query(ctx) => {
                ctx.vars.insert(name, value);
            }
            ExpressionContextEnum::Basic(ctx) => {
                // 将 Value 转换为 FieldValue
                let field_value = match value {
                    Value::Bool(b) => crate::core::types::query::FieldValue::Scalar(
                        crate::core::types::query::ScalarValue::Bool(b),
                    ),
                    Value::Int(i) => crate::core::types::query::FieldValue::Scalar(
                        crate::core::types::query::ScalarValue::Int(i),
                    ),
                    Value::Float(f) => crate::core::types::query::FieldValue::Scalar(
                        crate::core::types::query::ScalarValue::Float(f),
                    ),
                    Value::String(s) => crate::core::types::query::FieldValue::Scalar(
                        crate::core::types::query::ScalarValue::String(s),
                    ),
                    Value::Null(_) => crate::core::types::query::FieldValue::Scalar(
                        crate::core::types::query::ScalarValue::Null,
                    ),
                    _ => {
                        // 对于复杂类型，暂时返回空值
                        crate::core::types::query::FieldValue::Scalar(
                            crate::core::types::query::ScalarValue::Null,
                        )
                    }
                };
                ctx.set_variable(name, field_value);
            }
        }
    }

    fn get_vertex(&self) -> Option<&Vertex> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.vertex.as_ref(),
            ExpressionContextEnum::Query(ctx) => ctx.vertex.as_ref(),
            ExpressionContextEnum::Basic(ctx) => ctx.get_vertex(),
        }
    }

    fn get_edge(&self) -> Option<&Edge> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.edge.as_ref(),
            ExpressionContextEnum::Query(ctx) => ctx.edge.as_ref(),
            ExpressionContextEnum::Basic(ctx) => ctx.get_edge(),
        }
    }

    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.paths.get(name),
            ExpressionContextEnum::Query(ctx) => ctx.paths.get(name),
            ExpressionContextEnum::Basic(ctx) => ctx.get_path(name),
        }
    }

    fn set_vertex(&mut self, vertex: Vertex) {
        match self {
            ExpressionContextEnum::Default(ctx) => {
                ctx.vertex = Some(vertex);
            }
            ExpressionContextEnum::Query(ctx) => {
                ctx.vertex = Some(vertex);
            }
            ExpressionContextEnum::Basic(ctx) => {
                ctx.set_vertex(vertex);
            }
        }
    }

    fn set_edge(&mut self, edge: Edge) {
        match self {
            ExpressionContextEnum::Default(ctx) => {
                ctx.edge = Some(edge);
            }
            ExpressionContextEnum::Query(ctx) => {
                ctx.edge = Some(edge);
            }
            ExpressionContextEnum::Basic(ctx) => {
                ctx.set_edge(edge);
            }
        }
    }

    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        match self {
            ExpressionContextEnum::Default(ctx) => {
                ctx.paths.insert(name, path);
            }
            ExpressionContextEnum::Query(ctx) => {
                ctx.paths.insert(name, path);
            }
            ExpressionContextEnum::Basic(ctx) => {
                ctx.add_path(name, path);
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.is_empty(),
            ExpressionContextEnum::Query(ctx) => ctx.is_empty(),
            ExpressionContextEnum::Basic(ctx) => ctx.is_empty(),
        }
    }

    fn variable_count(&self) -> usize {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.variable_count(),
            ExpressionContextEnum::Query(ctx) => ctx.vars.len(),
            ExpressionContextEnum::Basic(ctx) => ctx.variable_count(),
        }
    }

    fn variable_names(&self) -> Vec<String> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.variable_names(),
            ExpressionContextEnum::Query(ctx) => ctx.vars.keys().cloned().collect(),
            ExpressionContextEnum::Basic(ctx) => ctx.variable_names(),
        }
    }

    fn get_all_variables(&self) -> Option<HashMap<String, Value>> {
        match self {
            ExpressionContextEnum::Default(ctx) => Some(ctx.vars.clone()),
            ExpressionContextEnum::Query(ctx) => Some(ctx.vars.clone()),
            ExpressionContextEnum::Basic(ctx) => ctx.get_all_variables(),
        }
    }

    fn clear(&mut self) {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.clear(),
            ExpressionContextEnum::Query(ctx) => {
                ctx.vertex = None;
                ctx.edge = None;
                ctx.vars.clear();
                ctx.paths.clear();
            }
            ExpressionContextEnum::Basic(ctx) => ctx.clear(),
        }
    }
    
    fn get_variable_names(&self) -> Vec<&str> {
        match self {
            ExpressionContextEnum::Default(ctx) => ctx.vars.keys().map(|k| k.as_str()).collect(),
            ExpressionContextEnum::Query(ctx) => ctx.vars.keys().map(|k| k.as_str()).collect(),
            ExpressionContextEnum::Basic(ctx) => ctx.get_variable_names(),
        }
    }
}

impl ExpressionContext for DefaultExpressionContext {
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
    
    fn get_variable_names(&self) -> Vec<&str> {
        self.vars.keys().map(|k| k.as_str()).collect()
    }
}

impl ExpressionContext for QueryContextAdapter {
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
    
    fn get_variable_names(&self) -> Vec<&str> {
        self.vars.keys().map(|k| k.as_str()).collect()
    }
}

impl ExpressionContextEnum {
    /// 创建简单上下文
    pub fn default() -> Self {
        ExpressionContextEnum::Default(DefaultExpressionContext::new())
    }

    /// 创建查询上下文适配器
    pub fn query() -> Self {
        ExpressionContextEnum::Query(QueryContextAdapter::new())
    }

    /// 创建基础表达式上下文
    pub fn basic() -> Self {
        ExpressionContextEnum::Basic(crate::core::expressions::BasicExpressionContext::new())
    }

    /// 从简单上下文创建
    pub fn from_default(default: DefaultExpressionContext) -> Self {
        ExpressionContextEnum::Default(default)
    }

    /// 从查询上下文适配器创建
    pub fn from_query(query: QueryContextAdapter) -> Self {
        ExpressionContextEnum::Query(query)
    }

    /// 从基础表达式上下文创建
    pub fn from_basic(basic: crate::core::expressions::BasicExpressionContext) -> Self {
        ExpressionContextEnum::Basic(basic)
    }

    /// 转换为简单上下文（如果可能）
    pub fn as_default(&self) -> Option<&DefaultExpressionContext> {
        match self {
            ExpressionContextEnum::Default(ctx) => Some(ctx),
            ExpressionContextEnum::Query(_) => None,
            ExpressionContextEnum::Basic(_) => None,
        }
    }

    /// 转换为可变简单上下文（如果可能）
    pub fn as_default_mut(&mut self) -> Option<&mut DefaultExpressionContext> {
        match self {
            ExpressionContextEnum::Default(ctx) => Some(ctx),
            ExpressionContextEnum::Query(_) => None,
            ExpressionContextEnum::Basic(_) => None,
        }
    }

    /// 转换为查询上下文适配器（如果可能）
    pub fn as_query(&self) -> Option<&QueryContextAdapter> {
        match self {
            ExpressionContextEnum::Query(ctx) => Some(ctx),
            ExpressionContextEnum::Default(_) => None,
            ExpressionContextEnum::Basic(_) => None,
        }
    }

    /// 转换为可变查询上下文适配器（如果可能）
    pub fn as_query_mut(&mut self) -> Option<&mut QueryContextAdapter> {
        match self {
            ExpressionContextEnum::Query(ctx) => Some(ctx),
            ExpressionContextEnum::Default(_) => None,
            ExpressionContextEnum::Basic(_) => None,
        }
    }

    /// 转换为基础表达式上下文（如果可能）
    pub fn as_basic(&self) -> Option<&crate::core::expressions::BasicExpressionContext> {
        match self {
            ExpressionContextEnum::Basic(ctx) => Some(ctx),
            ExpressionContextEnum::Default(_) => None,
            ExpressionContextEnum::Query(_) => None,
        }
    }

    /// 转换为可变基础表达式上下文（如果可能）
    pub fn as_basic_mut(&mut self) -> Option<&mut crate::core::expressions::BasicExpressionContext> {
        match self {
            ExpressionContextEnum::Basic(ctx) => Some(ctx),
            ExpressionContextEnum::Default(_) => None,
            ExpressionContextEnum::Query(_) => None,
        }
    }
}

impl Default for ExpressionContextEnum {
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
    context: ExpressionContextEnum,
}

impl ExpressionContextBuilder {
    pub fn new() -> Self {
        Self {
            context: ExpressionContextEnum::default(),
        }
    }

    pub fn query() -> Self {
        Self {
            context: ExpressionContextEnum::query(),
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
        for (name, value) in variables {
            self.context.set_variable(name, value);
        }
        self
    }

    pub fn with_path(mut self, name: String, path: crate::core::vertex_edge_path::Path) -> Self {
        self.context.add_path(name, path);
        self
    }

    pub fn build(self) -> ExpressionContextEnum {
        self.context
    }
}

impl Default for ExpressionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建带有初始数据的上下文
pub fn with_variables<I>(variables: I) -> ExpressionContextEnum
where
    I: IntoIterator<Item = (String, Value)>,
{
    ExpressionContextBuilder::new()
        .with_variables(variables)
        .build()
}

/// 便捷函数：创建带有顶点的上下文
pub fn with_vertex(vertex: Vertex) -> ExpressionContextEnum {
    ExpressionContextBuilder::new().with_vertex(vertex).build()
}

/// 便捷函数：创建带有边的上下文
pub fn with_edge(edge: Edge) -> ExpressionContextEnum {
    ExpressionContextBuilder::new().with_edge(edge).build()
}

// 为ExpressionContext添加批量设置变量的方法
impl ExpressionContextEnum {
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