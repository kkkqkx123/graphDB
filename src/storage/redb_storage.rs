use super::{StorageClient, TransactionId};
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::vertex_edge_path::Tag;
use crate::core::types::{
    SpaceInfo, TagInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo, EdgeTypeInfo, UpdateTarget, UpdateOp,
};
use crate::index::Index;
use crate::storage::{FieldDef, DataType, Schema};
use crate::storage::operations::{RedbReader, RedbWriter, ScanResult, VertexReader, EdgeReader, VertexWriter, EdgeWriter};
use crate::storage::metadata::{RedbSchemaManager, SchemaManager};
use crate::storage::index::MemoryIndexManager;
use crate::storage::redb_types::{ByteKey, TAG_INDEXES_TABLE, EDGE_INDEXES_TABLE, INDEX_DATA_TABLE, VERTEX_DATA_TABLE, EDGE_DATA_TABLE, PASSWORDS_TABLE};
use crate::storage::serializer::{serializer_index_to_bytes, serializer_index_from_bytes, value_to_bytes, vertex_to_bytes};
use crate::storage::utils::{property_defs_to_fields, property_defs_to_hashmap};
use redb::{Database, ReadableTable};
use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct RedbStorage {
    db: Arc<Database>,
    db_path: String,
    vertex_reader: Arc<Mutex<RedbReader>>,
    vertex_writer: Arc<Mutex<RedbWriter>>,
    schema_manager: Arc<Mutex<RedbSchemaManager>>,
    index_manager: Arc<Mutex<MemoryIndexManager>>,
    active_transactions: Arc<Mutex<HashMap<TransactionId, ()>>>,
}

impl std::fmt::Debug for RedbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl Clone for RedbStorage {
    fn clone(&self) -> Self {
        Self::new(&self.db_path).expect("Failed to clone RedbStorage")
    }
}

impl RedbStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();

        let db = Arc::new(
            Database::create(path.as_ref())
                .map_err(|e| StorageError::DbError(e.to_string()))?
        );

        let vertex_reader = Arc::new(Mutex::new(RedbReader::new(db.clone())?));
        let vertex_writer = Arc::new(Mutex::new(RedbWriter::new(db.clone())?));
        let schema_manager = Arc::new(Mutex::new(RedbSchemaManager::new(db.clone())));
        let index_manager = Arc::new(Mutex::new(MemoryIndexManager::new(PathBuf::from(&db_path))));
        let active_transactions = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            db: db.clone(),
            db_path,
            vertex_reader,
            vertex_writer,
            schema_manager,
            index_manager,
            active_transactions,
        })
    }
}

