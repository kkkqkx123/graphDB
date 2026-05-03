//! Edge Storage Manager
//!
//! Adapter layer for edge storage using CSR (Compressed Sparse Row) backend.
//! Responsible for edge additions, deletions, and dangling edge detection/repair.

use std::sync::Arc;
use parking_lot::RwLock;

use crate::core::types::{EdgeTypeInfo, InsertEdgeInfo};
use crate::core::{Edge, EdgeDirection, StorageError, Value};
use crate::storage::edge::{EdgeDirection as CsrEdgeDirection, EdgeId, EdgeRecord, EdgeStrategy, LabelId, Timestamp};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{IndexMetadataManager, Schema, SchemaManager};
use crate::storage::operations::{EdgeReader, EdgeWriter, ScanResult};
use crate::storage::property_graph::PropertyGraph;
use crate::storage::vertex::VertexId;
use crate::storage::version_manager::VersionManager;
use crate::sync::coordinator::ChangeType;

const INVALID_TIMESTAMP: Timestamp = u32::MAX;

#[derive(Clone)]
pub struct EdgeStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    version_manager: Arc<VersionManager>,
    schema_manager: Arc<dyn SchemaManager + Send + Sync>,
    index_data_manager: RedbIndexDataManager,
    sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
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
        index_data_manager: RedbIndexDataManager,
        sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
    ) -> Result<Self, StorageError> {
        Ok(Self {
            graph,
            version_manager,
            schema_manager,
            index_data_manager,
            sync_manager,
        })
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
        INVALID_TIMESTAMP - 1
    }

    fn get_write_timestamp(&self) -> Timestamp {
        INVALID_TIMESTAMP - 1
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
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        let src_vid = self.value_to_vertex_id(src)?;
        let dst_vid = self.value_to_vertex_id(dst)?;

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        if let Some(label_id) = graph.get_edge_label_id(edge_type) {
            if let Some(table) = graph.get_edge_table_by_label(label_id) {
                if let Some(record) = table.get_edge(src_vid, dst_vid, ts) {
                    return Ok(Some(self.edge_record_to_edge(&record, edge_type)));
                }
            }
        }

        Ok(None)
    }

    pub fn get_node_edges(
        &self,
        space: &str,
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
                let out_edges = self.get_edges_for_direction(&graph, vid, CsrEdgeDirection::Out, ts);
                let in_edges = self.get_edges_for_direction(&graph, vid, CsrEdgeDirection::In, ts);
                edges.extend(out_edges);
                edges.extend(in_edges);
                return Ok(edges);
            }
        };

        edges = self.get_edges_for_direction(&graph, vid, csr_direction, ts);
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
                    for nbr in table.out_edges(vid, ts) {
                        let mut props = std::collections::HashMap::new();
                        for (name, value) in &nbr.properties {
                            props.insert(name.clone(), value.clone());
                        }

                        edges.push(Edge {
                            src: Box::new(Value::BigInt(vid as i64)),
                            dst: Box::new(Value::BigInt(nbr.neighbor as i64)),
                            edge_type: edge_type.clone(),
                            ranking: 0,
                            id: nbr.edge_id as i64,
                            props,
                        });
                    }
                }
                CsrEdgeDirection::In => {
                    for nbr in table.in_edges(vid, ts) {
                        let mut props = std::collections::HashMap::new();
                        for (name, value) in &nbr.properties {
                            props.insert(name.clone(), value.clone());
                        }

                        edges.push(Edge {
                            src: Box::new(Value::BigInt(nbr.neighbor as i64)),
                            dst: Box::new(Value::BigInt(vid as i64)),
                            edge_type: edge_type.clone(),
                            ranking: 0,
                            id: nbr.edge_id as i64,
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
        space: &str,
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

        Ok(edges)
    }

    pub fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let mut edges = Vec::new();

        for (_, table) in graph.edge_tables() {
            let edge_type = table.label_name().to_string();
            for record in table.scan(ts) {
                edges.push(self.edge_record_to_edge(&record, &edge_type));
            }
        }

        Ok(edges)
    }

    pub fn insert_edge(&self, space: &str, space_id: u64, edge: Edge) -> Result<(), StorageError> {
        let txn_id = self.get_current_txn_id();

        let src_vid = self.value_to_vertex_id(&edge.src)?;
        let dst_vid = self.value_to_vertex_id(&edge.dst)?;

        let properties: Vec<(String, crate::core::Value)> = edge
            .props
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let ts = self.get_write_timestamp();

        {
            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                graph.insert_edge(label_id, src_vid, dst_vid, &properties, ts)?;
            }
        }

        let indexes = self.index_data_manager.list_edge_indexes(space_id)?;

        for index in indexes {
            if index.schema_name == edge.edge_type {
                let mut index_props = Vec::new();
                for field in &index.fields {
                    if let Some(value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), value.clone()));
                    }
                }

                if !index_props.is_empty() {
                    self.index_data_manager.update_edge_indexes(
                        space_id,
                        &edge.src,
                        &edge.dst,
                        &index.name,
                        &index_props,
                    )?;
                }
            }
        }

        if let Some(sync_manager) = self.get_sync_manager() {
            sync_manager
                .on_edge_insert(txn_id, space_id, &edge)
                .map_err(|e| StorageError::DbError(format!("Failed to sync edge insert: {}", e)))?;
        }

        Ok(())
    }

    pub fn delete_edge(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let old_edge = self.get_edge(space, src, dst, edge_type, rank)?;

        {
            let src_vid = self.value_to_vertex_id(src)?;
            let dst_vid = self.value_to_vertex_id(dst)?;
            let ts = self.get_write_timestamp();

            let mut graph = self.graph.write();

            if let Some(label_id) = graph.get_edge_label_id(edge_type) {
                graph.delete_edge(label_id, src_vid, dst_vid, ts)?;
            }
        }

        let indexes = self.index_data_manager.list_edge_indexes(space_id)?;
        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .map(|idx| idx.name)
            .collect();
        self.index_data_manager
            .delete_edge_indexes(space_id, src, dst, &index_names)?;

        if let Some(sync_manager) = self.get_sync_manager() {
            if let Some(edge) = old_edge {
                sync_manager
                    .on_edge_delete(0, space_id, &edge.src, &edge.dst, &edge.edge_type)
                    .map_err(|e| StorageError::DbError(format!("Failed to sync edge delete: {}", e)))?;
            }
        }

        Ok(())
    }

    pub fn batch_insert_edges(&self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let ts = self.get_write_timestamp();

        let mut graph = self.graph.write();

        for edge in edges {
            let src_vid = self.value_to_vertex_id(&edge.src)?;
            let dst_vid = self.value_to_vertex_id(&edge.dst)?;

            let properties: Vec<(String, crate::core::Value)> = edge
                .props
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                graph.insert_edge(label_id, src_vid, dst_vid, &properties, ts)?;
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
        let edges = self.scan_all_edges(space)?;

        for edge in edges {
            if *edge.src == *vertex_id || *edge.dst == *vertex_id {
                {
                    let src_vid = self.value_to_vertex_id(&edge.src)?;
                    let dst_vid = self.value_to_vertex_id(&edge.dst)?;
                    let ts = self.get_write_timestamp();

                    let mut graph = self.graph.write();

                    if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                        graph.delete_edge(label_id, src_vid, dst_vid, ts)?;
                    }
                }

                let indexes = self.index_data_manager.list_edge_indexes(space_id)?;
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
        }

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
                graph.insert_edge(label_id, src_vid, dst_vid, &properties, ts)?;
            }
        }

        self.index_data_manager.update_edge_indexes(
            space_id,
            &src_vertex_id,
            &dst_vertex_id,
            &edge_name,
            &props,
        )?;

        Ok(true)
    }

    pub fn delete_edge_data(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        rank: i64,
    ) -> Result<bool, StorageError> {
        let edges = self.scan_all_edges(space)?;
        let mut deleted = false;

        for edge in edges {
            if *edge.src == *src && *edge.dst == *dst && edge.ranking == rank {
                {
                    let src_vid = self.value_to_vertex_id(&edge.src)?;
                    let dst_vid = self.value_to_vertex_id(&edge.dst)?;
                    let ts = self.get_write_timestamp();

                    let mut graph = self.graph.write();

                    if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                        graph.delete_edge(label_id, src_vid, dst_vid, ts)?;
                    }
                }

                let indexes = self.index_data_manager.list_edge_indexes(space_id)?;
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

    fn vertex_exists(&self, space: &str, id: &Value) -> Result<bool, StorageError> {
        let vid = self.value_to_vertex_id(id)?;

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        for (_, table) in graph.vertex_tables() {
            if table.get_by_internal_id(vid as u32, ts).is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn repair_dangling_edges(&self, space: &str, space_id: u64) -> Result<usize, StorageError> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let count = dangling_edges.len();

        for edge in dangling_edges {
            {
                let src_vid = self.value_to_vertex_id(&edge.src)?;
                let dst_vid = self.value_to_vertex_id(&edge.dst)?;
                let ts = self.get_write_timestamp();

                let mut graph = self.graph.write();

                if let Some(label_id) = graph.get_edge_label_id(&edge.edge_type) {
                    graph.delete_edge(label_id, src_vid, dst_vid, ts)?;
                }
            }

            let indexes = self.index_data_manager.list_edge_indexes(space_id)?;
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

        Ok(count)
    }

    pub fn build_edge_schema(&self, edge_type_info: &EdgeTypeInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(edge_type_info.edge_type_name.clone(), 1);
        for prop in &edge_type_info.properties {
            let field_def = crate::storage::api::types::FieldDef {
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

impl EdgeReader for EdgeStorage {
    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        self.get_edge(space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        self.get_node_edges(space, node_id, direction).map(ScanResult::new)
    }

    fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<ScanResult<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.get_node_edges_filtered(space, node_id, direction, filter)
            .map(ScanResult::new)
    }

    fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
        self.scan_edges_by_type(space, edge_type).map(ScanResult::new)
    }

    fn scan_all_edges(&self, space: &str) -> Result<ScanResult<Edge>, StorageError> {
        self.scan_all_edges(space).map(ScanResult::new)
    }
}

impl EdgeWriter for EdgeStorage {
    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let space_id = self.get_space_id(space)?;
        self.insert_edge(space, space_id, edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let space_id = self.get_space_id(space)?;
        self.delete_edge(space, space_id, src, dst, edge_type, rank)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        self.batch_insert_edges(space, edges)
    }
}
