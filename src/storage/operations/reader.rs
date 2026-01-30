use crate::core::{Edge, EdgeDirection, Value, Vertex, StorageError};

pub trait VertexReader: Send + Sync {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError>;
    fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag_name: &str,
    ) -> Result<ScanResult<Vertex>, StorageError>;
    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag_name: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError>;
}

pub trait EdgeReader: Send + Sync {
    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError>;
    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        edge_type: Option<&str>,
    ) -> Result<ScanResult<Edge>, StorageError>;
    fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError>;
    fn scan_all_edges(&self, space: &str) -> Result<ScanResult<Edge>, StorageError>;
}

pub struct ScanResult<T> {
    data: Vec<T>,
    consumed: bool,
}

impl<T> ScanResult<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            consumed: false,
        }
    }

    pub fn into_vec(mut self) -> Vec<T> {
        self.consumed = true;
        std::mem::take(&mut self.data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T> IntoIterator for ScanResult<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a ScanResult<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
