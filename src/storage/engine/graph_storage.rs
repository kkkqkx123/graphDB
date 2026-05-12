//! Storage Interface Implementation
//!
//! Implements the StorageClient trait for PropertyGraph storage.
//! This module acts as an adapter layer between the high-level StorageClient API
//! and the low-level PropertyGraph storage engine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, UpdateInfo, UpdateOp, UpdateTarget, UserAlterInfo, UserInfo,
};
use crate::core::vertex_edge_path::Tag;
use crate::core::{
    Edge, EdgeDirection, RoleType, StorageError, StorageResult, Value, Vertex,
};
use crate::storage::interface::{StorageClient, StorageStats};
use crate::storage::metadata::{
    InMemoryExtendedSchemaManager, InMemoryIndexMetadataManager, InMemorySchemaManager,
    IndexMetadataManager, SchemaManager, ExtendedSchemaManager, Schema,
};
use crate::storage::engine::PropertyGraph;
use crate::storage::engine::property_graph::InsertEdgeParams;
use crate::api::server::auth::UserStorage;
use crate::storage::index::secondary::{IndexDataManager, InMemoryIndexDataManager};
use crate::storage::vertex::{LabelId, Timestamp, VertexRecord};
use crate::storage::edge::EdgeRecord;
use crate::transaction::context::TransactionContext;
use crate::transaction::version_manager::VersionManager;

#[derive(Clone)]
pub struct GraphStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    schema_manager: Arc<InMemorySchemaManager>,
    extended_schema_manager: Arc<InMemoryExtendedSchemaManager>,
    index_metadata_manager: Arc<InMemoryIndexMetadataManager>,
    index_data_manager: Arc<RwLock<InMemoryIndexDataManager>>,
    version_manager: Arc<VersionManager>,
    user_storage: Arc<UserStorage>,
    current_txn_context: Arc<Mutex<Option<Arc<TransactionContext>>>>,
    work_dir: Option<PathBuf>,
    db_path: String,
}

impl std::fmt::Debug for GraphStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorage")
            .field("work_dir", &self.work_dir)
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl GraphStorage {
    pub fn new() -> StorageResult<Self> {
        let graph = Arc::new(RwLock::new(PropertyGraph::new()));
        let schema_manager = Arc::new(InMemorySchemaManager::new());
        let extended_schema_manager = Arc::new(InMemoryExtendedSchemaManager::new());
        let index_metadata_manager = Arc::new(InMemoryIndexMetadataManager::new());
        let index_data_manager = Arc::new(RwLock::new(InMemoryIndexDataManager::new()));
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        Ok(Self {
            graph,
            schema_manager,
            extended_schema_manager,
            index_metadata_manager,
            index_data_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(Mutex::new(None)),
            work_dir: None,
            db_path: String::new(),
        })
    }

    pub fn new_with_path(path: PathBuf) -> StorageResult<Self> {
        let graph = Arc::new(RwLock::new(PropertyGraph::new()));
        let schema_manager = Arc::new(InMemorySchemaManager::new());
        let extended_schema_manager = Arc::new(InMemoryExtendedSchemaManager::new());
        let index_metadata_manager = Arc::new(InMemoryIndexMetadataManager::new());
        let index_data_manager = Arc::new(RwLock::new(InMemoryIndexDataManager::new()));
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        Ok(Self {
            graph,
            schema_manager,
            extended_schema_manager,
            index_metadata_manager,
            index_data_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(Mutex::new(None)),
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
        })
    }

    pub fn get_db(&self) -> Arc<RwLock<PropertyGraph>> {
        self.graph.clone()
    }

    pub fn get_schema_manager(&self) -> Arc<InMemorySchemaManager> {
        self.schema_manager.clone()
    }

    pub fn get_extended_schema_manager(&self) -> Arc<InMemoryExtendedSchemaManager> {
        self.extended_schema_manager.clone()
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        *self.current_txn_context.lock() = context;
    }

    pub fn compact_all(&self, ts: Timestamp) -> StorageResult<()> {
        let mut graph = self.graph.write();

        let label_ids = graph.vertex_label_ids();

        for label_id in label_ids {
            let removed = graph.compact_vertex_table_with_ts(label_id, ts);
            if !removed.is_empty() {
                log::info!(
                    "Compacted label {}: removed {} vertices",
                    label_id,
                    removed.len()
                );
            }
        }

        drop(graph);

        let index_manager = self.index_data_manager.read();
        let stats = index_manager.gc_tombstones(ts)?;
        if stats.total_removed() > 0 {
            log::info!(
                "Index GC: removed {} vertex entries, {} edge entries",
                stats.vertex_entries_removed,
                stats.edge_entries_removed
            );
        }

        Ok(())
    }

    fn get_read_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.get_transaction_context() {
            txn_ctx.timestamp()
        } else {
            self.version_manager.read_timestamp()
        }
    }

    fn get_write_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.get_transaction_context() {
            txn_ctx.timestamp()
        } else {
            self.version_manager.write_timestamp()
        }
    }

    fn value_to_string(id: &Value) -> String {
        match id {
            Value::String(s) => s.clone(),
            _ => id.to_string().unwrap_or_default(),
        }
    }

    fn vertex_record_to_vertex(record: &VertexRecord, tag_name: &str) -> Vertex {
        let vid_value = Value::String(record.vid.to_string());
        let properties: HashMap<String, Value> = record.properties.iter().cloned().collect();
        
        Vertex {
            vid: Box::new(vid_value),
            id: record.internal_id as i64,
            tags: vec![Tag {
                name: tag_name.to_string(),
                properties: properties.clone(),
            }],
            properties,
        }
    }

    fn edge_record_to_edge(record: &EdgeRecord, edge_type: &str, src_id: &str, dst_id: &str) -> Edge {
        let props: HashMap<String, Value> = record.properties.iter().cloned().collect();
        
        Edge {
            src: Box::new(Value::String(src_id.to_string())),
            dst: Box::new(Value::String(dst_id.to_string())),
            edge_type: edge_type.to_string(),
            ranking: 0,
            id: record.edge_id as i64,
            props,
        }
    }
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create GraphStorage")
    }
}

