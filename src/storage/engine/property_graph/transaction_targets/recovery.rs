use crate::core::{StorageError, StorageResult};
use crate::transaction::codec::bytes_to_value;
use crate::transaction::wal::{InsertEdgeRedo, RecoveryApplier, UpdateEdgePropRedo};
use crate::transaction::wal::types::{LabelId as TxnLabelId, Timestamp};

use crate::storage::engine::edge::EdgeOperationParams;
use crate::storage::engine::transaction::{AddEdgeParams, DeleteEdgeParams, TransactionOps};
use super::super::PropertyGraph;

impl RecoveryApplier for PropertyGraph {
    fn replay_insert_vertex(
        &mut self,
        label: TxnLabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        TransactionOps::add_vertex(
            &mut self.schema_ops,
            label,
            oid,
            properties,
            ts,
        )?;

        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_insert_edge(
        &mut self,
        redo: &InsertEdgeRedo,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(&redo.src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(&redo.dst_oid).to_string();

        let src_vid = match self.get_vertex(redo.src_label, &src_oid_str, ts) {
            Some(v) => v.internal_id as u64,
            None => {
                log::warn!(
                    "Source vertex not found during recovery: label={}, oid={}, ts={}. \
                     This may indicate out-of-order WAL entries or incomplete transaction.",
                    redo.src_label, src_oid_str, ts
                );
                return Err(StorageError::db_error(format!(
                    "Source vertex not found during recovery: label={}, oid={}",
                    redo.src_label, src_oid_str
                )));
            }
        };

        let dst_vid = match self.get_vertex(redo.dst_label, &dst_oid_str, ts) {
            Some(v) => v.internal_id as u64,
            None => {
                log::warn!(
                    "Destination vertex not found during recovery: label={}, oid={}, ts={}. \
                     This may indicate out-of-order WAL entries or incomplete transaction.",
                    redo.dst_label, dst_oid_str, ts
                );
                return Err(StorageError::db_error(format!(
                    "Destination vertex not found during recovery: label={}, oid={}",
                    redo.dst_label, dst_oid_str
                )));
            }
        };

        let params = AddEdgeParams {
            src_label: redo.src_label,
            src_vid,
            dst_label: redo.dst_label,
            dst_vid,
            edge_label: redo.edge_label,
        };

        TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            params,
            &redo.properties,
            ts,
        )
        .map_err(|e| StorageError::db_error(format!("Failed to replay insert edge: {}", e)))?;

        self.mark_edge_modified(redo.edge_label);
        Ok(())
    }

    fn replay_update_vertex_prop(
        &mut self,
        label: TxnLabelId,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: Timestamp,
    ) -> StorageResult<()> {
        let oid_str = String::from_utf8_lossy(oid).to_string();
        let prop_value = bytes_to_value(value).ok_or_else(|| {
            StorageError::deserialize_error("Failed to decode property value in WAL recovery".to_string())
        })?;

        self.schema_ops
            .update_vertex_property(label, &oid_str, prop_name, &prop_value, ts)?;

        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_update_edge_prop(
        &mut self,
        redo: &UpdateEdgePropRedo,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(&redo.src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(&redo.dst_oid).to_string();

        let prop_value = bytes_to_value(&redo.value).ok_or_else(|| {
            StorageError::deserialize_error("Failed to decode property value in WAL recovery".to_string())
        })?;

        let params = EdgeOperationParams {
            src_label: redo.src_label,
            src_id: &src_oid_str,
            dst_label: redo.dst_label,
            dst_id: &dst_oid_str,
            edge_label: redo.edge_label,
        };

        self.edge_ops.update_edge_property(
            params,
            &redo.prop_name,
            &prop_value,
            ts,
            self.schema_ops.vertex_tables(),
        )?;
        self.mark_edge_modified(redo.edge_label);

        Ok(())
    }

    fn replay_delete_vertex(
        &mut self,
        label: TxnLabelId,
        oid: &[u8],
        ts: Timestamp,
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
            self.mark_vertex_modified(label);
        } else {
            log::debug!(
                "Vertex not found during delete replay: label={}, oid={}. Already deleted or never existed.",
                label, oid_str
            );
        }

        Ok(())
    }

    fn replay_delete_edge(
        &mut self,
        src_label: TxnLabelId,
        src_oid: &[u8],
        dst_label: TxnLabelId,
        dst_oid: &[u8],
        edge_label: TxnLabelId,
        ts: Timestamp,
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
            self.mark_edge_modified(edge_label);
        } else {
            log::debug!(
                "Vertices not found during edge delete replay: src=({},{}) dst=({},{}). Edge already deleted or vertices removed.",
                src_label, src_oid_str, dst_label, dst_oid_str
            );
        }

        Ok(())
    }
}
