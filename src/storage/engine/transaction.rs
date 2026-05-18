use crate::core::types::{EdgeId, LabelId, Timestamp, VertexId};
use crate::core::Value;
use crate::storage::edge::UpdateEdgePropertyByOffsetParams;
use crate::transaction::insert_transaction::{
    InsertTransactionError, InsertTransactionResult,
};
use crate::transaction::codec::{bytes_to_value, property_value_to_value};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult};

// Type aliases for backward compatibility
type TxnLabelId = LabelId;
type TxnVertexId = VertexId;

use super::schema::SchemaOps;
use super::edge::{EdgeOps, EdgeOperationParams};

/// Parameters for add_edge operation
pub struct AddEdgeParams {
    pub src_label: TxnLabelId,
    pub src_vid: TxnVertexId,
    pub dst_label: TxnLabelId,
    pub dst_vid: TxnVertexId,
    pub edge_label: TxnLabelId,
}

/// Parameters for delete_edge operation
pub struct DeleteEdgeParams {
    pub src_label: TxnLabelId,
    pub src_vid: TxnVertexId,
    pub dst_label: TxnLabelId,
    pub dst_vid: TxnVertexId,
    pub edge_label: TxnLabelId,
}

/// Parameters for update_edge_property_undo operation
pub struct UpdateEdgePropertyUndoParams {
    pub src_label: TxnLabelId,
    pub src_vid: TxnVertexId,
    pub dst_label: TxnLabelId,
    pub dst_vid: TxnVertexId,
    pub edge_label: TxnLabelId,
}

/// Parameters for insert_edge_undo operation
pub struct InsertEdgeUndoParams {
    pub src_label: TxnLabelId,
    pub src_vid: TxnVertexId,
    pub dst_label: TxnLabelId,
    pub dst_vid: TxnVertexId,
    pub edge_label: TxnLabelId,
}

/// Parameters for revert_delete_edge operation
pub struct RevertDeleteEdgeParams {
    pub src_label: TxnLabelId,
    pub src_vid: TxnVertexId,
    pub dst_label: TxnLabelId,
    pub dst_vid: TxnVertexId,
    pub edge_label: TxnLabelId,
}

/// Parameters for delete_edge_type operation
pub struct DeleteEdgeTypeParams {
    pub src_label: TxnLabelId,
    pub dst_label: TxnLabelId,
    pub edge_label: TxnLabelId,
}

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
            .insert_vertex(label, external_id, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(VertexId::from_int64(internal_id as i64))
    }

    pub fn add_edge(
        edge_ops: &mut EdgeOps,
        schema_ops: &SchemaOps,
        params: AddEdgeParams,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeId> {
        let src_label_id = params.src_label;
        let dst_label_id = params.dst_label;
        let src_table = schema_ops
            .get_vertex_table(src_label_id)
            .ok_or(InsertTransactionError::LabelNotFound(params.src_label))?;
        let dst_table = schema_ops
            .get_vertex_table(dst_label_id)
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

        let src_id_str = match &src_external {
            crate::storage::vertex::IdKey::Text(s) => s.clone(),
            crate::storage::vertex::IdKey::Int(i) => i.to_string(),
        };
        let dst_id_str = match &dst_external {
            crate::storage::vertex::IdKey::Text(s) => s.clone(),
            crate::storage::vertex::IdKey::Int(i) => i.to_string(),
        };

        let edge_op_params = EdgeOperationParams {
            edge_label: params.edge_label,
            src_label: params.src_label,
            src_id: &src_id_str,
            dst_label: params.dst_label,
            dst_id: &dst_id_str,
        };

        let edge_id = edge_ops
            .insert_edge(edge_op_params, &props, ts, schema_ops.vertex_tables())
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
        schema_ops
            .get_vertex_internal_id(label, external_id, ts)
            .map(|id| VertexId::from_int64(id as i64))
    }

    pub fn get_vertex_oid(
        schema_ops: &SchemaOps,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> Option<Vec<u8>> {
        schema_ops
            .get_vertex_table(label)?
            .get_external_id(vid.as_int64().unwrap_or(0) as u32, ts)
            .map(|id_key| match id_key {
                crate::storage::vertex::IdKey::Text(s) => s.into_bytes(),
                crate::storage::vertex::IdKey::Int(i) => i.to_string().into_bytes(),
            })
    }

    pub fn get_vertex_property_types(schema_ops: &SchemaOps, label: TxnLabelId) -> Vec<String> {
        schema_ops
            .get_vertex_table(label)
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
        edge_ops
            .get_edge_table_by_label(edge_label)
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
        schema_ops
            .get_vertex_table(label)
            .map(|t| t.total_count())
            .unwrap_or(0)
    }

    pub fn delete_vertex_type(
        schema_ops: &mut SchemaOps,
        edge_ops: &mut EdgeOps,
        label: TxnLabelId,
    ) -> UndoLogResult<()> {
        let label_name = schema_ops
            .get_vertex_table(label)
            .map(|t| t.label_name().to_string());

        if let Some(name) = label_name {
            schema_ops.vertex_label_names.remove(&name);
        }

        schema_ops.vertex_tables.remove(&label);
        edge_ops.drop_edges_for_vertex_label(label);

        Ok(())
    }

    pub fn delete_edge_type(
        edge_ops: &mut EdgeOps,
        params: DeleteEdgeTypeParams,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
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
        if let Some(table) = schema_ops.vertex_tables.get_mut(&label) {
            table
                .delete_by_internal_id(vid.as_int64().unwrap_or(0) as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn delete_edge(
        edge_ops: &mut EdgeOps,
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
        if let Some(table) = edge_ops.edge_tables.get_mut(&key) {
            table
                .delete_edge_by_offset(params.src_vid, params.dst_vid, oe_offset, ie_offset, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub fn revert_delete_edge(
        edge_ops: &mut EdgeOps,
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
        if let Some(table) = edge_ops.edge_tables.get_mut(&key) {
            table
                .revert_delete_edge_by_offset(params.src_vid, params.dst_vid, oe_offset, ie_offset, ts)
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
            .insert_vertex(label, external_id, &props, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn insert_edge_undo(
        edge_ops: &mut EdgeOps,
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
        let table = edge_ops
            .edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(params.edge_label))?;

        table
            .insert_edge(params.src_vid, params.dst_vid, &props, ts)
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
        let table = schema_ops
            .vertex_tables
            .get_mut(&label)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_property_by_id(vid.as_int64().unwrap_or(0) as u32, col_id, &value, ts)
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        Ok(())
    }

    pub fn update_edge_property_undo(
        edge_ops: &mut EdgeOps,
        params: UpdateEdgePropertyUndoParams,
        oe_offset: i32,
        ie_offset: i32,
        col_id: i32,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (
            params.src_label,
            params.dst_label,
            params.edge_label,
        );
        let table = edge_ops
            .edge_tables
            .get_mut(&key)
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let value = property_value_to_value(old_value);
        table
            .update_edge_property_by_offset(UpdateEdgePropertyByOffsetParams {
                src: params.src_vid,
                dst: params.dst_vid,
                oe_offset,
                ie_offset,
                col_id,
                value,
                ts,
            })
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
            .ok_or(UndoLogError::LabelNotFound(0))?;

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
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = schema_ops
            .vertex_label_names
            .get(dst_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let edge_label_id = edge_ops
            .edge_label_names
            .get(edge_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

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
