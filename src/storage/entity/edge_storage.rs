//! Edge Storage Manager
//!
//! Adapter layer for edge storage using CSR (Compressed Sparse Row) backend.
//! Responsible for edge additions, deletions, and dangling edge detection/repair.

use parking_lot::RwLock;
use std::sync::Arc;

use crate::core::types::{EdgeTypeInfo, InsertEdgeInfo};
use crate::core::{Edge, EdgeDirection, StorageError, Value};
use crate::storage::edge::{EdgeDirection as CsrEdgeDirection, EdgeRecord, Timestamp};
use crate::storage::index::{DegreeIndex, EdgeIdIndex};
use crate::storage::index::{InMemoryIndexDataManager, IndexDataManager};
use crate::storage::metadata::{Schema, SchemaManager};
use crate::storage::engine::PropertyGraph;
use crate::storage::vertex::VertexId;
use crate::transaction::version_manager::VersionManager;

const INVALID_TIMESTAMP: Timestamp = u32::MAX;

#[derive(Clone)]
pub struct EdgeStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    version_manager: Arc<VersionManager>,
    schema_manager: Arc<dyn SchemaManager + Send + Sync>,
    index_data_manager: InMemoryIndexDataManager,
    sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
    edge_id_index: Arc<EdgeIdIndex>,
    degree_index: Arc<DegreeIndex>,
}

impl std::fmt::Debug for EdgeStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeStorage").finish()
    }
}

impl EdgeStorage {
    pub fn new(
        graph: Arc<RwLock<PropertyGraph>>,
        version_manager: Arc<VersionManager>,
        schema_manager: Arc<dyn SchemaManager + Send + Sync>,
        index_data_manager: InMemoryIndexDataManager,
        sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
    ) -> Result<Self, StorageError> {
        Ok(Self {
            graph,
            version_manager,
            schema_manager,
            index_data_manager,
            sync_manager,
            edge_id_index: Arc::new(EdgeIdIndex::new()),
            degree_index: Arc::new(DegreeIndex::new()),
        })
    }

    pub fn with_csr_indexes(
        graph: Arc<RwLock<PropertyGraph>>,
        version_manager: Arc<VersionManager>,
        schema_manager: Arc<dyn SchemaManager + Send + Sync>,
        index_data_manager: InMemoryIndexDataManager,
        sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
        edge_id_index: Arc<EdgeIdIndex>,
        degree_index: Arc<DegreeIndex>,
    ) -> Result<Self, StorageError> {
        Ok(Self {
            graph,
            version_manager,
            schema_manager,
            index_data_manager,
            sync_manager,
            edge_id_index,
            degree_index,
        })
    }

    pub fn edge_id_index(&self) -> &EdgeIdIndex {
        &self.edge_id_index
    }

