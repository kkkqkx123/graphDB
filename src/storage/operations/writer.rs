use crate::core::{Edge, Value, Vertex, StorageError};

pub trait VertexWriter: Send + Sync {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError>;
    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError>;
    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;
}

pub trait EdgeWriter: Send + Sync {
    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError>;
    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError>;
}
