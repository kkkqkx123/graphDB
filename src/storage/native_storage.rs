use crate::core::{Vertex, Edge, Value, Direction};
use super::{StorageError, StorageEngine, TransactionId};
use sled::{Db, Tree};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Native storage implementation using sled database
#[derive(Debug)]
pub struct NativeStorage {
    db: Db,
    nodes_tree: Tree,
    edges_tree: Tree,
    #[allow(dead_code)]
    schema_tree: Tree,
    node_edge_index: Tree, // Index: node_id -> [edge_id]
    db_path: String, // Store the path for cloning
}

impl Clone for NativeStorage {
    fn clone(&self) -> Self {
        // Note: This creates a new connection to the same database
        // In a real implementation, you might want to use a connection pool instead
        // For testing, we'll create a unique path to avoid locking issues
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let unique_path = format!("{}_test_{}", self.get_db_path(), timestamp);
        Self::new(unique_path).expect("Failed to clone NativeStorage")
    }
}

impl NativeStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();
        let db = sled::open(&db_path)?;
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
            db_path,
        })
    }

    // Get the database path for cloning
    fn get_db_path(&self) -> &str {
        &self.db_path
    }

    fn generate_id(&self) -> Value {
        // Simple ID generation - in production, this should be more robust
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;
        Value::Int(id as i64)
    }

    fn value_to_bytes(&self, value: &Value) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    #[allow(dead_code)]
    fn value_from_bytes(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        serde_json::from_slice(bytes)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn update_node_edge_index(&self, node_id: &Value, edge_key: &[u8], add: bool) -> Result<(), StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        let mut edge_list = match self.node_edge_index.get(&node_id_bytes)? {
            Some(list_bytes) => {
                let result: Vec<Vec<u8>> = serde_json::from_slice(&list_bytes)
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

        let list_bytes = serde_json::to_vec(&edge_list)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.node_edge_index.insert(&node_id_bytes, list_bytes)?;

        Ok(())
    }

    fn get_node_edge_keys(&self, node_id: &Value) -> Result<Vec<Vec<u8>>, StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        match self.node_edge_index.get(&node_id_bytes)? {
            Some(list_bytes) => {
                let edge_key_list: Vec<Vec<u8>> = serde_json::from_slice(&list_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(edge_key_list)
            }
            None => Ok(Vec::new()),
        }
    }
}

impl StorageEngine for NativeStorage {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        let id = self.generate_id();
        // 使用生成的id创建新顶点
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

        let vertex_bytes = serde_json::to_vec(&vertex_with_id)
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
                let vertex: Vertex = serde_json::from_slice(&vertex_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError> {
        // 检查顶点id是否为null
        if matches!(*vertex.vid, Value::Null(_)) {
            return Err(StorageError::NodeNotFound(Value::Null(Default::default())));
        }

        let vertex_bytes = serde_json::to_vec(&vertex)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let id_bytes = self.value_to_bytes(&vertex.vid)?;
        self.nodes_tree.insert(id_bytes, vertex_bytes)?;
        self.db.flush()?;

        Ok(())
    }

    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError> {
        // 首先删除与此顶点关联的所有边
        let edges_to_delete = self.get_node_edges(id, Direction::Both)?;
        for edge in edges_to_delete {
            self.delete_edge(&edge.src, &edge.dst, &edge.edge_type)?;
        }

        // 然后删除顶点
        let id_bytes = self.value_to_bytes(id)?;
        self.nodes_tree.remove(&id_bytes)?;

        // 从节点边索引中删除
        self.node_edge_index.remove(&id_bytes)?;

        self.db.flush()?;
        Ok(())
    }

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        // 为边键使用src、dst和edge_type的组合以使其唯一
        let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let edge_bytes = serde_json::to_vec(&edge)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // 存储边
        self.edges_tree.insert(&edge_key_bytes, edge_bytes)?;

        // 更新索引
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
                let edge: Edge = serde_json::from_slice(&edge_bytes)
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
                let edge: Edge = serde_json::from_slice(&edge_bytes)
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
            // 从边存储中删除
            self.edges_tree.remove(&edge_key_bytes)?;

            // 更新节点边索引
            self.update_node_edge_index(src, &edge_key_bytes, false)?;
            self.update_node_edge_index(dst, &edge_key_bytes, false)?;

            self.db.flush()?;
            Ok(())
        } else {
            Err(StorageError::EdgeNotFound(Value::String(edge_key)))
        }
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        // TODO: 实现实际的事务支持
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;
        Ok(id)
    }

    fn commit_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        // TODO: 实现实际的事务支持
        self.db.flush()?;
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        // TODO: 实现实际的事务支持
        Ok(())
    }
}