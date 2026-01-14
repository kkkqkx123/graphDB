//! 默认表达式上下文实现
//!
//! 包含默认上下文的实现

use crate::core::{Edge, Value, Vertex};
use crate::core::error::ExpressionError;
use crate::expression::evaluator::traits::ExpressionContext;
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

    /// 获取顶点
    fn get_vertex(&self, name: &str) -> Result<Value, ExpressionError>;

    /// 获取边
    fn get_edge(&self) -> Result<Value, ExpressionError>;
}
/// 简单的表达式上下文实现
///
/// 轻量级上下文，适用于大部分表达式求值场景
/// 如需更复杂的功能（函数注册、嵌套作用域等），请使用 BasicExpressionContext
#[derive(Clone, Debug)]
pub struct DefaultExpressionContext {
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

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageExpressionContext for DefaultExpressionContext {
    fn get_var(&self, name: &str) -> Result<Value, ExpressionError> {
        self.vars.get(name)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(name))
    }

    fn get_versioned_var(&self, name: &str, _version: i64) -> Result<Value, ExpressionError> {
        self.vars.get(name)
            .cloned()
            .ok_or_else(|| ExpressionError::undefined_variable(name))
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<(), ExpressionError> {
        self.vars.insert(name.to_string(), value);
        Ok(())
    }

    fn set_inner_var(&mut self, var: &str, value: Value) {
        self.vars.insert(var.to_string(), value);
    }

    fn get_inner_var(&self, var: &str) -> Option<Value> {
        self.vars.get(var).cloned()
    }

    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, ExpressionError> {
        let var_value = self.vars.get(var)
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
        self.vars.get(prop)
            .cloned()
            .ok_or_else(|| ExpressionError::property_not_found(prop))
    }

    fn get_input_prop_index(&self, prop: &str) -> Result<usize, ExpressionError> {
        if let Some(Value::List(list)) = self.vars.get(prop) {
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
        for value in self.vars.values() {
            if let Value::List(list) = value {
                if idx < list.len() {
                    return Ok(list[idx].clone());
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

    fn get_vertex(&self, name: &str) -> Result<Value, ExpressionError> {
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

    fn get_edge(&self) -> Result<Value, ExpressionError> {
        self.edge.as_ref()
            .map(|e| Value::Edge(e.clone()))
            .ok_or_else(|| ExpressionError::type_error("上下文中没有边"))
    }
}
