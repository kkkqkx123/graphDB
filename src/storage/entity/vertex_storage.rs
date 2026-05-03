//! Vertex Storage Manager
//!
//! Adapter layer for vertex storage using columnar storage backend.
//! Provides vertex additions, deletions and tag management.

use std::sync::Arc;
use parking_lot::RwLock;

use crate::core::types::{InsertVertexInfo, TagInfo, UpdateInfo, UpdateOp};
use crate::core::{StorageError, Value, Vertex};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{IndexMetadataManager, Schema, SchemaManager};
use crate::storage::operations::{ScanResult, VertexReader, VertexWriter};
use crate::storage::property_graph::PropertyGraph;
use crate::storage::vertex::{LabelId, PropertyDef, Timestamp, VertexRecord, VertexSchema};
use crate::storage::version_manager::VersionManager;
use crate::sync::coordinator::ChangeType;
use crate::transaction::wal::types::Timestamp as WalTimestamp;

const INVALID_TIMESTAMP: Timestamp = u32::MAX;

#[derive(Clone)]
pub struct VertexStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    version_manager: Arc<VersionManager>,
    schema_manager: Arc<dyn SchemaManager + Send + Sync>,
    index_data_manager: RedbIndexDataManager,
    sync_manager: Arc<RwLock<Option<Arc<crate::sync::SyncManager>>>>,
}

impl std::fmt::Debug for VertexStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexStorage").finish()
    }
}

impl VertexStorage {
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

    fn get_label_id(&self, space: &str, tag: &str) -> Result<LabelId, StorageError> {
        let graph = self.graph.read();
        graph
            .get_vertex_label_id(tag)
            .ok_or_else(|| StorageError::DbError(format!("Label '{}' not found", tag)))
    }

    fn value_to_string_id(&self, id: &Value) -> Result<String, StorageError> {
        match id {
            Value::BigInt(v) => Ok(v.to_string()),
            Value::String(v) => Ok(v.clone()),
            _ => Err(StorageError::DbError(format!(
                "Unsupported vertex ID type: {:?}",
                id
            ))),
        }
    }

    fn string_id_to_value(&self, id: &str) -> Value {
        if let Ok(v) = id.parse::<i64>() {
            Value::BigInt(v)
        } else {
            Value::String(id.to_string())
        }
    }

    fn detect_changed_properties(
        &self,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> Vec<(String, Value)> {
        let mut changed_props = Vec::new();

        for new_tag in &new_vertex.tags {
            if let Some(old_tag) = old_vertex.tags.iter().find(|t| t.name == new_tag.name) {
                for (prop_name, new_value) in &new_tag.properties {
                    if let Some(old_value) = old_tag.properties.get(prop_name) {
                        if old_value != new_value {
                            changed_props.push((prop_name.clone(), new_value.clone()));
                        }
                    } else {
                        changed_props.push((prop_name.clone(), new_value.clone()));
                    }
                }

                for (prop_name, old_value) in &old_tag.properties {
                    if !new_tag.properties.contains_key(prop_name) {
                        changed_props.push((prop_name.clone(), old_value.clone()));
                    }
                }
            } else {
                for (prop_name, value) in &new_tag.properties {
                    changed_props.push((prop_name.clone(), value.clone()));
                }
            }
        }

        changed_props
    }

    fn vertex_record_to_vertex(&self, record: &VertexRecord, tag_name: &str) -> Vertex {
        let mut properties = std::collections::HashMap::new();
        for (name, value) in &record.properties {
            properties.insert(name.clone(), value.clone());
        }

        let tag = crate::core::vertex_edge_path::Tag {
            name: tag_name.to_string(),
            properties,
        };

        Vertex {
            vid: Box::new(self.string_id_to_value(&record.vid.to_string())),
            id: record.vid,
            tags: vec![tag],
            properties: std::collections::HashMap::new(),
        }
    }

    pub fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let external_id = self.value_to_string_id(id)?;
        let label_id = self.get_label_id(space, "default").ok();

        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        if let Some(label) = label_id {
            if let Some(table) = graph.get_vertex_table(label) {
                if let Some(record) = table.get(&external_id, ts) {
                    return Ok(Some(self.vertex_record_to_vertex(&record, "default")));
                }
            }
        }

        Ok(None)
    }