impl StorageClient for GraphStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let _space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;
        
        let tags = self.list_tags(space)?;
        if tags.is_empty() {
            return Ok(None);
        }

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let id_str = Self::value_to_string(id);

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(record) = graph.get_vertex(label_id, &id_str, ts) {
                    let vertex = Self::vertex_record_to_vertex(&record, &tag.tag_name);
                    return Ok(Some(vertex));
                }
            }
        }

        Ok(None)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        let tags = self.list_tags(space)?;
        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut vertices = Vec::new();

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(iterator) = graph.scan_vertices(label_id, ts) {
                    for record in iterator {
                        let vertex = Self::vertex_record_to_vertex(&record, &tag.tag_name);
                        vertices.push(vertex);
                    }
                }
            }
        }

        Ok(vertices)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found in space {}", tag, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
            for record in iterator {
                let vertex = Self::vertex_record_to_vertex(&record, tag);
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found in space {}", tag, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
            for record in iterator {
                if record.properties.iter().any(|(k, v)| k == prop && v == value) {
                    let vertex = Self::vertex_record_to_vertex(&record, tag);
                    vertices.push(vertex);
                }
            }
        }

        Ok(vertices)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        _rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        let edge_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        
        let src_str = Self::value_to_string(src);
        let dst_str = Self::value_to_string(dst);

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(record) = graph.get_edge(
                    edge_label_id,
                    src_label_id,
                    &src_str,
                    dst_label_id,
                    &dst_str,
                    ts,
                ) {
                    let edge = Self::edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
                    return Ok(Some(edge));
                }
            }
        }

        Ok(None)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        let edge_types = self.list_edge_types(space)?;
        if edge_types.is_empty() {
            return Ok(Vec::new());
        }

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let node_str = Self::value_to_string(node_id);
        let mut edges = Vec::new();

        for edge_info in &edge_types {
            let edge_label_id = edge_info.edge_type_id;
            let edge_type_name = &edge_info.edge_type_name;
            
            if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
                if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                    match direction {
                        EdgeDirection::Out => {
                            if let Some(out_edges) = graph.out_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in out_edges {
                                    let edge = Self::edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &format!("{}", record.dst_vid),
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::In => {
                            if let Some(in_edges) = graph.in_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in in_edges {
                                    let edge = Self::edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &format!("{}", record.src_vid),
                                        &node_str,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::Both => {
                            if let Some(out_edges) = graph.out_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in out_edges {
                                    let edge = Self::edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &format!("{}", record.dst_vid),
                                    );
                                    edges.push(edge);
                                }
                            }
                            if let Some(in_edges) = graph.in_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in in_edges {
                                    let edge = Self::edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &format!("{}", record.src_vid),
                                        &node_str,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(edges)
    }

    fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        let edges = self.get_node_edges(space, node_id, direction)?;
        match filter {
            Some(f) => Ok(edges.into_iter().filter(f).collect()),
            None => Ok(edges),
        }
    }

    fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<Edge>, StorageError> {
        let edge_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut edges = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(table) = graph.get_edge_table(src_label_id, dst_label_id, edge_label_id) {
                    for record in table.scan(ts) {
                        let edge = Self::edge_record_to_edge(
                            &record,
                            edge_type,
                            &format!("{}", record.src_vid),
                            &format!("{}", record.dst_vid),
                        );
                        edges.push(edge);
                    }
                }
            }
        }

        Ok(edges)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let _space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let mut edges = Vec::new();
        let edge_types = self.list_edge_types(space)?;
        
        for et in edge_types {
            let type_edges = self.scan_edges_by_type(space, &et.edge_type_name)?;
            edges.extend(type_edges);
        }

        Ok(edges)
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        let mut inserted_tags: Vec<(LabelId, String)> = Vec::new();

        for tag in &vertex.tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.name) {
                let id_str = Self::value_to_string(&vertex.vid);
                let props: Vec<(String, Value)> = tag.properties.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if graph.insert_vertex(label_id, &id_str, &props, ts).is_err() {
                    for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                        let _ = graph.delete_vertex(*rollback_label, rollback_id, ts);
                    }
                    return Err(StorageError::vertex_already_exists(id_str));
                }

                if let Err(e) = self.update_vertex_indexes(space_info.space_id, &vertex.vid, &tag.name, &props, ts) {
                    for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                        let _ = graph.delete_vertex(*rollback_label, rollback_id, ts);
                    }
                    let _ = graph.delete_vertex(label_id, &id_str, ts);
                    return Err(e);
                }

                inserted_tags.push((label_id, id_str));
            }
        }

        Ok(*vertex.vid.clone())
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();
        let id_str = Self::value_to_string(&vertex.vid);

        for tag in &vertex.tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.name) {
                for (prop_name, value) in &tag.properties {
                    graph.update_vertex_property(label_id, &id_str, prop_name, value, ts)?;
                }

                let props: Vec<(String, Value)> = tag.properties.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                self.update_vertex_indexes(space_info.space_id, &vertex.vid, &tag.name, &props, ts)?;
            }
        }

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let tags = self.list_tags(space)?;
        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();
        let id_str = Self::value_to_string(id);

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                let _ = graph.delete_vertex(label_id, &id_str, ts);
                
                self.delete_vertex_indexes(space_info.space_id, id, &tag.tag_name)?;
            }
        }

        Ok(())
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let edges = self.get_node_edges(space, id, EdgeDirection::Both)?;
        
        for edge in edges {
            let _ = self.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type, edge.ranking);
        }

        self.delete_vertex(space, id)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            let id = self.insert_vertex(space, vertex)?;
            ids.push(id);
        }
        Ok(ids)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        if let Some(edge_label_id) = graph.get_edge_label_id(&edge.edge_type) {
            let edge_types = self.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == edge.edge_type {
                    if let Some(src_label_id) = graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = graph.get_vertex_label_id(&et.dst_tag_name) {
                            let src_str = Self::value_to_string(&edge.src);
                            let dst_str = Self::value_to_string(&edge.dst);
                            let props: Vec<(String, Value)> = edge.props.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();

                            graph.insert_edge(InsertEdgeParams {
                                edge_label: edge_label_id,
                                src_label: src_label_id,
                                src_id: &src_str,
                                dst_label: dst_label_id,
                                dst_id: &dst_str,
                                properties: &props,
                                ts,
                            })?;

                            self.update_edge_indexes(
                                space_info.space_id,
                                &edge.src,
                                &edge.dst,
                                &edge.edge_type,
                                &props,
                                ts,
                            )?;
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        _rank: i64,
    ) -> Result<(), StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        if let Some(edge_label_id) = graph.get_edge_label_id(edge_type) {
            let edge_types = self.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == edge_type {
                    if let Some(src_label_id) = graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = graph.get_vertex_label_id(&et.dst_tag_name) {
                            let src_str = Self::value_to_string(src);
                            let dst_str = Self::value_to_string(dst);

                            graph.delete_edge(
                                edge_label_id,
                                src_label_id,
                                &src_str,
                                dst_label_id,
                                &dst_str,
                                ts,
                            )?;

                            self.delete_edge_indexes(space_info.space_id, src, dst, edge_type)?;
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        for edge in edges {
            self.insert_edge(space, edge)?;
        }
        Ok(())
    }

    fn create_space(&mut self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        self.schema_manager.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        let tags = self.list_tags(space)?;
        let edge_types = self.list_edge_types(space)?;
        
        let mut graph = self.graph.write();
        for tag in tags {
            let _ = graph.drop_vertex_type(&tag.tag_name);
        }
        for et in edge_types {
            let _ = graph.drop_edge_type(&et.edge_type_name);
        }

        self.schema_manager.drop_space(space)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        self.schema_manager.get_space(space)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        self.schema_manager.get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        self.schema_manager.list_spaces()
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        self.schema_manager.get_space_id(space)
    }

    fn space_exists(&self, space: &str) -> bool {
        self.schema_manager
            .get_space(space)
            .ok()
            .flatten()
            .is_some()
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        let tags = self.list_tags(space)?;
        let edge_types = self.list_edge_types(space)?;
        
        {
            let mut graph = self.graph.write();
            for tag in tags {
                let _ = graph.drop_vertex_type(&tag.tag_name);
            }
            for et in edge_types {
                let _ = graph.drop_edge_type(&et.edge_type_name);
            }
        }

        self.schema_manager.clear_space(space)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        self.schema_manager.alter_space_comment(space_id, comment)
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<u32, StorageError> {
        let tag_id = self.schema_manager.create_tag(space, tag)?;

        let properties: Vec<crate::storage::vertex::PropertyDef> = 
            tag.properties.iter().map(|p| p.into()).collect();

        let primary_key = tag.properties.first()
            .map(|p| p.name.as_str())
            .unwrap_or("id");

        let mut graph = self.graph.write();
        graph.create_vertex_type_with_id(&tag.tag_name, tag_id, properties, primary_key)?;

        Ok(tag_id)
    }

    fn drop_tag(&mut self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        {
            let mut graph = self.graph.write();
            let _ = graph.drop_vertex_type(tag_name);
        }

        self.schema_manager.drop_tag(space, tag_name)
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        self.schema_manager.get_tag(space, tag_name)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        self.schema_manager.list_tags(space)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.schema_manager
            .alter_tag(space, tag_name, additions, deletions)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<u32, StorageError> {
        let edge_type_id = self.schema_manager.create_edge_type(space, edge_type)?;

        let mut graph = self.graph.write();

        let src_label_id = graph.get_vertex_label_id(&edge_type.src_tag_name)
            .ok_or_else(|| StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name)))?;
        let dst_label_id = graph.get_vertex_label_id(&edge_type.dst_tag_name)
            .ok_or_else(|| StorageError::not_found(format!("Destination tag {} not found", edge_type.dst_tag_name)))?;

        let properties: Vec<crate::storage::edge::PropertyDef> = 
            edge_type.properties.iter().map(|p| p.into()).collect();

        use crate::storage::edge::EdgeStrategy;
        graph.create_edge_type_with_id(
            &edge_type.edge_type_name,
            edge_type_id,
            src_label_id,
            dst_label_id,
            properties,
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )?;

        Ok(edge_type_id)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        {
            let mut graph = self.graph.write();
            let _ = graph.drop_edge_type(edge_type_name);
        }

        self.schema_manager.drop_edge_type(space, edge_type_name)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type_name: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        self.schema_manager.get_edge_type(space, edge_type_name)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        self.schema_manager.list_edge_types(space)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.schema_manager
            .alter_edge_type(space, edge_type_name, additions, deletions)
    }

    fn create_tag_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
            .space_id;
        self.index_metadata_manager
            .create_tag_index(space_id, index)?;
        Ok(true)
    }

    fn drop_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.drop_tag_index(space_id, index_name)
    }

    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.get_tag_index(space_id, index_name)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.list_tag_indexes(space_id)
    }

    fn create_edge_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
            .space_id;
        self.index_metadata_manager
            .create_edge_index(space_id, index)?;
        Ok(true)
    }

    fn drop_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.drop_edge_index(space_id, index_name)
    }

    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.get_edge_index(space_id, index_name)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        self.index_metadata_manager.list_edge_indexes(space_id)
    }

    fn get_schema_manager(&self) -> Option<Arc<InMemorySchemaManager>> {
        Some(self.schema_manager.clone())
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        None
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        self.user_storage.create_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        self.user_storage.drop_user(username)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        self.user_storage.alter_user(info)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: RoleType,
    ) -> Result<bool, StorageError> {
        self.user_storage.grant_role(username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        self.user_storage.revoke_role(username, space_id)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();
        let mut deleted_count = 0;

        let id_str = Self::value_to_string(vertex_id);

        for tag_name in tag_names {
            if let Some(label_id) = graph.get_vertex_label_id(tag_name) {
                if graph.delete_vertex(label_id, &id_str, ts).is_ok() {
                    self.delete_vertex_indexes(space_info.space_id, vertex_id, tag_name)?;
                    deleted_count += 1;
                }
            }
        }

        Ok(deleted_count)
    }

    fn rebuild_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        let index = self.index_metadata_manager.get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let vertices = self.scan_vertices_by_tag(space, &index.schema_name)?;
        
        let ts = self.get_write_timestamp();
        let index_data_manager = self.index_data_manager.read();
        for vertex in vertices {
            let props: Vec<(String, Value)> = vertex.properties.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            index_data_manager.update_vertex_indexes_mvcc(
                space_id,
                &vertex.vid,
                &index.name,
                &props,
                ts,
            )?;
        }

        Ok(true)
    }

    fn rebuild_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        let index = self.index_metadata_manager.get_edge_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let edges = self.scan_edges_by_type(space, &index.schema_name)?;
        
        let ts = self.get_write_timestamp();
        let index_data_manager = self.index_data_manager.read();
        for edge in edges {
            let props: Vec<(String, Value)> = edge.props.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            index_data_manager.update_edge_indexes_mvcc(
                space_id,
                &edge.src,
                &edge.dst,
                &index.name,
                &props,
                ts,
            )?;
        }

        Ok(true)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let _tag = self.get_tag(space, &info.tag_name)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found", info.tag_name)))?;

        if info.space_id != space_info.space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        if let Some(label_id) = graph.get_vertex_label_id(&info.tag_name) {
            let id_str = Self::value_to_string(&info.vertex_id);

            let result = graph.insert_vertex(label_id, &id_str, &info.props, ts);
            match result {
                Ok(_) => {
                    self.update_vertex_indexes(space_info.space_id, &info.vertex_id, &info.tag_name, &info.props, ts)?;
                    Ok(true)
                }
                Err(ref e) if e.kind() == crate::core::error::storage::StorageErrorKind::VertexAlreadyExists => Ok(false),
                Err(e) => Err(e),
            }
        } else {
            Err(StorageError::not_found(format!("Tag {} not found in graph", info.tag_name)))
        }
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let _edge_type = self.get_edge_type(space, &info.edge_name)?
            .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found", info.edge_name)))?;

        if info.space_id != space_info.space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        if let Some(edge_label_id) = graph.get_edge_label_id(&info.edge_name) {
            let src_id = Self::value_to_string(&info.src_vertex_id);
            let dst_id = Self::value_to_string(&info.dst_vertex_id);

            let edge_types = self.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == info.edge_name {
                    if let Some(src_label_id) = graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = graph.get_vertex_label_id(&et.dst_tag_name) {
                            let result = graph.insert_edge(
                                InsertEdgeParams {
                                    edge_label: edge_label_id,
                                    src_label: src_label_id,
                                    src_id: &src_id,
                                    dst_label: dst_label_id,
                                    dst_id: &dst_id,
                                    properties: &info.props,
                                    ts,
                                },
                            );
                            match result {
                                Ok(_) => {
                                    self.update_edge_indexes(
                                        space_info.space_id,
                                        &info.src_vertex_id,
                                        &info.dst_vertex_id,
                                        &info.edge_name,
                                        &info.props,
                                        ts,
                                    )?;
                                    return Ok(true);
                                }
                                Err(ref e) if e.kind() == crate::core::error::storage::StorageErrorKind::EdgeAlreadyExists => return Ok(false),
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
        }

        Err(StorageError::not_found(format!("Edge type {} not found in graph", info.edge_name)))
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let tags = self.list_tags(space)?;
        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();
        let mut deleted = false;

        for tag in tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if graph.delete_vertex(label_id, vertex_id, ts).is_ok() {
                    self.delete_vertex_indexes(space_info.space_id, &Value::String(vertex_id.to_string()), &tag.tag_name)?;
                    deleted = true;
                }
            }
        }

        Ok(deleted)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        _rank: i64,
    ) -> Result<bool, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let edge_types = self.list_edge_types(space)?;
        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();
        let mut deleted = false;

        for et in edge_types {
            if let Some(edge_label_id) = graph.get_edge_label_id(&et.edge_type_name) {
                if let Some(src_label_id) = graph.get_vertex_label_id(&et.src_tag_name) {
                    if let Some(dst_label_id) = graph.get_vertex_label_id(&et.dst_tag_name) {
                        if graph.delete_edge(edge_label_id, src_label_id, src, dst_label_id, dst, ts).is_ok() {
                            self.delete_edge_indexes(
                                space_info.space_id,
                                &Value::String(src.to_string()),
                                &Value::String(dst.to_string()),
                                &et.edge_type_name,
                            )?;
                            deleted = true;
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }

    fn update_data(&mut self, space: &str, space_id: u64, info: &UpdateInfo) -> Result<bool, StorageError> {
        let space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        if space_info.space_id != space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.get_write_timestamp();
        let mut graph = self.graph.write();

        let UpdateTarget { space_name, label, id, prop } = &info.update_target;
        
        if space_name != space {
            return Err(StorageError::db_error("Space name mismatch in update target".to_string()));
        }

        if let Some(label_id) = graph.get_vertex_label_id(label) {
            let id_str = Self::value_to_string(id);
            let value = match &info.update_op {
                UpdateOp::Set => info.value.clone(),
                UpdateOp::Add => {
                    if let Some(current) = graph.get_vertex(label_id, &id_str, ts) {
                        let current_val = current.properties.iter()
                            .find(|(k, _)| k == prop)
                            .map(|(_, v)| v);
                        if let (Some(crate::core::Value::Int(cv)), crate::core::Value::Int(add_val)) = 
                            (current_val, &info.value) {
                            crate::core::Value::Int(cv + add_val)
                        } else {
                            info.value.clone()
                        }
                    } else {
                        info.value.clone()
                    }
                }
                UpdateOp::Subtract => {
                    if let Some(current) = graph.get_vertex(label_id, &id_str, ts) {
                        let current_val = current.properties.iter()
                            .find(|(k, _)| k == prop)
                            .map(|(_, v)| v);
                        if let (Some(crate::core::Value::Int(cv)), crate::core::Value::Int(sub_val)) = 
                            (current_val, &info.value) {
                            crate::core::Value::Int(cv - sub_val)
                        } else {
                            info.value.clone()
                        }
                    } else {
                        info.value.clone()
                    }
                }
                _ => info.value.clone(),
            };

            graph.update_vertex_property(label_id, &id_str, prop, &value, ts)?;
            
            let props = vec![(prop.clone(), value)];
            self.update_vertex_indexes(space_info.space_id, id, label, &props, ts)?;
            Ok(true)
        } else {
            Err(StorageError::not_found(format!("Label {} not found", label)))
        }
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        self.user_storage.change_password(info)
    }

    fn lookup_index(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        
        let index = self.index_metadata_manager.get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;
        
        let index_data_manager = self.index_data_manager.read();
        let results = index_data_manager.lookup_tag_index(space_id, &index, value)?;
        Ok(results)
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        let space_id = self.schema_manager.get_space_id(space)?;
        
        let index = self.index_metadata_manager.get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;
        
        let index_data_manager = self.index_data_manager.read();
        let results = index_data_manager.lookup_tag_index(space_id, &index, value)?;
        Ok(results.into_iter().map(|v| (v, 1.0)).collect())
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found in space {}", tag, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let id_str = Self::value_to_string(id);

        let label_id = tag_info.tag_id;
        if let Some(record) = graph.get_vertex(label_id, &id_str, ts) {
            let schema = self.schema_manager.get_tag_schema(space, tag)?;
            let data = Self::serialize_properties(&record.properties);
            return Ok(Some((schema, data)));
        }

        Ok(None)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let edge_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let src_str = Self::value_to_string(src);
        let dst_str = Self::value_to_string(dst);

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(record) = graph.get_edge(
                    edge_label_id,
                    src_label_id,
                    &src_str,
                    dst_label_id,
                    &dst_str,
                    ts,
                ) {
                    let schema = self.schema_manager.get_edge_type_schema(space, edge_type)?;
                    let data = Self::serialize_properties(&record.properties);
                    return Ok(Some((schema, data)));
                }
            }
        }

        Ok(None)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found in space {}", tag, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut results = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
            let schema = self.schema_manager.get_tag_schema(space, tag)?;
            for record in iterator {
                let data = Self::serialize_properties(&record.properties);
                results.push((schema.clone(), data));
            }
        }

        Ok(results)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let edge_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut results = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(table) = graph.get_edge_table(src_label_id, dst_label_id, edge_label_id) {
                    let schema = self.schema_manager.get_edge_type_schema(space, edge_type)?;
                    
                    for record in table.scan(ts) {
                        let data = Self::serialize_properties(&record.properties);
                        results.push((schema.clone(), data));
                    }
                }
            }
        }

        Ok(results)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        if let Some(ref path) = self.work_dir {
            let schema_path = path.join("schema");
            self.schema_manager.load_schema(&schema_path)?;
            
            {
                let mut graph = self.graph.write();
                graph.load()?;
            }
            
            let index_path = path.join("indexes");
            let mut index_data_manager = self.index_data_manager.write();
            index_data_manager.load(&index_path)?;
        }
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        if let Some(ref path) = self.work_dir {
            std::fs::create_dir_all(path)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            
            let schema_path = path.join("schema");
            std::fs::create_dir_all(&schema_path)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            self.schema_manager.save_schema(&schema_path)?;
            
            {
                let graph = self.graph.read();
                graph.flush()?;
            }
            
            let index_path = path.join("indexes");
            std::fs::create_dir_all(&index_path)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            let index_data_manager = self.index_data_manager.read();
            index_data_manager.flush(&index_path)?;
        }
        Ok(())
    }

    fn get_storage_stats(&self) -> StorageStats {
        let graph = self.graph.read();
        
        let total_vertices: usize = graph.vertex_tables()
            .values()
            .map(|table| table.total_count())
            .sum();

        let total_edges: usize = graph.edge_tables()
            .map(|(_, table)| table.edge_count() as usize)
            .sum();

        let spaces = self.schema_manager.list_spaces().unwrap_or_default();
        let tags = spaces.iter()
            .filter_map(|s| self.schema_manager.list_tags(&s.space_name).ok())
            .flatten()
            .count();

        let edge_types = spaces.iter()
            .filter_map(|s| self.schema_manager.list_edge_types(&s.space_name).ok())
            .flatten()
            .count();

        StorageStats {
            total_vertices,
            total_edges,
            total_spaces: spaces.len(),
            total_tags: tags,
            total_edge_types: edge_types,
        }
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let _space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let ts = self.get_read_timestamp();
        let graph = self.graph.read();
        let mut dangling_edges = Vec::new();

        for ((src_label_id, dst_label_id, _edge_label_id), table) in graph.edge_tables() {
            let edge_type_name = table.label_name().to_string();
            for record in table.scan(ts) {
                let src_exists = graph.get_vertex_by_internal_id(
                    *src_label_id,
                    record.src_vid as u32,
                    ts,
                ).is_some();
                let dst_exists = graph.get_vertex_by_internal_id(
                    *dst_label_id,
                    record.dst_vid as u32,
                    ts,
                ).is_some();

                if !src_exists || !dst_exists {
                    let edge = Self::edge_record_to_edge(
                        &record,
                        &edge_type_name,
                        &format!("{}", record.src_vid),
                        &format!("{}", record.dst_vid),
                    );
                    dangling_edges.push(edge);
                }
            }
        }

        Ok(dangling_edges)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let mut repaired_count = 0;

        for edge in &dangling_edges {
            if self.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type, edge.ranking).is_ok() {
                repaired_count += 1;
            }
        }

        Ok(repaired_count)
    }

    fn get_db_path(&self) -> &str {
        &self.db_path
    }

    fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        *self.current_txn_context.lock() = context;
    }
}

