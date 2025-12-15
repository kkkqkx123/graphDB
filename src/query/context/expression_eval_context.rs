use std::collections::HashMap;
use crate::core::{Value, Vertex, Edge};
use serde::{Deserialize, Serialize};

/// Context for evaluating expressions, containing values for variables/properties
#[derive(Clone, Debug)]
pub struct EvalContext<'a> {
    pub vertex: Option<&'a Vertex>,
    pub edge: Option<&'a Edge>,
    pub vars: HashMap<String, Value>,
}

/// 可序列化的EvalContext变体
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializableEvalContext {
    pub vertex: Option<Vertex>,
    pub edge: Option<Edge>,
    pub vars: HashMap<String, Value>,
}

impl<'a> EvalContext<'a> {
    pub fn new() -> Self {
        Self {
            vertex: None,
            edge: None,
            vars: HashMap::new(),
        }
    }

    pub fn with_vertex(vertex: &'a Vertex) -> Self {
        Self {
            vertex: Some(vertex),
            edge: None,
            vars: HashMap::new(),
        }
    }

    pub fn with_edge(edge: &'a Edge) -> Self {
        Self {
            vertex: None,
            edge: Some(edge),
            vars: HashMap::new(),
        }
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

impl<'a> From<&'a SerializableEvalContext> for EvalContext<'a> {
    fn from(ctx: &'a SerializableEvalContext) -> Self {
        EvalContext {
            vertex: ctx.vertex.as_ref(),
            edge: ctx.edge.as_ref(),
            vars: ctx.vars.clone(),
        }
    }
}

impl From<EvalContext<'_>> for SerializableEvalContext {
    fn from(ctx: EvalContext) -> Self {
        SerializableEvalContext {
            vertex: ctx.vertex.cloned(),
            edge: ctx.edge.cloned(),
            vars: ctx.vars,
        }
    }
}