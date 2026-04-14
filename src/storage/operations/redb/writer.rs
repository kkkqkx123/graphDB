use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::engine::{ByteKey, EDGES_TABLE, NODES_TABLE};
use crate::storage::operations::traits::{EdgeWriter, VertexWriter};
use crate::transaction::types::OperationLog;
use crate::transaction::TransactionContext;
use crate::utils::id_gen::generate_id;
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use redb::{Database, ReadableTable};
use std::sync::Arc;

use crate::storage::operations::write_txn_executor::WriteTxnExecutor;

pub struct RedbWriter {
    db: Arc<Database>,
    txn_context: Option<Arc<TransactionContext>>,
}

impl Clone for RedbWriter {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            txn_context: self.txn_context.clone(),
        }
    }
}

impl std::fmt::Debug for RedbWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbWriter")
            .field("has_bound_context", &self.txn_context.is_some())
            .finish()
    }
}

impl RedbWriter {
    pub fn new(db: Arc<Database>) -> Result<Self, StorageError> {
        Ok(Self {
            db,
            txn_context: None,
        })
    }

    pub fn set_transaction_context(&mut self, context: Arc<TransactionContext>) {
        self.txn_context = Some(context);
    }

    pub fn clear_transaction_context(&mut self) {
        self.txn_context = None;
    }

    pub fn has_transaction_context(&self) -> bool {
        self.txn_context.is_some()
    }

    fn get_executor(&self) -> WriteTxnExecutor<'_> {
        match &self.txn_context {
            Some(ctx) => WriteTxnExecutor::bound(ctx.clone()),
            None => WriteTxnExecutor::independent(&self.db),
        }
    }

    fn log_operation(&self, operation: OperationLog) {
        if let Some(ctx) = &self.txn_context {
            ctx.add_operation_log(operation);
        }
    }

    fn record_table_modification(&self, table_name: &str) {
        if let Some(ctx) = &self.txn_context {
            ctx.record_table_modification(table_name);
        }
    }
}

impl RedbWriter {
    fn insert_vertex_internal(&self, vertex: Vertex) -> Result<Value, StorageError> {
        let id = match vertex.vid() {
            Value::Int(0) | Value::Null(_) => Value::Int(generate_id() as i64),
            _ => vertex.vid().clone(),
        };
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

        let vertex_bytes = encode_to_vec(&vertex_with_id, standard())?;
        let id_bytes = encode_to_vec(&id, standard())?;

        let previous_state = self.get_vertex_bytes(&id)?;
        if previous_state.is_some() {
            return Err(StorageError::AlreadyExists(format!(
                "Vertex with ID {} already exists",
                id
            )));
        }

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(())
        })?;

        self.log_operation(OperationLog::InsertVertex {
            space: "default".to_string(),
            vertex_id: encode_to_vec(&id, standard())?,
            previous_state,
        });

        self.record_table_modification("NODES_TABLE");

        Ok(id)
    }

    fn get_vertex_bytes(&self, id: &Value) -> Result<Option<Vec<u8>>, StorageError> {
        let id_bytes = encode_to_vec(id, standard())?;

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let result = table
                .get(ByteKey(id_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            match result {
                Some(value) => Ok(Some(value.value().0)),
                None => Ok(None),
            }
        })
    }

    fn get_edge_bytes(&self, edge_key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let result = table
                .get(ByteKey(edge_key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            match result {
                Some(value) => Ok(Some(value.value().0)),
                None => Ok(None),
            }
        })
    }

    fn update_vertex_internal(&self, vertex: Vertex) -> Result<(), StorageError> {
        if matches!(*vertex.vid, Value::Null(_)) {
            return Err(StorageError::NodeNotFound(Value::Null(Default::default())));
        }

        let vertex_bytes = encode_to_vec(&vertex, standard())?;
        let id_bytes = encode_to_vec(&vertex.vid, standard())?;

        let previous_data = self
            .get_vertex_bytes(&vertex.vid)?
            .ok_or_else(|| StorageError::NodeNotFound((*vertex.vid).clone()))?;

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(())
        })?;

        self.log_operation(OperationLog::UpdateVertex {
            space: "default".to_string(),
            vertex_id: encode_to_vec(&vertex.vid, standard())?,
            previous_data,
        });

        self.record_table_modification("NODES_TABLE");

        Ok(())
    }

    fn delete_vertex_internal(&self, id: &Value) -> Result<(), StorageError> {
        let id_bytes = encode_to_vec(id, standard())?;

        let deleted_data = self
            .get_vertex_bytes(id)?
            .ok_or_else(|| StorageError::NodeNotFound(id.clone()))?;

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .remove(ByteKey(id_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(())
        })?;

        self.log_operation(OperationLog::DeleteVertex {
            space: "default".to_string(),
            vertex_id: encode_to_vec(id, standard())?,
            vertex: deleted_data,
        });

        self.record_table_modification("NODES_TABLE");

        Ok(())
    }

    fn batch_insert_vertices_internal(
        &self,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::new();
        let mut previous_states = Vec::new();
        let mut operation_logs = Vec::new();

        for vertex in &vertices {
            let id = match vertex.vid() {
                Value::Int(0) | Value::Null(_) => Value::Int(generate_id() as i64),
                _ => vertex.vid().clone(),
            };
            let previous_state = self.get_vertex_bytes(&id)?;
            previous_states.push((id, previous_state));
        }

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for (i, vertex) in vertices.into_iter().enumerate() {
                let id = previous_states[i].0.clone();
                let vertex_with_id = Vertex::new(id.clone(), vertex.tags);
                let vertex_bytes = encode_to_vec(&vertex_with_id, standard())?;
                let id_bytes = encode_to_vec(&id, standard())?;

                table
                    .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                ids.push(id);
            }

            Ok(())
        })?;

        // After the data operation was successful, all operation logs were recorded.
        for (i, id) in ids.iter().enumerate() {
            let log = OperationLog::InsertVertex {
                space: "default".to_string(),
                vertex_id: encode_to_vec(id, standard())?,
                previous_state: previous_states[i].1.clone(),
            };
            operation_logs.push(log);
        }

        // Batch logging of operation logs (ensuring atomicity)
        if let Some(ctx) = &self.txn_context {
            ctx.add_operation_logs(operation_logs);
        }

        self.record_table_modification("NODES_TABLE");

        Ok(ids)
    }

    fn delete_tags_internal(
        &self,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let id_bytes = encode_to_vec(vertex_id, standard())?;
        let tag_names = tag_names.to_vec();

        let previous_data = self
            .get_vertex_bytes(vertex_id)?
            .ok_or_else(|| StorageError::NodeNotFound(vertex_id.clone()))?;

        let executor = self.get_executor();
        let deleted_count = executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let vertex: Vertex = match table
                .get(ByteKey(id_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let vertex_bytes = value.value();
                    decode_from_slice(&vertex_bytes.0, standard())?.0
                }
                None => return Err(StorageError::NodeNotFound(vertex_id.clone())),
            };

            let original_tag_count = vertex.tags.len();
            let remaining_tags: Vec<_> = vertex
                .tags
                .into_iter()
                .filter(|tag| !tag_names.contains(&tag.name))
                .collect();

            let deleted_count = original_tag_count - remaining_tags.len();

            let updated_vertex = Vertex::new(vertex_id.clone(), remaining_tags);
            let vertex_bytes = encode_to_vec(&updated_vertex, standard())?;

            table
                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(deleted_count)
        })?;

        self.log_operation(OperationLog::UpdateVertex {
            space: "default".to_string(),
            vertex_id: encode_to_vec(vertex_id, standard())?,
            previous_data,
        });

        self.record_table_modification("NODES_TABLE");

        Ok(deleted_count)
    }
}

