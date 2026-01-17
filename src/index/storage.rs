//! 并发安全的索引存储实现
//!
//! 使用 DashMap 提供高性能的并发访问：
//! - 细粒度锁，避免读操作阻塞
//! - 高并发读写性能
//! - 支持前缀查询和范围查询
//!
//! 复用 cache 模块的统计收集功能

use crate::core::error::{ManagerError, ManagerResult};
use crate::core::{Edge, Value, Vertex};
use crate::index::{IndexBinaryEncoder, IndexField, QueryType};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct IndexEntry {
    pub vertex: Option<Vertex>,
    pub edge: Option<Edge>,
    pub created_at: i64,
    pub access_count: Arc<AtomicU64>,
    pub last_accessed: Arc<AtomicU64>,
}

impl Clone for IndexEntry {
    fn clone(&self) -> Self {
        Self {
            vertex: self.vertex.clone(),
            edge: self.edge.clone(),
            created_at: self.created_at,
            access_count: Arc::clone(&self.access_count),
            last_accessed: Arc::clone(&self.last_accessed),
        }
    }
}

impl IndexEntry {
    pub fn new_vertex(vertex: Vertex) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            vertex: Some(vertex),
            edge: None,
            created_at: now as i64,
            access_count: Arc::new(AtomicU64::new(0)),
            last_accessed: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn new_edge(edge: Edge) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            vertex: None,
            edge: Some(edge),
            created_at: now as i64,
            access_count: Arc::new(AtomicU64::new(0)),
            last_accessed: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn touch(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_accessed.store(now, Ordering::Relaxed);
        self.access_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn is_vertex(&self) -> bool {
        self.vertex.is_some()
    }

    pub fn is_edge(&self) -> bool {
        self.edge.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ConcurrentIndexStorage {
    space_id: i32,
    index_id: i32,
    index_name: String,
    primary_index: DashMap<Vec<u8>, Vec<IndexEntry>>,
    vertex_by_id: DashMap<i64, Vertex>,
    edge_by_id: DashMap<i64, Edge>,
    field_indexes: DashMap<String, DashMap<Vec<u8>, Vec<IndexEntry>>>,
    query_stats: crate::index::IndexQueryStats,
    entry_count: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    last_updated: Arc<RwLock<i64>>,
}

impl ConcurrentIndexStorage {
    pub fn new(space_id: i32, index_id: i32, index_name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        Self {
            space_id,
            index_id,
            index_name,
            primary_index: DashMap::new(),
            vertex_by_id: DashMap::new(),
            edge_by_id: DashMap::new(),
            field_indexes: DashMap::new(),
            query_stats: crate::index::IndexQueryStats::new(),
            entry_count: Arc::new(AtomicU64::new(0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            last_updated: Arc::new(RwLock::new(now)),
        }
    }

    pub fn insert_vertex(&self, field_name: &str, field_value: &Value, vertex: Vertex) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let mut last = self.last_updated.write().unwrap();
        *last = now;
        drop(last);

        let key = IndexBinaryEncoder::encode_value(field_value);
        let entry = IndexEntry::new_vertex(vertex.clone());
        
        self.vertex_by_id.insert(vertex.id(), vertex);
        
        self.primary_index
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        let field_index = self.field_indexes
            .entry(field_name.to_string())
            .or_insert_with(DashMap::new);
        field_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push(entry);
        
        self.entry_count.fetch_add(1, Ordering::Relaxed);
        self.update_memory_usage();
    }

    pub fn insert_edge(&self, field_name: &str, field_value: &Value, edge: Edge) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let mut last = self.last_updated.write().unwrap();
        *last = now;
        drop(last);

        let key = IndexBinaryEncoder::encode_value(field_value);
        let entry = IndexEntry::new_edge(edge.clone());
        
        self.edge_by_id.insert(edge.id, edge);
        
        self.primary_index
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        let field_index = self.field_indexes
            .entry(field_name.to_string())
            .or_insert_with(DashMap::new);
        field_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push(entry);
        
        self.entry_count.fetch_add(1, Ordering::Relaxed);
        self.update_memory_usage();
    }

    pub fn exact_lookup(&self, field_name: &str, field_value: &Value) -> ManagerResult<(Vec<Vertex>, Vec<Edge>, Duration)> {
        let start = Instant::now();
        
        let key = IndexBinaryEncoder::encode_value(field_value);
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;
        
        if let Some(field_index) = self.field_indexes.get(field_name) {
            if let Some(entries) = field_index.get(&key) {
                for entry in entries.iter() {
                    entry.touch();
                    found = true;
                    if let Some(v) = &entry.vertex {
                        vertices.push(v.clone());
                    } else if let Some(e) = &entry.edge {
                        edges.push(e.clone());
                    }
                }
            }
        }
        
        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Exact);
        
        Ok((vertices, edges, duration))
    }

    pub fn prefix_lookup(&self, field_name: &str, prefix: &[Value]) -> ManagerResult<(Vec<Vertex>, Vec<Edge>, Duration)> {
        let start = Instant::now();
        
        let prefix_bytes = IndexBinaryEncoder::encode_prefix(prefix, prefix.len());
        let (start_key, end_key) = IndexBinaryEncoder::encode_prefix_range(&prefix_bytes);
        
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;
        
        for item in self.field_indexes.iter() {
            let key = item.key();
            let key_bytes = key.as_bytes();
            if key_bytes >= start_key.as_slice() && key_bytes < end_key.as_slice() {
                let inner_map = item.value();
                for inner_item in inner_map.iter() {
                    for entry in inner_item.value().iter() {
                        entry.touch();
                        found = true;
                        if let Some(v) = &entry.vertex {
                            vertices.push(v.clone());
                        } else if let Some(e) = &entry.edge {
                            edges.push(e.clone());
                        }
                    }
                }
            }
        }
        
        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Prefix);
        
        Ok((vertices, edges, duration))
    }

    pub fn range_lookup(
        &self,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> ManagerResult<(Vec<Vertex>, Vec<Edge>, Duration)> {
        let start = Instant::now();
        
        let start_key = IndexBinaryEncoder::encode_value(start_value);
        let mut end_key = IndexBinaryEncoder::encode_value(end_value);
        end_key.push(0xFFu8);
        
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;
        
        for item in self.field_indexes.iter() {
            let key = item.key();
            let key_bytes = key.as_bytes();
            if key_bytes >= start_key.as_slice() && key_bytes < end_key.as_slice() {
                let inner_map = item.value();
                for inner_item in inner_map.iter() {
                    for entry in inner_item.value().iter() {
                        entry.touch();
                        found = true;
                        if let Some(v) = &entry.vertex {
                            vertices.push(v.clone());
                        } else if let Some(e) = &entry.edge {
                            edges.push(e.clone());
                        }
                    }
                }
            }
        }
        
        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Range);
        
        Ok((vertices, edges, duration))
    }

    pub fn delete_vertex(&self, vertex: &Vertex) {
        self.vertex_by_id.remove(&vertex.id());
        self.entry_count.fetch_sub(1, Ordering::Relaxed);
        self.update_memory_usage();
    }

    pub fn delete_edge(&self, edge: &Edge) {
        self.edge_by_id.remove(&edge.id);
        self.entry_count.fetch_sub(1, Ordering::Relaxed);
        self.update_memory_usage();
    }

    pub fn clear(&self) {
        self.primary_index.clear();
        self.vertex_by_id.clear();
        self.edge_by_id.clear();
        self.field_indexes.clear();
        self.entry_count.store(0, Ordering::Relaxed);
        self.query_stats.reset();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let mut last = self.last_updated.write().unwrap();
        *last = now;
    }

    pub fn get_query_stats(&self) -> &crate::index::IndexQueryStats {
        &self.query_stats
    }

    pub fn get_entry_count(&self) -> usize {
        self.entry_count.load(Ordering::Relaxed) as usize
    }

    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed) as usize
    }

    fn update_memory_usage(&self) {
        let size = self.primary_index.len() + 
                   self.vertex_by_id.len() + 
                   self.edge_by_id.len() + 
                   self.field_indexes.len();
        self.memory_usage.store(size as u64, Ordering::Relaxed);
    }
}

