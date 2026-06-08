//! Transaction Operations
//!
//! Core transaction operations for vertex and edge manipulation.
//! These operations are used by the transaction system for insert, delete, and update operations.

use std::collections::HashMap;

use crate::core::types::{LabelId, Timestamp, VertexId};
use crate::core::Value;
use crate::storage::edge::UpdateEdgePropertyByOffsetParams;
use crate::storage::types::EdgeOffset;
use crate::transaction::codec::{bytes_to_value, property_value_to_value};
use crate::transaction::insert_transaction::{InsertTransactionError, InsertTransactionResult};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult};

use crate::storage::edge::EdgeTable;
use crate::storage::engine::data_store::EdgeTableKey;
use crate::storage::vertex::VertexTable;

/// Parameters for add_edge operation
pub struct AddEdgeParams {
    pub src_label: LabelId,
    pub src_vid: u32,
    pub dst_label: LabelId,
    pub dst_vid: u32,
    pub edge_label: LabelId,
    pub rank: i64,
}

/// Parameters for delete_edge operation
pub struct DeleteEdgeParams {
    pub src_label: LabelId,
    pub src_vid: u32,
    pub dst_label: LabelId,
    pub dst_vid: u32,
    pub edge_label: LabelId,
    pub rank: i64,
}

/// Parameters for update_edge_property_undo operation
pub struct UpdateEdgePropertyUndoParams {
    pub src_label: LabelId,
    pub src_vid: u32,
    pub dst_label: LabelId,
    pub dst_vid: u32,
    pub edge_label: LabelId,
    pub rank: i64,
}

/// Parameters for revert_delete_edge operation
pub struct RevertDeleteEdgeParams {
    pub src_label: LabelId,
    pub src_vid: u32,
    pub dst_label: LabelId,
    pub dst_vid: u32,
    pub edge_label: LabelId,
    pub rank: i64,
}

/// Parameters for delete_edge_type operation
pub struct DeleteEdgeTypeParams {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
}

/// Parameters identifying an edge type by label names
pub struct EdgeTypeLabelParams<'a> {
    pub src_label: &'a str,
    pub dst_label: &'a str,
    pub edge_label: &'a str,
}

pub struct TransactionOps;

impl TransactionOps {
    /// Resolve an external VertexId to an internal row ID.
    pub fn resolve_vertex_id(table: &VertexTable, vid: VertexId, ts: Timestamp) -> Option<u32> {
        if let Some(int_id) = vid.as_int64() {
            table.get_internal_id_by_i64(int_id, ts)
        } else if let Some(str_id) = vid.as_str() {
            table.get_internal_id(str_id, ts)
        } else {
            None
        }
    }
    pub fn add_vertex(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<VertexId> {
        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| bytes_to_value(v).map(|val| (k.clone(), val)))
            .collect();

        let table = vertex_tables
            .get_mut(&label)
            .ok_or(InsertTransactionError::LabelNotFound(label))?;

        let internal_id = if let Some(int_id) = vid.as_int64() {
            table
                .insert_by_i64(int_id, &props, ts)
                .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?
        } else if let Some(str_id) = vid.as_str() {
            table
                .insert(str_id, &props, ts)
                .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?
        } else {
            return Err(InsertTransactionError::SerializationError(
                "Invalid VertexId: neither int64 nor string".to_string(),
            ));
        };

        Ok(VertexId::from_int64(internal_id as i64))
    }

