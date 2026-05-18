use crate::core::types::{ColumnId, LabelId, Timestamp};
use crate::storage::edge::EdgeStrategy;
use crate::storage::engine::edge::CreateEdgeTypeParams;
use crate::storage::{EdgeDeletionContext, EdgeIdentifier, EdgeKey, VertexIdentifier};
use crate::core::types::{PropertyValue, UndoLogError, UndoLogResult, UndoTarget};

// Type alias for backward compatibility
type TxnLabelId = LabelId;

use crate::storage::engine::transaction::{
    DeleteEdgeParams, DeleteEdgeTypeParams, RevertDeleteEdgeParams, TransactionOps,
    UpdateEdgePropertyUndoParams,
};
use super::super::PropertyGraph;

impl UndoTarget for PropertyGraph {
    fn delete_vertex_type(&self, label: TxnLabelId) -> UndoLogResult<()> {
        {
            let mut schema = self.schema_ops.write();
            let mut edge = self.edge_ops.write();
            TransactionOps::delete_vertex_type(&mut schema, &mut edge, label)?;
        }
        self.mark_vertex_modified(label);
        Ok(())
    }

    fn delete_edge_type(&self, edge_key: EdgeKey) -> UndoLogResult<()> {
        let params = DeleteEdgeTypeParams {
            src_label: edge_key.src_label,
            dst_label: edge_key.dst_label,
            edge_label: edge_key.edge_label,
        };
        {
            let mut edge = self.edge_ops.write();
            TransactionOps::delete_edge_type(&mut edge, params)?;
        }
        self.mark_edge_modified(edge_key.edge_label);
        Ok(())
    }

    fn delete_vertex(
        &self,
        vertex: VertexIdentifier,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        {
            let mut schema = self.schema_ops.write();
            TransactionOps::delete_vertex(&mut schema, vertex.label, vertex.vid, ts)?;
        }
        self.mark_vertex_modified(vertex.label);
        Ok(())
    }

    fn delete_edge(&self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = DeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_label: edge_ctx.edge_id.dst_label,
            dst_vid: edge_ctx.edge_id.dst_vid,
            edge_label: edge_ctx.edge_id.edge_label,
        };
        {
            let mut edge = self.edge_ops.write();
            TransactionOps::delete_edge(
                &mut edge,
                params,
                edge_ctx.oe_offset,
                edge_ctx.ie_offset,
                edge_ctx.timestamp,
            )?;
        }
        self.mark_edge_modified(edge_ctx.edge_id.edge_label);
        Ok(())
    }

    fn undo_update_vertex_property(
        &self,
        vertex: VertexIdentifier,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        {
            let mut schema = self.schema_ops.write();
            TransactionOps::update_vertex_property_undo(
                &mut schema,
                vertex.label,
                vertex.vid,
                col_id,
                value,
                ts,
            )?;
        }
        self.mark_vertex_modified(vertex.label);
        Ok(())
    }

    fn undo_update_edge_property(
        &self,
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
        {
            let mut edge = self.edge_ops.write();
            TransactionOps::update_edge_property_undo(
                &mut edge,
                params,
                oe_offset,
                ie_offset,
                col_id as u16,
                value,
                ts,
            )?;
        }
        self.mark_edge_modified(edge_id.edge_label);
        Ok(())
    }

    fn revert_delete_vertex(
        &self,
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
        {
            let mut edge = self.edge_ops.write();
            TransactionOps::revert_delete_edge(
                &mut edge,
                params,
                0,
                0,
                ts,
            )?;
        }
        self.mark_vertex_modified(vertex.label);
        Ok(())
    }

    fn revert_delete_edge(&self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = RevertDeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            dst_label: edge_ctx.edge_id.dst_label,
            edge_label: edge_ctx.edge_id.edge_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_vid: edge_ctx.edge_id.dst_vid,
        };
        {
            let mut edge = self.edge_ops.write();
            TransactionOps::revert_delete_edge(
                &mut edge,
                params,
                edge_ctx.oe_offset,
                edge_ctx.ie_offset,
                edge_ctx.timestamp,
            )?;
        }
        self.mark_edge_modified(edge_ctx.edge_id.edge_label);
        Ok(())
    }

    fn revert_delete_vertex_properties(
        &self,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        let label_id = {
            let mut schema = self.schema_ops.write();
            TransactionOps::revert_delete_vertex_properties(
                &mut schema,
                label_name,
                prop_names,
            )?;
            schema.vertex_label_names.get(label_name).copied()
        };
        if let Some(label) = label_id {
            self.mark_vertex_modified(label);
        }
        Ok(())
    }

    fn revert_delete_edge_properties(
        &self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        let edge_label_id = {
            let schema = self.schema_ops.read();
            let mut edge = self.edge_ops.write();
            TransactionOps::revert_delete_edge_properties(
                &mut edge,
                src_label,
                dst_label,
                edge_label,
                &schema,
                prop_names,
            )?;
            edge.edge_label_names.get(edge_label).copied()
        };
        if let Some(label) = edge_label_id {
            self.mark_edge_modified(label);
        }
        Ok(())
    }

    fn revert_delete_vertex_label(&self, label_name: &str) -> UndoLogResult<()> {
        let label;
        {
            let mut schema = self.schema_ops.write();
            label = schema.vertex_label_counter;
            TransactionOps::create_vertex_type_undo(
                &mut schema,
                label_name,
                label,
            )?;
        }
        self.mark_vertex_modified(label);
        Ok(())
    }

    fn revert_delete_edge_label(
        &self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
    ) -> UndoLogResult<()> {
        let (src_label_id, dst_label_id);
        {
            let schema = self.schema_ops.read();
            src_label_id = schema
                .vertex_label_names
                .get(src_label)
                .copied()
                .ok_or(UndoLogError::LabelNotFound(0))?;
            dst_label_id = schema
                .vertex_label_names
                .get(dst_label)
                .copied()
                .ok_or(UndoLogError::LabelNotFound(0))?;
        }

        let params = CreateEdgeTypeParams {
            name: edge_label,
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties: Vec::new(),
            oe_strategy: EdgeStrategy::None,
            ie_strategy: EdgeStrategy::None,
        };
        let edge_label_id = {
            let schema = self.schema_ops.read();
            let mut edge = self.edge_ops.write();
            edge
                .create_edge_type(params, &schema.vertex_tables)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
            edge.edge_label_names.get(edge_label).copied().ok_or(UndoLogError::LabelNotFound(0))?
        };

        self.mark_edge_modified(edge_label_id);
        Ok(())
    }

    fn revert_rename_vertex_properties(
        &self,
        label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        let label_id = {
            let mut schema = self.schema_ops.write();
            TransactionOps::revert_rename_vertex_properties(
                &mut schema,
                label,
                current_names,
                original_names,
            )?;
            schema.vertex_label_names.get(label).copied()
        };
        if let Some(label_id) = label_id {
            self.mark_vertex_modified(label_id);
        }
        Ok(())
    }

    fn revert_rename_edge_properties(
        &self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        let edge_label_id = {
            let schema = self.schema_ops.read();
            let mut edge = self.edge_ops.write();
            TransactionOps::revert_rename_edge_properties(
                &mut edge,
                src_label,
                dst_label,
                edge_label,
                &schema,
                current_names,
                original_names,
            )?;
            edge.edge_label_names.get(edge_label).copied()
        };
        if let Some(label) = edge_label_id {
            self.mark_edge_modified(label);
        }
        Ok(())
    }
}