impl StorageClient for RedbStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        (*self.vertex_reader.lock().unwrap()).get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).scan_vertices(space)?.into_vec())
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).scan_vertices_by_tag(space, tag)?.into_vec())
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).scan_vertices_by_prop(space, tag, prop, value)?.into_vec())
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        (*self.vertex_reader.lock().unwrap()).get_edge(space, src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).get_node_edges(space, node_id, direction)?.into_vec())
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).get_node_edges_filtered(space, node_id, direction, filter)?.into_vec())
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).scan_edges_by_type(space, edge_type)?.into_vec())
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        Ok((*self.vertex_reader.lock().unwrap()).scan_all_edges(space)?.into_vec())
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        (*self.vertex_writer.lock().unwrap()).insert_vertex(space, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        (*self.vertex_writer.lock().unwrap()).update_vertex(space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        (*self.vertex_writer.lock().unwrap()).delete_vertex(space, id)
    }

    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        (*self.vertex_writer.lock().unwrap()).batch_insert_vertices(space, vertices)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        (*self.vertex_writer.lock().unwrap()).insert_edge(space, edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        (*self.vertex_writer.lock().unwrap()).delete_edge(space, src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        (*self.vertex_writer.lock().unwrap()).batch_insert_edges(space, edges)
    }

    fn begin_transaction(&mut self, space: &str) -> Result<TransactionId, StorageError> {
        let tx_id = TransactionId::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos() as u64,
        );

        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        active_transactions.insert(tx_id, ());

        Ok(tx_id)
    }

    fn commit_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        if !active_transactions.contains_key(&tx_id) {
            return Err(StorageError::TransactionNotFound(tx_id.as_u64()));
        }

        active_transactions.remove(&tx_id);

        Ok(())
    }

    fn rollback_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        if !active_transactions.contains_key(&tx_id) {
            return Err(StorageError::TransactionNotFound(tx_id.as_u64()));
        }

        active_transactions.remove(&tx_id);

        Ok(())
    }

    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).create_space(space)
    }

    fn drop_space(&mut self, space_name: &str) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).drop_space(space_name)
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).get_space(space_name)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).list_spaces()
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).create_tag(space, tag)
    }

    fn alter_tag(&mut self, space: &str, tag: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let tag_info = match (*self.schema_manager.lock().unwrap()).get_tag(space, tag)? {
            Some(t) => t,
            None => return Err(StorageError::NotFound(format!("Tag {} not found in space {}", tag, space))),
        };

        let mut updated_info = tag_info;
        for prop in additions {
            updated_info.properties.retain(|p| p.name != prop.name);
            updated_info.properties.push(prop);
        }

        for prop_name in deletions {
            updated_info.properties.retain(|p| p.name != prop_name);
        }

        (*self.schema_manager.lock().unwrap()).drop_tag(space, tag)?;
        (*self.schema_manager.lock().unwrap()).create_tag(space, &updated_info)?;

        Ok(true)
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).get_tag(space, tag_name)
    }

    fn drop_tag(&mut self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).drop_tag(space, tag_name)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).list_tags(space)
    }

    fn create_edge_type(&mut self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).create_edge_type(space, edge)
    }

    fn alter_edge_type(&mut self, space: &str, edge_type: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let edge_info = match (*self.schema_manager.lock().unwrap()).get_edge_type(space, edge_type)? {
            Some(e) => e,
            None => return Err(StorageError::NotFound(format!("Edge type {} not found in space {}", edge_type, space))),
        };

        let mut updated_info = edge_info;
        for prop in additions {
            updated_info.properties.retain(|p| p.name != prop.name);
            updated_info.properties.push(prop);
        }

        for prop_name in deletions {
            updated_info.properties.retain(|p| p.name != prop_name);
        }

        (*self.schema_manager.lock().unwrap()).drop_edge_type(space, edge_type)?;
        (*self.schema_manager.lock().unwrap()).create_edge_type(space, &updated_info)?;

        Ok(true)
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).get_edge_type(space, edge_type_name)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        (*self.schema_manager.lock().unwrap()).drop_edge_type(space, edge_type_name)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        (*self.schema_manager.lock().unwrap()).list_edge_types(space)
    }

    fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, info.name);
        let index_bytes = serializer_index_to_bytes(info)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAG_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(index_key.as_bytes().to_vec()), ByteKey(index_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(TAG_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let index_bytes = value.value();
                let index_info = serializer_index_from_bytes(&index_bytes.0)?;
                Ok(Some(index_info))
            }
            None => Ok(None),
        }
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut indexes = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (key_bytes, index_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&format!("{}:", space)) {
                let index_info = serializer_index_from_bytes(&index_bytes.value().0)?;
                indexes.push(index_info);
            }
        }

        Ok(indexes)
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let tag_indexes_table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        if tag_indexes_table.get(ByteKey(index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .is_none() {
            return Err(StorageError::NotFound(format!("Tag index {} not found in space {}", index, space)));
        }

        drop(tag_indexes_table);
        drop(read_txn);

        let vertex_reader = self.vertex_reader.lock().unwrap();
        let vertices = vertex_reader.scan_vertices(space)?;

        for vertex in vertices.into_vec() {
            for tag in &vertex.tags {
                if tag.name == index {
                    let value_bytes = vertex_to_bytes(&vertex)?;
                    let index_data_key = format!("{}:{}:{:?}", space, index, vertex.id);

                    let write_txn = self.db.begin_write()
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                    {
                        let mut index_data_table = write_txn.open_table(INDEX_DATA_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        index_data_table.insert(
                            ByteKey(index_data_key.as_bytes().to_vec()),
                            ByteKey(value_bytes)
                        ).map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                    write_txn.commit()
                        .map_err(|e| StorageError::DbError(e.to_string()))?;
                }
            }
        }

        Ok(true)
    }

    fn create_edge_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, info.name);
        let index_bytes = serializer_index_to_bytes(info)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(index_key.as_bytes().to_vec()), ByteKey(index_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(EDGE_INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            if table.get(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Ok(false);
            }

            table.remove(ByteKey(index_key.as_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let index_bytes = value.value();
                let index_info = serializer_index_from_bytes(&index_bytes.0)?;
                Ok(Some(index_info))
            }
            None => Ok(None),
        }
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut indexes = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (key_bytes, index_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);
            if key_str.starts_with(&format!("{}:", space)) {
                let index_info = serializer_index_from_bytes(&index_bytes.value().0)?;
                indexes.push(index_info);
            }
        }

        Ok(indexes)
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let index_key = format!("{}:{}", space, index);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_indexes_table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        if edge_indexes_table.get(ByteKey(index_key.as_bytes().to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .is_none() {
            return Err(StorageError::NotFound(format!("Edge index {} not found in space {}", index, space)));
        }

        drop(edge_indexes_table);
        drop(read_txn);

        let vertex_reader = self.vertex_reader.lock().unwrap();
        let edges = vertex_reader.scan_all_edges(space)?;

        for edge in edges.into_vec() {
            if edge.edge_type == index {
                let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
                let edge_data_key = format!("{}:{}:{}", space, index, edge_key);

                let write_txn = self.db.begin_write()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                {
                    let mut index_data_table = write_txn.open_table(INDEX_DATA_TABLE)
                        .map_err(|e| StorageError::DbError(e.to_string()))?;

                    let edge_bytes = serde_json::to_vec(&edge)
                        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    index_data_table.insert(
                        ByteKey(edge_data_key.as_bytes().to_vec()),
                        ByteKey(edge_bytes)
                    ).map_err(|e| StorageError::DbError(e.to_string()))?;
                }
                write_txn.commit()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }

        Ok(true)
    }

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        let tag_info = match (*self.schema_manager.lock().unwrap()).get_tag(space, &info.tag_name)? {
            Some(t) => t,
            None => return Err(StorageError::NotFound(format!("Tag {} not found in space {}", info.tag_name, space))),
        };

        let tag_properties = property_defs_to_hashmap(&tag_info.properties);
        let vertex = Vertex {
            vid: Box::new(info.vertex_id.clone()),
            id: 0,
            tags: vec![Tag { name: info.tag_name.clone(), properties: tag_properties }],
            properties: info.properties.iter().cloned().collect(),
        };

        let vertex_id = (*self.vertex_writer.lock().unwrap()).insert_vertex(space, vertex)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut vertex_data_table = write_txn.open_table(VERTEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let vertex_data_key = format!("{}:{}:{:?}", space, info.tag_name, vertex_id);
            let vertex_bytes = serde_json::to_vec(&info.properties)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            vertex_data_table.insert(
                ByteKey(vertex_data_key.as_bytes().to_vec()),
                ByteKey(vertex_bytes)
            ).map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        let _edge_type_info = match (*self.schema_manager.lock().unwrap()).get_edge_type(space, &info.edge_name)? {
            Some(e) => e,
            None => return Err(StorageError::NotFound(format!("Edge type {} not found in space {}", info.edge_name, space))),
        };

        let edge = Edge {
            src: Box::new(info.src_vertex_id.clone()),
            dst: Box::new(info.dst_vertex_id.clone()),
            edge_type: info.edge_name.clone(),
            ranking: info.rank,
            id: 0,
            props: info.properties.iter().cloned().collect(),
        };

        (*self.vertex_writer.lock().unwrap()).insert_edge(space, edge)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut edge_data_table = write_txn.open_table(EDGE_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let edge_data_key = format!("{}:{}:{:?}:{:?}:{}", space, info.edge_name, info.src_vertex_id, info.dst_vertex_id, info.rank);
            let edge_bytes = serde_json::to_vec(&info.properties)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            edge_data_table.insert(
                ByteKey(edge_data_key.as_bytes().to_vec()),
                ByteKey(edge_bytes)
            ).map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let vertex_id_value = Value::String(vertex_id.to_string());
        (*self.vertex_writer.lock().unwrap()).delete_vertex(space, &vertex_id_value)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut vertex_data_table = write_txn.open_table(VERTEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let keys_to_remove: Vec<Vec<u8>> = vertex_data_table.iter()
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .filter_map(|result| {
                    let (key_bytes, _) = result.ok()?;
                    let key_data = key_bytes.value().0.clone();
                    let key_str = String::from_utf8_lossy(&key_data);
                    if key_str.starts_with(&format!("{}:", space)) && key_str.contains(&format!(":{:?}:", vertex_id_value)) {
                        Some(key_data)
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                vertex_data_table.remove(ByteKey(key))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        let src_value = Value::String(src.to_string());
        let dst_value = Value::String(dst.to_string());

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_data_table = read_txn.open_table(EDGE_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edge_types: Vec<String> = edge_data_table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
            .filter_map(|result| {
                let (key_bytes, _) = result.ok()?;
                let key_data = key_bytes.value().0.clone();
                let key_str = String::from_utf8_lossy(&key_data);
                if key_str.starts_with(&format!("{}:", space)) &&
                   key_str.contains(&format!(":{:?}:", src_value)) &&
                   key_str.contains(&format!(":{:?}:", dst_value)) &&
                   key_str.ends_with(&format!(":{}", rank)) {
                    let parts: Vec<&str> = key_str.split(':').collect();
                    if parts.len() >= 3 {
                        Some(parts[1].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        drop(edge_data_table);
        drop(read_txn);

        for edge_type in edge_types {
            (*self.vertex_writer.lock().unwrap()).delete_edge(space, &src_value, &dst_value, &edge_type)?;
        }

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut edge_data_table = write_txn.open_table(EDGE_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let keys_to_remove: Vec<Vec<u8>> = edge_data_table.iter()
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .filter_map(|result| {
                    let (key_bytes, _) = result.ok()?;
                    let key_data = key_bytes.value().0.clone();
                    let key_str = String::from_utf8_lossy(&key_data);
                    if key_str.starts_with(&format!("{}:", space)) &&
                       key_str.contains(&format!(":{:?}:", src_value)) &&
                       key_str.contains(&format!(":{:?}:", dst_value)) &&
                       key_str.ends_with(&format!(":{}", rank)) {
                        Some(key_data)
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                edge_data_table.remove(ByteKey(key))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        match &info.target {
            UpdateTarget::Vertex { vertex_id, tag_name: _ } => {
                let vertex_id_value = vertex_id.clone();
                let vertex = match (*self.vertex_reader.lock().unwrap()).get_vertex(space, &vertex_id_value)? {
                    Some(v) => v,
                    None => return Err(StorageError::NotFound(format!("Vertex {:?} not found in space {}", vertex_id, space))),
                };

                let mut updated_props = vertex.properties.clone();
                for op in &info.operations {
                    match op {
                        UpdateOp::Set { property, value } => {
                            updated_props.insert(property.clone(), value.clone());
                        }
                        UpdateOp::Delete { property } => {
                            updated_props.remove(property);
                        }
                    }
                }

                let updated_vertex = Vertex {
                    vid: vertex.vid,
                    id: vertex.id,
                    tags: vertex.tags,
                    properties: updated_props,
                };
                (*self.vertex_writer.lock().unwrap()).update_vertex(space, updated_vertex)?;
            }
            UpdateTarget::Edge { src_vertex_id, dst_vertex_id, rank: _, edge_name } => {
                let src_value = src_vertex_id.clone();
                let dst_value = dst_vertex_id.clone();
                let edge = match (*self.vertex_reader.lock().unwrap()).get_edge(space, &src_value, &dst_value, edge_name)? {
                    Some(e) => e,
                    None => return Err(StorageError::NotFound(format!("Edge from {:?} to {:?} with type {} not found", src_vertex_id, dst_vertex_id, edge_name))),
                };

                let mut updated_props = edge.props.clone();
                for op in &info.operations {
                    match op {
                        UpdateOp::Set { property, value } => {
                            updated_props.insert(property.clone(), value.clone());
                        }
                        UpdateOp::Delete { property } => {
                            updated_props.remove(property);
                        }
                    }
                }

                let updated_edge = Edge {
                    src: edge.src,
                    dst: edge.dst,
                    edge_type: edge.edge_type,
                    ranking: edge.ranking,
                    id: edge.id,
                    props: updated_props,
                };
                (*self.vertex_writer.lock().unwrap()).insert_edge(space, updated_edge)?;
                (*self.vertex_writer.lock().unwrap()).delete_edge(space, &src_value, &dst_value, edge_name)?;
            }
        }

        Ok(true)
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let username_key = info.username.as_bytes();
        let password_bytes = serde_json::to_vec(info)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn.open_table(PASSWORDS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            match table.get(ByteKey(username_key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(existing) => {
                    let existing_info: PasswordInfo = serde_json::from_slice(&existing.value().0)
                        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    if existing_info.old_password != info.old_password {
                        return Err(StorageError::InvalidInput("Old password does not match".to_string()));
                    }
                }
                None => {
                    return Err(StorageError::NotFound(format!("User {} not found", info.username)));
                }
            }

            table.insert(ByteKey(username_key.to_vec()), ByteKey(password_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(true)
    }

    fn lookup_index(&self, space: &str, index: &str, value: &Value) -> Result<Vec<Value>, StorageError> {
        let _index_key = format!("{}:{}:{:?}", space, index, value);

        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let index_data_table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut results = Vec::new();
        for result in index_data_table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (key_bytes, value_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_data = key_bytes.value().0.clone();
            let key_str = String::from_utf8_lossy(&key_data);

            if key_str.starts_with(&format!("{}:{}:", space, index)) {
                if key_str.ends_with(&format!(":{:?}", value)) || key_str.contains(&format!(":{:?}:", value)) {
                    let vertex_data: Vec<(String, Value)> = serde_json::from_slice(&value_bytes.value().0)
                        .map_err(|e| StorageError::SerializeError(e.to_string()))?;

                    for (_, prop_value) in vertex_data {
                        if prop_value == *value {
                            results.push(prop_value);
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let vertex = (*self.vertex_reader.lock().unwrap()).get_vertex(space, id)?;

        match vertex {
            Some(v) => {
                let tag_info = match (*self.schema_manager.lock().unwrap()).get_tag(space, tag)? {
                    Some(t) => t,
                    None => return Err(StorageError::NotFound(format!("Tag {} not found in space {}", tag, space))),
                };

                let fields = property_defs_to_fields(&tag_info.properties);
                let schema = Schema {
                    name: tag.to_string(),
                    fields,
                    version: tag_info.tag_id as i32,
                };

                let vertex_bytes = serde_json::to_vec(&v)
                    .map_err(|e| StorageError::SerializeError(e.to_string()))?;

                Ok(Some((schema, vertex_bytes)))
            }
            None => Ok(None),
        }
    }

    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let edge = (*self.vertex_reader.lock().unwrap()).get_edge(space, src, dst, edge_type)?;

        match edge {
            Some(e) => {
                let edge_type_info = match (*self.schema_manager.lock().unwrap()).get_edge_type(space, edge_type)? {
                    Some(et) => et,
                    None => return Err(StorageError::NotFound(format!("Edge type {} not found in space {}", edge_type, space))),
                };

                let fields = property_defs_to_fields(&edge_type_info.properties);
                let schema = Schema {
                    name: edge_type.to_string(),
                    fields,
                    version: edge_type_info.edge_type_id as i32,
                };

                let edge_bytes = serde_json::to_vec(&e)
                    .map_err(|e| StorageError::SerializeError(e.to_string()))?;

                Ok(Some((schema, edge_bytes)))
            }
            None => Ok(None),
        }
    }

    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let tag_info = match (*self.schema_manager.lock().unwrap()).get_tag(space, tag)? {
            Some(t) => t,
            None => return Err(StorageError::NotFound(format!("Tag {} not found in space {}", tag, space))),
        };

        let fields = property_defs_to_fields(&tag_info.properties);
        let schema = Schema {
            name: tag.to_string(),
            fields,
            version: tag_info.tag_id as i32,
        };

        let vertices = (*self.vertex_reader.lock().unwrap()).scan_vertices_by_tag(space, tag)?;

        let mut results = Vec::new();
        for vertex in vertices.into_vec() {
            let vertex_bytes = serde_json::to_vec(&vertex)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            results.push((schema.clone(), vertex_bytes));
        }

        Ok(results)
    }

    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let edge_type_info = match (*self.schema_manager.lock().unwrap()).get_edge_type(space, edge_type)? {
            Some(et) => et,
            None => return Err(StorageError::NotFound(format!("Edge type {} not found in space {}", edge_type, space))),
        };

        let fields = property_defs_to_fields(&edge_type_info.properties);
        let schema = Schema {
            name: edge_type.to_string(),
            fields,
            version: edge_type_info.edge_type_id as i32,
        };

        let edges = (*self.vertex_reader.lock().unwrap()).scan_edges_by_type(space, edge_type)?;

        let mut results = Vec::new();
        for edge in edges.into_vec() {
            let edge_bytes = serde_json::to_vec(&edge)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            results.push((schema.clone(), edge_bytes));
        }

        Ok(results)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _tag_indexes_table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _edge_indexes_table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _index_data_table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _vertex_data_table = read_txn.open_table(VERTEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _edge_data_table = read_txn.open_table(EDGE_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _passwords_table = read_txn.open_table(PASSWORDS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _tag_indexes_table = read_txn.open_table(TAG_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _edge_indexes_table = read_txn.open_table(EDGE_INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _index_data_table = read_txn.open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _vertex_data_table = read_txn.open_table(VERTEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _edge_data_table = read_txn.open_table(EDGE_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let _passwords_table = read_txn.open_table(PASSWORDS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        log::info!("Successfully saved to disk");

        Ok(())
    }
}
