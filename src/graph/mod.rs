//! 图操作核心模块
//!
//! 包含图相关的核心操作，包括事务管理、索引系统和表达式计算

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::core::{Vertex, Edge, Value, Path};

pub mod transaction;
pub mod index;
pub mod expression;

// 重新导出图操作相关功能
pub use transaction::*;
pub use index::*;
pub use expression::*;

/// Represents the response from a graph database query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub data: GraphData,
    pub execution_time_ms: u64,
    pub message: Option<String>,
    pub success: bool,
}

/// Represents the data part of a graph response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphData {
    /// Single vertex result
    Vertex(Vertex),
    /// List of vertices
    Vertices(Vec<Vertex>),
    /// Single edge result
    Edge(Edge),
    /// List of edges
    Edges(Vec<Edge>),
    /// Path result
    Path(Path),
    /// List of paths
    Paths(Vec<Path>),
    /// Scalar value result (e.g., count)
    Scalar(Value),
    /// Multiple scalar values
    Scalars(Vec<Value>),
    /// Key-value pairs result
    KeyValue(HashMap<String, Value>),
    /// Multiple key-value pairs
    KeyValues(Vec<HashMap<String, Value>>),
    /// Empty result
    Empty,
}

/// Represents a result set with potentially multiple types of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub stats: ExecutionStats,
}

/// Statistics about query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_vertices: usize,
    pub total_edges: usize,
    pub vertices_scanned: usize,
    pub edges_scanned: usize,
    pub execution_time_ms: u64,
    pub memory_used_bytes: usize,
}

impl ExecutionStats {
    pub fn new() -> Self {
        Self {
            total_vertices: 0,
            total_edges: 0,
            vertices_scanned: 0,
            edges_scanned: 0,
            execution_time_ms: 0,
            memory_used_bytes: 0,
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a formatted response for API output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            code: 200,
            message,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String, code: u16) -> Self {
        Self {
            code,
            message: "Error occurred".to_string(),
            data: None,
            error: Some(error),
        }
    }

    pub fn empty(message: String) -> Self {
        Self {
            code: 200,
            message,
            data: None,
            error: None,
        }
    }
}

/// Represents a batch of graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub operations: Vec<GraphOperation>,
    pub atomic: bool, // Whether all operations should succeed or fail together
}

/// Represents a single graph operation within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphOperation {
    CreateVertex { vertex: Vertex },
    CreateEdge { edge: Edge },
    UpdateVertex { vid: Value, properties: HashMap<String, Value> },
    UpdateEdge { src: Value, dst: Value, edge_type: String, properties: HashMap<String, Value> },
    DeleteVertex { vid: Value },
    DeleteEdge { src: Value, dst: Value, edge_type: String },
    ReadVertex { vid: Value },
    ReadEdge { src: Value, dst: Value, edge_type: String },
}

/// Represents a graph schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDef {
    pub vertex_types: HashMap<String, Vec<PropertyDef>>,
    pub edge_types: HashMap<String, Vec<PropertyDef>>,
    pub indexes: Vec<IndexDef>,
}

/// Represents a property definition in the schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub indexed: bool,
}

/// Represents the data type of a property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List(Box<DataType>),
    Map(String, Box<DataType>), // (key_type, value_type)
    Custom(String), // Custom type name
}

/// Represents an index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: String,
    pub entity_type: EntityType,
    pub property_name: String,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Vertex(String), // vertex type name
    Edge(String),   // edge type name
}

impl GraphResponse {
    pub fn new(data: GraphData, execution_time_ms: u64, success: bool) -> Self {
        Self {
            data,
            execution_time_ms,
            message: None,
            success,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn success_with_data(data: GraphData, execution_time_ms: u64) -> Self {
        Self::new(data, execution_time_ms, true)
    }

    pub fn error_with_message(message: String, execution_time_ms: u64) -> Self {
        Self {
            data: GraphData::Empty,
            execution_time_ms,
            message: Some(message),
            success: false,
        }
    }
}

impl ResultSet {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            stats: ExecutionStats::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<Value>) {
        if row.len() == self.columns.len() {
            self.rows.push(row);
        }
    }

    pub fn with_stats(mut self, stats: ExecutionStats) -> Self {
        self.stats = stats;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Tag, NullType};

    #[test]
    fn test_graph_response() {
        let vertex = Vertex::new(
            Value::Int(1),
            vec![Tag::new("person".to_string(), HashMap::new())]
        );
        
        let response = GraphResponse::success_with_data(
            GraphData::Vertex(vertex),
            10
        );
        
        assert!(response.success);
        assert_eq!(response.execution_time_ms, 10);
    }

    #[test]
    fn test_result_set() {
        let mut result_set = ResultSet::new(vec!["name".to_string(), "age".to_string()]);
        result_set.add_row(vec![Value::String("Alice".to_string()), Value::Int(30)]);
        result_set.add_row(vec![Value::String("Bob".to_string()), Value::Int(25)]);
        
        assert_eq!(result_set.columns, vec!["name", "age"]);
        assert_eq!(result_set.rows.len(), 2);
    }

    #[test]
    fn test_api_response() {
        let data = vec![1, 2, 3];
        let api_response = ApiResponse::success(data, "Query executed successfully".to_string());
        
        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());
        assert!(api_response.error.is_none());
    }
}