impl GraphStorage {
    fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
        props: &[(String, Value)],
        ts: u32,
    ) -> Result<(), StorageError> {
        let indexes = self.index_metadata_manager.list_tag_indexes(space_id)?;
        let index_data_manager = self.index_data_manager.read();
        for index in indexes {
            if index.schema_name == tag_name {
                index_data_manager.update_vertex_indexes_mvcc(
                    space_id,
                    vertex_id,
                    &index.name,
                    props,
                    ts,
                )?;
            }
        }
        Ok(())
    }

    fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        props: &[(String, Value)],
        ts: u32,
    ) -> Result<(), StorageError> {
        let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
        let index_data_manager = self.index_data_manager.read();
        for index in indexes {
            if index.schema_name == edge_type {
                index_data_manager.update_edge_indexes_mvcc(
                    space_id,
                    src,
                    dst,
                    &index.name,
                    props,
                    ts,
                )?;
            }
        }
        Ok(())
    }

    fn delete_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        let indexes = self.index_metadata_manager.list_tag_indexes(space_id)?;
        let ts = self.get_write_timestamp();
        let index_data_manager = self.index_data_manager.read();
        for index in indexes {
            if index.schema_name == tag_name {
                index_data_manager.delete_vertex_indexes_mvcc(
                    space_id,
                    vertex_id,
                    ts,
                )?;
            }
        }
        Ok(())
    }

    fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
        let index_names: Vec<String> = indexes.iter()
            .filter(|index| index.schema_name == edge_type)
            .map(|index| index.name.clone())
            .collect();
        
        if !index_names.is_empty() {
            let ts = self.get_write_timestamp();
            let index_data_manager = self.index_data_manager.read();
            index_data_manager.delete_edge_indexes_mvcc(
                space_id,
                src,
                dst,
                &index_names,
                ts,
            )?;
        }
        Ok(())
    }

    fn serialize_properties(props: &[(String, Value)]) -> Vec<u8> {
        let mut data = Vec::new();
        for (key, value) in props {
            data.extend_from_slice(key.as_bytes());
            data.push(0);
            match value {
                Value::String(s) => {
                    data.push(1);
                    data.extend_from_slice(s.as_bytes());
                }
                Value::Int(i) => {
                    data.push(2);
                    data.extend_from_slice(&i.to_le_bytes());
                }
                Value::Float(f) => {
                    data.push(3);
                    data.extend_from_slice(&f.to_le_bytes());
                }
                Value::Bool(b) => {
                    data.push(4);
                    data.push(if *b { 1 } else { 0 });
                }
                _ => {
                    data.push(0);
                }
            }
            data.push(0);
        }
        data
    }
}