    pub fn add_edge(
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
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

        let src_external = src_table.get_external_id(params.src_vid, ts).ok_or(
            InsertTransactionError::VertexNotFound(VertexId::from_int64(params.src_vid as i64)),
        )?;
        let dst_external = dst_table.get_external_id(params.dst_vid, ts).ok_or(
            InsertTransactionError::VertexNotFound(VertexId::from_int64(params.dst_vid as i64)),
        )?;

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

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let edge_table = edge_tables
            .get_mut(&key)
            .ok_or(InsertTransactionError::LabelNotFound(params.edge_label))?;

        let edge_offset = edge_table
            .insert_edge(params.src_vid, params.dst_vid, params.rank, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(edge_offset)
    }

    pub fn delete_vertex_type(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
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
            if key.src_label == label || key.dst_label == label {
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
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        params: DeleteEdgeTypeParams,
    ) -> UndoLogResult<()> {
        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
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

    pub fn delete_vertex_by_external_vid(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let table = vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(label))?;

        let internal_id =
            Self::resolve_vertex_id(table, vid, ts).ok_or(UndoLogError::LabelNotFound(0))?;

        table
            .delete_by_internal_id(internal_id, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
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
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        params: DeleteEdgeParams,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        if let Some(table) = edge_tables.get_mut(&key) {
            table
                .delete_edge_by_offset(
                    params.src_vid,
                    params.dst_vid,
                    params.rank,
                    oe_offset,
                    ie_offset,
                    ts,
                )
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn revert_delete_edge(
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        params: RevertDeleteEdgeParams,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        if let Some(table) = edge_tables.get_mut(&key) {
            table
                .revert_delete_edge_by_offset(
                    params.src_vid,
                    params.dst_vid,
                    params.rank,
                    crate::storage::types::EdgeOffset(oe_offset),
                    crate::storage::types::EdgeOffset(ie_offset),
                    ts,
                )
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn update_vertex_property_by_vid(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        label: LabelId,
        vid: VertexId,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let table = vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(label))?;

        let internal_id = if let Some(int_id) = vid.as_int64() {
            table.get_internal_id_by_i64(int_id, ts)
        } else if let Some(str_id) = vid.as_str() {
            table.get_internal_id(str_id, ts)
        } else {
            None
        }
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
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        params: crate::storage::engine::params::EdgeOperationParams,
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

        let src_internal = Self::resolve_vertex_id(src_table, params.src_id, ts)
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_internal = Self::resolve_vertex_id(dst_table, params.dst_id, ts)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(params.edge_label))?;

        table
            .update_edge_property(
                src_internal,
                dst_internal,
                params.rank,
                prop_name,
                value,
                ts,
            )
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_edge_property_undo(
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        params: UpdateEdgePropertyUndoParams,
        _oe_offset: i32,
        _ie_offset: i32,
        prop_id: u16,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let table = edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_edge_property_by_offset(UpdateEdgePropertyByOffsetParams {
                src: params.src_vid,
                dst: params.dst_vid,
                rank: params.rank,
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
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
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
        let key = EdgeTableKey::new(src_label_id, dst_label_id, label);
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
                if let Some(prop) = new_schema
                    .properties
                    .iter_mut()
                    .find(|p| p.name == *current)
                {
                    prop.name = original.clone();
                }
            }
            table.set_schema(new_schema);
        }

        Ok(())
    }

    pub fn revert_rename_edge_properties(
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        edge_labels: &EdgeTypeLabelParams,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        let src_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_labels.src_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_labels.dst_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_label_names
            .get(edge_labels.edge_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = EdgeTableKey::new(src_label_id, dst_label_id, edge_label_id);
        if let Some(table) = edge_tables.get_mut(&key) {
            let mut new_schema = table.schema().clone();
            for (current, original) in current_names.iter().zip(original_names.iter()) {
                if let Some(prop) = new_schema
                    .properties
                    .iter_mut()
                    .find(|p| p.name == *current)
                {
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
        edge_tables: &mut HashMap<EdgeTableKey, EdgeTable>,
        edge_label_names: &mut HashMap<String, LabelId>,
        vertex_tables: &HashMap<LabelId, VertexTable>,
        prop_names: &[String],
        edge_labels: &EdgeTypeLabelParams,
    ) -> UndoLogResult<()> {
        let src_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_labels.src_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = vertex_tables
            .values()
            .find(|t| t.label_name() == edge_labels.dst_label)
            .map(|t| t.label())
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_label_names
            .get(edge_labels.edge_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let key = EdgeTableKey::new(src_label_id, dst_label_id, edge_label_id);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::types::{LabelId, VertexId};
    use crate::core::Value;
    use crate::storage::edge::{EdgeSchema, EdgeStrategy, EdgeTable};
    use crate::storage::engine::data_store::EdgeTableKey;
    use crate::storage::types::StoragePropertyDef;
    use crate::storage::vertex::{VertexSchema, VertexTable};

    use super::{
        AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, RevertDeleteEdgeParams,
        TransactionOps,
    };

    fn create_vertex_table(label: LabelId, name: &str) -> VertexTable {
        let schema = VertexSchema {
            label_id: label,
            label_name: name.to_string(),
            properties: vec![
                StoragePropertyDef::new("name".to_string(), crate::core::DataType::String),
                StoragePropertyDef::new("age".to_string(), crate::core::DataType::BigInt),
            ],
            primary_key_index: 0,
        };
        VertexTable::new(label, name.to_string(), schema)
    }

    fn create_edge_table(
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        name: &str,
    ) -> EdgeTable {
        let schema = EdgeSchema {
            label_id: edge_label,
            label_name: name.to_string(),
            src_label,
            dst_label,
            properties: vec![StoragePropertyDef::new(
                "since".to_string(),
                crate::core::DataType::Int,
            )],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        };
        EdgeTable::new(schema).expect("Failed to create EdgeTable")
    }

    #[test]
    fn test_add_vertex_int_id() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        let vid = VertexId::from_int64(100);
        let properties = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::BigInt(30)),
        ];
        let props_bytes: Vec<(String, Vec<u8>)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), crate::transaction::codec::value_to_bytes(v)))
            .collect();

        let result = TransactionOps::add_vertex(&mut vertex_tables, 0, vid, &props_bytes, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VertexId::from_int64(0));

        let table = vertex_tables.get(&0).unwrap();
        let internal = table.get_internal_id_by_i64(100, 1);
        assert!(internal.is_some());
    }

    #[test]
    fn test_add_vertex_string_id() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        // Use a string ID that is NOT 8 bytes (avoids as_int64() collision)
        let vid = VertexId::from_string("user-alice");
        let properties = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::BigInt(30)),
        ];
        let props_bytes: Vec<(String, Vec<u8>)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), crate::transaction::codec::value_to_bytes(v)))
            .collect();

        let result = TransactionOps::add_vertex(&mut vertex_tables, 0, vid, &props_bytes, 1);
        assert!(result.is_ok());

        let table = vertex_tables.get(&0).unwrap();
        let internal = table.get_internal_id("user-alice", 1);
        assert!(internal.is_some());
    }

