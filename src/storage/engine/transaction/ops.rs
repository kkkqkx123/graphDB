//! Transaction Operations
//!
//! Core transaction operations for vertex and edge manipulation.
//! These operations are used by the transaction system for insert, delete, and update operations.

use std::collections::HashMap;

use crate::core::types::{LabelId, Timestamp, VertexId};
use crate::core::Value;
use crate::storage::edge::UpdateEdgePropertyByOffsetParams;
use crate::storage::storage_types::EdgeOffset;
use crate::transaction::insert_transaction::{
    InsertTransactionError, InsertTransactionResult,
};
use crate::transaction::codec::{bytes_to_value, property_value_to_value};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult};

use crate::storage::edge::EdgeTable;
use crate::storage::vertex::VertexTable;

/// Parameters for add_edge operation
pub struct AddEdgeParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

/// Parameters for delete_edge operation
pub struct DeleteEdgeParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

/// Parameters for update_edge_property_undo operation
pub struct UpdateEdgePropertyUndoParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

/// Parameters for insert_edge_undo operation
pub struct InsertEdgeUndoParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

/// Parameters for revert_delete_edge operation
pub struct RevertDeleteEdgeParams {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
}

/// Parameters for delete_edge_type operation
pub struct DeleteEdgeTypeParams {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
}

/// Parameters identifying an edge type by label names
pub struct EdgeLabelParams<'a> {
    pub src_label: &'a str,
    pub dst_label: &'a str,
    pub edge_label: &'a str,
}

pub struct TransactionOps;

