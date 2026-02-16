use crate::core::{Edge, Value, Vertex, StorageError};

pub trait VertexWriter: Send + Sync {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError>;
    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError>;
    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;
    
    /// 删除顶点上的指定标签
    /// 
    /// # Arguments
    /// * `space` - 空间名称
    /// * `vertex_id` - 顶点ID
    /// * `tag_names` - 要删除的标签名列表
    /// 
    /// # Returns
    /// * `Ok(usize)` - 成功删除的标签数量
    /// * `Err(StorageError)` - 存储错误
    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError>;
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
