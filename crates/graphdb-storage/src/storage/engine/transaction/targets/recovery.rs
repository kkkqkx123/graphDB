use crate::core::types::{LabelId, Timestamp, VertexId};
use crate::core::wal::traits::RecoveryApplier;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::edge_params::EdgeOperationParams;
use crate::storage::engine::property_graph::PropertyGraph;
use crate::storage::engine::transaction::{AddEdgeParams, DeleteEdgeParams, TransactionOps};
use crate::transaction::codec::bytes_to_value;
use crate::transaction::wal::{InsertEdgeRedo, UpdateEdgePropRedo};

impl RecoveryApplier for PropertyGraph {
    fn replay_insert_vertex(
        &self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            TransactionOps::add_vertex(&mut vertex_tables, label, oid, properties, ts)?;
        }
        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_insert_edge(&self, redo: &InsertEdgeRedo, ts: Timestamp) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(&redo.src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(&redo.dst_oid).to_string();

        let src_vid = match self.get_vertex(redo.src_label, &src_oid_str, ts) {
            Some(v) => v.internal_id as u64,
            None => {
                log::warn!(
                    "Source vertex not found during recovery: label={}, oid={}, ts={}. \
                     This may indicate out-of-order WAL entries or incomplete transaction.",
                    redo.src_label,
                    src_oid_str,
                    ts
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
                    redo.dst_label,
                    dst_oid_str,
                    ts
                );
                return Err(StorageError::db_error(format!(
                    "Destination vertex not found during recovery: label={}, oid={}",
                    redo.dst_label, dst_oid_str
                )));
            }
        };

        let params = AddEdgeParams {
            src_label: redo.src_label,
            src_vid: VertexId::from_u64(src_vid),
            dst_label: redo.dst_label,
            dst_vid: VertexId::from_u64(dst_vid),
            edge_label: redo.edge_label,
        };

        {
            let vertex_tables = self.data_store.vertex_tables().read();
            let mut edge_tables = self.data_store.edge_tables().write();
            TransactionOps::add_edge(
                &mut edge_tables,
                &vertex_tables,
                params,
                &redo.properties,
                ts,
            )
            .map_err(|e| StorageError::db_error(format!("Failed to replay insert edge: {}", e)))?;
        }

        self.mark_edge_modified(redo.edge_label);
        Ok(())
    }

    fn replay_update_vertex_prop(
        &self,
        label: LabelId,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: Timestamp,
    ) -> StorageResult<()> {
        let oid_str = String::from_utf8_lossy(oid).to_string();
        let prop_value = bytes_to_value(value).ok_or_else(|| {
            StorageError::deserialize_error(
                "Failed to decode property value in WAL recovery".to_string(),
            )
        })?;

        {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            TransactionOps::update_vertex_property(
                &mut vertex_tables,
                label,
                &oid_str,
                prop_name,
                &prop_value,
                ts,
            )?;
        }

        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_update_edge_prop(
        &self,
        redo: &UpdateEdgePropRedo,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(&redo.src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(&redo.dst_oid).to_string();

        let prop_value = bytes_to_value(&redo.value).ok_or_else(|| {
            StorageError::deserialize_error(
                "Failed to decode property value in WAL recovery".to_string(),
            )
        })?;

        let params = EdgeOperationParams {
            src_label: redo.src_label,
            src_id: &src_oid_str,
            dst_label: redo.dst_label,
            dst_id: &dst_oid_str,
            edge_label: redo.edge_label,
        };

        {
            let vertex_tables = self.data_store.vertex_tables().read();
            let mut edge_tables = self.data_store.edge_tables().write();
            TransactionOps::update_edge_property(
                &mut edge_tables,
                &vertex_tables,
                params,
                &redo.prop_name,
                &prop_value,
                ts,
            )?;
        }
        self.mark_edge_modified(redo.edge_label);

        Ok(())
    }

    fn replay_delete_vertex(&self, label: LabelId, oid: &[u8], ts: Timestamp) -> StorageResult<()> {
        let oid_str = String::from_utf8_lossy(oid).to_string();

        if let Some(vertex) = self.get_vertex(label, &oid_str, ts) {
            {
                let mut vertex_tables = self.data_store.vertex_tables().write();
                TransactionOps::delete_vertex(
                    &mut vertex_tables,
                    label,
                    VertexId::from_u64(vertex.internal_id as u64),
                    ts,
                )
                .map_err(|e| {
                    StorageError::db_error(format!("Failed to replay delete vertex: {}", e))
                })?;
            }
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
        &self,
        src_label: LabelId,
        src_oid: &[u8],
        dst_label: LabelId,
        dst_oid: &[u8],
        edge_label: LabelId,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let src_oid_str = String::from_utf8_lossy(src_oid).to_string();
        let dst_oid_str = String::from_utf8_lossy(dst_oid).to_string();

        let src = self.get_vertex(src_label, &src_oid_str, ts);
        let dst = self.get_vertex(dst_label, &dst_oid_str, ts);

        if let (Some(src), Some(dst)) = (src, dst) {
            let params = DeleteEdgeParams {
                src_label,
                src_vid: VertexId::from_u64(src.internal_id as u64),
                dst_label,
                dst_vid: VertexId::from_u64(dst.internal_id as u64),
                edge_label,
            };

            {
                let mut edge_tables = self.data_store.edge_tables().write();
                TransactionOps::delete_edge(&mut edge_tables, params, 0, 0, ts).map_err(|e| {
                    StorageError::db_error(format!("Failed to replay delete edge: {}", e))
                })?;
            }
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