    pub fn degree_index(&self) -> &DegreeIndex {
        &self.degree_index
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        let space_info = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::DbError(format!("Space '{}' not found", space)))?;
        Ok(space_info.space_id)
    }

    fn get_current_txn_id(&self) -> crate::transaction::types::TransactionId {
        0
    }

    fn value_to_vertex_id(&self, id: &Value) -> Result<VertexId, StorageError> {
        match id {
            Value::BigInt(v) => Ok(*v as VertexId),
            Value::String(v) => v
                .parse::<VertexId>()
                .map_err(|e| StorageError::DbError(format!("Invalid vertex ID: {}", e))),
            _ => Err(StorageError::DbError(format!(
                "Unsupported vertex ID type: {:?}",
                id
            ))),
        }
    }

    fn get_read_timestamp(&self) -> Timestamp {
        self.version_manager.acquire_read_timestamp()
    }

    fn release_read_timestamp(&self) {
        self.version_manager.release_read_timestamp();
    }

    fn get_write_timestamp(&self) -> Timestamp {
        self.version_manager.acquire_insert_timestamp()
    }

    fn release_write_timestamp(&self, ts: Timestamp) {
        self.version_manager.release_insert_timestamp(ts);
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.sync_manager.read().clone()
    }

    fn edge_record_to_edge(&self, record: &EdgeRecord, edge_type: &str) -> Edge {
        let mut props = std::collections::HashMap::new();
        for (name, value) in &record.properties {
            props.insert(name.clone(), value.clone());
        }

        Edge {
            src: Box::new(Value::BigInt(record.src_vid as i64)),
            dst: Box::new(Value::BigInt(record.dst_vid as i64)),
            edge_type: edge_type.to_string(),
            ranking: 0,
            id: record.edge_id as i64,
            props,
        }
    }

    pub fn get_edge(
        &self,
        _space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        _rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        let src_vid = self.value_to_vertex_id(src)?;
        let dst_vid = self.value_to_vertex_id(dst)?;

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let result = if let Some(label_id) = graph.get_edge_label_id(edge_type) {
            if let Some(table) = graph.get_edge_table_by_label(label_id) {
                if let Some(record) = table.get_edge(src_vid, dst_vid, ts) {
                    Some(self.edge_record_to_edge(&record, edge_type))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        self.release_read_timestamp();
        Ok(result)
    }

    pub fn get_node_edges(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        let vid = self.value_to_vertex_id(node_id)?;

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let mut edges = Vec::new();

        let csr_direction = match direction {
            EdgeDirection::Out => CsrEdgeDirection::Out,
            EdgeDirection::In => CsrEdgeDirection::In,
            EdgeDirection::Both => {
                let out_edges =
                    self.get_edges_for_direction(&graph, vid, CsrEdgeDirection::Out, ts);
                let in_edges = self.get_edges_for_direction(&graph, vid, CsrEdgeDirection::In, ts);
                edges.extend(out_edges);
                edges.extend(in_edges);
                self.release_read_timestamp();
                return Ok(edges);
            }
        };

        edges = self.get_edges_for_direction(&graph, vid, csr_direction, ts);
        self.release_read_timestamp();
        Ok(edges)
    }

    fn get_edges_for_direction(
        &self,
        graph: &PropertyGraph,
        vid: VertexId,
        direction: CsrEdgeDirection,
        ts: Timestamp,
    ) -> Vec<Edge> {
        let mut edges = Vec::new();

        for (_, table) in graph.edge_tables() {
            let edge_type = table.label_name().to_string();

            match direction {
                CsrEdgeDirection::Out => {
                    for edge_record in table.out_edges(vid, ts) {
                        let mut props = std::collections::HashMap::new();
                        for (name, value) in &edge_record.properties {
                            props.insert(name.clone(), value.clone());
                        }

                        edges.push(Edge {
                            src: Box::new(Value::BigInt(vid as i64)),
                            dst: Box::new(Value::BigInt(edge_record.dst_vid as i64)),
                            edge_type: edge_type.clone(),
                            ranking: 0,
                            id: edge_record.edge_id as i64,
                            props,
                        });
                    }
                }
                CsrEdgeDirection::In => {
                    for edge_record in table.in_edges(vid, ts) {
                        let mut props = std::collections::HashMap::new();
                        for (name, value) in &edge_record.properties {
                            props.insert(name.clone(), value.clone());
                        }

                        edges.push(Edge {
                            src: Box::new(Value::BigInt(edge_record.src_vid as i64)),
                            dst: Box::new(Value::BigInt(vid as i64)),
                            edge_type: edge_type.clone(),
                            ranking: 0,
                            id: edge_record.edge_id as i64,
                            props,
                        });
                    }
                }
            }
        }

        edges
    }

    pub fn get_node_edges_filtered<F>(
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

        if let Some(f) = filter {
            Ok(edges.into_iter().filter(f).collect())
        } else {
            Ok(edges)
        }
    }

    pub fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<Vec<Edge>, StorageError> {
        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let mut edges = Vec::new();

        if let Some(label_id) = graph.get_edge_label_id(edge_type) {
            if let Some(table) = graph.get_edge_table_by_label(label_id) {
                for record in table.scan(ts) {
                    edges.push(self.edge_record_to_edge(&record, edge_type));
                }
            }
        }

        self.release_read_timestamp();
        Ok(edges)
    }

    pub fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let mut edges = Vec::new();

        for (_, table) in graph.edge_tables() {
            let edge_type = table.label_name().to_string();
            for record in table.scan(ts) {
                edges.push(self.edge_record_to_edge(&record, &edge_type));
            }
        }

        self.release_read_timestamp();
        Ok(edges)
    }

    pub fn insert_edge(&self, space: &str, space_id: u64, edge: Edge) -> Result<(), StorageError> {
        let src_vid = self.value_to_vertex_id(&edge.src)?;
        let dst_vid = self.value_to_vertex_id(&edge.dst)?;

        let properties: Vec<(String, crate::core::Value)> = edge
            .props
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let ts = self.get_write_timestamp();

        let _edge_type_info = self
            .schema_manager
            .get_edge_type(space, &edge.edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge.edge_type, space
                ))
            })?;

        {
            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                let (src_label_id, dst_label_id) = graph
                    .get_edge_table_by_label(label_id)
                    .map(|table| (table.src_label(), table.dst_label()))
                    .ok_or_else(|| {
                        StorageError::DbError(format!(
                            "Edge table not found for edge type '{}'",
                            edge.edge_type
                        ))
                    })?;

                let src_id_str = match &*edge.src {
                    Value::String(s) => s.as_str(),
                    _ => &src_vid.to_string(),
                };
                let dst_id_str = match &*edge.dst {
                    Value::String(s) => s.as_str(),
                    _ => &dst_vid.to_string(),
                };
                let edge_id = graph.insert_edge(
                    label_id,
                    src_label_id,
                    src_id_str,
                    dst_label_id,
                    dst_id_str,
                    &properties,
                    ts,
                )?;

                self.edge_id_index.insert(edge_id, src_vid, dst_vid, 0);
                self.degree_index.insert_edge(src_vid, dst_vid);
            }
        }

        let indexes = self.schema_manager.list_edge_indexes(space)?;

        for index in indexes {
            if index.schema_name == edge.edge_type {
                let mut index_props = Vec::new();
                for field in &index.fields {
                    if let Some(value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), value.clone()));
                    }
                }

                if !index_props.is_empty() {
                    self.index_data_manager.update_edge_indexes_mvcc(
                        space_id,
                        &edge.src,
                        &edge.dst,
                        &index.name,
                        &index_props,
                        ts,
                    )?;
                }
            }
        }

        self.release_write_timestamp(ts);
        Ok(())
    }

    pub fn delete_edge(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        _rank: i64,
    ) -> Result<(), StorageError> {
        let src_vid = self.value_to_vertex_id(src)?;
        let dst_vid = self.value_to_vertex_id(dst)?;
        let ts = self.get_write_timestamp();

        let _edge_type_info = self
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;

        {
            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(edge_type) {
                let (src_label_id, dst_label_id) = graph
                    .get_edge_table_by_label(label_id)
                    .map(|table| (table.src_label(), table.dst_label()))
                    .ok_or_else(|| {
                        StorageError::DbError(format!(
                            "Edge table not found for edge type '{}'",
                            edge_type
                        ))
                    })?;

                let src_id_str = match src {
                    Value::String(s) => s.as_str(),
                    _ => &src_vid.to_string(),
                };
                let dst_id_str = match dst {
                    Value::String(s) => s.as_str(),
                    _ => &dst_vid.to_string(),
                };
                graph.delete_edge(
                    label_id,
                    src_label_id,
                    src_id_str,
                    dst_label_id,
                    dst_id_str,
                    ts,
                )?;

                self.degree_index.remove_edge(src_vid, dst_vid);
            }
        }

        let indexes = self.schema_manager.list_edge_indexes(space)?;
        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .map(|idx| idx.name)
            .collect();
        self.index_data_manager
            .delete_edge_indexes_mvcc(space_id, src, dst, &index_names, ts)?;

        self.release_write_timestamp(ts);
        Ok(())
    }

    pub fn batch_insert_edges(&self, space: &str, space_id: u64, edges: Vec<Edge>) -> Result<(), StorageError> {
        let ts = self.get_write_timestamp();

        {
            let mut graph = self.graph.write();

            for edge in &edges {
                let src_vid = self.value_to_vertex_id(&edge.src)?;
                let dst_vid = self.value_to_vertex_id(&edge.dst)?;

                let properties: Vec<(String, crate::core::Value)> = edge
                    .props
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                    let _edge_type_info = self
                        .schema_manager
                        .get_edge_type(space, &edge.edge_type)?
                        .ok_or_else(|| {
                            StorageError::DbError(format!(
                                "Edge type '{}' not found in space '{}'",
                                edge.edge_type, space
                            ))
                        })?;

                    let (src_label_id, dst_label_id) = graph
                        .get_edge_table_by_label(label_id)
                        .map(|table| (table.src_label(), table.dst_label()))
                        .ok_or_else(|| {
                            StorageError::DbError(format!(
                                "Edge table not found for edge type '{}'",
                                edge.edge_type
                            ))
                        })?;

                    let src_id_str = match &*edge.src {
                        Value::String(s) => s.as_str(),
                        _ => &src_vid.to_string(),
                    };
                    let dst_id_str = match &*edge.dst {
                        Value::String(s) => s.as_str(),
                        _ => &dst_vid.to_string(),
                    };
                    let edge_id = graph.insert_edge(
                        label_id,
                        src_label_id,
                        src_id_str,
                        dst_label_id,
                        dst_id_str,
                        &properties,
                        ts,
                    )?;

                    self.edge_id_index.insert(edge_id, src_vid, dst_vid, 0);
                    self.degree_index.insert_edge(src_vid, dst_vid);
                }
            }
        }

        self.release_write_timestamp(ts);

        for edge in &edges {
            let indexes = self.schema_manager.list_edge_indexes(space)?;

            for index in indexes {
                if index.schema_name == edge.edge_type {
                    let mut index_props = Vec::new();
                    for field in &index.fields {
                        if let Some(value) = edge.props.get(&field.name) {
                            index_props.push((field.name.clone(), value.clone()));
                        }
                    }

                    if !index_props.is_empty() {
                        self.index_data_manager.update_edge_indexes_mvcc(
                            space_id,
                            &edge.src,
                            &edge.dst,
                            &index.name,
                            &index_props,
                            ts,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn delete_vertex_edges(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let vid = self.value_to_vertex_id(vertex_id)?;
        let ts = self.get_write_timestamp();

        let edge_types: Vec<_> = {
            let graph = self.graph.read();
            graph
                .edge_tables()
                .map(|(key, table)| (*key, table.label(), table.label_name().to_string()))
                .collect()
        };

        for ((src_label, dst_label, edge_label), label_id, edge_type_name) in edge_types {
            let edges_to_delete = {
                let graph = self.graph.read();
                if let Some(table) = graph.get_edge_table(src_label, dst_label, edge_label) {
                    let read_ts = self.get_read_timestamp();
                    table
                        .scan(read_ts)
                        .into_iter()
                        .filter(|record| record.src_vid == vid || record.dst_vid == vid)
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            };

            for record in edges_to_delete {
                let src_id_str;
                let dst_id_str;
                let src_value;
                let dst_value;

                {
                    let graph = self.graph.read();
                    if let Some(src_table) = graph.get_vertex_table(src_label) {
                        if let Some(ext_id) = src_table.get_external_id(record.src_vid as u32) {
                            src_id_str = ext_id;
                            src_value = Value::String(src_id_str.clone());
                        } else {
                            src_id_str = record.src_vid.to_string();
                            src_value = Value::BigInt(record.src_vid as i64);
                        }
                    } else {
                        src_id_str = record.src_vid.to_string();
                        src_value = Value::BigInt(record.src_vid as i64);
                    }

                    if let Some(dst_table) = graph.get_vertex_table(dst_label) {
                        if let Some(ext_id) = dst_table.get_external_id(record.dst_vid as u32) {
                            dst_id_str = ext_id;
                            dst_value = Value::String(dst_id_str.clone());
                        } else {
                            dst_id_str = record.dst_vid.to_string();
                            dst_value = Value::BigInt(record.dst_vid as i64);
                        }
                    } else {
                        dst_id_str = record.dst_vid.to_string();
                        dst_value = Value::BigInt(record.dst_vid as i64);
                    }
                }

                {
                    let mut graph = self.graph.write();
                    graph.delete_edge(
                        label_id,
                        src_label,
                        &src_id_str,
                        dst_label,
                        &dst_id_str,
                        ts,
                    )?;

                    self.degree_index.remove_edge(record.src_vid, record.dst_vid);
                }

                let indexes = self.schema_manager.list_edge_indexes(space)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge_type_name)
                    .map(|idx| idx.name)
                    .collect();

                self.index_data_manager.delete_edge_indexes_mvcc(
                    space_id,
                    &src_value,
                    &dst_value,
                    &index_names,
                    ts,
                )?;
            }
        }

        self.release_write_timestamp(ts);
        Ok(())
    }

    pub fn insert_edge_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let edge_name = info.edge_name.clone();
        let src_vertex_id = info.src_vertex_id.clone();
        let dst_vertex_id = info.dst_vertex_id.clone();
        let props = info.props.clone();

        let _edge_type_info = self
            .schema_manager
            .get_edge_type(space, &edge_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_name, space
                ))
            })?;

        let src_vid = self.value_to_vertex_id(&src_vertex_id)?;
        let dst_vid = self.value_to_vertex_id(&dst_vertex_id)?;

        let properties: Vec<(String, crate::core::Value)> = props.clone();

        let ts = self.get_write_timestamp();

        {
            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(&edge_name) {
                let (src_label_id, dst_label_id) = graph
                    .get_edge_table_by_label(label_id)
                    .map(|table| (table.src_label(), table.dst_label()))
                    .ok_or_else(|| {
                        StorageError::DbError(format!(
                            "Edge table not found for edge type '{}'",
                            edge_name
                        ))
                    })?;

                let src_id_str = match &src_vertex_id {
                    Value::String(s) => s.as_str(),
                    _ => &src_vid.to_string(),
                };
                let dst_id_str = match &dst_vertex_id {
                    Value::String(s) => s.as_str(),
                    _ => &dst_vid.to_string(),
                };
                let edge_id = graph.insert_edge(
                    label_id,
                    src_label_id,
                    src_id_str,
                    dst_label_id,
                    dst_id_str,
                    &properties,
                    ts,
                )?;

                self.edge_id_index.insert(edge_id, src_vid, dst_vid, 0);
                self.degree_index.insert_edge(src_vid, dst_vid);
            }
        }

        self.index_data_manager.update_edge_indexes(
            space_id,
            &src_vertex_id,
            &dst_vertex_id,
            &edge_name,
            &props,
        )?;

        self.release_write_timestamp(ts);
        Ok(true)
    }

    pub fn delete_edge_data(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        _rank: i64,
    ) -> Result<bool, StorageError> {
        let edges = self.scan_all_edges(space)?;
        let mut deleted = false;
        let ts = self.get_write_timestamp();

        for edge in edges {
            if *edge.src == *src && *edge.dst == *dst {
                let src_vid = self.value_to_vertex_id(&edge.src)?;
                let dst_vid = self.value_to_vertex_id(&edge.dst)?;

                let _edge_type_info = self
                    .schema_manager
                    .get_edge_type(space, &edge.edge_type)?
                    .ok_or_else(|| {
                        StorageError::DbError(format!(
                            "Edge type '{}' not found in space '{}'",
                            edge.edge_type, space
                        ))
                    })?;

                {
                    let mut graph = self.graph.write();

                    if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                        let (src_label_id, dst_label_id) = graph
                            .get_edge_table_by_label(label_id)
                            .map(|table| (table.src_label(), table.dst_label()))
                            .ok_or_else(|| {
                                StorageError::DbError(format!(
                                    "Edge table not found for edge type '{}'",
                                    edge.edge_type
                                ))
                            })?;

                        let src_id_str = match &*edge.src {
                            Value::String(s) => s.as_str(),
                            _ => &src_vid.to_string(),
                        };
                        let dst_id_str = match &*edge.dst {
                            Value::String(s) => s.as_str(),
                            _ => &dst_vid.to_string(),
                        };
                        graph.delete_edge(
                            label_id,
                            src_label_id,
                            src_id_str,
                            dst_label_id,
                            dst_id_str,
                            ts,
                        )?;

                        self.degree_index.remove_edge(src_vid, dst_vid);
                    }
                }

                let indexes = self.schema_manager.list_edge_indexes(space)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge.edge_type)
                    .map(|idx| idx.name)
                    .collect();
                self.index_data_manager.delete_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
                    &index_names,
                )?;
                deleted = true;
                break;
            }
        }

        self.release_write_timestamp(ts);
        Ok(deleted)
    }

    pub fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let mut dangling_edges = Vec::new();
        let edges = self.scan_all_edges(space)?;

        for edge in edges {
            let src_exists = self.vertex_exists(space, &edge.src)?;
            let dst_exists = self.vertex_exists(space, &edge.dst)?;

            if !src_exists || !dst_exists {
                dangling_edges.push(edge);
            }
        }

        Ok(dangling_edges)
    }

    fn vertex_exists(&self, _space: &str, id: &Value) -> Result<bool, StorageError> {
        let vid = self.value_to_vertex_id(id)?;

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let exists = graph
            .vertex_tables()
            .iter()
            .any(|(_, table)| table.get_by_internal_id(vid as u32, ts).is_some());

        self.release_read_timestamp();
        Ok(exists)
    }

    pub fn repair_dangling_edges(&self, space: &str, space_id: u64) -> Result<usize, StorageError> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let count = dangling_edges.len();
        let ts = self.get_write_timestamp();

        for edge in dangling_edges {
            let src_vid = self.value_to_vertex_id(&edge.src)?;
            let dst_vid = self.value_to_vertex_id(&edge.dst)?;

            let _edge_type_info = self
                .schema_manager
                .get_edge_type(space, &edge.edge_type)?
                .ok_or_else(|| {
                    StorageError::DbError(format!(
                        "Edge type '{}' not found in space '{}'",
                        edge.edge_type, space
                    ))
                })?;

            {
                let mut graph = self.graph.write();

                if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                    let (src_label_id, dst_label_id) = graph
                        .get_edge_table_by_label(label_id)
                        .map(|table| (table.src_label(), table.dst_label()))
                        .ok_or_else(|| {
                            StorageError::DbError(format!(
                                "Edge table not found for edge type '{}'",
                                edge.edge_type
                            ))
                        })?;

                    let src_id_str = match &*edge.src {
                        Value::String(s) => s.as_str(),
                        _ => &src_vid.to_string(),
                    };
                    let dst_id_str = match &*edge.dst {
                        Value::String(s) => s.as_str(),
                        _ => &dst_vid.to_string(),
                    };
                    graph.delete_edge(
                        label_id,
                        src_label_id,
                        src_id_str,
                        dst_label_id,
                        dst_id_str,
                        ts,
                    )?;
                }
            }

            let indexes = self.schema_manager.list_edge_indexes(space)?;
            let index_names: Vec<String> = indexes
                .into_iter()
                .filter(|idx| idx.schema_name == edge.edge_type)
                .map(|idx| idx.name)
                .collect();
            self.index_data_manager.delete_edge_indexes(
                space_id,
                &edge.src,
                &edge.dst,
                &index_names,
            )?;
        }

        self.release_write_timestamp(ts);
        Ok(count)
    }

    pub fn update_edge_property(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
        prop_name: &str,
        value: &Value,
    ) -> Result<(), StorageError> {
        // Note: rank parameter is reserved for future multi-edge support.
        // Current implementation uses (src, dst, edge_type) as unique identifier.
        let _ = rank;

        let src_vid = self.value_to_vertex_id(src)?;
        let dst_vid = self.value_to_vertex_id(dst)?;
        let ts = self.get_write_timestamp();

        let _edge_type_info = self
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;

        {
            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(edge_type) {
                let (src_label_id, dst_label_id) = graph
                    .get_edge_table_by_label(label_id)
                    .map(|table| (table.src_label(), table.dst_label()))
                    .ok_or_else(|| {
                        StorageError::DbError(format!(
                            "Edge table not found for edge type '{}'",
                            edge_type
                        ))
                    })?;

                let src_id_str = match src {
                    Value::String(s) => s.as_str(),
                    _ => &src_vid.to_string(),
                };
                let dst_id_str = match dst {
                    Value::String(s) => s.as_str(),
                    _ => &dst_vid.to_string(),
                };
                graph.update_edge_property(
                    label_id,
                    src_label_id,
                    src_id_str,
                    dst_label_id,
                    dst_id_str,
                    prop_name,
                    value,
                    ts,
                )?;
            }
        }

        let indexes = self.schema_manager.list_edge_indexes(space)?;
        for index in indexes {
            if index.schema_name == edge_type && index.fields.iter().any(|f| &f.name == prop_name) {
                self.index_data_manager.update_edge_indexes_mvcc(
                    space_id,
                    src,
                    dst,
                    &index.name,
                    &[(prop_name.to_string(), value.clone())],
                    ts,
                )?;
            }
        }

        self.release_write_timestamp(ts);
        Ok(())
    }

    pub fn build_edge_schema(&self, edge_type_info: &EdgeTypeInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(edge_type_info.edge_type_name.clone(), 1);
        for prop in &edge_type_info.properties {
            let field_def = crate::storage::interface::FieldDef {
                name: prop.name.clone(),
                field_type: prop.data_type.clone(),
                nullable: prop.nullable,
                default_value: prop.default.clone(),
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            };
            schema = schema.add_field(field_def);
        }
        Ok(schema)
    }

    pub fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        if let Some(edge) = self.get_edge(space, src, dst, edge_type, 0)? {
            let edge_type_info = self
                .schema_manager
                .get_edge_type(space, edge_type)?
                .ok_or_else(|| {
                    StorageError::DbError(format!(
                        "Edge type '{}' not found in space '{}'",
                        edge_type, space
                    ))
                })?;
            let schema = self.build_edge_schema(&edge_type_info)?;
            let edge_data = encode_to_vec(&edge)?;
            return Ok(Some((schema, edge_data)));
        }
        Ok(None)
    }

    pub fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        let mut results = Vec::new();
        let edge_type_info = self
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;
        let schema = self.build_edge_schema(&edge_type_info)?;

        let edges = self.scan_edges_by_type(space, edge_type)?;
        for edge in edges {
            let edge_data = encode_to_vec(&edge)?;
            results.push((schema.clone(), edge_data));
        }

        Ok(results)
    }
}
