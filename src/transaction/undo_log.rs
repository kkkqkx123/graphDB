//! Undo Log
//!
//! Provides transaction rollback support through undo log entries.
//! Each undo log entry can reverse a specific operation during transaction abort.

use std::collections::HashMap;
use std::sync::Arc;

use super::wal::types::{ColumnId, EdgeId, LabelId, Timestamp, VertexId};

/// Undo log error
#[derive(Debug, Clone, thiserror::Error)]
pub enum UndoLogError {
    #[error("Undo operation failed: {0}")]
    UndoFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Label not found: {0}")]
    LabelNotFound(LabelId),

    #[error("Vertex not found: {0}")]
    VertexNotFound(VertexId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),
}

/// Undo log result type
pub type UndoLogResult<T> = Result<T, UndoLogError>;

/// Property value type for undo operations
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Null,
}

impl PropertyValue {
    pub fn is_null(&self) -> bool {
        matches!(self, PropertyValue::Null)
    }
}

/// Trait for undo log entries
pub trait UndoLog: Send + Sync {
    /// Execute the undo operation
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()>;

    /// Get a description of this undo operation
    fn description(&self) -> String;
}

/// Target for undo operations (will be PropertyGraph in phase 2)
pub trait UndoTarget: Send + Sync {
    fn delete_vertex_type(&mut self, label: LabelId) -> UndoLogResult<()>;
    fn delete_edge_type(&mut self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> UndoLogResult<()>;
    fn delete_vertex(&mut self, label: LabelId, vid: VertexId, ts: Timestamp) -> UndoLogResult<()>;
    fn delete_edge(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, ts: Timestamp) -> UndoLogResult<()>;
    fn update_vertex_property(&mut self, label: LabelId, vid: VertexId, col_id: ColumnId, value: PropertyValue, ts: Timestamp) -> UndoLogResult<()>;
    fn update_edge_property(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, col_id: ColumnId, value: PropertyValue, ts: Timestamp) -> UndoLogResult<()>;
    fn revert_delete_vertex(&mut self, label: LabelId, vid: VertexId, ts: Timestamp) -> UndoLogResult<()>;
    fn revert_delete_edge(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, ts: Timestamp) -> UndoLogResult<()>;
    fn revert_delete_vertex_properties(&mut self, label_name: &str, prop_names: &[String]) -> UndoLogResult<()>;
    fn revert_delete_edge_properties(&mut self, src_label: &str, dst_label: &str, edge_label: &str, prop_names: &[String]) -> UndoLogResult<()>;
    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()>;
    fn revert_delete_edge_label(&mut self, src_label: &str, dst_label: &str, edge_label: &str) -> UndoLogResult<()>;
}

/// Undo log for create vertex type operation
#[derive(Debug, Clone)]
pub struct CreateVertexTypeUndo {
    pub vertex_type: LabelId,
}

impl UndoLog for CreateVertexTypeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.delete_vertex_type(self.vertex_type)
    }

    fn description(&self) -> String {
        format!("CreateVertexTypeUndo(label={})", self.vertex_type)
    }
}

/// Undo log for create edge type operation
#[derive(Debug, Clone)]
pub struct CreateEdgeTypeUndo {
    pub src_type: LabelId,
    pub dst_type: LabelId,
    pub edge_type: LabelId,
}

impl UndoLog for CreateEdgeTypeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.delete_edge_type(self.src_type, self.dst_type, self.edge_type)
    }

    fn description(&self) -> String {
        format!(
            "CreateEdgeTypeUndo(src={}, dst={}, edge={})",
            self.src_type, self.dst_type, self.edge_type
        )
    }
}

/// Undo log for insert vertex operation
#[derive(Debug, Clone)]
pub struct InsertVertexUndo {
    pub v_label: LabelId,
    pub vid: VertexId,
}

impl UndoLog for InsertVertexUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.delete_vertex(self.v_label, self.vid, ts)
    }

    fn description(&self) -> String {
        format!("InsertVertexUndo(label={}, vid={})", self.v_label, self.vid)
    }
}

/// Undo log for insert edge operation
#[derive(Debug, Clone)]
pub struct InsertEdgeUndo {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub oe_offset: i32,
    pub ie_offset: i32,
}