impl VertexWriter for RedbWriter {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let _ = space;
        self.insert_vertex_internal(vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let _ = space;
        self.update_vertex_internal(vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let _ = space;
        self.delete_vertex_internal(id)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let _ = space;
        self.batch_insert_vertices_internal(vertices)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let _ = space;
        self.delete_tags_internal(vertex_id, tag_names)
    }
}

impl RedbWriter {
    fn insert_edge_internal(&self, edge: Edge) -> Result<(), StorageError> {
        let edge_key = format!(
            "{:?}_{:?}_{}_{}",
            edge.src, edge.dst, edge.edge_type, edge.ranking
        );
        let edge_key_bytes = edge_key.as_bytes().to_vec();
        let edge_bytes = encode_to_vec(&edge, standard())?;

        let previous_state = self.get_edge_bytes(&edge_key_bytes)?;

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(edge_key_bytes.clone()), ByteKey(edge_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(())
        })?;

        self.log_operation(OperationLog::InsertEdge {
            space: "default".to_string(),
            edge_id: edge_key_bytes,
            previous_state,
        });

        self.record_table_modification("EDGES_TABLE");

        Ok(())
    }

    fn delete_edge_internal(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}_{}", src, dst, edge_type, rank);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let deleted_data = self
            .get_edge_bytes(&edge_key_bytes)?
            .ok_or_else(|| StorageError::EdgeNotFound(Value::String(edge_key.clone())))?;

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .remove(ByteKey(edge_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            Ok(())
        })?;

        self.log_operation(OperationLog::DeleteEdge {
            space: "default".to_string(),
            edge_id: edge_key_bytes,
            edge: deleted_data,
        });

        self.record_table_modification("EDGES_TABLE");

        Ok(())
    }

    fn batch_insert_edges_internal(&self, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut edge_keys = Vec::new();
        let mut previous_states = Vec::new();
        let mut operation_logs = Vec::new();

        for edge in &edges {
            let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
            let edge_key_bytes = edge_key.as_bytes().to_vec();
            let previous_state = self.get_edge_bytes(&edge_key_bytes)?;
            edge_keys.push(edge_key_bytes);
            previous_states.push(previous_state);
        }

        let executor = self.get_executor();
        executor.execute(|write_txn| {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for (i, edge) in edges.into_iter().enumerate() {
                let edge_bytes = encode_to_vec(&edge, standard())?;

                table
                    .insert(ByteKey(edge_keys[i].clone()), ByteKey(edge_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }

            Ok(())
        })?;

        // After the data operation was successful, all operation logs were recorded.
        for (i, edge_key) in edge_keys.iter().enumerate() {
            let log = OperationLog::InsertEdge {
                space: "default".to_string(),
                edge_id: edge_key.clone(),
                previous_state: previous_states[i].clone(),
            };
            operation_logs.push(log);
        }

        // Batch logging of operation logs (ensuring atomicity)
        if let Some(ctx) = &self.txn_context {
            ctx.add_operation_logs(operation_logs);
        }

        self.record_table_modification("EDGES_TABLE");

        Ok(())
    }
}

impl EdgeWriter for RedbWriter {
    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let _ = space;
        self.insert_edge_internal(edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let _ = space;
        self.delete_edge_internal(src, dst, edge_type, rank)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let _ = space;
        self.batch_insert_edges_internal(edges)
    }
}
