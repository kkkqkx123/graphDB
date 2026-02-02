//! 索引管理器接口
//!
//! 定义索引管理的基本操作接口

use crate::core::{Edge, Value, Vertex};
use crate::core::StorageResult;
use crate::index::{Index, IndexStatus, IndexStats, IndexOptimization};

pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn get_index(&self, name: &str) -> Option<Index>;
    fn list_indexes(&self) -> Vec<String>;
    fn has_index(&self, name: &str) -> bool;

    fn create_index(&self, space_id: i32, index: Index) -> StorageResult<i32>;
    fn drop_index(&self, space_id: i32, index_id: i32) -> StorageResult<()>;
    fn get_index_status(&self, space_id: i32, index_id: i32) -> Option<IndexStatus>;
    fn list_indexes_by_space(&self, space_id: i32) -> StorageResult<Vec<Index>>;

    fn lookup_vertex_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> StorageResult<Vec<Vertex>>;

    fn lookup_edge_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> StorageResult<Vec<Edge>>;

    fn range_lookup_vertex(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> StorageResult<Vec<Vertex>>;

    fn range_lookup_edge(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> StorageResult<Vec<Edge>>;

    fn insert_vertex_to_index(&self, space_id: i32, vertex: &Vertex) -> StorageResult<()>;
    fn delete_vertex_from_index(&self, space_id: i32, vertex: &Vertex) -> StorageResult<()>;
    fn update_vertex_in_index(
        &self,
        space_id: i32,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> StorageResult<()>;

    fn insert_edge_to_index(&self, space_id: i32, edge: &Edge) -> StorageResult<()>;
    fn delete_edge_from_index(&self, space_id: i32, edge: &Edge) -> StorageResult<()>;
    fn update_edge_in_index(
        &self,
        space_id: i32,
        old_edge: &Edge,
        new_edge: &Edge,
    ) -> StorageResult<()>;

    fn load_from_disk(&self) -> StorageResult<()>;
    fn save_to_disk(&self) -> StorageResult<()>;

    fn rebuild_index(&self, space_id: i32, index_id: i32) -> StorageResult<()>;
    fn rebuild_all_indexes(&self, space_id: i32) -> StorageResult<()>;
    fn get_index_stats(&self, space_id: i32, index_id: i32) -> StorageResult<IndexStats>;
    fn get_all_index_stats(&self, space_id: i32) -> StorageResult<Vec<IndexStats>>;
    fn analyze_index(&self, space_id: i32, index_id: i32) -> StorageResult<IndexOptimization>;
    fn analyze_all_indexes(&self, space_id: i32) -> StorageResult<Vec<IndexOptimization>>;
    fn check_index_consistency(&self, space_id: i32, index_id: i32) -> StorageResult<bool>;
    fn repair_index(&self, space_id: i32, index_id: i32) -> StorageResult<()>;
    fn cleanup_index(&self, space_id: i32, index_id: i32) -> StorageResult<()>;

    fn batch_insert_vertices(&self, space_id: i32, vertices: &[Vertex]) -> StorageResult<()>;
    fn batch_delete_vertices(&self, space_id: i32, vertices: &[Vertex]) -> StorageResult<()>;
    fn batch_insert_edges(&self, space_id: i32, edges: &[Edge]) -> StorageResult<()>;
    fn batch_delete_edges(&self, space_id: i32, edges: &[Edge]) -> StorageResult<()>;
}
