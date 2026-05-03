//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::core::{DataType, StorageError, StorageResult, Value};

use super::edge::{EdgeDirection, EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable, PropertyDef as EdgePropertyDef};
use super::vertex::{LabelId, PropertyDef as VertexPropertyDef, Timestamp, VertexId, VertexRecord, VertexSchema, VertexTable};

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub work_dir: PathBuf,
}

impl Default for PropertyGraphConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            work_dir: PathBuf::from("./data"),
        }
    }
}

pub struct PropertyGraph {
    vertex_tables: HashMap<LabelId, VertexTable>,
    edge_tables: HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
    vertex_label_names: HashMap<String, LabelId>,
    edge_label_names: HashMap<String, LabelId>,
    vertex_label_counter: LabelId,
    edge_label_counter: LabelId,
    config: PropertyGraphConfig,
    is_open: bool,
}

impl PropertyGraph {
    pub fn new() -> Self {
        Self::with_config(PropertyGraphConfig::default())
    }

    pub fn with_config(config: PropertyGraphConfig) -> Self {
        Self {
            vertex_tables: HashMap::new(),
            edge_tables: HashMap::new(),
            vertex_label_names: HashMap::new(),
            edge_label_names: HashMap::new(),
            vertex_label_counter: 0,
            edge_label_counter: 0,
            config,
            is_open: true,
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        Ok(Self::with_config(config))
    }

    pub fn close(&mut self) {
        self.is_open = false;
        for table in self.vertex_tables.values_mut() {
            table.close();
        }
        for table in self.edge_tables.values_mut() {
            table.close();
        }
    }

    pub fn create_vertex_type(
        &mut self,
        name: &str,
        properties: Vec<VertexPropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if self.vertex_label_names.contains_key(name) {
            return Err(StorageError::LabelAlreadyExists(name.to_string()));
        }

        let label_id = self.vertex_label_counter;
        self.vertex_label_counter += 1;

        let primary_key_index = properties
            .iter()
            .position(|p| p.name == primary_key)
            .ok_or_else(|| StorageError::PropertyNotFound(primary_key.to_string()))?;

        let schema = VertexSchema {
            label_id,
            label_name: name.to_string(),
            properties,
            primary_key_index,
        };

        let table = VertexTable::new(label_id, name.to_string(), schema);
        self.vertex_tables.insert(label_id, table);
        self.vertex_label_names.insert(name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn create_edge_type(
        &mut self,
        name: &str,
        src_label: LabelId,
        dst_label: LabelId,
        properties: Vec<EdgePropertyDef>,
        oe_strategy: EdgeStrategy,
        ie_strategy: EdgeStrategy,
    ) -> StorageResult<LabelId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if !self.vertex_tables.contains_key(&src_label) {
            return Err(StorageError::LabelNotFound(format!("source label {}", src_label)));
        }
        if !self.vertex_tables.contains_key(&dst_label) {
            return Err(StorageError::LabelNotFound(format!("destination label {}", dst_label)));
        }

        if self.edge_label_names.contains_key(name) {
            return Err(StorageError::LabelAlreadyExists(name.to_string()));
        }

        let label_id = self.edge_label_counter;
        self.edge_label_counter += 1;

        let schema = EdgeSchema {
            label_id,
            label_name: name.to_string(),
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
        };

        let table = EdgeTable::new(schema);
        let key = (src_label, dst_label, label_id);
        self.edge_tables.insert(key, table);
        self.edge_label_names.insert(name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let label_id = self.vertex_label_names
            .remove(name)
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;

        self.vertex_tables.remove(&label_id);

        let keys_to_remove: Vec<_> = self.edge_tables
            .keys()
            .filter(|(src, dst, _)| *src == label_id || *dst == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let label_id = self.edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;

        let keys_to_remove: Vec<_> = self.edge_tables
            .keys()
            .filter(|(_, _, e)| *e == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
    }

    pub fn insert_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        table.insert(external_id, properties, ts)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        self.vertex_tables.get(&label)?.get(external_id, ts)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        self.vertex_tables.get(&label)?.get_by_internal_id(internal_id, ts)
    }

    pub fn delete_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        table.delete(external_id, ts)
    }

    pub fn update_vertex_property(
        &mut self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        let internal_id = table.get_internal_id(external_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        table.update_property(internal_id, property_name, value, ts)
    }

    pub fn insert_edge(
        &mut self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<EdgeId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let src_table = self.vertex_tables.get(&src_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("source vertex label {}", src_label)))?;
        let dst_table = self.vertex_tables.get(&dst_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("destination vertex label {}", dst_label)))?;

        let src_internal = src_table.get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.insert_edge(src_internal as VertexId, dst_internal as VertexId, properties, ts)
    }

    pub fn get_edge(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let src_table = self.vertex_tables.get(&src_label)?;
        let dst_table = self.vertex_tables.get(&dst_label)?;

        let src_internal = src_table.get_internal_id(src_id, ts)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        edge_table.get_edge(src_internal as VertexId, dst_internal as VertexId, ts)
    }

    pub fn delete_edge(
        &mut self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let src_table = self.vertex_tables.get(&src_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("source vertex label {}", src_label)))?;
        let dst_table = self.vertex_tables.get(&dst_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("destination vertex label {}", dst_label)))?;

        let src_internal = src_table.get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.delete_edge(src_internal as VertexId, dst_internal as VertexId, ts)
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        src_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.is_open {
            return None;
        }

        let src_table = self.vertex_tables.get(&src_label)?;
        let src_internal = src_table.get_internal_id(src_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.out_edges(src_internal as VertexId, ts))
    }

