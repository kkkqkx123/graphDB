use crate::core::{Node, Edge, Value, Direction};
use thiserror::Error;
use sled::{Db, Tree};
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DbError(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(u64),
    #[error("Edge not found: {0}")]
    EdgeNotFound(u64),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Storage engine trait defining the interface for graph storage
pub trait StorageEngine {
    fn insert_node(&mut self, node: Node) -> Result<u64, StorageError>;
    fn get_node(&self, id: u64) -> Result<Option<Node>, StorageError>;
    fn update_node(&mut self, node: Node) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: u64) -> Result<(), StorageError>;
    
    fn insert_edge(&mut self, edge: Edge) -> Result<u64, StorageError>;
    fn get_edge(&self, id: u64) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(&self, node_id: u64, direction: Direction) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(&mut self, id: u64) -> Result<(), StorageError>;

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
    
    fn generate_id(&self) -> u64 {
        // Simple ID generation - in production, this should be more robust
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64
    }
}

impl StorageEngine for NativeStorage {
    fn insert_node(&mut self, mut node: Node) -> Result<u64, StorageError> {
        let id = self.generate_id();
        node.id = id;
        
        let node_bytes = bincode::serialize(&node)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.nodes_tree.insert(id.to_be_bytes(), node_bytes)?;
        self.db.flush()?;
        
        Ok(id)
    }
    
    fn get_node(&self, id: u64) -> Result<Option<Node>, StorageError> {
        match self.nodes_tree.get(id.to_be_bytes())? {
            Some(node_bytes) => {
                let node: Node = bincode::deserialize(&node_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }
    
    fn update_node(&mut self, node: Node) -> Result<(), StorageError> {
        if node.id == 0 {
            return Err(StorageError::NodeNotFound(0));
        }
        
        let node_bytes = bincode::serialize(&node)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.nodes_tree.insert(node.id.to_be_bytes(), node_bytes)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    fn delete_node(&mut self, id: u64) -> Result<(), StorageError> {
        // First, delete all edges associated with this node
        let edges_to_delete = self.get_node_edges(id, Direction::Both)?;
        for edge in edges_to_delete {
            self.delete_edge(edge.id)?;
        }

        // Then delete the node
        self.nodes_tree.remove(id.to_be_bytes())?;

        // Remove from node-edge index
        self.node_edge_index.remove(id.to_be_bytes())?;

        self.db.flush()?;
        Ok(())
    }
    
    fn insert_edge(&mut self, mut edge: Edge) -> Result<u64, StorageError> {
        let id = self.generate_id();
        edge.id = id;
        
        let edge_bytes = bincode::serialize(&edge)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store the edge
        self.edges_tree.insert(id.to_be_bytes(), edge_bytes)?;
        
        // Update indices
        self.update_node_edge_index(edge.from_node, id, true)?;
        self.update_node_edge_index(edge.to_node, id, true)?;
        
        self.db.flush()?;
        
        Ok(id)
    }
    
    fn get_edge(&self, id: u64) -> Result<Option<Edge>, StorageError> {
        match self.edges_tree.get(id.to_be_bytes())? {
            Some(edge_bytes) => {
                let edge: Edge = bincode::deserialize(&edge_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }
    
    fn get_node_edges(&self, node_id: u64, direction: Direction) -> Result<Vec<Edge>, StorageError> {
        let edge_ids = self.get_node_edge_ids(node_id, &direction)?;
        let mut edges = Vec::new();
        
        for edge_id in edge_ids {
            if let Some(edge) = self.get_edge(edge_id)? {
                match direction {
                    Direction::Out if edge.from_node == node_id => edges.push(edge),
                    Direction::In if edge.to_node == node_id => edges.push(edge),
                    Direction::Both => edges.push(edge),
                    _ => continue,
                }
            }
        }
        
        Ok(edges)
    }

    fn delete_edge(&mut self, id: u64) -> Result<(), StorageError> {
        if let Some(edge) = self.get_edge(id)? {
            // Remove from edge storage
            self.edges_tree.remove(id.to_be_bytes())?;

            // Update node-edge indices
            self.update_node_edge_index(edge.from_node, id, false)?;
            self.update_node_edge_index(edge.to_node, id, false)?;

            self.db.flush()?;
            Ok(())
        } else {
            Err(StorageError::EdgeNotFound(id))
        }
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        // TODO: Implement actual transaction support
        Ok(self.generate_id())
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
    fn update_node_edge_index(&self, node_id: u64, edge_id: u64, add: bool) -> Result<(), StorageError> {
        let key = node_id.to_be_bytes();
        let mut edge_list = match self.node_edge_index.get(&key)? {
            Some(list_bytes) => bincode::deserialize::<Vec<u64>>(&list_bytes)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?,
            None => Vec::new(),
        };
        
        if add {
            edge_list.push(edge_id);
        } else {
            edge_list.retain(|&id| id != edge_id);
        }
        
        let list_bytes = bincode::serialize(&edge_list)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.node_edge_index.insert(&key, list_bytes)?;
        
        Ok(())
    }
    
    fn get_node_edge_ids(&self, node_id: u64, _direction: &Direction) -> Result<Vec<u64>, StorageError> {
        let key = node_id.to_be_bytes();
        match self.node_edge_index.get(&key)? {
            Some(list_bytes) => {
                let edge_list: Vec<u64> = bincode::deserialize(&list_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(edge_list)
            }
            None => Ok(Vec::new()),
        }
    }
}