impl UndoLog for InsertEdgeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.delete_edge(
            self.src_label,
            self.src_vid,
            self.dst_label,
            self.dst_vid,
            self.edge_label,
            self.oe_offset,
            self.ie_offset,
            ts,
        )
    }

    fn description(&self) -> String {
        format!(
            "InsertEdgeUndo(src={}, dst={}, edge={}, src_vid={}, dst_vid={})",
            self.src_label, self.dst_label, self.edge_label, self.src_vid, self.dst_vid
        )
    }
}

/// Undo log for update vertex property operation
#[derive(Debug, Clone)]
pub struct UpdateVertexPropUndo {
    pub v_label: LabelId,
    pub vid: VertexId,
    pub col_id: ColumnId,
    pub old_value: PropertyValue,
}

impl UndoLog for UpdateVertexPropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.update_vertex_property(self.v_label, self.vid, self.col_id, self.old_value.clone(), ts)
    }

    fn description(&self) -> String {
        format!(
            "UpdateVertexPropUndo(label={}, vid={}, col={})",
            self.v_label, self.vid, self.col_id
        )
    }
}

/// Undo log for update edge property operation
#[derive(Debug, Clone)]
pub struct UpdateEdgePropUndo {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
    pub oe_offset: i32,
    pub ie_offset: i32,
    pub col_id: ColumnId,
    pub old_value: PropertyValue,
}

impl UndoLog for UpdateEdgePropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.update_edge_property(
            self.src_label,
            self.src_vid,
            self.dst_label,
            self.dst_vid,
            self.edge_label,
            self.oe_offset,
            self.ie_offset,
            self.col_id,
            self.old_value.clone(),
            ts,
        )
    }

    fn description(&self) -> String {
        format!(
            "UpdateEdgePropUndo(src={}, dst={}, edge={}, col={})",
            self.src_label, self.dst_label, self.edge_label, self.col_id
        )
    }
}

/// Related edge information for remove vertex undo
#[derive(Debug, Clone)]
pub struct RelatedEdgeInfo {
    pub src_vid: VertexId,
    pub dst_vid: VertexId,
    pub oe_offset: i32,
    pub ie_offset: i32,
}

/// Undo log for remove vertex operation
#[derive(Debug, Clone)]
pub struct RemoveVertexUndo {
    pub v_label: LabelId,
    pub vid: VertexId,
    pub related_edges: Vec<(LabelId, LabelId, LabelId, Vec<RelatedEdgeInfo>)>,
}

impl UndoLog for RemoveVertexUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_vertex(self.v_label, self.vid, ts)?;

        for (src_label, dst_label, edge_label, edges) in &self.related_edges {
            for edge in edges {
                graph.revert_delete_edge(
                    *src_label,
                    edge.src_vid,
                    *dst_label,
                    edge.dst_vid,
                    *edge_label,
                    edge.oe_offset,
                    edge.ie_offset,
                    ts,
                )?;
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "RemoveVertexUndo(label={}, vid={}, edges={})",
            self.v_label,
            self.vid,
            self.related_edges.len()
        )
    }
}

/// Undo log for remove edge operation
#[derive(Debug, Clone)]
pub struct RemoveEdgeUndo {
    pub src_label: LabelId,
    pub src_vid: VertexId,
    pub dst_label: LabelId,
    pub dst_vid: VertexId,
    pub edge_label: LabelId,
    pub oe_offset: i32,
    pub ie_offset: i32,
}

impl UndoLog for RemoveEdgeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_edge(
            self.src_label,
            self.src_vid,
            self.dst_label,
            self.dst_vid,
            self.edge_label,
            self.oe_offset,
            self.ie_offset,
            ts,
        )
    }

    fn description(&self) -> String {
        format!(
            "RemoveEdgeUndo(src={}, dst={}, edge={})",
            self.src_label, self.dst_label, self.edge_label
        )
    }
}

/// Undo log for add vertex property operation
#[derive(Debug, Clone)]
pub struct AddVertexPropUndo {
    pub label: LabelId,
    pub label_name: String,
    pub prop_names: Vec<String>,
}

impl UndoLog for AddVertexPropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_vertex_properties(&self.label_name, &self.prop_names)
    }

    fn description(&self) -> String {
        format!(
            "AddVertexPropUndo(label={}, props={:?})",
            self.label_name, self.prop_names
        )
    }
}

