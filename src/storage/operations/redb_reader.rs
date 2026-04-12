use crate::core::{Edge, EdgeDirection, StorageError, Value, Vertex};
use crate::storage::engine::{ByteKey, EDGES_TABLE, NODES_TABLE};
use crate::storage::operations::{EdgeReader, ScanResult, VertexReader};
use crate::transaction::TransactionContext;
use bincode::{config::standard, decode_from_slice, encode_to_vec};
use lru::LruCache;
use parking_lot::Mutex;
use redb::{Database, ReadableTable};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedbReader {
    db: Arc<Database>,
    vertex_cache: Arc<Mutex<LruCache<Vec<u8>, Vertex>>>,
    edge_cache: Arc<Mutex<LruCache<Vec<u8>, Edge>>>,
    txn_context: Option<Arc<TransactionContext>>,
}

impl std::fmt::Debug for RedbReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbReader")
            .field("has_txn_context", &self.txn_context.is_some())
            .finish()
    }
}

impl RedbReader {
    pub fn new(db: Arc<Database>) -> Result<Self, StorageError> {
        let vertex_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for vertex cache");
        let edge_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for edge cache");
        let vertex_cache = Arc::new(Mutex::new(LruCache::new(vertex_cache_size)));
        let edge_cache = Arc::new(Mutex::new(LruCache::new(edge_cache_size)));

        Ok(Self {
            db,
            vertex_cache,
            edge_cache,
            txn_context: None,
        })
    }

    pub fn set_transaction_context(&mut self, context: Option<Arc<TransactionContext>>) {
        self.txn_context = context;
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.txn_context.clone()
    }

    pub fn invalidate_vertex_cache(&self, id: &Value) {
        if let Ok(id_bytes) = encode_to_vec(id, standard()) {
            let mut cache = self.vertex_cache.lock();
            cache.pop(&id_bytes);
        }
    }

    pub fn invalidate_edge_cache(&self, src: &Value, dst: &Value, edge_type: &str) {
        if let Ok(key) = encode_to_vec(
            &(src.clone(), dst.clone(), edge_type.to_string()),
            standard(),
        ) {
            let mut cache = self.edge_cache.lock();
            cache.pop(&key);
        }
    }

