use crate::core::{Direction, Edge, StorageError, Value, Vertex};

/// Transaction identifier
pub type TransactionId = u64;

/// Storage engine trait defining the interface for graph storage
pub trait StorageEngine: Send + Sync {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    /// 全表扫描所有顶点
    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError>;
    /// 按标签扫描顶点
    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError>;
    /// 按属性扫描顶点
    fn scan_vertices_by_prop(&self, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError>;

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: Direction,
    ) -> Result<Vec<Edge>, StorageError>;
    fn get_node_edges_filtered(
        &self,
        node_id: &Value,
        direction: Direction,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
    /// 按类型扫描边
    fn scan_edges_by_type(&self, edge_type: &str) -> Result<Vec<Edge>, StorageError>;
    /// 全表扫描所有边
    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError>;

    /// 批量插入顶点
    fn batch_insert_nodes(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;
    /// 批量插入边
    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError>;

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
}