    #[test]
    fn test_add_vertex_label_not_found() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();

        let vid = VertexId::from_int64(1);
        let result = TransactionOps::add_vertex(&mut vertex_tables, 99, vid, &[], 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));
        vertex_tables.insert(1, create_vertex_table(1, "Person"));

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        edge_tables.insert(
            EdgeTableKey::new(0, 1, 0),
            create_edge_table(0, 0, 1, "KNOWS"),
        );

        let vid1 = VertexId::from_int64(100);
        let vid2 = VertexId::from_int64(101);

        TransactionOps::add_vertex(&mut vertex_tables, 0, vid1, &[], 1).unwrap();
        TransactionOps::add_vertex(&mut vertex_tables, 1, vid2, &[], 1).unwrap();

        let src_internal = vertex_tables
            .get(&0)
            .unwrap()
            .get_internal_id_by_i64(100, 1)
            .unwrap();
        let dst_internal = vertex_tables
            .get(&1)
            .unwrap()
            .get_internal_id_by_i64(101, 1)
            .unwrap();

        let params = AddEdgeParams {
            src_label: 0,
            src_vid: src_internal,
            dst_label: 1,
            dst_vid: dst_internal,
            edge_label: 0,
            rank: 0,
        };

        let result = TransactionOps::add_edge(&mut edge_tables, &vertex_tables, params, &[], 1);
        assert!(result.is_ok(), "add_edge failed: {:?}", result.err());
    }

    #[test]
    fn test_add_edge_missing_src_label() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        edge_tables.insert(
            EdgeTableKey::new(0, 1, 0),
            create_edge_table(0, 0, 1, "KNOWS"),
        );

        let params = AddEdgeParams {
            src_label: 0,
            src_vid: 0,
            dst_label: 1,
            dst_vid: 0,
            edge_label: 0,
            rank: 0,
        };

        let result = TransactionOps::add_edge(&mut edge_tables, &vertex_tables, params, &[], 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_vertex_id() {
        let mut table = create_vertex_table(0, "Person");
        table
            .insert_by_i64(
                100,
                &[("name".to_string(), Value::String("Alice".to_string()))],
                1,
            )
            .unwrap();
        table
            .insert(
                "user-bob-ext",
                &[("name".to_string(), Value::String("Bob".to_string()))],
                1,
            )
            .unwrap();

        let resolved_int = TransactionOps::resolve_vertex_id(&table, VertexId::from_int64(100), 1);
        assert_eq!(resolved_int, Some(0));

        let resolved_str =
            TransactionOps::resolve_vertex_id(&table, VertexId::from_string("user-bob-ext"), 1);
        assert_eq!(resolved_str, Some(1));

        let not_found = TransactionOps::resolve_vertex_id(&table, VertexId::from_int64(999), 1);
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_delete_vertex() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        TransactionOps::add_vertex(&mut vertex_tables, 0, VertexId::from_int64(1), &[], 1).unwrap();

        let result =
            TransactionOps::delete_vertex(&mut vertex_tables, 0, VertexId::from_int64(0), 2);
        assert!(result.is_ok());

        // After deletion, the vertex should not be visible at timestamp 2
        let table = vertex_tables.get(&0).unwrap();
        let v = table.get_by_internal_id(0, 2);
        assert!(v.is_none());
    }

    #[test]
    fn test_update_vertex_property_by_vid() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        // Insert with external ID = 1 (int64 vid), gets internal ID 0
        let vid = VertexId::from_int64(1);
        TransactionOps::add_vertex(
            &mut vertex_tables,
            0,
            vid,
            &[(
                "name".to_string(),
                crate::transaction::codec::value_to_bytes(&Value::String("Alice".to_string())),
            )],
            1,
        )
        .unwrap();

        // update_vertex_property_by_vid looks up external ID in the table
        let result = TransactionOps::update_vertex_property_by_vid(
            &mut vertex_tables,
            0,
            VertexId::from_int64(1),
            "name",
            &Value::String("AliceUpdated".to_string()),
            2,
        );
        assert!(result.is_ok());

        let table = vertex_tables.get(&0).unwrap();
        let record = table.get_by_internal_id(0, 2).unwrap();
        let name_val = record
            .properties
            .iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v);
        assert_eq!(name_val, Some(&Value::String("AliceUpdated".to_string())));
    }

    #[test]
    fn test_delete_vertex_type_cascades_to_edge_types() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));
        vertex_tables.insert(1, create_vertex_table(1, "Employee"));

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        edge_tables.insert(
            EdgeTableKey::new(0, 0, 0),
            create_edge_table(0, 0, 0, "KNOWS"),
        );
        edge_tables.insert(
            EdgeTableKey::new(0, 1, 1),
            create_edge_table(1, 0, 1, "WORKS_AT"),
        );

        let mut vertex_label_names: HashMap<String, LabelId> = HashMap::new();
        vertex_label_names.insert("Person".to_string(), 0);
        vertex_label_names.insert("Employee".to_string(), 1);

        let mut edge_label_names: HashMap<String, LabelId> = HashMap::new();
        edge_label_names.insert("KNOWS".to_string(), 0);
        edge_label_names.insert("WORKS_AT".to_string(), 1);

        TransactionOps::delete_vertex_type(
            &mut vertex_tables,
            &mut edge_tables,
            &mut vertex_label_names,
            &mut edge_label_names,
            1,
        )
        .unwrap();

        assert!(vertex_tables.get(&1).is_none());
        assert!(vertex_label_names.get("Employee").is_none());
        // WORKS_AT edge type should be removed because its src or dst label is gone
        assert!(edge_label_names.get("WORKS_AT").is_none());
        // KNOWS (0,0,0) should still exist since neither src=0 nor dst=0 was removed
        assert!(edge_tables.contains_key(&EdgeTableKey::new(0, 0, 0)));
    }

    #[test]
    fn test_delete_edge_type() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));
        vertex_tables.insert(1, create_vertex_table(1, "Person"));

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        edge_tables.insert(
            EdgeTableKey::new(0, 1, 0),
            create_edge_table(0, 0, 1, "KNOWS"),
        );

        let mut edge_label_names: HashMap<String, LabelId> = HashMap::new();
        edge_label_names.insert("KNOWS".to_string(), 0);

        let params = DeleteEdgeTypeParams {
            src_label: 0,
            dst_label: 1,
            edge_label: 0,
        };

        TransactionOps::delete_edge_type(&mut edge_tables, &mut edge_label_names, params).unwrap();

        assert!(edge_label_names.get("KNOWS").is_none());
        assert!(edge_tables.get(&EdgeTableKey::new(0, 1, 0)).is_none());
    }

    #[test]
    fn test_revert_delete_vertex() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        TransactionOps::add_vertex(
            &mut vertex_tables,
            0,
            VertexId::from_int64(1),
            &[(
                "name".to_string(),
                crate::transaction::codec::value_to_bytes(&Value::String("Alice".to_string())),
            )],
            1,
        )
        .unwrap();

        TransactionOps::delete_vertex(&mut vertex_tables, 0, VertexId::from_int64(0), 2).unwrap();

        let result =
            TransactionOps::revert_delete_vertex(&mut vertex_tables, 0, VertexId::from_int64(0), 3);
        assert!(result.is_ok());

        let table = vertex_tables.get(&0).unwrap();
        let record = table.get_by_internal_id(0, 3);
        assert!(record.is_some());
    }

    #[test]
    fn test_revert_delete_edge() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        vertex_tables.insert(0, create_vertex_table(0, "Person"));

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        edge_tables.insert(
            EdgeTableKey::new(0, 0, 0),
            create_edge_table(0, 0, 0, "KNOWS"),
        );

        TransactionOps::add_vertex(&mut vertex_tables, 0, VertexId::from_int64(1), &[], 1).unwrap();
        TransactionOps::add_vertex(&mut vertex_tables, 0, VertexId::from_int64(2), &[], 1).unwrap();

        let add_params = AddEdgeParams {
            src_label: 0,
            src_vid: 0,
            dst_label: 0,
            dst_vid: 1,
            edge_label: 0,
            rank: 0,
        };
        let offset =
            TransactionOps::add_edge(&mut edge_tables, &vertex_tables, add_params, &[], 1).unwrap();

        let del_params = DeleteEdgeParams {
            src_label: 0,
            src_vid: 0,
            dst_label: 0,
            dst_vid: 1,
            edge_label: 0,
            rank: 0,
        };
        TransactionOps::delete_edge(&mut edge_tables, del_params, offset.0, offset.0, 2).unwrap();

        let revert_params = RevertDeleteEdgeParams {
            src_label: 0,
            dst_label: 0,
            edge_label: 0,
            src_vid: 0,
            dst_vid: 1,
            rank: 0,
        };
        let result = TransactionOps::revert_delete_edge(
            &mut edge_tables,
            revert_params,
            offset.0,
            offset.0,
            3,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_vertex_type_undo() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        let mut vertex_label_names: HashMap<String, LabelId> = HashMap::new();
        let mut vertex_label_counter: LabelId = 0;

        TransactionOps::create_vertex_type_undo(
            &mut vertex_tables,
            &mut vertex_label_names,
            &mut vertex_label_counter,
            "Person",
        )
        .unwrap();

        assert!(vertex_tables.contains_key(&0));
        assert_eq!(vertex_label_names.get("Person"), Some(&0));
        assert!(vertex_label_counter >= 1);

        // Create another
        TransactionOps::create_vertex_type_undo(
            &mut vertex_tables,
            &mut vertex_label_names,
            &mut vertex_label_counter,
            "Employee",
        )
        .unwrap();

        assert!(vertex_tables.contains_key(&1));
        assert_eq!(vertex_label_names.get("Employee"), Some(&1));
    }

    #[test]
    fn test_create_edge_type_undo() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        let mut vertex_label_names: HashMap<String, LabelId> = HashMap::new();
        let mut vertex_label_counter: LabelId = 0;

        TransactionOps::create_vertex_type_undo(
            &mut vertex_tables,
            &mut vertex_label_names,
            &mut vertex_label_counter,
            "Person",
        )
        .unwrap();

        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        let mut edge_label_names: HashMap<String, LabelId> = HashMap::new();
        let mut edge_label_counter: LabelId = 0;

        let result = TransactionOps::create_edge_type_undo(
            &mut edge_tables,
            &mut edge_label_names,
            &mut edge_label_counter,
            &vertex_tables,
            "KNOWS",
            "Person",
            "Person",
        );
        assert!(result.is_ok());

        let key = EdgeTableKey::new(0, 0, 0);
        assert!(edge_tables.contains_key(&key));
        assert_eq!(edge_label_names.get("KNOWS"), Some(&0));
    }

    #[test]
    fn test_create_edge_type_undo_missing_vertex_label() {
        let vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        let mut edge_tables: HashMap<EdgeTableKey, EdgeTable> = HashMap::new();
        let mut edge_label_names: HashMap<String, LabelId> = HashMap::new();
        let mut edge_label_counter: LabelId = 0;

        let result = TransactionOps::create_edge_type_undo(
            &mut edge_tables,
            &mut edge_label_names,
            &mut edge_label_counter,
            &vertex_tables,
            "KNOWS",
            "Person",
            "Person",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_revert_rename_vertex_properties() {
        let mut vertex_tables: HashMap<LabelId, VertexTable> = HashMap::new();
        let mut vertex_label_names: HashMap<String, LabelId> = HashMap::new();
        let mut vertex_label_counter: LabelId = 0;

        // First create a vertex type with property "name"
        TransactionOps::create_vertex_type_undo(
            &mut vertex_tables,
            &mut vertex_label_names,
            &mut vertex_label_counter,
            "Person",
        )
        .unwrap();

        // Add the "name" property to the schema before renaming
        {
            let table = vertex_tables.get_mut(&0).unwrap();
            table
                .add_property(crate::storage::types::StoragePropertyDef::new(
                    "name".to_string(),
                    crate::core::DataType::String,
                ))
                .unwrap();
        }

        // Rename "name" -> "full_name" in schema
        {
            let table = vertex_tables.get_mut(&0).unwrap();
            table.rename_property("name", "full_name").unwrap();
        }

        let result = TransactionOps::revert_rename_vertex_properties(
            &mut vertex_tables,
            &mut vertex_label_names,
            "Person",
            &["full_name".to_string()],
            &["name".to_string()],
        );
        assert!(result.is_ok());

        let table = vertex_tables.get(&0).unwrap();
        let schema = table.schema();
        assert!(schema.properties.iter().any(|p| p.name == "name"));
    }
}