pub struct ConcurrentIndexManager {
    space_id: i32,
    storages: DashMap<i32, ConcurrentIndexStorage>,
    index_metadata: DashMap<i32, IndexMetadata>,
    global_stats: Arc<crate::index::IndexQueryStats>,
}

#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub id: i32,
    pub name: String,
    pub space_id: i32,
    pub schema_name: String,
    pub fields: Vec<IndexField>,
    pub is_unique: bool,
    pub status: IndexStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexStatus {
    Active,
    Building,
    Dropped,
    Failed(String),
}



impl Default for ConcurrentIndexManager {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ConcurrentIndexManager {
    pub fn new(space_id: i32) -> Self {
        Self {
            space_id,
            storages: DashMap::new(),
            index_metadata: DashMap::new(),
            global_stats: Arc::new(crate::index::IndexQueryStats::new()),
        }
    }

    pub fn create_index(&self, name: &str, schema_name: &str, fields: Vec<IndexField>, is_unique: bool) -> ManagerResult<i32> {
        let index_id = self.storages.len() as i32 + 1;
        
        let storage = ConcurrentIndexStorage::new(self.space_id, index_id, name.to_string());
        self.storages.insert(index_id, storage);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let metadata = IndexMetadata {
            id: index_id,
            name: name.to_string(),
            space_id: self.space_id,
            schema_name: schema_name.to_string(),
            fields,
            is_unique,
            status: IndexStatus::Active,
            created_at: now,
        };
        self.index_metadata.insert(index_id, metadata);
        
        Ok(index_id)
    }

