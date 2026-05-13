//! InsertTarget Implementation
//!
//! Implements the InsertTarget trait for PropertyGraph.

use crate::storage::edge::EdgeId;
use crate::storage::metadata::TableId;
use crate::transaction::insert_transaction::{
    AddEdgeInsertParam, InsertTarget, InsertTransactionResult,
};
use crate::transaction::wal::types::{
    LabelId as TxnLabelId, Timestamp, VertexId as TxnVertexId,
};

use super::super::transaction::{AddEdgeParams, TransactionOps};
use super::PropertyGraph;

impl InsertTarget for PropertyGraph {
    fn add_vertex(
        &mut self,
        label: TxnLabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<TxnVertexId> {
        let result = TransactionOps::add_vertex(
                &mut self.schema_ops,
                label,
                oid,
                properties,
                ts,
            )?;

        self.table_tracker.mark_modified(TableId::vertex(label));
        self.table_tracker
            .mark_modified_since_checkpoint(TableId::vertex(label));

        Ok(result)
    }

    fn add_edge(
        &mut self,
        param: AddEdgeInsertParam,
    ) -> InsertTransactionResult<EdgeId> {
        let params = AddEdgeParams {
            src_label: param.src_label,
            src_vid: param.src_vid,
            dst_label: param.dst_label,
            dst_vid: param.dst_vid,
            edge_label: param.edge_label,
        };
        let result = TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            params,
            param.properties,
            param.ts,
        )?;

        self.table_tracker
            .mark_modified(TableId::edge(param.edge_label));
        self.table_tracker
            .mark_modified_since_checkpoint(TableId::edge(param.edge_label));

        Ok(result)
    }

    fn get_vertex_id(
        &self,
        label: TxnLabelId,
        oid: &[u8],
        ts: Timestamp,
    ) -> Option<TxnVertexId> {
        let oid_str = String::from_utf8_lossy(oid).to_string();
        self.get_vertex(label, &oid_str, ts)
            .map(|v| v.internal_id as TxnVertexId)
    }

    fn get_vertex_oid(
        &self,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> Option<Vec<u8>> {
        TransactionOps::get_vertex_oid(
            &self.schema_ops,
            label,
            vid,
            ts,
        )
    }

    fn get_vertex_property_types(&self, label: TxnLabelId) -> Vec<String> {
        TransactionOps::get_vertex_property_types(&self.schema_ops, label)
    }

    fn get_edge_property_types(
        &self,
        src_label: TxnLabelId,
        dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> Vec<String> {
        TransactionOps::get_edge_property_types(
            &self.edge_ops,
            src_label,
            dst_label,
            edge_label,
        )
    }

    fn vertex_label_num(&self) -> usize {
        TransactionOps::vertex_label_num(&self.schema_ops)
    }

    fn lid_num(&self, label: TxnLabelId) -> usize {
        TransactionOps::lid_num(&self.schema_ops, label)
    }
}
