use std::collections::HashMap;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::{
    EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable,
    PropertyDef as EdgePropertyDef, VertexId,
};
use crate::storage::vertex::{LabelId, Timestamp};

pub struct EdgeOps {
    pub edge_tables: HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
    pub edge_label_names: HashMap<String, LabelId>,
    pub edge_label_counter: LabelId,
}

impl EdgeOps {
    pub fn new() -> Self {
        Self {
            edge_tables: HashMap::new(),
            edge_label_names: HashMap::new(),
            edge_label_counter: 0,
        }
    }

    pub fn create_edge_type(
        &mut self,
        name: &str,
        src_label: LabelId,
        dst_label: LabelId,
        properties: Vec<EdgePropertyDef>,
        oe_strategy: EdgeStrategy,
        ie_strategy: EdgeStrategy,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<LabelId> {
        if !vertex_tables.contains_key(&src_label) {
            return Err(StorageError::LabelNotFound(format!(
                "source label {}",
                src_label
            )));
        }
        if !vertex_tables.contains_key(&dst_label) {
            return Err(StorageError::LabelNotFound(format!(
                "destination label {}",
                dst_label
            )));
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

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        let label_id = self
            .edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;

        let keys_to_remove: Vec<_> = self
            .edge_tables
            .keys()
            .filter(|(_, _, e)| *e == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
    }

    pub fn drop_edges_for_vertex_label(&mut self, label_id: LabelId) {
        let keys_to_remove: Vec<_> = self
            .edge_tables
            .keys()
            .filter(|(src, dst, _)| *src == label_id || *dst == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.edge_label_names.get(name).copied()
    }

    pub fn edge_label_names(&self) -> Vec<&str> {
        self.edge_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_edge_table(
        &self,
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
    ) -> Option<&EdgeTable> {
        self.edge_tables.get(&(src_label, dst_label, edge_label))
    }

    pub fn get_edge_table_by_label(&self, edge_label: LabelId) -> Option<&EdgeTable> {
        self.edge_tables.values().find(|t| t.label() == edge_label)
    }

    pub fn edge_tables(&self) -> impl Iterator<Item = (&(LabelId, LabelId, LabelId), &EdgeTable)> {
        self.edge_tables.iter()
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
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<EdgeId> {
        let src_table = vertex_tables.get(&src_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("source vertex label {}", src_label))
        })?;
        let dst_table = vertex_tables.get(&dst_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("destination vertex label {}", dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table
            .get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.insert_edge(
            src_internal as VertexId,
            dst_internal as VertexId,
            properties,
            ts,
        )
    }

    pub fn get_edge(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<EdgeRecord> {
        let src_table = vertex_tables.get(&src_label)?;
        let dst_table = vertex_tables.get(&dst_label)?;

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
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<bool> {
        let src_table = vertex_tables.get(&src_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("source vertex label {}", src_label))
        })?;
        let dst_table = vertex_tables.get(&dst_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("destination vertex label {}", dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table
            .get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.delete_edge(src_internal as VertexId, dst_internal as VertexId, ts)
    }

    pub fn update_edge_property(
        &mut self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<bool> {
        let src_table = vertex_tables.get(&src_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("source vertex label {}", src_label))
        })?;
        let dst_table = vertex_tables.get(&dst_label).ok_or_else(|| {
            StorageError::LabelNotFound(format!("destination vertex label {}", dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table
            .get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.update_edge_property(
            src_internal as VertexId,
            dst_internal as VertexId,
            prop_name,
            value,
            ts,
        )
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        src_id: &str,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<Vec<EdgeRecord>> {
        let src_table = vertex_tables.get(&src_label)?;
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
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<Vec<EdgeRecord>> {
        let dst_table = vertex_tables.get(&dst_label)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.in_edges(dst_internal as VertexId, ts))
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
}