    pub fn drop_index(&self, index_id: i32) -> ManagerResult<()> {
        self.storages.remove(&index_id);
        self.index_metadata.remove(&index_id);
        Ok(())
    }

    pub fn insert_vertex(&self, index_id: i32, field_name: &str, field_value: &Value, vertex: Vertex) -> ManagerResult<()> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.insert_vertex(field_name, field_value, vertex);
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn insert_edge(&self, index_id: i32, field_name: &str, field_value: &Value, edge: Edge) -> ManagerResult<()> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.insert_edge(field_name, field_value, edge);
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn exact_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        field_value: &Value,
    ) -> ManagerResult<(Vec<Vertex>, Vec<Edge>)> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.exact_lookup(field_name, field_value)?;
            Ok((vertices, edges))
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn prefix_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        prefix: &[Value],
    ) -> ManagerResult<(Vec<Vertex>, Vec<Edge>)> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.prefix_lookup(field_name, prefix)?;
            Ok((vertices, edges))
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn range_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> ManagerResult<(Vec<Vertex>, Vec<Edge>)> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.range_lookup(field_name, start_value, end_value)?;
            Ok((vertices, edges))
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn delete_vertex(&self, index_id: i32, vertex: &Vertex) -> ManagerResult<()> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.delete_vertex(vertex);
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn delete_edge(&self, index_id: i32, edge: &Edge) -> ManagerResult<()> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.delete_edge(edge);
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("索引 {} 不存在", index_id)))
        }
    }

    pub fn get_global_stats(&self) -> &crate::index::IndexQueryStats {
        &self.global_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;

    fn create_test_vertex(id: i64, name: &str, age: i64) -> Vertex {
        Vertex {
            vid: Box::new(Value::Int(id)),
            id,
            tags: vec![Tag {
                name: "person".to_string(),
                properties: vec![
                    ("name".to_string(), Value::String(name.to_string())),
                    ("age".to_string(), Value::Int(age)),
                ]
                .into_iter()
                .collect(),
            }],
            properties: vec![
                ("name".to_string(), Value::String(name.to_string())),
                ("age".to_string(), Value::Int(age)),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn create_test_edge(id: i64, edge_type: &str, weight: f64) -> Edge {
        Edge {
            src: Box::new(Value::Int(1)),
            dst: Box::new(Value::Int(2)),
            edge_type: edge_type.to_string(),
            props: vec![
                ("weight".to_string(), Value::Float(weight)),
            ]
            .into_iter()
            .collect(),
            ranking: 0,
            id,
        }
    }

    #[test]
    fn test_concurrent_index_storage_insert() {
        let storage = ConcurrentIndexStorage::new(1, 1, "test".to_string());

        let vertex = create_test_vertex(1, "Alice", 30);
        storage.insert_vertex("name", &Value::String("Alice".to_string()), vertex.clone());

        assert_eq!(storage.get_entry_count(), 1);

        let (vertices, _, _) = storage.exact_lookup("name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].vid, Box::new(Value::Int(1)));
    }

    #[test]
    fn test_concurrent_index_storage_prefix_lookup() {
        let storage = ConcurrentIndexStorage::new(1, 1, "test".to_string());

        let vertex1 = create_test_vertex(1, "Alice", 30);
        let vertex2 = create_test_vertex(2, "Bob", 25);
        let vertex3 = create_test_vertex(3, "Alex", 35);

        storage.insert_vertex("name", &Value::String("Alice".to_string()), vertex1);
        storage.insert_vertex("name", &Value::String("Bob".to_string()), vertex2);
        storage.insert_vertex("name", &Value::String("Alex".to_string()), vertex3);

        let prefix = vec![Value::String("A".to_string())];
        let (vertices, _, _) = storage.prefix_lookup("name", &prefix).expect("Failed to perform prefix lookup in test");

        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn test_concurrent_index_storage_range_lookup() {
        let storage = ConcurrentIndexStorage::new(1, 1, "test".to_string());

        let vertex1 = create_test_vertex(1, "Alice", 20);
        let vertex2 = create_test_vertex(2, "Bob", 30);
        let vertex3 = create_test_vertex(3, "Charlie", 40);

        storage.insert_vertex("age", &Value::Int(20), vertex1.clone());
        storage.insert_vertex("age", &Value::Int(30), vertex2.clone());
        storage.insert_vertex("age", &Value::Int(40), vertex3.clone());

        let (vertices, _, _) = storage.range_lookup("age", &Value::Int(25), &Value::Int(35)).expect("Failed to perform range lookup in test");

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].vid, Box::new(Value::Int(2)));
    }

    #[test]
    fn test_concurrent_index_manager() {
        let manager = ConcurrentIndexManager::new(1);

        let fields = vec![
            IndexField {
                name: "name".to_string(),
                value_type: Value::String("".to_string()),
                is_nullable: false,
            },
        ];

        let index_id = manager.create_index("person_name", "person", fields, false).expect("Failed to create index in test");

        let vertex = create_test_vertex(1, "Alice", 30);
        manager.insert_vertex(index_id, "name", &Value::String("Alice".to_string()), vertex).expect("Failed to insert vertex in test");

        let (vertices, _) = manager.exact_lookup(index_id, "name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        assert_eq!(vertices.len(), 1);
    }

    #[test]
    fn test_concurrent_edge_index() {
        let storage = ConcurrentIndexStorage::new(1, 1, "test".to_string());

        let edge = create_test_edge(1, "friend", 0.9);
        storage.insert_edge("weight", &Value::Float(0.9), edge.clone());

        assert_eq!(storage.get_entry_count(), 1);

        let (_, edges, _) = storage.exact_lookup("weight", &Value::Float(0.9)).expect("Failed to perform exact lookup in test");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].id, 1);
    }

    #[test]
    fn test_query_stats() {
        let storage = ConcurrentIndexStorage::new(1, 1, "test".to_string());

        let vertex = create_test_vertex(1, "Alice", 30);
        storage.insert_vertex("name", &Value::String("Alice".to_string()), vertex);

        let _ = storage.exact_lookup("name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        let _ = storage.exact_lookup("name", &Value::String("NonExistent".to_string())).expect("Failed to perform exact lookup in test");

        let stats = storage.get_query_stats();
        assert_eq!(stats.query_count.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(stats.hit_count.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(stats.miss_count.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert!((stats.hit_rate() - 0.5).abs() < 0.01);
    }
}
