//! RecoveryApplier Implementation
//!
//! Implements the RecoveryApplier trait for PropertyGraph.

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::metadata::TableId;
use crate::transaction::wal::RecoveryApplier;

use super::super::edge::EdgeOperationParams;
use super::super::transaction::{AddEdgeParams, DeleteEdgeParams, TransactionOps};
use super::PropertyGraph;

impl RecoveryApplier for PropertyGraph {
    fn replay_insert_vertex(
        &mut self,
        label: u32,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: u32,
    ) -> StorageResult<()> {
        TransactionOps::add_vertex(
            &mut self.schema_ops,
            label,
            oid,
            properties,
            ts,
        )?;

        self.table_tracker.mark_modified(TableId::vertex(label));
        Ok(())
    }

    fn replay_insert_edge(
        &mut self,
        src_label: u32,
        src_oid: &[u8],
        dst_label: u32,
        dst_oid: &[u8],
        edge_label: u32,
        properties: &[(String, Vec<u8>)],
        ts: u32,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(dst_oid).to_string();

        let src_vid = self
            .get_vertex(src_label, &src_oid_str, ts)
            .map(|v| v.internal_id as u64)
            .ok_or_else(|| {
                StorageError::db_error("Source vertex not found during recovery".to_string())
            })?;

        let dst_vid = self
            .get_vertex(dst_label, &dst_oid_str, ts)
            .map(|v| v.internal_id as u64)
            .ok_or_else(|| {
                StorageError::db_error("Destination vertex not found during recovery".to_string())
            })?;

        let params = AddEdgeParams {
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
        };

        TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            params,
            properties,
            ts,
        )
        .map_err(|e| StorageError::db_error(format!("Failed to replay insert edge: {}", e)))?;

        self.table_tracker.mark_modified(TableId::edge(edge_label));
        Ok(())
    }

    fn replay_update_vertex_prop(
        &mut self,
        label: u32,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: u32,
    ) -> StorageResult<()> {
        let oid_str = String::from_utf8_lossy(oid).to_string();
        let prop_value: Value =
            serde_json::from_slice(value).unwrap_or_else(|_| Value::Empty);

        self.schema_ops
            .update_vertex_property(label, &oid_str, prop_name, &prop_value, ts)?;

        self.table_tracker.mark_modified(TableId::vertex(label));
        Ok(())
    }

    fn replay_update_edge_prop(
        &mut self,
        src_label: u32,
        src_oid: &[u8],
        dst_label: u32,
        dst_oid: &[u8],
        edge_label: u32,
        prop_name: &str,
        value: &[u8],
        ts: u32,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(dst_oid).to_string();

        let prop_value: Value =
            serde_json::from_slice(value).unwrap_or_else(|_| Value::Empty);

        let params = EdgeOperationParams {
            src_label,
            src_id: &src_oid_str,
            dst_label,
            dst_id: &dst_oid_str,
            edge_label,
        };

        self.edge_ops.update_edge_property(
            params,
            prop_name,
            &prop_value,
            ts,
            self.schema_ops.vertex_tables(),
        )?;
        self.table_tracker.mark_modified(TableId::edge(edge_label));

        Ok(())
    }

    fn replay_delete_vertex(
        &mut self,
        label: u32,
        oid: &[u8],
        ts: u32,
    ) -> StorageResult<()> {
        let oid_str = String::from_utf8_lossy(oid).to_string();

        if let Some(vertex) = self.get_vertex(label, &oid_str, ts) {
            TransactionOps::delete_vertex(
                &mut self.schema_ops,
                label,
                vertex.internal_id as u64,
                ts,
            )
            .map_err(|e| StorageError::db_error(format!("Failed to replay delete vertex: {}", e)))?;
            self.table_tracker.mark_modified(TableId::vertex(label));
        }

        Ok(())
    }

    fn replay_delete_edge(
        &mut self,
        src_label: u32,
        src_oid: &[u8],
        dst_label: u32,
        dst_oid: &[u8],
        edge_label: u32,
        ts: u32,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(dst_oid).to_string();

        if let (Some(src), Some(dst)) = (
            self.get_vertex(src_label, &src_oid_str, ts),
            self.get_vertex(dst_label, &dst_oid_str, ts),
        ) {
            let params = DeleteEdgeParams {
                src_label,
                src_vid: src.internal_id as u64,
                dst_label,
                dst_vid: dst.internal_id as u64,
                edge_label,
            };

            TransactionOps::delete_edge(
                &mut self.edge_ops,
                params,
                0,
                0,
                ts,
            )
            .map_err(|e| StorageError::db_error(format!("Failed to replay delete edge: {}", e)))?;
            self.table_tracker.mark_modified(TableId::edge(edge_label));
        }

        Ok(())
    }
}
