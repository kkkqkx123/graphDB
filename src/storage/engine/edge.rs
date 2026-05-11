use std::collections::HashMap;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::{
    EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable,
    PropertyDef as EdgePropertyDef, VertexId,
};
use crate::storage::vertex::{LabelId, Timestamp};

use super::schema::SchemaOps;

/// Parameters for creating an edge type
pub struct CreateEdgeTypeParams<'a> {
    pub name: &'a str,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<EdgePropertyDef>,
    pub oe_strategy: EdgeStrategy,
    pub ie_strategy: EdgeStrategy,
}

/// Parameters for edge operations that need vertex/edge labels and IDs
pub struct EdgeOperationParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
}

/// Parameters for edge traversal operations
pub struct EdgeTraversalParams {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub dst_label: LabelId,
}

pub struct EdgeOps {
    pub edge_tables: HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
    pub edge_label_names: HashMap<String, LabelId>,
    pub edge_label_counter: LabelId,
}

impl Default for EdgeOps {
    fn default() -> Self {
        Self::new()
    }
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
        params: CreateEdgeTypeParams,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<LabelId> {
        if !vertex_tables.contains_key(&params.src_label) {
            return Err(StorageError::label_not_found(format!(
                "source label {}",
                params.src_label
            )));
        }
        if !vertex_tables.contains_key(&params.dst_label) {
            return Err(StorageError::label_not_found(format!(
                "destination label {}",
                params.dst_label
            )));
        }

        if self.edge_label_names.contains_key(params.name) {
            return Err(StorageError::label_already_exists(params.name.to_string()));
        }

        let label_id = self.edge_label_counter;
        self.edge_label_counter += 1;

        let schema = EdgeSchema {
            label_id,
            label_name: params.name.to_string(),
            src_label: params.src_label,
            dst_label: params.dst_label,
            properties: params.properties,
            oe_strategy: params.oe_strategy,
            ie_strategy: params.ie_strategy,
        };

        let table = EdgeTable::new(schema);
        let key = (params.src_label, params.dst_label, label_id);
        self.edge_tables.insert(key, table);
        self.edge_label_names.insert(params.name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn create_edge_type_with_id(
        &mut self,
        params: CreateEdgeTypeParams,
        label_id: LabelId,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<LabelId> {
        if !vertex_tables.contains_key(&params.src_label) {
            return Err(StorageError::label_not_found(format!(
                "source label {}",
                params.src_label
            )));
        }
        if !vertex_tables.contains_key(&params.dst_label) {
            return Err(StorageError::label_not_found(format!(
                "destination label {}",
                params.dst_label
            )));
        }

        if self.edge_label_names.contains_key(params.name) {
            return Err(StorageError::label_already_exists(params.name.to_string()));
        }

        if label_id >= self.edge_label_counter {
            self.edge_label_counter = label_id + 1;
        }

        let schema = EdgeSchema {
            label_id,
            label_name: params.name.to_string(),
            src_label: params.src_label,
            dst_label: params.dst_label,
            properties: params.properties,
            oe_strategy: params.oe_strategy,
            ie_strategy: params.ie_strategy,
        };

        let table = EdgeTable::new(schema);
        let key = (params.src_label, params.dst_label, label_id);
        self.edge_tables.insert(key, table);
        self.edge_label_names.insert(params.name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        let label_id = self
            .edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?;

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
        params: EdgeOperationParams,
        properties: &[(String, Value)],
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<EdgeId> {
        let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
            StorageError::label_not_found(format!("source vertex label {}", params.src_label))
        })?;
        let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
            StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(params.src_id, ts)
            .ok_or(StorageError::vertex_not_found())?;
        let dst_internal = dst_table
            .get_internal_id(params.dst_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::label_not_found(format!("edge label {}", params.edge_label)))?;

        edge_table.insert_edge(
            src_internal as VertexId,
            dst_internal as VertexId,
            properties,
            ts,
        )
    }

    pub fn get_edge(
        &self,
        params: EdgeOperationParams,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<EdgeRecord> {
        let src_table = vertex_tables.get(&params.src_label)?;
        let dst_table = vertex_tables.get(&params.dst_label)?;

        let src_internal = src_table.get_internal_id(params.src_id, ts)?;
        let dst_internal = dst_table.get_internal_id(params.dst_id, ts)?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        edge_table.get_edge(src_internal as VertexId, dst_internal as VertexId, ts)
    }

    pub fn delete_edge(
        &mut self,
        params: EdgeOperationParams,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<bool> {
        let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
            StorageError::label_not_found(format!("source vertex label {}", params.src_label))
        })?;
        let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
            StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(params.src_id, ts)
            .ok_or(StorageError::vertex_not_found())?;
        let dst_internal = dst_table
            .get_internal_id(params.dst_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::label_not_found(format!("edge label {}", params.edge_label)))?;

        edge_table.delete_edge(src_internal as VertexId, dst_internal as VertexId, ts)
    }

    pub fn update_edge_property(
        &mut self,
        params: EdgeOperationParams,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> StorageResult<bool> {
        let src_table = vertex_tables.get(&params.src_label).ok_or_else(|| {
            StorageError::label_not_found(format!("source vertex label {}", params.src_label))
        })?;
        let dst_table = vertex_tables.get(&params.dst_label).ok_or_else(|| {
            StorageError::label_not_found(format!("destination vertex label {}", params.dst_label))
        })?;

        let src_internal = src_table
            .get_internal_id(params.src_id, ts)
            .ok_or(StorageError::vertex_not_found())?;
        let dst_internal = dst_table
            .get_internal_id(params.dst_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::label_not_found(format!("edge label {}", params.edge_label)))?;

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
        params: EdgeTraversalParams,
        src_id: &str,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<Vec<EdgeRecord>> {
        let src_table = vertex_tables.get(&params.src_label)?;
        let src_internal = src_table.get_internal_id(src_id, ts)?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.out_edges(src_internal as VertexId, ts))
    }

    pub fn in_edges(
        &self,
        params: EdgeTraversalParams,
        dst_id: &str,
        ts: Timestamp,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> Option<Vec<EdgeRecord>> {
        let dst_table = vertex_tables.get(&params.dst_label)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (params.src_label, params.dst_label, params.edge_label);
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

    pub fn revert_delete_edge_properties(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        schema_ops: &SchemaOps,
        prop_names: &[String],
    ) -> StorageResult<()> {
        let src_label_id = schema_ops
            .vertex_label_names
            .get(src_label)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(src_label.to_string()))?;
        let dst_label_id = schema_ops
            .vertex_label_names
            .get(dst_label)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(dst_label.to_string()))?;
        let edge_label_id = self
            .edge_label_names
            .get(edge_label)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(edge_label.to_string()))?;

        let key = (src_label_id, dst_label_id, edge_label_id);
        let table = self
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::label_not_found(edge_label.to_string()))?;

        for prop_name in prop_names {
            if table.schema().properties.iter().any(|p| p.name == *prop_name) {
                continue;
            }

            table.add_property(
                prop_name.clone(),
                crate::core::DataType::String,
                false,
            )?;
        }

        Ok(())
    }
}
