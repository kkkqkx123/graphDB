//! UndoTarget Implementation
//!
//! Implements the UndoTarget trait for PropertyGraph.

use crate::storage::edge::EdgeStrategy;
use crate::storage::metadata::TableId;
use crate::storage::{EdgeDeletionContext, EdgeIdentifier, EdgeKey, VertexIdentifier};
use crate::transaction::undo_log::{
    PropertyValue, UndoLogError, UndoLogResult, UndoTarget,
};
use crate::transaction::wal::types::{ColumnId, LabelId as TxnLabelId, Timestamp};

use super::super::edge::CreateEdgeTypeParams;
use super::super::transaction::{
    DeleteEdgeParams, DeleteEdgeTypeParams, RevertDeleteEdgeParams, TransactionOps,
    UpdateEdgePropertyUndoParams,
};
use super::PropertyGraph;

impl UndoTarget for PropertyGraph {
    fn delete_vertex_type(&mut self, label: TxnLabelId) -> UndoLogResult<()> {
        TransactionOps::delete_vertex_type(
            &mut self.schema_ops,
            &mut self.edge_ops,
            label,
        )?;
        self.table_tracker.mark_modified(TableId::vertex(label));
        Ok(())
    }

    fn delete_edge_type(&mut self, edge_key: EdgeKey) -> UndoLogResult<()> {
        let params = DeleteEdgeTypeParams {
            src_label: edge_key.src_label,
            dst_label: edge_key.dst_label,
            edge_label: edge_key.edge_label,
        };
        TransactionOps::delete_edge_type(&mut self.edge_ops, params)?;
        self.table_tracker
            .mark_modified(TableId::edge(edge_key.edge_label));
        Ok(())
    }

    fn delete_vertex(
        &mut self,
        vertex: VertexIdentifier,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_vertex(
            &mut self.schema_ops,
            vertex.label,
            vertex.vid,
            ts,
        )?;
        self.table_tracker.mark_modified(TableId::vertex(vertex.label));
        Ok(())
    }

    fn delete_edge(&mut self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = DeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_label: edge_ctx.edge_id.dst_label,
            dst_vid: edge_ctx.edge_id.dst_vid,
            edge_label: edge_ctx.edge_id.edge_label,
        };
        TransactionOps::delete_edge(
            &mut self.edge_ops,
            params,
            edge_ctx.oe_offset,
            edge_ctx.ie_offset,
            edge_ctx.timestamp,
        )?;
        self.table_tracker
            .mark_modified(TableId::edge(edge_ctx.edge_id.edge_label));
        Ok(())
    }

    fn undo_update_vertex_property(
        &mut self,
        vertex: VertexIdentifier,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_vertex_property_undo(
            &mut self.schema_ops,
            vertex.label,
            vertex.vid,
            col_id,
            value,
            ts,
        )?;
        self.table_tracker.mark_modified(TableId::vertex(vertex.label));
        Ok(())
    }

    fn undo_update_edge_property(
        &mut self,
        edge_id: EdgeIdentifier,
        oe_offset: i32,
        ie_offset: i32,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let params = UpdateEdgePropertyUndoParams {
            src_label: edge_id.src_label,
            src_vid: edge_id.src_vid,
            dst_label: edge_id.dst_label,
            dst_vid: edge_id.dst_vid,
            edge_label: edge_id.edge_label,
        };
        TransactionOps::update_edge_property_undo(
            &mut self.edge_ops,
            params,
            oe_offset,
            ie_offset,
            col_id,
            value,
            ts,
        )?;
        self.table_tracker
            .mark_modified(TableId::edge(edge_id.edge_label));
        Ok(())
    }

    fn revert_delete_vertex(
        &mut self,
        vertex: VertexIdentifier,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let params = RevertDeleteEdgeParams {
            src_label: vertex.label,
            src_vid: vertex.vid,
            dst_label: vertex.label,
            dst_vid: vertex.vid,
            edge_label: vertex.label,
        };
        TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            params,
            0,
            0,
            ts,
        )?;
        self.table_tracker.mark_modified(TableId::vertex(vertex.label));
        Ok(())
    }

    fn revert_delete_edge(&mut self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = RevertDeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_label: edge_ctx.edge_id.dst_label,
            dst_vid: edge_ctx.edge_id.dst_vid,
            edge_label: edge_ctx.edge_id.edge_label,
        };
        super::super::transaction::TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            params,
            edge_ctx.oe_offset,
            edge_ctx.ie_offset,
            edge_ctx.timestamp,
        )?;
        self.table_tracker
            .mark_modified(TableId::edge(edge_ctx.edge_id.edge_label));
        Ok(())
    }

    fn revert_delete_vertex_properties(
        &mut self,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_vertex_properties(
            &mut self.schema_ops,
            label_name,
            prop_names,
        )?;
        if let Some(label) = self.schema_ops.vertex_label_names.get(label_name) {
            self.table_tracker.mark_modified(TableId::vertex(*label));
        }
        Ok(())
    }

    fn revert_delete_edge_properties(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_edge_properties(
            &mut self.edge_ops,
            src_label,
            dst_label,
            edge_label,
            &self.schema_ops,
            prop_names,
        )?;
        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.table_tracker.mark_modified(TableId::edge(*label));
        }
        Ok(())
    }

    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
        let label = self.schema_ops.vertex_label_counter;
        TransactionOps::create_vertex_type_undo(
            &mut self.schema_ops,
            label_name,
            label,
        )?;
        self.table_tracker.mark_modified(TableId::vertex(label));
        Ok(())
    }

    fn revert_delete_edge_label(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
    ) -> UndoLogResult<()> {
        let src_label_id = self
            .schema_ops
            .vertex_label_names
            .get(src_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = self
            .schema_ops
            .vertex_label_names
            .get(dst_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        let params = CreateEdgeTypeParams {
            name: edge_label,
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties: Vec::new(),
            oe_strategy: EdgeStrategy::None,
            ie_strategy: EdgeStrategy::None,
        };
        self.edge_ops
            .create_edge_type(params, self.schema_ops.vertex_tables())
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;

        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.table_tracker.mark_modified(TableId::edge(*label));
        }

        Ok(())
    }

    fn revert_rename_vertex_properties(
        &mut self,
        label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_rename_vertex_properties(
            &mut self.schema_ops,
            label,
            current_names,
            original_names,
        )?;
        if let Some(label_id) = self.schema_ops.vertex_label_names.get(label) {
            self.table_tracker.mark_modified(TableId::vertex(*label_id));
        }
        Ok(())
    }

    fn revert_rename_edge_properties(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_rename_edge_properties(
            &mut self.edge_ops,
            src_label,
            dst_label,
            edge_label,
            &self.schema_ops,
            current_names,
            original_names,
        )?;
        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.table_tracker.mark_modified(TableId::edge(*label));
        }
        Ok(())
    }
}
