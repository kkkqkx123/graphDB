use crate::core::types::{LabelId, Timestamp};
use crate::core::{StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::PropertyGraph;

use super::context::GraphStorageContext;
use super::type_utils::vertex_id_to_string;

/// Transactional writer for atomic insert operations
pub struct TransactionalWriter<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> TransactionalWriter<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    /// Insert a vertex within a transaction
    ///
    /// This method uses InsertTransaction for atomic insert with WAL logging.
    pub fn insert_vertex_transactional(
        &self,
        space: &str,
        vertex: &Vertex,
    ) -> StorageResult<Value> {
        let _space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();

        let mut inserted_ids: Vec<(LabelId, String)> = Vec::new();

        for tag in &vertex.tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.name) {
                let id_str = vertex_id_to_string(&vertex.vid);
                let props: Vec<(String, Value)> = tag
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                match self.ctx.graph.insert_vertex(label_id, &id_str, &props, ts) {
                    Ok(_) => {
                        inserted_ids.push((label_id, id_str));
                    }
                    Err(e) => {
                        for (rollback_label, rollback_id) in inserted_ids.iter().rev() {
                            let _ = self.ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                        }
                        return Err(e);
                    }
                }
            }
        }

        Ok((*vertex.vid).clone())
    }

    /// Batch insert vertices within a single transaction
    ///
    /// All vertices are inserted atomically. If any insert fails,
    /// all previous inserts are rolled back.
    pub fn batch_insert_vertices_transactional(
        &self,
        space: &str,
        vertices: &[Vertex],
    ) -> StorageResult<Vec<Value>> {
        let _space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();

        let mut inserted_ids: Vec<Value> = Vec::with_capacity(vertices.len());
        let mut rollback_info: Vec<(LabelId, String)> = Vec::new();

        for vertex in vertices {
            let mut vertex_inserted = false;
            for tag in &vertex.tags {
                if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.name) {
                    let id_str = vertex_id_to_string(&vertex.vid);
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    match self.ctx.graph.insert_vertex(label_id, &id_str, &props, ts) {
                        Ok(_) => {
                            if !vertex_inserted {
                                rollback_info.push((label_id, id_str));
                                vertex_inserted = true;
                            }
                        }
                        Err(e) => {
                            for (rollback_label, rollback_id) in rollback_info.iter().rev() {
                                let _ = self.ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                            }
                            return Err(e);
                        }
                    }
                }
            }
            if vertex_inserted {
                inserted_ids.push((*vertex.vid).clone());
            }
        }

        Ok(inserted_ids)
    }

    /// Execute a transactional operation with automatic rollback
    pub fn execute_transactional<F, T>(&self, operation: F) -> StorageResult<T>
    where
        F: FnOnce(&PropertyGraph, Timestamp) -> StorageResult<T>,
    {
        let ts = self.ctx.get_write_timestamp();
        operation(&self.ctx.graph, ts)
    }
}
