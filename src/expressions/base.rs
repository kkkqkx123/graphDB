use std::collections::HashMap;
use crate::core::Value;

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationError {
    InvalidOperation(String),
    TypeError(String),
    UndefinedVariable(String),
    DivisionByZero,
    Other(String),
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            EvaluationError::TypeError(msg) => write!(f, "Type error: {}", msg),
            EvaluationError::UndefinedVariable(var) => write!(f, "Undefined variable: {}", var),
            EvaluationError::DivisionByZero => write!(f, "Division by zero"),
            EvaluationError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for EvaluationError {}

use crate::core::{Vertex, Edge, Path};

/// The expression evaluation context trait
pub trait ExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError>;
    fn get_tag_property(&self, tag: &str, property: &str) -> Result<Value, EvaluationError>;
    fn get_edge_property(&self, edge: &str, property: &str) -> Result<Value, EvaluationError>;
    fn get_src_vertex(&self) -> Result<Value, EvaluationError>;
    fn get_dst_vertex(&self) -> Result<Value, EvaluationError>;
    fn get_current_vertex(&self) -> Result<Value, EvaluationError>;
    fn get_current_edge(&self) -> Result<Value, EvaluationError>;
}

/// A default implementation of the expression context
#[derive(Default, Debug)]
pub struct DefaultExpressionContext {
    variables: HashMap<String, Value>,
    tag_properties: HashMap<String, HashMap<String, Value>>,
    edge_properties: HashMap<String, HashMap<String, Value>>,
    current_vertex: Option<Vertex>,
    current_edge: Option<Edge>,
    current_path: Option<Path>,
}

impl DefaultExpressionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            tag_properties: HashMap::new(),
            edge_properties: HashMap::new(),
            current_vertex: None,
            current_edge: None,
            current_path: None,
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Result<Value, EvaluationError> {
        self.variables.get(name).cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.to_string()))
    }

    pub fn set_tag_property(&mut self, tag: String, property: String, value: Value) {
        self.tag_properties.entry(tag).or_insert_with(HashMap::new).insert(property, value);
    }

    pub fn set_edge_property(&mut self, edge: String, property: String, value: Value) {
        self.edge_properties.entry(edge).or_insert_with(HashMap::new).insert(property, value);
    }

    pub fn set_current_vertex(&mut self, vertex: Vertex) {
        self.current_vertex = Some(vertex);
    }

    pub fn set_current_edge(&mut self, edge: Edge) {
        self.current_edge = Some(edge);
    }

    pub fn set_current_path(&mut self, path: Path) {
        self.current_path = Some(path);
    }
}

impl ExpressionContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError> {
        self.get_variable(name)
    }

    fn get_tag_property(&self, tag: &str, property: &str) -> Result<Value, EvaluationError> {
        self.tag_properties.get(tag)
            .and_then(|properties| properties.get(property))
            .cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(format!("{}.{}", tag, property)))
    }

    fn get_edge_property(&self, edge: &str, property: &str) -> Result<Value, EvaluationError> {
        self.edge_properties.get(edge)
            .and_then(|properties| properties.get(property))
            .cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(format!("{}.{}", edge, property)))
    }

    fn get_src_vertex(&self) -> Result<Value, EvaluationError> {
        self.current_vertex.as_ref().cloned().map(|v| Value::Vertex(Box::new(v))).ok_or_else(||
            EvaluationError::Other("No source vertex available".to_string())
        )
    }

    fn get_dst_vertex(&self) -> Result<Value, EvaluationError> {
        // For simplicity, returning the current vertex as destination
        // In a real implementation, we would have separate source/dst vertices
        self.current_vertex.as_ref().cloned().map(|v| Value::Vertex(Box::new(v))).ok_or_else(||
            EvaluationError::Other("No destination vertex available".to_string())
        )
    }

    fn get_current_vertex(&self) -> Result<Value, EvaluationError> {
        self.current_vertex.as_ref().cloned().map(|v| Value::Vertex(Box::new(v))).ok_or_else(||
            EvaluationError::Other("No current vertex available".to_string())
        )
    }

    fn get_current_edge(&self) -> Result<Value, EvaluationError> {
        self.current_edge.as_ref().cloned().map(|e| Value::Edge(e)).ok_or_else(||
            EvaluationError::Other("No current edge available".to_string())
        )
    }
}