use std::collections::HashMap;
use crate::core::{Value, Vertex, Edge};

/// Context for evaluating expressions, containing values for variables/properties
#[derive(Clone)]
pub struct EvalContext<'a> {
    pub vertex: Option<&'a Vertex>,
    pub edge: Option<&'a Edge>,
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
}