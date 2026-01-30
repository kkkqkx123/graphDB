use crate::core::{Edge, Value, Vertex, StorageError};

pub trait VertexWriter: Send + Sync {
    fn insert_vertex(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn update_vertex(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_vertex(&mut self, id: &Value) -> Result<(), StorageError>;
    fn batch_insert_vertices(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;
}

pub trait EdgeWriter: Send + Sync {
    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError>;
}
