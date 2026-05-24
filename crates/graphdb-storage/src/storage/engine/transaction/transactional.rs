//! Transactional Support
//!
//! Provides utilities for undo-log-based rollback and atomic write operations.
//! Merged from previous transaction_support.rs and transactional_writer.rs.

use crate::core::types::{LabelId, Timestamp};
use crate::core::{StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::PropertyGraph;
use crate::core::metadata::SchemaManager;
use crate::transaction::undo_log::UndoLogManager;

use super::super::graph_storage::type_utils::vertex_id_to_string;

/// Execute an operation with automatic rollback on failure
pub fn with_rollback<T, F>(
    graph: &PropertyGraph,
    undo_logs: &mut UndoLogManager,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce() -> StorageResult<T>,
{
    let result = operation();

    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            if !undo_logs.is_empty() {
                log::warn!("Operation failed, attempting rollback: {}", e);
                if let Err(rollback_err) = undo_logs.execute_undo(graph, ts) {
                    log::error!("Rollback also failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

/// Execute an operation in a transaction context
pub fn execute_in_transaction<T, F>(
    graph: &PropertyGraph,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce() -> StorageResult<T>,
{
    let mut undo_logs = UndoLogManager::new();

    let result = operation();

    match result {
        Ok(value) => {
            undo_logs.clear();
            Ok(value)
        }
        Err(e) => {
            if !undo_logs.is_empty() {
                log::warn!("Transaction failed, rolling back: {}", e);
                if let Err(rollback_err) = undo_logs.execute_undo(graph, ts) {
                    log::error!("Rollback failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

/// Transaction writer for atomic vertex insert operations
///
/// Provides atomic insert operations with manual rollback on failure.
/// Unlike `execute_in_transaction`, this uses PropertyGraph methods directly
/// (which do not record undo logs), so rollback is done by calling delete
/// on previously inserted items.
pub struct TransactionWriter<'a> {
    graph: &'a PropertyGraph,
    schema_manager: &'a SchemaManager,
}

impl<'a> TransactionWriter<'a> {
    pub fn new(graph: &'a PropertyGraph, schema_manager: &'a SchemaManager) -> Self {
        Self {
            graph,
            schema_manager,
        }
    }

    pub fn insert_vertex_transactional(
        &self,
        space: &str,
        vertex: &Vertex,
        ts: Timestamp,
    ) -> StorageResult<Value> {
        let _space_info = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let mut inserted_ids: Vec<(LabelId, String)> = Vec::new();

        for tag in &vertex.tags {
            if let Some(label_id) = self.graph.get_vertex_label_id(&tag.name) {
                let id_str = vertex_id_to_string(&vertex.vid);
                let props: Vec<(String, Value)> = tag
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                match self.graph.insert_vertex(label_id, &id_str, &props, ts) {
                    Ok(_) => {
                        inserted_ids.push((label_id, id_str));
                    }
                    Err(e) => {
                        for (rollback_label, rollback_id) in inserted_ids.iter().rev() {
                            let _ = self
                                .graph
                                .delete_vertex(*rollback_label, rollback_id, ts);
                        }
                        return Err(e);
                    }
                }
            }
        }

        Ok(Value::from(vertex.vid))
    }

    pub fn batch_insert_vertices_transactional(
        &self,
        space: &str,
        vertices: &[Vertex],
        ts: Timestamp,
    ) -> StorageResult<Vec<Value>> {
        let _space_info = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let mut inserted_ids: Vec<Value> = Vec::with_capacity(vertices.len());
        let mut rollback_info: Vec<(LabelId, String)> = Vec::new();

        for vertex in vertices {
            let mut vertex_inserted = false;
            for tag in &vertex.tags {
                if let Some(label_id) = self.graph.get_vertex_label_id(&tag.name) {
                    let id_str = vertex_id_to_string(&vertex.vid);
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    match self.graph.insert_vertex(label_id, &id_str, &props, ts) {
                        Ok(_) => {
                            if !vertex_inserted {
                                rollback_info.push((label_id, id_str));
                                vertex_inserted = true;
                            }
                        }
                        Err(e) => {
                            for (rollback_label, rollback_id) in rollback_info.iter().rev() {
                                let _ =
                                    self.graph
                                        .delete_vertex(*rollback_label, rollback_id, ts);
                            }
                            return Err(e);
                        }
                    }
                }
            }
            if vertex_inserted {
                inserted_ids.push(Value::from(vertex.vid));
            }
        }

        Ok(inserted_ids)
    }

    pub fn execute_transactional<F, T>(
        &self,
        ts: Timestamp,
        operation: F,
    ) -> StorageResult<T>
    where
        F: FnOnce(&PropertyGraph, Timestamp) -> StorageResult<T>,
    {
        operation(self.graph, ts)
    }
}
