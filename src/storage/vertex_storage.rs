use crate::core::types::{InsertVertexInfo, TagInfo, UpdateInfo, UpdateOp};
use crate::core::{StorageError, Value, Vertex};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{
    IndexMetadataManager, RedbIndexMetadataManager, RedbSchemaManager, SchemaManager,
};
use crate::storage::operations::{RedbReader, RedbWriter, VertexReader, VertexWriter};
use crate::storage::Schema;
use parking_lot::Mutex;
use redb::Database;
use std::sync::Arc;

/// Vertex Storage Manager
///
/// Responsible for vertex additions, deletions and tag management
#[derive(Clone)]
pub struct VertexStorage {
    reader: Arc<Mutex<RedbReader>>,
    writer: Arc<Mutex<RedbWriter>>,
    index_data_manager: RedbIndexDataManager,
    schema_manager: Arc<RedbSchemaManager>,
    index_metadata_manager: Arc<RedbIndexMetadataManager>,
}

impl std::fmt::Debug for VertexStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexStorage").finish()
    }
}

impl VertexStorage {
    /// Creating a New Vertex Store Instance
    pub fn new(
        db: Arc<Database>,
        reader: Arc<Mutex<RedbReader>>,
        writer: Arc<Mutex<RedbWriter>>,
        schema_manager: Arc<RedbSchemaManager>,
        index_metadata_manager: Arc<RedbIndexMetadataManager>,
    ) -> Result<Self, StorageError> {
        let index_data_manager = RedbIndexDataManager::new(db);

        Ok(Self {
            reader,
            writer,
            index_data_manager,
            schema_manager,
            index_metadata_manager,
        })
    }

    /// Get a single vertex
    pub fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.reader.lock().get_vertex(space, id)
    }

    /// Scan all vertices
    pub fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.reader
            .lock()
            .scan_vertices(space)
            .map(|r| r.into_vec())
    }

    /// Scanning vertices by label
    pub fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.reader
            .lock()
            .scan_vertices_by_tag(space, tag)
            .map(|r| r.into_vec())
    }

    /// Scanning vertices by attribute
    pub fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.reader
            .lock()
            .scan_vertices_by_prop(space, tag, prop, value)
            .map(|r| r.into_vec())
    }

    /// Insert vertex
    pub fn insert_vertex(
        &self,
        space: &str,
        space_id: u64,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        let id = {
            let mut writer = self.writer.lock();
            writer.insert_vertex(space, vertex.clone())?
        };

        // Update Index
        for tag in &vertex.tags {
            let indexes = self.index_metadata_manager.list_tag_indexes(space_id)?;

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
                            &id,
                            &index.name,
                            &index_props,
                        )?;
                    }
                }
            }
        }

        Ok(id)
    }

    /// Update Vertex
    pub fn update_vertex(&self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let mut writer = self.writer.lock();
        writer.update_vertex(space, vertex)
    }

    /// Delete Vertex
    pub fn delete_vertex(
        &self,
        space: &str,
        space_id: u64,
        id: &Value,
    ) -> Result<(), StorageError> {
        {
            let mut writer = self.writer.lock();
            writer.delete_vertex(space, id)?;
        }

        // Delete Index
        self.index_data_manager
            .delete_vertex_indexes(space_id, id)?;

        Ok(())
    }

    /// Batch insertion of vertices
    pub fn batch_insert_vertices(
        &self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let mut writer = self.writer.lock();
        writer.batch_insert_vertices(space, vertices)
    }

    /// Deletes the specified label on a vertex
    pub fn delete_tags(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let deleted_count = {
            let mut writer = self.writer.lock();
            writer.delete_tags(space, vertex_id, tag_names)?
        };

        // Delete Related Indexes
        for tag_name in tag_names {
            self.index_data_manager
                .delete_tag_indexes(space_id, vertex_id, tag_name)?;
        }

        Ok(deleted_count)
    }

    /// Insert vertex data (advanced interface)
    pub fn insert_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        // Get label information
        let tag_name = info.tag_name.clone();
        let _tag_info = self
            .schema_manager
            .get_tag(space, &tag_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag_name, space))
            })?;

        // Constructing vertex attribute mappings
        let mut properties = std::collections::HashMap::new();
        for (prop_name, prop_value) in &info.props {
            properties.insert(prop_name.clone(), prop_value.clone());
        }

        // Creating Tags
        let tag = crate::core::vertex_edge_path::Tag {
            name: tag_name.clone(),
            properties,
        };

        // Getting or creating vertices
        let vertex = match self.reader.lock().get_vertex(space, &info.vertex_id)? {
            Some(mut existing_vertex) => {
                existing_vertex.tags.retain(|t| t.name != tag_name);
                existing_vertex.tags.push(tag);
                existing_vertex
            }
            None => crate::core::Vertex {
                vid: Box::new(info.vertex_id.clone()),
                id: 0,
                tags: vec![tag],
                properties: std::collections::HashMap::new(),
            },
        };

        // Insert vertex
        {
            let mut writer = self.writer.lock();
            writer.update_vertex(space, vertex)?;
        }

        // Update Index
        self.index_data_manager.update_vertex_indexes(
            space_id,
            &info.vertex_id,
            &tag_name,
            &info.props,
        )?;

        Ok(true)
    }

    /// Delete vertex data (advanced interface)
    pub fn delete_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<bool, StorageError> {
        // Delete Vertex Index
        self.index_data_manager
            .delete_vertex_indexes(space_id, vertex_id)?;

        // Delete the vertex itself
        let mut writer = self.writer.lock();
        writer.delete_vertex(space, vertex_id)?;

        Ok(true)
    }

    /// Updating vertex properties
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

    /// Update vertex properties (internal method)
    fn update_vertex_property(
        &self,
        space: &str,
        vertex_id: &Value,
        tag: &str,
        prop: &str,
        op: &UpdateOp,
        value: &Value,
    ) -> Result<(), StorageError> {
        if let Some(mut vertex) = self.reader.lock().get_vertex(space, vertex_id)? {
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
            let mut writer = self.writer.lock();
            writer.update_vertex(space, vertex)?;
        }
        Ok(())
    }

    /// Build the vertex schema
    pub fn build_vertex_schema(&self, tag_info: &TagInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(tag_info.tag_name.clone(), 1);
        for prop in &tag_info.properties {
            let field_def = crate::storage::types::FieldDef {
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

    /// Get vertices with schema
    pub fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        if let Some(vertex) = self.reader.lock().get_vertex(space, id)? {
            let tag_info = self.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
            })?;
            let schema = self.build_vertex_schema(&tag_info)?;
            let vertex_data = encode_to_vec(&vertex, standard())?;
            return Ok(Some((schema, vertex_data)));
        }
        Ok(None)
    }

    /// Scanning vertices with schema
    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        let mut results = Vec::new();
        let tag_info = self.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
        })?;
        let schema = self.build_vertex_schema(&tag_info)?;

        let vertices = self.reader.lock().scan_vertices(space)?;
        for vertex in vertices {
            if vertex.tags.iter().any(|t| t.name == tag) {
                let vertex_data = encode_to_vec(&vertex, standard())?;
                results.push((schema.clone(), vertex_data));
            }
        }

        Ok(results)
    }
}