    pub fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let mut vertices = Vec::new();
        for (_, table) in graph.vertex_tables() {
            for record in table.scan(ts) {
                let vertex = self.vertex_record_to_vertex(&record, &table.label_name());
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    pub fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<Vertex>, StorageError> {
        let graph = self.graph.read();
        let ts = self.get_read_timestamp();

        let label_id = match graph.get_vertex_label_id(tag) {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let mut vertices = Vec::new();
        if let Some(table) = graph.get_vertex_table(label_id) {
            for record in table.scan(ts) {
                let vertex = self.vertex_record_to_vertex(&record, tag);
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    pub fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        let all_vertices = self.scan_vertices_by_tag(space, tag)?;
        let filtered: Vec<Vertex> = all_vertices
            .into_iter()
            .filter(|v| {
                v.tags
                    .iter()
                    .any(|t| t.properties.get(prop) == Some(value))
            })
            .collect();

        Ok(filtered)
    }

    fn get_read_timestamp(&self) -> Timestamp {
        INVALID_TIMESTAMP - 1
    }

    fn get_write_timestamp(&self) -> Timestamp {
        INVALID_TIMESTAMP - 1
    }

    pub fn insert_vertex(
        &self,
        space: &str,
        space_id: u64,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        let txn_id = self.get_current_txn_id();
        let vid = vertex.vid.clone();
        let external_id = self.value_to_string_id(&vid)?;

        let tag = vertex
            .tags
            .first()
            .ok_or_else(|| StorageError::DbError("Vertex has no tags".to_string()))?;

        let label_id = self.get_label_id(space, &tag.name)?;
        let properties: Vec<(String, Value)> = tag
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let ts = self.get_write_timestamp();

        {
            let mut graph = self.graph.write();
            graph.insert_vertex(label_id, &external_id, &properties, ts)?;
        }

        for tag in &vertex.tags {
            let indexes = self.index_data_manager.list_tag_indexes(space_id)?;

            for index in indexes {
                if index.schema_name == tag.name {
                    let mut index_props = Vec::new();
                    for field in &index.fields {
                        if let Some(value) = tag.properties.get(&field.name) {
                            index_props.push((field.name.clone(), value.clone()));
                        }
                    }

                    if !index_props.is_empty() {
                        self.index_data_manager.update_vertex_indexes(
                            space_id,
                            &vid,
                            &index.name,
                            &index_props,
                        )?;
                    }
                }
            }

            if let Some(sync_manager) = self.get_sync_manager() {
                let props: Vec<(String, Value)> = tag
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if !props.is_empty() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            &vid,
                            &props,
                            ChangeType::Insert,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex insert: {}", e))
                        })?;
                }
            }
        }

        Ok(vid)
    }

    pub fn update_vertex(&self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let vid = vertex.vid.clone();
        let txn_id = self.get_current_txn_id();

        let old_vertex = self.get_vertex(space, &vid)?;

        {
            let external_id = self.value_to_string_id(&vid)?;
            let tag = vertex
                .tags
                .first()
                .ok_or_else(|| StorageError::DbError("Vertex has no tags".to_string()))?;

            let label_id = self.get_label_id(space, &tag.name)?;
            let properties: Vec<(String, Value)> = tag
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let ts = self.get_write_timestamp();

            let mut graph = self.graph.write();
            graph.update_vertex_properties(label_id, &external_id, &properties, ts)?;
        }

        if let Some(sync_manager) = self.get_sync_manager() {
            if let Some(old_v) = old_vertex {
                let changed_props = self.detect_changed_properties(&old_v, &vertex);
                if !changed_props.is_empty() {
                    let space_id = self.get_space_id(space)?;
                    let tag_name = vertex
                        .tags
                        .first()
                        .map(|t| t.name.as_str())
                        .unwrap_or("default");

                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            tag_name,
                            &vid,
                            &changed_props,
                            ChangeType::Update,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex update: {}", e))
                        })?;
                }
            }
        }

        Ok(())
    }

