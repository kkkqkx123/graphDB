use crate::core::Value;
use crate::storage::edge::EdgeId;
use crate::storage::vertex::{LabelId, Timestamp};
use crate::transaction::insert_transaction::{
    InsertTransactionError, InsertTransactionResult,
};
use crate::transaction::codec::{bytes_to_value, property_value_to_value};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult};
use crate::transaction::wal::types::{
    LabelId as TxnLabelId, VertexId as TxnVertexId,
};

use super::schema::SchemaOps;
use super::edge::EdgeOps;

pub struct TransactionOps;

impl TransactionOps {
    pub fn add_vertex(
        schema_ops: &mut SchemaOps,
        label: TxnLabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<TxnVertexId> {
        let external_id = std::str::from_utf8(oid)
            .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| bytes_to_value(v).map(|val| (k.clone(), val)))
            .collect();

        let internal_id = schema_ops
            .insert_vertex(label as LabelId, external_id, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(internal_id as TxnVertexId)
    }

    pub fn add_edge(
        edge_ops: &mut EdgeOps,
        schema_ops: &SchemaOps,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeId> {
        let src_label_id = src_label as LabelId;
        let dst_label_id = dst_label as LabelId;
        let src_table = schema_ops
            .get_vertex_table(src_label_id)
            .ok_or_else(|| InsertTransactionError::LabelNotFound(src_label))?;
        let dst_table = schema_ops
            .get_vertex_table(dst_label_id)
            .ok_or_else(|| InsertTransactionError::LabelNotFound(dst_label))?;

        let src_external = src_table
            .get_external_id(src_vid as u32)
            .ok_or(InsertTransactionError::VertexNotFound(src_vid))?;
        let dst_external = dst_table
            .get_external_id(dst_vid as u32)
            .ok_or(InsertTransactionError::VertexNotFound(dst_vid))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| bytes_to_value(v).map(|val| (k.clone(), val)))
            .collect();

        let edge_id = edge_ops
            .insert_edge(
                edge_label as LabelId,
                src_label as LabelId,
                &src_external,
                dst_label as LabelId,
                &dst_external,
                &props,
                ts,
                schema_ops.vertex_tables(),
            )
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(edge_id)
    }

    pub fn get_vertex_id(
        schema_ops: &SchemaOps,
        label: TxnLabelId,
        oid: &[u8],
        ts: Timestamp,
    ) -> Option<TxnVertexId> {
        let external_id = std::str::from_utf8(oid).ok()?;
        let label_id = label as LabelId;
        schema_ops
            .get_vertex_internal_id(label_id, external_id, ts)
            .map(|id| id as TxnVertexId)
    }

    pub fn get_vertex_oid(
        schema_ops: &SchemaOps,
        label: TxnLabelId,
        vid: TxnVertexId,
        _ts: Timestamp,
    ) -> Option<Vec<u8>> {
        let label_id = label as LabelId;
        schema_ops
            .get_vertex_table(label_id)?
            .get_external_id(vid as u32)
            .map(|s| s.into_bytes())
    }

    pub fn get_vertex_property_types(schema_ops: &SchemaOps, label: TxnLabelId) -> Vec<String> {
        let label_id = label as LabelId;
        schema_ops
            .get_vertex_table(label_id)
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
        edge_ops: &EdgeOps,
        _src_label: TxnLabelId,
        _dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> Vec<String> {
        let edge_label_id = edge_label as LabelId;
        edge_ops
            .get_edge_table_by_label(edge_label_id)
            .map(|t| {
                t.schema()
                    .properties
                    .iter()
                    .map(|p| p.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn vertex_label_num(schema_ops: &SchemaOps) -> usize {
        schema_ops.vertex_tables.len()
    }

    pub fn lid_num(schema_ops: &SchemaOps, label: TxnLabelId) -> usize {
        let label_id = label as LabelId;
        schema_ops
            .get_vertex_table(label_id)
            .map(|t| t.total_count())
            .unwrap_or(0)
    }

    pub fn delete_vertex_type(
        schema_ops: &mut SchemaOps,
        edge_ops: &mut EdgeOps,
        label: TxnLabelId,
    ) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        let label_name = schema_ops
            .get_vertex_table(label_id)
            .map(|t| t.label_name().to_string());

        if let Some(name) = label_name {
            schema_ops.vertex_label_names.remove(&name);
        }

        schema_ops.vertex_tables.remove(&label_id);
        edge_ops.drop_edges_for_vertex_label(label_id);

        Ok(())
    }

    pub fn delete_edge_type(
        edge_ops: &mut EdgeOps,
        src_label: TxnLabelId,
        dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> UndoLogResult<()> {
        let key = (
            src_label as LabelId,
            dst_label as LabelId,
            edge_label as LabelId,
        );
        edge_ops.edge_tables.remove(&key);
        Ok(())
    }

    pub fn delete_vertex(
        schema_ops: &mut SchemaOps,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        if let Some(table) = schema_ops.vertex_tables.get_mut(&label_id) {
            table
                .delete_by_internal_id(vid as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn delete_edge(
        edge_ops: &mut EdgeOps,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            src_label as LabelId,
            dst_label as LabelId,
            edge_label as LabelId,
        );
        if let Some(table) = edge_ops.edge_tables.get_mut(&key) {
            table
                .delete_edge_by_offset(src_vid as u64, dst_vid as u64, oe_offset, ie_offset, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn revert_delete_edge(
        edge_ops: &mut EdgeOps,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            src_label as LabelId,
            dst_label as LabelId,
            edge_label as LabelId,
        );
        if let Some(table) = edge_ops.edge_tables.get_mut(&key) {
            table
                .revert_delete_edge_by_offset(src_vid as u64, dst_vid as u64, oe_offset, ie_offset, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn insert_vertex_undo(
        schema_ops: &mut SchemaOps,
        label: TxnLabelId,
        external_id: &str,
        properties: &[(String, PropertyValue)],
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let props: Vec<(String, Value)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
            .collect();

        schema_ops
            .insert_vertex(label as LabelId, external_id, &props, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn insert_edge_undo(
        edge_ops: &mut EdgeOps,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        properties: &[(String, PropertyValue)],
        ts: Timestamp,
        _vertex_tables: &std::collections::HashMap<LabelId, crate::storage::vertex::VertexTable>,
    ) -> UndoLogResult<()> {
        let props: Vec<(String, Value)> = properties
            .iter()
            .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
            .collect();

        let key = (
            src_label as LabelId,
            dst_label as LabelId,
            edge_label as LabelId,
        );
        let table = edge_ops
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| UndoLogError::LabelNotFound(edge_label as LabelId))?;

        table
            .insert_edge(src_vid as u64, dst_vid as u64, &props, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_vertex_property_undo(
        schema_ops: &mut SchemaOps,
        label: TxnLabelId,
        vid: TxnVertexId,
        col_id: i32,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        let table = schema_ops
            .vertex_tables
            .get_mut(&label_id)
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_property_by_id(vid as u32, col_id, &value, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_edge_property_undo(
        edge_ops: &mut EdgeOps,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        oe_offset: i32,
        ie_offset: i32,
        col_id: i32,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            src_label as LabelId,
            dst_label as LabelId,
            edge_label as LabelId,
        );
        let table = edge_ops
            .edge_tables
            .get_mut(&key)
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_edge_property_by_offset(
                src_vid as u64,
                dst_vid as u64,
                oe_offset,
                ie_offset,
                col_id,
                &value,
                ts,
            )
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn create_vertex_type_undo(
        schema_ops: &mut SchemaOps,
        name: &str,
        label: LabelId,
    ) -> UndoLogResult<()> {
        schema_ops.vertex_label_names.insert(name.to_string(), label);
        schema_ops.vertex_label_counter = schema_ops.vertex_label_counter.max(label + 1);
        Ok(())
    }

    pub fn create_edge_type_undo(
        edge_ops: &mut EdgeOps,
        name: &str,
        label: LabelId,
    ) -> UndoLogResult<()> {
        edge_ops.edge_label_names.insert(name.to_string(), label);
        edge_ops.edge_label_counter = edge_ops.edge_label_counter.max(label + 1);
        Ok(())
    }

    pub fn revert_rename_vertex_properties(
        schema_ops: &mut SchemaOps,
        label: &str,
        _current_names: &[String],
        _original_names: &[String],
    ) -> UndoLogResult<()> {
        let label_id = schema_ops
            .vertex_label_names
            .get(label)
            .copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;

        if let Some(table) = schema_ops.vertex_tables.get_mut(&label_id) {
            let mut new_schema = table.schema().clone();
            for (current, original) in _current_names.iter().zip(_original_names.iter()) {
                if let Some(prop) = new_schema.properties.iter_mut().find(|p| p.name == *current) {
                    prop.name = original.clone();
                }
            }
        }

        Ok(())
    }

    pub fn revert_rename_edge_properties(
        edge_ops: &mut EdgeOps,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        schema_ops: &SchemaOps,
        _current_names: &[String],
        _original_names: &[String],
    ) -> UndoLogResult<()> {
        let src_label_id = schema_ops
            .vertex_label_names
            .get(src_label)
            .copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;
        let dst_label_id = schema_ops
            .vertex_label_names
            .get(dst_label)
            .copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_ops
            .edge_label_names
            .get(edge_label)
            .copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;

        let key = (src_label_id, dst_label_id, edge_label_id);
        if let Some(table) = edge_ops.edge_tables.get_mut(&key) {
            let mut new_schema = table.schema().clone();
            for (current, original) in _current_names.iter().zip(_original_names.iter()) {
                if let Some(prop) = new_schema.properties.iter_mut().find(|p| p.name == *current) {
                    prop.name = original.clone();
                }
            }
        }

        Ok(())
    }

    pub fn revert_delete_vertex_properties(
        schema_ops: &mut SchemaOps,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        schema_ops
            .revert_delete_vertex_properties(label_name, prop_names)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))
    }

    pub fn revert_delete_edge_properties(
        edge_ops: &mut EdgeOps,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        schema_ops: &SchemaOps,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        edge_ops
            .revert_delete_edge_properties(src_label, dst_label, edge_label, schema_ops, prop_names)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))
    }
}
