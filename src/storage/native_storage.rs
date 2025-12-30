use super::{StorageEngine, TransactionId};
use crate::core::{Direction, Edge, StorageError, Value, Vertex};
use serde_json;
use sled::{Db, Tree};
use std::time::{SystemTime, UNIX_EPOCH};

/// Native storage implementation using sled database
#[derive(Debug)]
pub struct NativeStorage {
    db: Db,
    nodes_tree: Tree,
    edges_tree: Tree,

    schema_tree: Tree,
    node_edge_index: Tree, // Index: node_id -> [edge_id]
    edge_type_index: Tree, // Index: edge_type -> [edge_key]
    db_path: String,       // Store the path for cloning
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
        let db = sled::open(&db_path).map_err(|e| StorageError::DbError(e.to_string()))?;
        let nodes_tree = db
            .open_tree("nodes")
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let edges_tree = db
            .open_tree("edges")
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let schema_tree = db
            .open_tree("schema")
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let node_edge_index = db
            .open_tree("node_edge_index")
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_type_index = db
            .open_tree("edge_type_index")
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(Self {
            db,
            nodes_tree,
            edges_tree,
            schema_tree,
            node_edge_index,
            edge_type_index,
            db_path,
        })
    }

    fn sled_error_to_storage_error(e: sled::Error) -> StorageError {
        StorageError::DbError(e.to_string())
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
        serde_json::to_vec(value).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn value_from_bytes(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        serde_json::from_slice(bytes).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn update_node_edge_index(
        &self,
        node_id: &Value,
        edge_key: &[u8],
        add: bool,
    ) -> Result<(), StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        let mut edge_list = match self
            .node_edge_index
            .get(&node_id_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
            Some(list_bytes) => {
                let result: Vec<Vec<u8>> = serde_json::from_slice(&list_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                result
            }
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

        self.node_edge_index
            .insert(&node_id_bytes, list_bytes)
            .map_err(Self::sled_error_to_storage_error)?;

        Ok(())
    }

    fn get_node_edge_keys(&self, node_id: &Value) -> Result<Vec<Vec<u8>>, StorageError> {
        let node_id_bytes = self.value_to_bytes(node_id)?;
        match self
            .node_edge_index
            .get(&node_id_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
            Some(list_bytes) => {
                let edge_key_list: Vec<Vec<u8>> = serde_json::from_slice(&list_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(edge_key_list)
            }
            None => Ok(Vec::new()),
        }
    }

    fn update_edge_type_index(
        &self,
        edge_type: &str,
        edge_key: &[u8],
        add: bool,
    ) -> Result<(), StorageError> {
        let edge_type_bytes = edge_type.as_bytes().to_vec();
        let mut edge_list = match self
            .edge_type_index
            .get(&edge_type_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
            Some(list_bytes) => {
                let result: Vec<Vec<u8>> = serde_json::from_slice(&list_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                result
            }
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

        self.edge_type_index
            .insert(&edge_type_bytes, list_bytes)
            .map_err(Self::sled_error_to_storage_error)?;

        Ok(())
    }

    fn get_edge_keys_by_type(&self, edge_type: &str) -> Result<Vec<Vec<u8>>, StorageError> {
        let edge_type_bytes = edge_type.as_bytes().to_vec();
        match self
            .edge_type_index
            .get(&edge_type_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
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
        self.nodes_tree
            .insert(id_bytes, vertex_bytes)
            .map_err(Self::sled_error_to_storage_error)?;
        self.db.flush().map_err(Self::sled_error_to_storage_error)?;

        Ok(id)
    }

    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = self.value_to_bytes(id)?;
        match self
            .nodes_tree
            .get(id_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
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
        self.nodes_tree
            .insert(id_bytes, vertex_bytes)
            .map_err(Self::sled_error_to_storage_error)?;
        self.db.flush().map_err(Self::sled_error_to_storage_error)?;

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
        self.nodes_tree
            .remove(&id_bytes)
            .map_err(Self::sled_error_to_storage_error)?;

        // 从节点边索引中删除
        self.node_edge_index
            .remove(&id_bytes)
            .map_err(Self::sled_error_to_storage_error)?;

        self.db.flush().map_err(Self::sled_error_to_storage_error)?;
        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
        let mut vertices = Vec::new();

        // 遍历nodes_tree中的所有顶点
        for item in self.nodes_tree.iter() {
            let (_, vertex_bytes) = item.map_err(Self::sled_error_to_storage_error)?;
            let vertex: Vertex = serde_json::from_slice(&vertex_bytes)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            vertices.push(vertex);
        }

        Ok(vertices)
    }

    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let all_vertices = self.scan_all_vertices()?;
        let filtered_vertices = all_vertices
            .into_iter()
            .filter(|vertex| vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag))
            .collect();

        Ok(filtered_vertices)
    }

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        // 为边键使用src、dst和edge_type的组合以使其唯一
        let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let edge_bytes = serde_json::to_vec(&edge)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // 存储边
        self.edges_tree
            .insert(&edge_key_bytes, edge_bytes)
            .map_err(Self::sled_error_to_storage_error)?;

        // 更新索引
        self.update_node_edge_index(&edge.src, &edge_key_bytes, true)?;
        self.update_node_edge_index(&edge.dst, &edge_key_bytes, true)?;
        self.update_edge_type_index(&edge.edge_type, &edge_key_bytes, true)?;

        self.db.flush().map_err(Self::sled_error_to_storage_error)?;

        Ok(())
    }

    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        match self
            .edges_tree
            .get(&edge_key_bytes)
            .map_err(Self::sled_error_to_storage_error)?
        {
            Some(edge_bytes) => {
                let edge: Edge = serde_json::from_slice(&edge_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: Direction,
    ) -> Result<Vec<Edge>, StorageError> {
        let edge_keys = self.get_node_edge_keys(node_id)?;
        let mut edges = Vec::new();

        for edge_key_bytes in edge_keys {
            if let Some(edge_bytes) = self
                .edges_tree
                .get(&edge_key_bytes)
                .map_err(Self::sled_error_to_storage_error)?
            {
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

    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        if self
            .edges_tree
            .get(&edge_key_bytes)
            .map_err(Self::sled_error_to_storage_error)?
            .is_some()
        {
            self.edges_tree
                .remove(&edge_key_bytes)
                .map_err(Self::sled_error_to_storage_error)?;

            self.update_node_edge_index(src, &edge_key_bytes, false)?;
            self.update_node_edge_index(dst, &edge_key_bytes, false)?;
            self.update_edge_type_index(edge_type, &edge_key_bytes, false)?;

            self.db.flush().map_err(Self::sled_error_to_storage_error)?;
            Ok(())
        } else {
            Err(StorageError::EdgeNotFound(Value::String(edge_key)))
        }
    }

    fn scan_edges_by_type(&self, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        let mut edges = Vec::new();

        let edge_keys = self.get_edge_keys_by_type(edge_type)?;
        for edge_key_bytes in edge_keys {
            if let Some(edge_bytes) = self
                .edges_tree
                .get(&edge_key_bytes)
                .map_err(Self::sled_error_to_storage_error)?
            {
                let edge: Edge = serde_json::from_slice(&edge_bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError> {
        let mut edges = Vec::new();

        for item in self.edges_tree.iter() {
            let (_, edge_bytes) = item.map_err(Self::sled_error_to_storage_error)?;
            let edge: Edge = serde_json::from_slice(&edge_bytes)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            edges.push(edge);
        }

        Ok(edges)
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
        self.db.flush().map_err(Self::sled_error_to_storage_error)?;
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        // TODO: 实现实际的事务支持
        Ok(())
    }
}