    pub fn delete_vertex(
        &self,
        space: &str,
        space_id: u64,
        id: &Value,
    ) -> Result<(), StorageError> {
        let txn_id = self.get_current_txn_id();

        let old_vertex = self.get_vertex(space, id)?;

        {
            let external_id = self.value_to_string_id(id)?;
            let ts = self.get_write_timestamp();

            let mut graph = self.graph.write();
            if let Some(label_id) = graph.get_vertex_label_id("default") {
                graph.delete_vertex(label_id, &external_id, ts)?;
            }
        }

        self.index_data_manager.delete_vertex_indexes(space_id, id)?;

        if let Some(sync_manager) = self.get_sync_manager() {
            if let Some(vertex) = old_vertex {
                if let Some(tag) = vertex.tags.first() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            id,
                            &[],
                            ChangeType::Delete,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex delete: {}", e))
                        })?;
                }
            }
        }

        Ok(())
    }

    pub fn batch_insert_vertices(
        &self,
        space: &str,
        space_id: u64,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let txn_id = self.get_current_txn_id();
        let mut ids = Vec::with_capacity(vertices.len());

        for vertex in &vertices {
            ids.push(vertex.vid.clone());
        }

        {
            let ts = self.get_write_timestamp();
            let mut graph = self.graph.write();

            for vertex in &vertices {
                let external_id = self.value_to_string_id(&vertex.vid)?;
                let tag = vertex
                    .tags
                    .first()
                    .ok_or_else(|| StorageError::DbError("Vertex has no tags".to_string()))?;

                let label_id = self.get_label_id(space, &tag.name)?;
                let properties: Vec<(String, Value)> = tag
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                graph.insert_vertex(label_id, &external_id, &properties, ts)?;
            }
        }

        if let Some(sync_manager) = self.get_sync_manager() {
            for vertex in &vertices {
                let vid = vertex.vid.clone();
                for tag in &vertex.tags {
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                &tag.name,
                                &vid,
                                &props,
                                ChangeType::Insert,
                            )
                            .map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to sync vertex insert in batch: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(ids)
    }

    pub fn delete_tags(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let mut deleted_count = 0;

        for tag_name in tag_names {
            self.index_data_manager
                .delete_tag_indexes(space_id, vertex_id, tag_name)?;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    pub fn insert_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let txn_id = self.get_current_txn_id();

        let tag_name = info.tag_name.clone();
        let _tag_info = self
            .schema_manager
            .get_tag(space, &tag_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag_name, space))
            })?;

        let external_id = self.value_to_string_id(&info.vertex_id)?;
        let properties: Vec<(String, Value)> = info.props.clone();

        let label_id = self.get_label_id(space, &tag_name)?;
        let ts = self.get_write_timestamp();

        {
            let mut graph = self.graph.write();
            graph.insert_vertex(label_id, &external_id, &properties, ts)?;
        }

        self.index_data_manager.update_vertex_indexes(
            space_id,
            &info.vertex_id,
            &tag_name,
            &info.props,
        )?;

        if let Some(sync_manager) = self.get_sync_manager() {
            if !info.props.is_empty() {
                sync_manager
                    .on_vertex_change_with_txn(
                        txn_id,
                        space_id,
                        &tag_name,
                        &info.vertex_id,
                        &info.props,
                        ChangeType::Insert,
                    )
                    .map_err(|e| {
                        StorageError::DbError(format!("Failed to sync vertex data insert: {}", e))
                    })?;
            }
        }

        Ok(true)
    }

    pub fn delete_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<bool, StorageError> {
        let txn_id = self.get_current_txn_id();

        let old_vertex = self.get_vertex(space, vertex_id)?;

        self.index_data_manager
            .delete_vertex_indexes(space_id, vertex_id)?;

        {
            let external_id = self.value_to_string_id(vertex_id)?;
            let ts = self.get_write_timestamp();

            let mut graph = self.graph.write();
            if let Some(label_id) = graph.get_vertex_label_id("default") {
                graph.delete_vertex(label_id, &external_id, ts)?;
            }
        }

        if let Some(sync_manager) = self.get_sync_manager() {
            if let Some(vertex) = old_vertex {
                if let Some(tag) = vertex.tags.first() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            vertex_id,
                            &[],
                            ChangeType::Delete,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to sync vertex data delete: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        Ok(true)
    }

    pub fn update_data(&self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.update_vertex_property(
            space,
            &info.update_target.id,
            &info.update_target.label,
            &info.update_target.prop,
            &info.update_op,
            &info.value,
        )?;
        Ok(true)
    }

    fn update_vertex_property(
        &self,
        space: &str,
        vertex_id: &Value,
        tag: &str,
        prop: &str,
        op: &UpdateOp,
        value: &Value,
    ) -> Result<(), StorageError> {
        if let Some(mut vertex) = self.get_vertex(space, vertex_id)? {
            for tag_data in &mut vertex.tags {
                if tag_data.name == tag {
                    match op {
                        UpdateOp::Set => {
                            tag_data.properties.insert(prop.to_string(), value.clone());
                        }
                        UpdateOp::Add => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a + b));
                                }
                            }
                        }
                        UpdateOp::Subtract => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a - b));
                                }
                            }
                        }
                        _ => {}
                    }
                    break;
                }
            }

            let external_id = self.value_to_string_id(vertex_id)?;
            let tag_data = vertex
                .tags
                .iter()
                .find(|t| t.name == tag)
                .ok_or_else(|| StorageError::DbError(format!("Tag '{}' not found", tag)))?;

            let properties: Vec<(String, Value)> = tag_data
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let label_id = self.get_label_id(space, tag)?;
            let ts = self.get_write_timestamp();

            let mut graph = self.graph.write();
            graph.update_vertex_properties(label_id, &external_id, &properties, ts)?;
        }
        Ok(())
    }

    pub fn build_vertex_schema(&self, tag_info: &TagInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(tag_info.tag_name.clone(), 1);
        for prop in &tag_info.properties {
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

    pub fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        if let Some(vertex) = self.get_vertex(space, id)? {
            let tag_info = self
                .schema_manager
                .get_tag(space, tag)?
                .ok_or_else(|| {
                    StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
                })?;
            let schema = self.build_vertex_schema(&tag_info)?;
            let vertex_data = encode_to_vec(&vertex)?;
            return Ok(Some((schema, vertex_data)));
        }
        Ok(None)
    }

    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        let mut results = Vec::new();
        let tag_info = self
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
            })?;
        let schema = self.build_vertex_schema(&tag_info)?;

        let vertices = self.scan_vertices(space)?;
        for vertex in vertices {
            if vertex.tags.iter().any(|t| t.name == tag) {
                let vertex_data = encode_to_vec(&vertex)?;
                results.push((schema.clone(), vertex_data));
            }
        }

        Ok(results)
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.sync_manager.read().clone()
    }
}

impl VertexReader for VertexStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        self.scan_vertices(space).map(ScanResult::new)
    }

    fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag_name: &str,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        self.scan_vertices_by_tag(space, tag_name).map(ScanResult::new)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag_name: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        self.scan_vertices_by_prop(space, tag_name, prop_name, value)
            .map(ScanResult::new)
    }
}

impl VertexWriter for VertexStorage {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let space_id = self.get_space_id(space)?;
        self.insert_vertex(space, space_id, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        self.update_vertex(space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let space_id = self.get_space_id(space)?;
        self.delete_vertex(space, space_id, id)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let space_id = self.get_space_id(space)?;
        self.batch_insert_vertices(space, space_id, vertices)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let space_id = self.get_space_id(space)?;
        self.delete_tags(space, space_id, vertex_id, tag_names)
    }
}