impl TransactionOps {
    pub fn add_vertex(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<VertexId> {
        let external_id = std::str::from_utf8(oid)
            .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| bytes_to_value(v).map(|val| (k.clone(), val)))
            .collect();

        let table = vertex_tables
            .get_mut(&label)
            .ok_or(InsertTransactionError::LabelNotFound(label))?;

        let internal_id = table
            .insert(external_id, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(VertexId::from_int64(internal_id as i64))
    }

    pub fn add_edge(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        params: AddEdgeParams,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeOffset> {
        let src_table = vertex_tables
            .get(&params.src_label)
            .ok_or(InsertTransactionError::LabelNotFound(params.src_label))?;
        let dst_table = vertex_tables
            .get(&params.dst_label)
            .ok_or(InsertTransactionError::LabelNotFound(params.dst_label))?;

        let src_external = src_table
            .get_external_id(params.src_vid.as_int64().unwrap_or(0) as u32, ts)
            .ok_or(InsertTransactionError::VertexNotFound(params.src_vid))?;
        let dst_external = dst_table
            .get_external_id(params.dst_vid.as_int64().unwrap_or(0) as u32, ts)
            .ok_or(InsertTransactionError::VertexNotFound(params.dst_vid))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| bytes_to_value(v).map(|val| (k.clone(), val)))
            .collect();

        let _src_id_str = match &src_external {
            crate::storage::vertex::IdKey::Text(s) => s.clone(),
            crate::storage::vertex::IdKey::Int(i) => i.to_string(),
        };
        let _dst_id_str = match &dst_external {
            crate::storage::vertex::IdKey::Text(s) => s.clone(),
            crate::storage::vertex::IdKey::Int(i) => i.to_string(),
        };

        let key = (params.src_label, params.dst_label, params.edge_label);
        let edge_table = edge_tables
            .get_mut(&key)
            .ok_or(InsertTransactionError::LabelNotFound(params.edge_label))?;

        let edge_id = edge_table
            .insert_edge(params.src_vid, params.dst_vid, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(edge_id)
    }

    pub fn get_vertex_id(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        label: LabelId,
        oid: &[u8],
        ts: Timestamp,
    ) -> Option<VertexId> {
        let external_id = std::str::from_utf8(oid).ok()?;
        vertex_tables
            .get(&label)?
            .get_internal_id(external_id, ts)
            .map(|id| VertexId::from_int64(id as i64))
    }

    pub fn get_vertex_oid(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<u8>> {
        vertex_tables
            .get(&label)?
            .get_external_id(vid.as_int64().unwrap_or(0) as u32, ts)
            .map(|id_key| match id_key {
                crate::storage::vertex::IdKey::Text(s) => s.into_bytes(),
                crate::storage::vertex::IdKey::Int(i) => i.to_string().into_bytes(),
            })
    }

    pub fn get_vertex_property_types(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        label: LabelId,
    ) -> Vec<String> {
        vertex_tables
            .get(&label)
            .map(|t| {
                t.schema()
                    .properties
                    .iter()
                    .map(|p| p.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_edge_property_types(
        edge_tables: &HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        edge_label: LabelId,
    ) -> Vec<String> {
        edge_tables
            .values()
            .find(|t| t.label() == edge_label)
            .map(|t| {
                t.schema()
                    .properties
                    .iter()
                    .map(|p| p.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn vertex_label_num(vertex_tables: &HashMap<LabelId, VertexTable>) -> usize {
        vertex_tables.len()
    }

    pub fn lid_num(vertex_tables: &HashMap<LabelId, VertexTable>, label: LabelId) -> usize {
        vertex_tables
            .get(&label)
            .map(|t| t.total_count())
            .unwrap_or(0)
    }

    pub fn delete_vertex_type(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        vertex_label_names: &mut HashMap<String, LabelId>,
        edge_label_names: &mut HashMap<String, LabelId>,
        label: LabelId,
    ) -> UndoLogResult<()> {
        let label_name = vertex_tables
            .get(&label)
            .map(|t| t.label_name().to_string());

        if let Some(name) = label_name {
            vertex_label_names.remove(&name);
        }

        vertex_tables.remove(&label);

        let mut removed_edge_keys = Vec::new();
        for (key, table) in &*edge_tables {
            let (src, dst, _edge) = *key;
            if src == label || dst == label {
                let edge_name = table.label_name().to_string();
                edge_label_names.remove(&edge_name);
                removed_edge_keys.push(*key);
            }
        }
        for key in removed_edge_keys {
            edge_tables.remove(&key);
        }

        Ok(())
    }

    pub fn delete_edge_type(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        params: DeleteEdgeTypeParams,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        if let Some(table) = edge_tables.get(&key) {
            let label_name = table.label_name().to_string();
            edge_label_names.remove(&label_name);
        }
        edge_tables.remove(&key);
        Ok(())
    }

    pub fn delete_vertex(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        if let Some(table) = vertex_tables.get_mut(&label) {
            table
                .delete_by_internal_id(vid.as_int64().unwrap_or(0) as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn revert_delete_vertex(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        if let Some(table) = vertex_tables.get_mut(&label) {
            table
                .revert_delete(vid.as_int64().unwrap_or(0) as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn delete_edge(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        params: DeleteEdgeParams,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        if let Some(table) = edge_tables.get_mut(&key) {
            table
                .delete_edge_by_offset(params.src_vid, params.dst_vid, oe_offset, ie_offset, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn revert_delete_edge(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        params: RevertDeleteEdgeParams,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        if let Some(table) = edge_tables.get_mut(&key) {
            table
                .revert_delete_edge_by_offset(
                    params.src_vid,
                    params.dst_vid,
                    crate::storage::storage_types::EdgeOffset(oe_offset),
                    crate::storage::storage_types::EdgeOffset(ie_offset),
                    ts,
                )
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn insert_vertex_undo(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        external_id: &str,
        properties: &[(String, PropertyValue)],
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let props: Vec<(String, Value)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
            .collect();

        let table = vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(label))?;

        table
            .insert(external_id, &props, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn insert_edge_undo(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        params: InsertEdgeUndoParams,
        properties: &[(String, PropertyValue)],
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let props: Vec<(String, Value)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
            .collect();

        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(params.edge_label))?;

        table
            .insert_edge(params.src_vid, params.dst_vid, &props, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_vertex_property(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        external_id: &str,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let table = vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(label))?;

        let internal_id = table
            .get_internal_id(external_id, ts)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        table
            .update_property(internal_id, prop_name, value, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_vertex_property_undo(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        col_id: i32,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let table = vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(label))?;

        let value = property_value_to_value(old_value);
        table
            .update_property_by_id(vid.as_int64().unwrap_or(0) as u32, col_id, &value, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_edge_property(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        params: crate::storage::engine::edge_params::EdgeOperationParams,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let src_table = vertex_tables
            .get(&params.src_label)
            .ok_or(UndoLogError::LabelNotFound(params.src_label))?;
        let dst_table = vertex_tables
            .get(&params.dst_label)
            .ok_or(UndoLogError::LabelNotFound(params.dst_label))?;

        let src_internal = src_table
            .get_internal_id(params.src_id, ts)
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_internal = dst_table
            .get_internal_id(params.dst_id, ts)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = (params.src_label, params.dst_label, params.edge_label);
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(params.edge_label))?;

        table
            .update_edge_property(
                VertexId::from_int64(src_internal as i64),
                VertexId::from_int64(dst_internal as i64),
                prop_name,
                value,
                ts,
            )
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_edge_property_undo(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        params: UpdateEdgePropertyUndoParams,
        oe_offset: i32,
        ie_offset: i32,
        prop_id: u16,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_edge_property_by_offset(UpdateEdgePropertyByOffsetParams {
                src: params.src_vid,
                dst: params.dst_vid,
                oe_offset: crate::storage::storage_types::EdgeOffset(oe_offset),
                ie_offset: crate::storage::storage_types::EdgeOffset(ie_offset),
                prop_id,
                value,
                ts,
            })
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn create_vertex_type_undo(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        vertex_label_names: &mut HashMap<String, LabelId>,
        vertex_label_counter: &mut LabelId,
        name: &str,
    ) -> UndoLogResult<()> {
        let label = *vertex_label_counter;
        vertex_label_names.insert(name.to_string(), label);
        *vertex_label_counter = (*vertex_label_counter).max(label + 1);

        let schema = crate::storage::vertex::VertexSchema {
            label_id: label,
            label_name: name.to_string(),
            properties: Vec::new(),
            primary_key_index: 0,
        };

        let table = VertexTable::new(label, name.to_string(), schema);
        vertex_tables.insert(label, table);

        Ok(())
    }

    pub fn create_edge_type_undo(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        edge_label_counter: &mut LabelId,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        name: &str,
        src_label_name: &str,
        dst_label_name: &str,
    ) -> UndoLogResult<()> {
        let src_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == src_label_name)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == dst_label_name)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let label = *edge_label_counter;
        edge_label_names.insert(name.to_string(), label);
        *edge_label_counter = (*edge_label_counter).max(label + 1);

        let schema = crate::storage::edge::EdgeSchema {
            label_id: label,
            label_name: name.to_string(),
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties: Vec::new(),
            oe_strategy: crate::storage::edge::EdgeStrategy::Multiple,
            ie_strategy: crate::storage::edge::EdgeStrategy::Multiple,
        };

        let table = crate::storage::edge::EdgeTable::new(schema)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        let key = (src_label_id, dst_label_id, label);
        edge_tables.insert(key, table);

        Ok(())
    }

    pub fn revert_rename_vertex_properties(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        vertex_label_names: &mut HashMap<String, LabelId>,
        label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        let label_id = vertex_label_names
            .get(label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        if let Some(table) = vertex_tables.get_mut(&label_id) {
            let mut new_schema = table.schema().clone();
            for (current, original) in current_names.iter().zip(original_names.iter()) {
                if let Some(prop) = new_schema.properties.iter_mut().find(|p| p.name == *current) {
                    prop.name = original.clone();
                }
            }
            table.set_schema(new_schema);
        }

        Ok(())
    }

    pub fn revert_rename_edge_properties(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        edge_params: &EdgeLabelParams,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        let src_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_params.src_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_params.dst_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_label_names
            .get(edge_params.edge_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = (src_label_id, dst_label_id, edge_label_id);
        if let Some(table) = edge_tables.get_mut(&key) {
            let mut new_schema = table.schema().clone();
            for (current, original) in current_names.iter().zip(original_names.iter()) {
                if let Some(prop) = new_schema.properties.iter_mut().find(|p| p.name == *current) {
                    prop.name = original.clone();
                }
            }
            table.set_schema(new_schema);
        }

        Ok(())
    }

    pub fn revert_delete_vertex_properties(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        vertex_label_names: &mut HashMap<String, LabelId>,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        let label_id = vertex_label_names
            .get(label_name)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let table = vertex_tables
            .get_mut(&label_id)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let mut schema = table.schema().clone();
        for prop_name in prop_names {
            schema.properties.retain(|p| p.name != *prop_name);
        }

        Ok(())
    }

    pub fn revert_delete_edge_properties(
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        prop_names: &[String],
        edge_params: &EdgeLabelParams,
    ) -> UndoLogResult<()> {
        let src_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_params.src_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_params.dst_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_label_names
            .get(edge_params.edge_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = (src_label_id, dst_label_id, edge_label_id);
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let mut schema = table.schema().clone();
        for prop_name in prop_names {
            schema.properties.retain(|p| p.name != *prop_name);
        }

        Ok(())
    }
}
