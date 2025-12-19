use crate::core::{Edge, Value, Vertex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for evaluating expressions, containing values for variables/properties
#[derive(Clone, Debug)]
pub struct EvalContext<'a> {
    pub vertex: Option<&'a Vertex>,
    pub edge: Option<&'a Edge>,
    pub vars: HashMap<String, Value>,
    pub paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

/// 可序列化的EvalContext变体
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializableEvalContext {
    pub vertex: Option<Vertex>,
    pub edge: Option<Edge>,
    pub vars: HashMap<String, Value>,
    pub paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

impl<'a> EvalContext<'a> {
    pub fn new() -> Self {
        Self {
            vertex: None,
            edge: None,
            vars: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn with_vertex(vertex: &'a Vertex) -> Self {
        Self {
            vertex: Some(vertex),
            edge: None,
            vars: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn with_edge(edge: &'a Edge) -> Self {
        Self {
            vertex: None,
            edge: Some(edge),
            vars: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    /// 添加路径
    pub fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        self.paths.insert(name, path);
    }

    /// 获取路径
    pub fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.paths.get(name)
    }
}

impl<'a> From<&'a SerializableEvalContext> for EvalContext<'a> {
    fn from(ctx: &'a SerializableEvalContext) -> Self {
        EvalContext {
            vertex: ctx.vertex.as_ref(),
            edge: ctx.edge.as_ref(),
            vars: ctx.vars.clone(),
            paths: ctx.paths.clone(),
        }
    }
}

impl From<EvalContext<'_>> for SerializableEvalContext {
    fn from(ctx: EvalContext) -> Self {
        SerializableEvalContext {
            vertex: ctx.vertex.cloned(),
            edge: ctx.edge.cloned(),
            vars: ctx.vars,
            paths: ctx.paths,
        }
    }
}
