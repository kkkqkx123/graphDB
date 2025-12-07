use crate::core::{Vertex, Edge, Value, Direction, Tag};
use thiserror::Error;
use sled::{Db, Tree};
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DbError(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Node not found: {0:?}")]
    NodeNotFound(Value),
    #[error("Edge not found: {0:?}")]
    EdgeNotFound(Value),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Storage engine trait defining the interface for graph storage
pub trait StorageEngine {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(&self, src: &Value, dst: &Value, edge_type: &str) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(&self, node_id: &Value, direction: Direction) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(&mut self, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError>;

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
}

/// Transaction identifier
pub type TransactionId = u64;

/// Native storage implementation using sled database
pub struct NativeStorage {
    db: Db,
    nodes_tree: Tree,
    edges_tree: Tree,
    schema_tree: Tree,
    node_edge_index: Tree, // Index: node_id -> [edge_id]
}

impl NativeStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        let nodes_tree = db.open_tree("nodes")?;
        let edges_tree = db.open_tree("edges")?;
        let schema_tree = db.open_tree("schema")?;
        let node_edge_index = db.open_tree("node_edge_index")?;

        Ok(Self {
            db,
            nodes_tree,
            edges_tree,
            schema_tree,
            node_edge_index,
        })
    }

    fn generate_id(&self) -> Value {
        // Simple ID generation - in production, this should be more robust
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;
        Value::Int(id as i64)
    }

    fn value_to_bytes(&self, value: &Value) -> Result<Vec<u8>, StorageError> {
        bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn value_from_bytes(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        let (value, _len) = bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(value)
    }
}

impl StorageEngine for NativeStorage {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        let id = self.generate_id();
        // We create a new vertex with the generated id
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

        let vertex_bytes = bincode::encode_to_vec(&vertex_with_id, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let id_bytes = self.value_to_bytes(&id)?;
        self.nodes_tree.insert(id_bytes, vertex_bytes)?;
        self.db.flush()?;

        Ok(id)
    }

    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = self.value_to_bytes(id)?;
        match self.nodes_tree.get(id_bytes)? {
            Some(vertex_bytes) => {
                let (vertex, _len): (Vertex, usize) = bincode::decode_from_slice(&vertex_bytes, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError> {
        // Check if vertex id is null
        if matches!(*vertex.vid, Value::Null(_)) {
            return Err(StorageError::NodeNotFound(Value::Null(Default::default())));
        }

        let vertex_bytes = bincode::encode_to_vec(&vertex, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let id_bytes = self.value_to_bytes(&vertex.vid)?;
        self.nodes_tree.insert(id_bytes, vertex_bytes)?;
        self.db.flush()?;

        Ok(())
    }

    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError> {
        // First, delete all edges associated with this vertex
        let edges_to_delete = self.get_node_edges(id, Direction::Both)?;
        for edge in edges_to_delete {
            self.delete_edge(&edge.src, &edge.dst, &edge.edge_type)?;
        }

        // Then delete the vertex
        let id_bytes = self.value_to_bytes(id)?;
        self.nodes_tree.remove(&id_bytes)?;

        // Remove from node-edge index
        self.node_edge_index.remove(&id_bytes)?;

        self.db.flush()?;
        Ok(())
    }

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        // For edge key, we'll use a combination of src, dst, and edge_type to make it unique
        let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let edge_bytes = bincode::encode_to_vec(&edge, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Store the edge
        self.edges_tree.insert(&edge_key_bytes, edge_bytes)?;

        // Update indices
        self.update_node_edge_index(&edge.src, &edge_key_bytes, true)?;
        self.update_node_edge_index(&edge.dst, &edge_key_bytes, true)?;

        self.db.flush()?;

        Ok(())
    }

    fn get_edge(&self, src: &Value, dst: &Value, edge_type: &str) -> Result<Option<Edge>, StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        match self.edges_tree.get(&edge_key_bytes)? {
            Some(edge_bytes) => {
                let (edge, _len): (Edge, usize) = bincode::decode_from_slice(&edge_bytes, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn get_node_edges(&self, node_id: &Value, direction: Direction) -> Result<Vec<Edge>, StorageError> {
        let edge_keys = self.get_node_edge_keys(node_id)?;
        let mut edges = Vec::new();

        for edge_key_bytes in edge_keys {
            if let Some(edge_bytes) = self.edges_tree.get(&edge_key_bytes)? {
                let (edge, _len): (Edge, usize) = bincode::decode_from_slice(&edge_bytes, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                match direction {
                    Direction::Out if *edge.src == *node_id => edges.push(edge),
                    Direction::In if *edge.dst == *node_id => edges.push(edge),
                    Direction::Both => edges.push(edge),
                    _ => continue,
                }
            }
        }

        Ok(edges)
    }

    fn delete_edge(&mut self, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        if self.edges_tree.get(&edge_key_bytes)?.is_some() {
            // Remove from edge storage
            self.edges_tree.remove(&edge_key_bytes)?;

            // Update node-edge indices
            self.update_node_edge_index(src, &edge_key_bytes, false)?;
            self.update_node_edge_index(dst, &edge_key_bytes, false)?;

            self.db.flush()?;
            Ok(())
        } else {
            Err(StorageError::EdgeNotFound(Value::String(edge_key)))
        }
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        // TODO: Implement actual transaction support
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;
        Ok(id)
    }

    fn commit_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        // TODO: Implement actual transaction support
        self.db.flush()?;
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        // TODO: Implement actual transaction support
        Ok(())
    }
}

impl NativeStorage {
    fn update_node_edge_index(&self, node_id: &Value, edge_key: &[u8], add: bool) -> Result<(), StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        let mut edge_list = match self.node_edge_index.get(&node_id_bytes)? {
            Some(list_bytes) => {
                let (result, _len): (Vec<Vec<u8>>, usize) = bincode::decode_from_slice(&list_bytes, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                result
            },
            None => Vec::new(),
        };

        if add {
            if !edge_list.contains(&edge_key.to_vec()) {
                edge_list.push(edge_key.to_vec());
            }
        } else {
            edge_list.retain(|key| key != edge_key);
        }

        let list_bytes = bincode::encode_to_vec(&edge_list, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.node_edge_index.insert(&node_id_bytes, list_bytes)?;

        Ok(())
    }

    fn get_node_edge_keys(&self, node_id: &Value) -> Result<Vec<Vec<u8>>, StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        match self.node_edge_index.get(&node_id_bytes)? {
            Some(list_bytes) => {
                let (edge_key_list, _len): (Vec<Vec<u8>>, usize) = bincode::decode_from_slice(&list_bytes, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(edge_key_list)
            }
            None => Ok(Vec::new()),
        }
    }
}