/// Undo log for add edge property operation
#[derive(Debug, Clone)]
pub struct AddEdgePropUndo {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
    pub src_label_name: String,
    pub dst_label_name: String,
    pub edge_label_name: String,
    pub prop_names: Vec<String>,
}

impl UndoLog for AddEdgePropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_edge_properties(
            &self.src_label_name,
            &self.dst_label_name,
            &self.edge_label_name,
            &self.prop_names,
        )
    }

    fn description(&self) -> String {
        format!(
            "AddEdgePropUndo(edge={}, props={:?})",
            self.edge_label_name, self.prop_names
        )
    }
}

/// Undo log for rename vertex property operation
#[derive(Debug, Clone)]
pub struct RenameVertexPropUndo {
    pub label: LabelId,
    pub label_name: String,
    pub old_names_to_new_names: Vec<(String, String)>,
}

impl UndoLog for RenameVertexPropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        let new_names_to_old_names: Vec<_> = self
            .old_names_to_new_names
            .iter()
            .map(|(old, new)| (new.clone(), old.clone()))
            .collect();
        graph.revert_delete_vertex_properties(&self.label_name, &new_names_to_old_names.iter().map(|(_, old)| old.clone()).collect::<Vec<_>>())
    }

    fn description(&self) -> String {
        format!(
            "RenameVertexPropUndo(label={}, renames={:?})",
            self.label_name, self.old_names_to_new_names
        )
    }
}

/// Undo log for rename edge property operation
#[derive(Debug, Clone)]
pub struct RenameEdgePropUndo {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
    pub src_label_name: String,
    pub dst_label_name: String,
    pub edge_label_name: String,
    pub old_names_to_new_names: Vec<(String, String)>,
}

impl UndoLog for RenameEdgePropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        let new_names_to_old_names: Vec<_> = self
            .old_names_to_new_names
            .iter()
            .map(|(old, new)| (new.clone(), old.clone()))
            .collect();
        graph.revert_delete_edge_properties(
            &self.src_label_name,
            &self.dst_label_name,
            &self.edge_label_name,
            &new_names_to_old_names.iter().map(|(_, old)| old.clone()).collect::<Vec<_>>(),
        )
    }

    fn description(&self) -> String {
        format!(
            "RenameEdgePropUndo(edge={}, renames={:?})",
            self.edge_label_name, self.old_names_to_new_names
        )
    }
}

/// Undo log for delete vertex property operation
#[derive(Debug, Clone)]
pub struct DeleteVertexPropUndo {
    pub label: LabelId,
    pub label_name: String,
    pub prop_names: Vec<String>,
}

impl UndoLog for DeleteVertexPropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_vertex_properties(&self.label_name, &self.prop_names)
    }

    fn description(&self) -> String {
        format!(
            "DeleteVertexPropUndo(label={}, props={:?})",
            self.label_name, self.prop_names
        )
    }
}

/// Undo log for delete edge property operation
#[derive(Debug, Clone)]
pub struct DeleteEdgePropUndo {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
    pub src_label_name: String,
    pub dst_label_name: String,
    pub edge_label_name: String,
    pub prop_names: Vec<String>,
}

impl UndoLog for DeleteEdgePropUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_edge_properties(
            &self.src_label_name,
            &self.dst_label_name,
            &self.edge_label_name,
            &self.prop_names,
        )
    }

    fn description(&self) -> String {
        format!(
            "DeleteEdgePropUndo(edge={}, props={:?})",
            self.edge_label_name, self.prop_names
        )
    }
}

/// Undo log for delete vertex type operation
#[derive(Debug, Clone)]
pub struct DeleteVertexTypeUndo {
    pub v_label: String,
}

impl UndoLog for DeleteVertexTypeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_vertex_label(&self.v_label)
    }

    fn description(&self) -> String {
        format!("DeleteVertexTypeUndo(label={})", self.v_label)
    }
}

/// Undo log for delete edge type operation
#[derive(Debug, Clone)]
pub struct DeleteEdgeTypeUndo {
    pub src_label: String,
    pub dst_label: String,
    pub edge_label: String,
}