    pub fn in_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.is_open {
            return None;
        }

        let dst_table = self.vertex_tables.get(&dst_label)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.in_edges(dst_internal as VertexId, ts))
    }

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<super::vertex::VertexIterator> {
        if !self.is_open {
            return None;
        }
        self.vertex_tables.get(&label).map(|t| t.scan(ts))
    }

    pub fn vertex_count(&self, label: LabelId, ts: Timestamp) -> usize {
        self.vertex_tables
            .get(&label)
            .map(|t| t.vertex_count(ts))
            .unwrap_or(0)
    }

    pub fn edge_count(&self, edge_label: LabelId) -> u64 {
        self.edge_tables
            .values()
            .filter_map(|t| {
                if t.label() == edge_label {
                    Some(t.edge_count())
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.vertex_label_names.get(name).copied()
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.edge_label_names.get(name).copied()
    }

    pub fn vertex_label_names(&self) -> Vec<&str> {
        self.vertex_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn edge_label_names(&self) -> Vec<&str> {
        self.edge_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_vertex_table(&self, label: LabelId) -> Option<&VertexTable> {
        self.vertex_tables.get(&label)
    }

    pub fn get_edge_table(&self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> Option<&EdgeTable> {
        self.edge_tables.get(&(src_label, dst_label, edge_label))
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn compact(&mut self) {
        for table in self.vertex_tables.values_mut() {
            table.compact();
        }
        for table in self.edge_tables.values_mut() {
            table.compact();
        }
    }

    pub fn clear(&mut self) {
        self.vertex_tables.clear();
        self.edge_tables.clear();
        self.vertex_label_names.clear();
        self.edge_label_names.clear();
        self.vertex_label_counter = 0;
        self.edge_label_counter = 0;
    }
}

impl Default for PropertyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vertex_type() {
        let mut graph = PropertyGraph::new();

        let label = graph.create_vertex_type(
            "person",
            vec![
                VertexPropertyDef::new("name".to_string(), DataType::String),
                VertexPropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            "name",
        ).unwrap();

        assert_eq!(label, 0);
        assert_eq!(graph.get_vertex_label_id("person"), Some(0));
    }

    #[test]
    fn test_insert_and_get_vertex() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(
            0,
            "v1",
            &[("name".to_string(), Value::String("Alice".to_string()))],
            100,
        ).unwrap();

        let vertex = graph.get_vertex(0, "v1", 100).unwrap();
        assert_eq!(vertex.properties.len(), 1);
    }

    #[test]
    fn test_create_and_insert_edge() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(0, "v1", &[("name".to_string(), Value::String("Alice".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v2", &[("name".to_string(), Value::String("Bob".to_string()))], 100).unwrap();

        graph.create_edge_type(
            "knows",
            0,
            0,
            vec![EdgePropertyDef::new("since".to_string(), DataType::Int)],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        ).unwrap();

        let edge_id = graph.insert_edge(
            0,
            0,
            "v1",
            0,
            "v2",
            &[("since".to_string(), Value::Int(2020))],
            100,
        ).unwrap();

        let edge = graph.get_edge(0, 0, "v1", 0, "v2", 100).unwrap();
        assert_eq!(edge.edge_id, edge_id);
    }

    #[test]
    fn test_out_in_edges() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(0, "v1", &[("name".to_string(), Value::String("Alice".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v2", &[("name".to_string(), Value::String("Bob".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v3", &[("name".to_string(), Value::String("Charlie".to_string()))], 100).unwrap();

        graph.create_edge_type(
            "knows",
            0,
            0,
            vec![],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        ).unwrap();

        graph.insert_edge(0, 0, "v1", 0, "v2", &[], 100).unwrap();
        graph.insert_edge(0, 0, "v1", 0, "v3", &[], 100).unwrap();

        let out_edges = graph.out_edges(0, 0, 0, "v1", 100).unwrap();
        assert_eq!(out_edges.len(), 2);

        let in_edges = graph.in_edges(0, 0, 0, "v2", 100).unwrap();
        assert_eq!(in_edges.len(), 1);
    }
}