    fn get_node_from_bytes(&self, id_bytes: &[u8]) -> Result<Option<Vertex>, StorageError> {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        match table
                            .get(ByteKey(id_bytes.to_vec()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            Some(value) => {
                                let byte_key_value = value.value();
                                let vertex_bytes = byte_key_value.0.clone();
                                let vertex: Vertex =
                                    decode_from_slice(&vertex_bytes, standard())?.0;
                                Ok(Some(vertex))
                            }
                            None => Ok(None),
                        }
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let result = table
                            .get(ByteKey(id_bytes.to_vec()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        match result {
                            Some(value) => {
                                let byte_key_value = value.value();
                                let vertex_bytes = byte_key_value.0.clone();
                                let vertex: Vertex =
                                    decode_from_slice(&vertex_bytes, standard())?.0;
                                Ok(Some(vertex))
                            }
                            None => Ok(None),
                        }
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(NODES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let result = table
            .get(ByteKey(id_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match result {
            Some(value) => {
                let byte_key_value = value.value();
                let vertex_bytes = byte_key_value.0.clone();
                let vertex: Vertex = decode_from_slice(&vertex_bytes, standard())?.0;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn get_edge_from_bytes(&self, edge_key_bytes: &[u8]) -> Result<Option<Edge>, StorageError> {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let result = table
                            .get(ByteKey(edge_key_bytes.to_vec()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        match result {
                            Some(value) => {
                                let byte_key_value = value.value();
                                let edge_bytes = byte_key_value.0.clone();
                                let edge: Edge = decode_from_slice(&edge_bytes, standard())?.0;
                                Ok(Some(edge))
                            }
                            None => Ok(None),
                        }
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let result = table
                            .get(ByteKey(edge_key_bytes.to_vec()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        match result {
                            Some(value) => {
                                let byte_key_value = value.value();
                                let edge_bytes = byte_key_value.0.clone();
                                let edge: Edge = decode_from_slice(&edge_bytes, standard())?.0;
                                Ok(Some(edge))
                            }
                            None => Ok(None),
                        }
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let result = table
            .get(ByteKey(edge_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match result {
            Some(value) => {
                let byte_key_value = value.value();
                let edge_bytes = byte_key_value.0.clone();
                let edge: Edge = decode_from_slice(&edge_bytes, standard())?.0;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }
}

impl VertexReader for RedbReader {
    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = encode_to_vec(id, standard())?;

        {
            let mut cache = self.vertex_cache.lock();
            if let Some(vertex) = cache.get(&id_bytes) {
                return Ok(Some(vertex.clone()));
            }
        }

        match self.get_node_from_bytes(&id_bytes)? {
            Some(vertex) => {
                {
                    let mut cache = self.vertex_cache.lock();
                    cache.put(id_bytes.clone(), vertex.clone());
                }
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn scan_vertices(&self, _space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut vertices = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, vertex_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let vertex: Vertex =
                                decode_from_slice(&vertex_bytes.value().0, standard())?.0;
                            vertices.push(vertex);
                        }

                        Ok(ScanResult::new(vertices))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut vertices = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, vertex_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let vertex: Vertex =
                                decode_from_slice(&vertex_bytes.value().0, standard())?.0;
                            vertices.push(vertex);
                        }

                        Ok(ScanResult::new(vertices))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(NODES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut vertices = Vec::new();
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, vertex_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let vertex: Vertex = decode_from_slice(&vertex_bytes.value().0, standard())?.0;
            vertices.push(vertex);
        }

        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_tag(
        &self,
        _space: &str,
        tag: &str,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = self.scan_vertices(_space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|vertex| vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag))
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = self.scan_vertices(_space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|vertex| {
                vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag)
                    && (vertex.properties.get(prop) == Some(value))
            })
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }
}

impl EdgeReader for RedbReader {
    fn get_edge(
        &self,
        _space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        let edge_key = format!("{:?}_{:?}_{}_{}", src, dst, edge_type, rank);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        {
            let mut cache = self.edge_cache.lock();
            if let Some(edge) = cache.get(&edge_key_bytes) {
                return Ok(Some(edge.clone()));
            }
        }

        match self.get_edge_from_bytes(&edge_key_bytes)? {
            Some(edge) => {
                {
                    let mut cache = self.edge_cache.lock();
                    cache.put(edge_key_bytes.clone(), edge.clone());
                }
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn get_node_edges(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        self.get_node_edges_filtered(_space, node_id, direction, None::<fn(&Edge) -> bool>)
    }

    fn get_node_edges_filtered<F>(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<ScanResult<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;

                            let matches_direction = match direction {
                                EdgeDirection::Out => *edge.src == *node_id,
                                EdgeDirection::In => *edge.dst == *node_id,
                                EdgeDirection::Both => {
                                    *edge.src == *node_id || *edge.dst == *node_id
                                }
                            };

                            if matches_direction {
                                if let Some(ref f) = filter {
                                    if !f(&edge) {
                                        continue;
                                    }
                                }
                                edges.push(edge);
                            }
                        }

                        Ok(ScanResult::new(edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;

                            let matches_direction = match direction {
                                EdgeDirection::Out => *edge.src == *node_id,
                                EdgeDirection::In => *edge.dst == *node_id,
                                EdgeDirection::Both => {
                                    *edge.src == *node_id || *edge.dst == *node_id
                                }
                            };

                            if matches_direction {
                                if let Some(ref f) = filter {
                                    if !f(&edge) {
                                        continue;
                                    }
                                }
                                edges.push(edge);
                            }
                        }

                        Ok(ScanResult::new(edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, edge_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let edge: Edge = decode_from_slice(&edge_bytes.value().0, standard())?.0;

            let matches_direction = match direction {
                EdgeDirection::Out => *edge.src == *node_id,
                EdgeDirection::In => *edge.dst == *node_id,
                EdgeDirection::Both => *edge.src == *node_id || *edge.dst == *node_id,
            };

            if matches_direction {
                if let Some(ref f) = filter {
                    if !f(&edge) {
                        continue;
                    }
                }
                edges.push(edge);
            }
        }

        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;
                            edges.push(edge);
                        }

                        let filtered_edges: Vec<Edge> = edges
                            .into_iter()
                            .filter(|e| e.edge_type == edge_type)
                            .collect();

                        Ok(ScanResult::new(filtered_edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;
                            edges.push(edge);
                        }

                        let filtered_edges: Vec<Edge> = edges
                            .into_iter()
                            .filter(|e| e.edge_type == edge_type)
                            .collect();

                        Ok(ScanResult::new(filtered_edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, edge_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let edge: Edge = decode_from_slice(&edge_bytes.value().0, standard())?.0;
            edges.push(edge);
        }

        let filtered_edges: Vec<Edge> = edges
            .into_iter()
            .filter(|e| e.edge_type == edge_type)
            .collect();

        Ok(ScanResult::new(filtered_edges))
    }

    fn scan_all_edges(&self, _space: &str) -> Result<ScanResult<Edge>, StorageError> {
        if let Some(ref ctx) = self.txn_context {
            if ctx.read_only {
                return ctx
                    .with_read_txn(|read_txn| {
                        let table = read_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;
                            edges.push(edge);
                        }

                        Ok(ScanResult::new(edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            } else {
                return ctx
                    .with_write_txn(|write_txn| {
                        let table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;

                        let mut edges = Vec::new();
                        for result in table
                            .iter()
                            .map_err(|e| StorageError::DbError(e.to_string()))?
                        {
                            let (_, edge_bytes) =
                                result.map_err(|e| StorageError::DbError(e.to_string()))?;
                            let edge: Edge =
                                decode_from_slice(&edge_bytes.value().0, standard())?.0;
                            edges.push(edge);
                        }

                        Ok(ScanResult::new(edges))
                    })
                    .map_err(|e| StorageError::DbError(e.to_string()));
            }
        }

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, edge_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let edge: Edge = decode_from_slice(&edge_bytes.value().0, standard())?.0;
            edges.push(edge);
        }

        Ok(ScanResult::new(edges))
    }
}