impl UndoLog for DeleteEdgeTypeUndo {
    fn undo(&self, graph: &mut dyn UndoTarget, _ts: Timestamp) -> UndoLogResult<()> {
        graph.revert_delete_edge_label(&self.src_label, &self.dst_label, &self.edge_label)
    }

    fn description(&self) -> String {
        format!(
            "DeleteEdgeTypeUndo(src={}, dst={}, edge={})",
            self.src_label, self.dst_label, self.edge_label
        )
    }
}

/// Undo log manager for collecting and executing undo logs
pub struct UndoLogManager {
    logs: Vec<Box<dyn UndoLog>>,
}

impl UndoLogManager {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn add(&mut self, log: Box<dyn UndoLog>) {
        self.logs.push(log);
    }

    pub fn add_insert_vertex(&mut self, label: LabelId, vid: VertexId) {
        self.add(Box::new(InsertVertexUndo { v_label: label, vid }));
    }

    pub fn add_insert_edge(
        &mut self,
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
        src_vid: VertexId,
        dst_vid: VertexId,
        oe_offset: i32,
        ie_offset: i32,
    ) {
        self.add(Box::new(InsertEdgeUndo {
            src_label,
            dst_label,
            edge_label,
            src_vid,
            dst_vid,
            oe_offset,
            ie_offset,
        }));
    }

    pub fn add_update_vertex_prop(
        &mut self,
        label: LabelId,
        vid: VertexId,
        col_id: ColumnId,
        old_value: PropertyValue,
    ) {
        self.add(Box::new(UpdateVertexPropUndo {
            v_label: label,
            vid,
            col_id,
            old_value,
        }));
    }

    pub fn add_update_edge_prop(
        &mut self,
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
        oe_offset: i32,
        ie_offset: i32,
        col_id: ColumnId,
        old_value: PropertyValue,
    ) {
        self.add(Box::new(UpdateEdgePropUndo {
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
            col_id,
            old_value,
        }));
    }

    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.logs.len()
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn execute_undo(&mut self, graph: &mut dyn UndoTarget, ts: Timestamp) -> UndoLogResult<()> {
        while let Some(log) = self.logs.pop() {
            log.undo(graph, ts)?;
        }
        Ok(())
    }
}

impl Default for UndoLogManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUndoTarget;

    impl UndoTarget for MockUndoTarget {
        fn delete_vertex_type(&mut self, label: LabelId) -> UndoLogResult<()> {
            Ok(())
        }

        fn delete_edge_type(&mut self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> UndoLogResult<()> {
            Ok(())
        }

        fn delete_vertex(&mut self, label: LabelId, vid: VertexId, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn delete_edge(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn update_vertex_property(&mut self, label: LabelId, vid: VertexId, col_id: ColumnId, value: PropertyValue, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn update_edge_property(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, col_id: ColumnId, value: PropertyValue, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_vertex(&mut self, label: LabelId, vid: VertexId, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_edge(&mut self, src_label: LabelId, src_vid: VertexId, dst_label: LabelId, dst_vid: VertexId, edge_label: LabelId, oe_offset: i32, ie_offset: i32, ts: Timestamp) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_vertex_properties(&mut self, label_name: &str, prop_names: &[String]) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_edge_properties(&mut self, src_label: &str, dst_label: &str, edge_label: &str, prop_names: &[String]) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
            Ok(())
        }

        fn revert_delete_edge_label(&mut self, src_label: &str, dst_label: &str, edge_label: &str) -> UndoLogResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_undo_log_manager() {
        let mut manager = UndoLogManager::new();

        manager.add_insert_vertex(1, 100);
        manager.add_insert_edge(1, 2, 3, 100, 200, 0, 0);

        assert_eq!(manager.len(), 2);

        let mut target = MockUndoTarget;
        manager.execute_undo(&mut target, 1).expect("Undo failed");

        assert!(manager.is_empty());
    }

    #[test]
    fn test_create_vertex_type_undo() {
        let undo = CreateVertexTypeUndo { vertex_type: 1 };
        assert!(undo.description().contains("CreateVertexTypeUndo"));
    }

    #[test]
    fn test_insert_vertex_undo() {
        let undo = InsertVertexUndo {
            v_label: 1,
            vid: 100,
        };

        let mut target = MockUndoTarget;
        undo.undo(&mut target, 1).expect("Undo failed");
    }
}
